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
    create_user_handler, create_user_identity_handler, delete_user_identity_handler,
    get_current_user_handler, get_users_handler, patch_user_handler,
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
        .route("/api/v1/users/{user_id}", patch(patch_user_handler))
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

    use axum::body::{to_bytes, Body};
    use axum::http::{header, Method, Request, StatusCode};
    use serde_json::Value;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;
    use uuid::Uuid;

    fn database_url_for_e2e() -> Option<String> {
        std::env::var("DATABASE_URL")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    async fn request_json(
        app: &Router,
        method: Method,
        uri: &str,
        token: Option<&str>,
        cookie: Option<&str>,
        body: Option<Value>,
    ) -> axum::response::Response {
        let mut builder = Request::builder().method(method).uri(uri);
        builder = builder.header(header::ACCEPT, "application/json");
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        if let Some(cookie) = cookie {
            builder = builder.header(header::COOKIE, cookie);
        }

        let request = if let Some(body) = body {
            builder
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .expect("构造请求失败")
        } else {
            builder.body(Body::empty()).expect("构造请求失败")
        };

        app.clone().oneshot(request).await.expect("执行请求失败")
    }

    async fn response_json(response: axum::response::Response) -> Value {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("读取响应体失败");
        serde_json::from_slice::<Value>(&bytes).expect("响应体应为 JSON")
    }

    fn parse_cookie_pair(set_cookie_value: &str) -> String {
        set_cookie_value
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_string()
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

    async fn login_and_get_tokens(
        app: &Router,
        username: &str,
        password: &str,
    ) -> (String, String) {
        let response = request_json(
            app,
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

        assert_eq!(response.status(), StatusCode::OK);

        let set_cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|v| v.to_str().ok())
            .expect("登录响应缺少 Set-Cookie")
            .to_string();
        let cookie_pair = parse_cookie_pair(&set_cookie);
        assert!(cookie_pair.starts_with("refresh_token="));

        let body = response_json(response).await;
        let token = body
            .get("token")
            .and_then(Value::as_str)
            .expect("登录响应缺少 token")
            .to_string();

        (token, cookie_pair)
    }

    #[tokio::test]
    async fn password_change_should_only_invalidate_current_user_sessions() {
        let Some(database_url) = database_url_for_e2e() else {
            eprintln!("跳过 e2e：未设置 DATABASE_URL");
            return;
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

        let jwt_secret = format!("e2e-secret-{}", Uuid::new_v4());
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

        let username_a = format!("e2e_user_a_{}", Uuid::new_v4().simple());
        let username_b = format!("e2e_user_b_{}", Uuid::new_v4().simple());
        let password_a_old = "OldPassword#A123";
        let password_a_new = "NewPassword#A123";
        let password_b = "StablePassword#B123";

        let user_a_id = create_user_with_password(&pool, &username_a, password_a_old).await;
        let user_b_id = create_user_with_password(&pool, &username_b, password_b).await;

        let runtime = crate::config::runtime::RuntimeConfig::load_from_db(&pool)
            .await
            .expect("加载运行时配置失败");
        let state = AppState {
            config: Arc::new(ArcSwap::from_pointee(runtime)),
            db: pool.clone(),
        };
        let app = app_router(state);

        let (access_a_old, refresh_a_old) =
            login_and_get_tokens(&app, &username_a, password_a_old).await;
        let (access_b_old, refresh_b_old) =
            login_and_get_tokens(&app, &username_b, password_b).await;

        let patch_response = request_json(
            &app,
            Method::PATCH,
            "/api/v1/security/password",
            Some(&access_a_old),
            None,
            Some(serde_json::json!({
                "current_password": password_a_old,
                "new_password": password_a_new,
            })),
        )
        .await;
        assert_eq!(patch_response.status(), StatusCode::NO_CONTENT);

        let me_a_old = request_json(
            &app,
            Method::GET,
            "/api/v1/users/me",
            Some(&access_a_old),
            None,
            None,
        )
        .await;
        assert_eq!(me_a_old.status(), StatusCode::UNAUTHORIZED);
        let me_a_old_body = response_json(me_a_old).await;
        assert_eq!(
            me_a_old_body.get("code").and_then(Value::as_u64),
            Some(1001)
        );

        let refresh_a_old_response = request_json(
            &app,
            Method::POST,
            "/api/v1/sessions/refresh",
            None,
            Some(&refresh_a_old),
            None,
        )
        .await;
        assert_eq!(refresh_a_old_response.status(), StatusCode::UNAUTHORIZED);
        let refresh_a_old_body = response_json(refresh_a_old_response).await;
        assert_eq!(
            refresh_a_old_body.get("code").and_then(Value::as_u64),
            Some(1001)
        );

        let me_b_old = request_json(
            &app,
            Method::GET,
            "/api/v1/users/me",
            Some(&access_b_old),
            None,
            None,
        )
        .await;
        assert_eq!(me_b_old.status(), StatusCode::OK);

        let refresh_b_old_response = request_json(
            &app,
            Method::POST,
            "/api/v1/sessions/refresh",
            None,
            Some(&refresh_b_old),
            None,
        )
        .await;
        assert_eq!(refresh_b_old_response.status(), StatusCode::OK);

        sqlx::query!(
            "DELETE FROM auth_sessions WHERE user_id = $1 OR user_id = $2",
            user_a_id,
            user_b_id
        )
        .execute(&pool)
        .await
        .expect("清理测试会话失败");
        sqlx::query!(
            "DELETE FROM users WHERE id = $1 OR id = $2",
            user_a_id,
            user_b_id
        )
        .execute(&pool)
        .await
        .expect("清理测试用户失败");
    }
}
