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
    use serde_json::{json, Value};
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

    async fn setup_user_management_test_app() -> Option<(crate::db::DbPool, Router)> {
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

        let runtime = crate::config::runtime::RuntimeConfig::load_from_db(&pool)
            .await
            .expect("加载运行时配置失败");
        let state = AppState {
            config: Arc::new(ArcSwap::from_pointee(runtime)),
            db: pool.clone(),
        };

        Some((pool, app_router(state)))
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
    async fn delete_user_should_soft_delete_and_hide_from_default_list() {
        let Some((pool, app)) = setup_user_management_test_app().await else {
            return;
        };

        let admin_password = "AdminPassword#A123";
        let admin_id = create_or_update_user_with_password(
            &pool,
            "admin",
            "admin@local.invalid",
            admin_password,
        )
        .await;

        let victim_username = format!("soft_delete_target_{}", Uuid::new_v4().simple());
        let victim_email = format!("{victim_username}@example.invalid");
        let victim_id = create_or_update_user_with_password(
            &pool,
            &victim_username,
            &victim_email,
            "TargetPassword#A123",
        )
        .await;

        let (admin_token, _) = login_and_get_tokens(&app, "admin", admin_password).await;

        let delete_response = request_json(
            &app,
            Method::DELETE,
            &format!("/api/v1/users/{victim_id}"),
            Some(&admin_token),
            None,
            None,
        )
        .await;
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        let list_response = request_json(
            &app,
            Method::GET,
            "/api/v1/users",
            Some(&admin_token),
            None,
            None,
        )
        .await;
        assert_eq!(list_response.status(), StatusCode::OK);
        let users = response_json(list_response).await;
        let users = users.as_array().expect("用户列表响应体应为 JSON 数组");
        let victim_visible = users.iter().any(|user| {
            user.get("id").and_then(Value::as_str) == Some(victim_id.to_string().as_str())
        });
        assert!(!victim_visible, "默认列表不应包含已逻辑删除用户");

        cleanup_test_users(&pool, &[admin_id, victim_id]).await;
    }

    #[tokio::test]
    async fn deleted_user_email_should_be_reusable() {
        let Some((pool, app)) = setup_user_management_test_app().await else {
            return;
        };

        let admin_password = "AdminPassword#A123";
        let admin_id = create_or_update_user_with_password(
            &pool,
            "admin",
            "admin@local.invalid",
            admin_password,
        )
        .await;

        let reusable_email = format!("reusable_{}@example.invalid", Uuid::new_v4().simple());
        let user_a_username = format!("reuse_user_a_{}", Uuid::new_v4().simple());
        let user_a_id = create_or_update_user_with_password(
            &pool,
            &user_a_username,
            &reusable_email,
            "UserAPassword#A123",
        )
        .await;

        let (admin_token, _) = login_and_get_tokens(&app, "admin", admin_password).await;

        let delete_response = request_json(
            &app,
            Method::DELETE,
            &format!("/api/v1/users/{user_a_id}"),
            Some(&admin_token),
            None,
            None,
        )
        .await;
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        let user_b_username = format!("reuse_user_b_{}", Uuid::new_v4().simple());
        let create_response = request_json(
            &app,
            Method::POST,
            "/api/v1/users",
            Some(&admin_token),
            None,
            Some(json!({
                "username": user_b_username,
                "display_name": "Reusable Email User",
                "email": reusable_email,
            })),
        )
        .await;
        assert_eq!(create_response.status(), StatusCode::CREATED);
        let created = response_json(create_response).await;
        let user_b_id = created
            .get("id")
            .and_then(Value::as_str)
            .and_then(|id| Uuid::parse_str(id).ok())
            .expect("创建用户响应应包含合法 id");

        cleanup_test_users(&pool, &[admin_id, user_a_id, user_b_id]).await;
    }

    #[tokio::test]
    async fn non_admin_should_forbidden_on_user_management_routes() {
        let Some((pool, app)) = setup_user_management_test_app().await else {
            return;
        };

        let username = format!("normal_user_{}", Uuid::new_v4().simple());
        let email = format!("{username}@example.invalid");
        let password = "NormalUserPassword#A123";
        let normal_user_id =
            create_or_update_user_with_password(&pool, &username, &email, password).await;

        let target_username = format!("target_user_{}", Uuid::new_v4().simple());
        let target_email = format!("{target_username}@example.invalid");
        let target_user_id = create_or_update_user_with_password(
            &pool,
            &target_username,
            &target_email,
            "TargetPassword#A123",
        )
        .await;

        let (token, _) = login_and_get_tokens(&app, &username, password).await;

        let routes = vec![
            (Method::GET, "/api/v1/users".to_string(), None),
            (
                Method::POST,
                "/api/v1/users".to_string(),
                Some(json!({
                    "username": format!("created_by_normal_{}", Uuid::new_v4().simple()),
                    "display_name": "should fail",
                    "email": format!("create_by_normal_{}@example.invalid", Uuid::new_v4().simple()),
                })),
            ),
            (
                Method::PATCH,
                format!("/api/v1/users/{target_user_id}"),
                Some(json!({
                    "display_name": "updated by normal",
                })),
            ),
            (
                Method::DELETE,
                format!("/api/v1/users/{target_user_id}"),
                None,
            ),
            (
                Method::POST,
                format!("/api/v1/users/{target_user_id}/restore"),
                None,
            ),
        ];

        for (method, uri, body) in routes {
            let response = request_json(&app, method, &uri, Some(&token), None, body).await;
            assert_eq!(
                response.status(),
                StatusCode::FORBIDDEN,
                "非管理员访问 {uri} 应返回 403"
            );
            let body = response_json(response).await;
            assert_eq!(body.get("code").and_then(Value::as_u64), Some(2002));
        }

        cleanup_test_users(&pool, &[normal_user_id, target_user_id]).await;
    }

    #[tokio::test]
    async fn restore_user_should_reactivate_soft_deleted_user() {
        let Some((pool, app)) = setup_user_management_test_app().await else {
            return;
        };

        let admin_password = "AdminPassword#A123";
        let admin_id = create_or_update_user_with_password(
            &pool,
            "admin",
            "admin@local.invalid",
            admin_password,
        )
        .await;

        let username = format!("restore_target_{}", Uuid::new_v4().simple());
        let email = format!("{username}@example.invalid");
        let user_id =
            create_or_update_user_with_password(&pool, &username, &email, "TargetPassword#A123")
                .await;

        let (admin_token, _) = login_and_get_tokens(&app, "admin", admin_password).await;

        let delete_response = request_json(
            &app,
            Method::DELETE,
            &format!("/api/v1/users/{user_id}"),
            Some(&admin_token),
            None,
            None,
        )
        .await;
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        let restore_response = request_json(
            &app,
            Method::POST,
            &format!("/api/v1/users/{user_id}/restore"),
            Some(&admin_token),
            None,
            None,
        )
        .await;
        assert_eq!(restore_response.status(), StatusCode::OK);
        let restored = response_json(restore_response).await;
        assert_eq!(
            restored.get("id").and_then(Value::as_str),
            Some(user_id.to_string().as_str())
        );

        let list_response = request_json(
            &app,
            Method::GET,
            "/api/v1/users?include_deleted=true",
            Some(&admin_token),
            None,
            None,
        )
        .await;
        assert_eq!(list_response.status(), StatusCode::OK);
        let users = response_json(list_response).await;
        let users = users.as_array().expect("用户列表响应体应为 JSON 数组");
        let restored_user = users
            .iter()
            .find(|user| {
                user.get("id").and_then(Value::as_str) == Some(user_id.to_string().as_str())
            })
            .expect("include_deleted 列表应包含恢复后的用户");
        assert_eq!(
            restored_user.get("is_active").and_then(Value::as_bool),
            Some(true)
        );

        cleanup_test_users(&pool, &[admin_id, user_id]).await;
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
