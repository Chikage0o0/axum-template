use arc_swap::ArcSwap;
use axum::{
    middleware,
    routing::{any, delete, get, patch, post},
    Router,
};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::auth::auth_middleware;
use crate::api::openapi::ApiDoc;
use crate::config::runtime::RuntimeConfig;
use crate::db::DbPool;
use crate::error::AppError;
use crate::modules::security::handlers::patch_current_user_password_handler;
use crate::modules::sessions::handlers::{
    create_session_handler, delete_current_session_handler, refresh_session_handler,
};
use crate::modules::settings::handlers::{get_settings_handler, patch_settings_handler};
use crate::modules::users::handlers::{
    create_user_handler, create_user_identity_handler, delete_user_handler,
    delete_user_identity_handler, get_current_user_handler, get_users_handler, patch_user_handler,
    restore_user_handler,
};
use crate::web_assets::{serve_frontend_index, serve_frontend_path};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ArcSwap<RuntimeConfig>>,
    pub db: DbPool,
}

impl AppState {
    pub async fn reload_runtime(&self) -> Result<(), AppError> {
        let runtime = RuntimeConfig::load_from_db(&self.db)
            .await
            .map_err(|e| AppError::InternalError(format!("从数据库加载运行期配置失败: {e}")))?;
        self.config.store(Arc::new(runtime));
        Ok(())
    }
}

