use serde::Serialize;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi, ToSchema};

use crate::modules::security::handlers as security_handlers;
use crate::modules::sessions::handlers as sessions;
use crate::modules::settings::handlers as settings;
use crate::modules::users::handlers as users;

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
        (name = "security", description = "安全与凭证管理"),
        (name = "users", description = "用户管理")
    ),
    modifiers(&SecurityAddon),
    paths(
        sessions::create_session_handler,
        sessions::refresh_session_handler,
        sessions::delete_current_session_handler,
        settings::get_settings_handler,
        settings::patch_settings_handler,
        security_handlers::patch_current_user_password_handler,
        users::get_current_user_handler,
        users::patch_current_user_handler,
        users::get_users_handler,
        users::create_user_handler,
        users::patch_user_handler,
        users::delete_user_handler,
        users::restore_user_handler
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
        security_handlers::PatchCurrentUserPasswordRequest,
        users::UserResponse,
        users::CreateUserRequest,
        users::PatchCurrentUserRequest,
        users::PatchUserRequest
    ))
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use utoipa::OpenApi;

    use super::ApiDoc;

    #[test]
    fn should_expose_user_management_paths() {
        let doc = ApiDoc::openapi();

        assert!(doc.paths.paths.contains_key("/api/v1/users"));
        assert!(doc.paths.paths.contains_key("/api/v1/users/me"));
        assert!(doc.paths.paths.contains_key("/api/v1/users/{user_id}"));
        assert!(doc
            .paths
            .paths
            .contains_key("/api/v1/users/{user_id}/restore"));
        assert!(!doc
            .paths
            .paths
            .contains_key("/api/v1/users/{user_id}/identities"));
    }

    #[test]
    fn should_expose_session_identifier_schema() {
        let doc = ApiDoc::openapi();
        let schemas = doc.components.expect("openapi components 应存在").schemas;
        let session_schema = schemas
            .get("CreateSessionRequest")
            .expect("CreateSessionRequest schema 应存在")
            .clone();
        let session_obj = match session_schema {
            utoipa::openapi::RefOr::T(utoipa::openapi::schema::Schema::Object(obj)) => obj,
            _ => panic!("CreateSessionRequest schema 类型应为 object"),
        };

        assert!(session_obj.properties.contains_key("identifier"));
        assert!(session_obj.properties.contains_key("password"));
    }

    #[test]
    fn should_expose_current_user_password_path() {
        let doc = ApiDoc::openapi();

        assert!(doc.paths.paths.contains_key("/api/v1/security/password"));
        assert!(!doc
            .paths
            .paths
            .contains_key("/api/v1/security/admin-password"));
    }

    #[test]
    fn should_expose_session_refresh_and_current_paths() {
        let doc = ApiDoc::openapi();

        assert!(doc.paths.paths.contains_key("/api/v1/sessions/refresh"));
        assert!(doc.paths.paths.contains_key("/api/v1/sessions/current"));
    }

    #[test]
    fn should_not_expose_locale_timezone_in_user_schemas() {
        let doc = ApiDoc::openapi();
        let schemas = doc.components.expect("openapi components 应存在").schemas;

        for schema_name in ["CreateUserRequest", "PatchUserRequest", "UserResponse"] {
            let schema = schemas
                .get(schema_name)
                .unwrap_or_else(|| panic!("{schema_name} schema 应存在"))
                .clone();
            let obj = match schema {
                utoipa::openapi::RefOr::T(utoipa::openapi::schema::Schema::Object(obj)) => obj,
                _ => panic!("{schema_name} schema 类型应为 object"),
            };

            assert!(
                !obj.properties.contains_key("locale"),
                "{schema_name} 不应包含 locale"
            );
            assert!(
                !obj.properties.contains_key("timezone"),
                "{schema_name} 不应包含 timezone"
            );
        }
    }
}
