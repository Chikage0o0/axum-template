use serde::Serialize;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi, ToSchema};

use crate::api::handlers::{security as security_handlers, sessions, settings};

/// 失败时的统一错误体（与 `AppError` 的序列化保持一致）。
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponseBody {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub request_id: String,
}

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "PROJECT_NAME API",
        version = "0.1.0",
        description = "PROJECT_NAME 对外 REST API（以 OpenAPI 作为规范中心）。"
    ),
    tags(
        (name = "sessions", description = "认证与会话"),
        (name = "settings", description = "运行期配置"),
        (name = "security", description = "安全与密钥轮换")
    ),
    modifiers(&SecurityAddon),
    paths(
        sessions::create_session_handler,
        settings::get_settings_handler,
        settings::patch_settings_handler,
        security_handlers::patch_admin_password_handler
    ),
    components(schemas(
        ErrorResponseBody,
        sessions::CreateSessionRequest,
        sessions::CreateSessionResponse,
        settings::SettingsResponse,
        settings::AppSettings,
        settings::IntegrationsSettings,
        settings::PatchSettingsRequest,
        settings::PatchAppSettings,
        settings::PatchIntegrationsSettings,
        security_handlers::PatchAdminPasswordRequest
    ))
)]
pub struct ApiDoc;