pub fn app_router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/sessions", post(create_session_handler))
        .route("/api/v1/sessions/refresh", post(refresh_session_handler));

    let protected_routes = Router::new()
        .route(
            "/api/v1/settings",
            get(get_settings_handler).patch(patch_settings_handler),
        )
        .route(
            "/api/v1/security/password",
            patch(patch_current_user_password_handler),
        )
        .route(
            "/api/v1/sessions/current",
            delete(delete_current_session_handler),
        )
        .route(
            "/api/v1/users",
            get(get_users_handler).post(create_user_handler),
        )
        .route("/api/v1/users/me", get(get_current_user_handler))
        .route(
            "/api/v1/users/{user_id}",
            patch(patch_user_handler).delete(delete_user_handler),
        )
        .route(
            "/api/v1/users/{user_id}/restore",
            post(restore_user_handler),
        )
        .route(
            "/api/v1/users/{user_id}/identities",
            post(create_user_identity_handler),
        )
        .route(
            "/api/v1/users/{user_id}/identities/{identity_id}",
            axum::routing::delete(delete_user_identity_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let mut router = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .route("/api", any(api_not_found))
        .route("/api/", any(api_not_found))
        .route("/api/{*path}", any(api_not_found));

    if cfg!(embed_frontend) {
        router = router
            .route("/", get(serve_frontend_index))
            .route("/{*path}", get(serve_frontend_path));
    }

    if should_expose_openapi() {
        router = router.merge(
            SwaggerUi::new("/api/v1/swagger-ui").url("/api/v1/openapi.json", ApiDoc::openapi()),
        );
    }

    router.with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn api_not_found() -> AppError {
    AppError::NotFound("API 路径不存在".to_string())
}

fn should_expose_openapi() -> bool {
    let Ok(v) = std::env::var("PROJECT_NAME_EXPOSE_OPENAPI") else {
        return cfg!(debug_assertions);
    };
    let v = v.trim();
    if v == "0" || v.eq_ignore_ascii_case("false") {
        return false;
    }
    if v == "1" || v.eq_ignore_ascii_case("true") {
        return true;
    }
    cfg!(debug_assertions)
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::http::{header, Method, StatusCode};
    use axum_test::{TestResponse, TestServer};
    use serde_json::Value;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    fn database_url_for_e2e() -> Option<String> {
        std::env::var("DATABASE_URL")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    const E2E_JWT_SECRET: &str = "router-tests-shared-jwt-secret";

    async fn request_json(
        server: &TestServer,
        method: Method,
        uri: &str,
        token: Option<&str>,
        cookie: Option<&str>,
        body: Option<Value>,
    ) -> TestResponse {
        let mut request = server
            .method(method, uri)
            .add_header(header::ACCEPT, "application/json");
        if let Some(token) = token {
            request = request.authorization_bearer(token);
        }
        if let Some(cookie) = cookie {
            request = request.add_header(header::COOKIE, cookie);
        }

        if let Some(body) = body {
            request = request.json(&body);
        }

        request.await
    }

    async fn create_user_with_password(
        pool: &crate::db::DbPool,
        username: &str,
        password: &str,
    ) -> Uuid {
        let password_hash =
            crate::password::hash_password_argon2id(password).expect("生成测试用户密码哈希失败");

        sqlx::query_scalar!(
            r#"
INSERT INTO users (
    username,
    display_name,
    email,
    is_active,
    metadata,
    password_hash,
    auth_version
)
VALUES ($1, $2, $3, TRUE, '{}'::jsonb, $4, 0)
RETURNING id
            "#,
            username,
            format!("{username}-display"),
            format!("{username}@example.invalid"),
            password_hash,
        )
        .fetch_one(pool)
        .await
        .expect("创建测试用户失败")
    }

    async fn ensure_admin_user_with_password(pool: &crate::db::DbPool, password: &str) -> Uuid {
        let password_hash =
            crate::password::hash_password_argon2id(password).expect("生成管理员密码哈希失败");

        sqlx::query_scalar::<_, Uuid>(
            r#"
INSERT INTO users (
    username,
    display_name,
    email,
    is_active,
    metadata,
    password_hash,
    auth_version,
    deleted_at
)
VALUES ('admin', 'admin-display', 'admin@local.invalid', TRUE, '{}'::jsonb, $1, 0, NULL)
ON CONFLICT (username) WHERE deleted_at IS NULL AND username IS NOT NULL
DO UPDATE SET
    display_name = EXCLUDED.display_name,
    email = EXCLUDED.email,
    is_active = TRUE,
    password_hash = EXCLUDED.password_hash,
    auth_version = 0,
    deleted_at = NULL,
    updated_at = NOW()
RETURNING id
            "#,
        )
        .bind(password_hash)
        .fetch_one(pool)
        .await
        .expect("创建或更新管理员测试用户失败")
    }

    async fn create_or_update_user_with_password(
        pool: &crate::db::DbPool,
        username: &str,
        email: &str,
        password: &str,
    ) -> Uuid {
        let password_hash =
            crate::password::hash_password_argon2id(password).expect("生成测试用户密码哈希失败");

        sqlx::query!("DELETE FROM users WHERE username = $1", username)
            .execute(pool)
            .await
            .expect("清理同名测试用户失败");

        sqlx::query_scalar!(
            r#"
INSERT INTO users (
    username,
    display_name,
    email,
    is_active,
    metadata,
    password_hash,
    auth_version
)
VALUES ($1, $2, $3, TRUE, '{}'::jsonb, $4, 0)
RETURNING id
            "#,
            username,
            format!("{username}-display"),
            email,
            password_hash,
        )
        .fetch_one(pool)
        .await
        .expect("创建测试用户失败")
    }

    async fn cleanup_test_users(pool: &crate::db::DbPool, user_ids: &[Uuid]) {
        if user_ids.is_empty() {
            return;
        }

        sqlx::query!(
            "DELETE FROM users WHERE id = ANY($1::uuid[])",
            &user_ids[..]
        )
        .execute(pool)
        .await
        .expect("清理测试用户失败");
    }

    async fn setup_user_management_test_app() -> Option<(crate::db::DbPool, TestServer)> {
        let Some(database_url) = database_url_for_e2e() else {
            eprintln!("跳过 e2e：未设置 DATABASE_URL");
            return None;
        };

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("连接测试数据库失败");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("执行迁移失败");

        let jwt_secret = E2E_JWT_SECRET.to_string();
        sqlx::query!(
            r#"
INSERT INTO system_config (key, value, description)
VALUES ('security.jwt_secret', $1, 'e2e test secret')
ON CONFLICT (key) DO UPDATE
SET value = EXCLUDED.value,
    updated_at = NOW()
            "#,
            Value::String(jwt_secret),
        )
        .execute(&pool)
        .await
        .expect("写入测试 jwt secret 失败");

        let runtime = crate::config::runtime::RuntimeConfig::load_from_db(&pool)
            .await
            .expect("加载运行时配置失败");
        let state = AppState {
            config: Arc::new(ArcSwap::from_pointee(runtime)),
            db: pool.clone(),
        };

        let server = TestServer::new(app_router(state)).expect("创建测试服务器失败");

        Some((pool, server))
    }

    async fn login_and_get_tokens(
        server: &TestServer,
        username: &str,
        password: &str,
    ) -> (String, String) {
        let response = request_json(
            server,
            Method::POST,
            "/api/v1/sessions",
            None,
            None,
            Some(serde_json::json!({
                "username": username,
                "password": password,
            })),
        )
        .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let refresh_cookie = response.cookie("refresh_token");
        let cookie_pair = format!("refresh_token={}", refresh_cookie.value());
        assert!(cookie_pair.starts_with("refresh_token="));

        let body = response.json::<Value>();
        let token = body
            .get("token")
            .and_then(Value::as_str)
            .expect("登录响应缺少 token")
            .to_string();

        (token, cookie_pair)
    }

    mod security;
    mod sessions;
    mod settings;
    mod users;
}
