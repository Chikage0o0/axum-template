pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod password;
pub mod services;
pub mod web_assets;

use crate::api::request_id::request_id_middleware;
use crate::api::routes::{app_router, AppState};
use crate::config::bootstrap::BootstrapConfig;
use crate::config::runtime::RuntimeConfig;
use crate::config::seed::{seed_if_needed, SeedOptions};
use crate::db::connect as connect_db;
use anyhow::Context;
use arc_swap::ArcSwap;
use axum::extract::ConnectInfo;
use axum::middleware;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn, Span};
use tracing_subscriber::fmt::time::ChronoLocal;
use utoipa::OpenApi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--export-openapi") {
        let json = crate::api::openapi::ApiDoc::openapi()
            .to_pretty_json()
            .context("序列化 OpenAPI 失败")?;
        print!("{json}");
        return Ok(());
    }

    init_tracing();
    info!("Starting PROJECT_NAME...");

    let bootstrap = BootstrapConfig::load_from_env().context("加载启动期配置失败")?;

    let db = connect_db(&bootstrap.database_url)
        .await
        .context("连接数据库失败")?;

    if auto_migrate_enabled() {
        run_migrations(&db).await.context("数据库迁移失败")?;
    } else {
        info!("已禁用启动时自动数据库迁移（PROJECT_NAME_AUTO_MIGRATE=0）");
    }

    seed_if_needed(&db, &SeedOptions::from_env())
        .await
        .context("配置初始化（seed）失败")?;

    let runtime = RuntimeConfig::load_from_db(&db)
        .await
        .context("从数据库加载运行期配置失败")?;

    let state = AppState {
        config: Arc::new(ArcSwap::from_pointee(runtime)),
        db,
    };

    let cors = CorsLayer::permissive();

    let access_log = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            let path = request.uri().path();
            if should_skip_access_log(path) {
                return tracing::Span::none();
            }

            let user_agent = request
                .headers()
                .get(axum::http::header::USER_AGENT)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("-");

            let request_id = request
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("-");

            let forwarded_for = request
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());

            let remote_ip = request
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ConnectInfo(addr)| addr.ip().to_string());

            let client_ip = forwarded_for
                .map(str::to_string)
                .or(remote_ip)
                .unwrap_or_else(|| "-".to_string());

            tracing::info_span!(
                "access",
                method = %request.method(),
                path = %path,
                client_ip = %client_ip,
                user_agent = %user_agent,
                request_id = %request_id,
            )
        })
        .on_response(
            |response: &axum::http::Response<_>, latency: Duration, span: &Span| {
                if span.is_disabled() {
                    return;
                }
                let status = response.status().as_u16();
                let latency_ms = latency.as_millis() as u64;
                if status >= 500 {
                    return;
                }
                tracing::info!(parent: span, status, latency_ms, "访问请求完成");
            },
        )
        .on_failure(
            |failure: ServerErrorsFailureClass, latency: Duration, span: &Span| {
                if span.is_disabled() {
                    return;
                }
                let latency_ms = latency.as_millis() as u64;
                match failure {
                    ServerErrorsFailureClass::StatusCode(code) => {
                        let status = code.as_u16();
                        tracing::warn!(parent: span, status, latency_ms, "访问请求失败");
                    }
                    ServerErrorsFailureClass::Error(error) => {
                        tracing::error!(parent: span, latency_ms, error = %error, "访问请求异常");
                    }
                }
            },
        );

    let app = app_router(state)
        .layer(cors)
        .layer(access_log)
        .layer(middleware::from_fn(request_id_middleware));

    let addr_str = format!("{}:{}", bootstrap.server_host, bootstrap.server_port);
    let addr: SocketAddr = addr_str.parse().context("Invalid server address")?;
    info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_timer(ChronoLocal::rfc_3339())
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "info,project_name=debug,hyper=warn,h2=warn,rustls=warn,tower=warn".into()
            }),
        )
        .init();
}

fn auto_migrate_enabled() -> bool {
    let Ok(v) = std::env::var("PROJECT_NAME_AUTO_MIGRATE") else {
        return true;
    };
    let v = v.trim();
    !(v == "0" || v.eq_ignore_ascii_case("false"))
}

fn should_skip_access_log(path: &str) -> bool {
    !path.starts_with("/api")
}

async fn run_migrations(db: &crate::db::DbPool) -> anyhow::Result<()> {
    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();
    MIGRATOR.run(db).await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        match signal::ctrl_c().await {
            Ok(()) => {}
            Err(e) => {
                tracing::error!(error = %e, "安装 Ctrl+C handler 失败，将忽略 Ctrl+C 信号");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::error!(error = %e, "安装 SIGTERM handler 失败，将忽略 SIGTERM 信号");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    warn!("Signal received, starting graceful shutdown");
}
