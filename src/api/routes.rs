use arc_swap::ArcSwap;
use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::auth::auth_middleware;
use crate::api::handlers::security::patch_current_user_password_handler;
use crate::api::handlers::sessions::create_session_handler;
use crate::api::handlers::settings::{get_settings_handler, patch_settings_handler};
use crate::api::handlers::users::{
    create_user_handler, create_user_identity_handler, delete_user_identity_handler,
    get_current_user_handler, get_users_handler, patch_user_handler,
};
use crate::api::openapi::ApiDoc;
use crate::config::runtime::RuntimeConfig;
use crate::db::DbPool;
use crate::error::AppError;

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
        .route("/health", get(health_check))
        .route("/api/v1/sessions", post(create_session_handler));

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

    let mut router = Router::new().merge(public_routes).merge(protected_routes);

    if should_expose_openapi() {
        router = router
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
    }

    router.with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
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
