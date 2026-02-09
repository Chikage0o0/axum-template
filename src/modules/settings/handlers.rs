use axum::extract::State;
use axum::Json;
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::AppError;
use crate::http::router::AppState;
use crate::services::system_config;

#[derive(Debug, Serialize, ToSchema)]
pub struct SettingsResponse {
    pub app: AppSettings,
    pub integrations: IntegrationsSettings,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AppSettings {
    pub check_interval_secs: u64,
    pub welcome_message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IntegrationsSettings {
    pub example_api_base: String,
    pub example_api_key_is_set: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/settings",
    tag = "settings",
    responses(
        (status = 200, description = "获取运行期配置", body = SettingsResponse),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_settings_handler(
    State(state): State<AppState>,
) -> Result<Json<SettingsResponse>, AppError> {
    let cfg = state.config.load_full();

    Ok(Json(SettingsResponse {
        app: AppSettings {
            check_interval_secs: cfg.app.check_interval_secs,
            welcome_message: cfg.app.welcome_message.clone(),
        },
        integrations: IntegrationsSettings {
            example_api_base: cfg.integrations.example_api_base.clone(),
            example_api_key_is_set: !cfg.integrations.example_api_key.trim().is_empty(),
        },
    }))
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct PatchSettingsRequest {
    #[garde(dive)]
    pub app: Option<PatchAppSettings>,
    #[garde(dive)]
    pub integrations: Option<PatchIntegrationsSettings>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct PatchAppSettings {
    #[schema(minimum = 10)]
    #[garde(custom(crate::api::garde_helpers::opt_u64_min_10))]
    pub check_interval_secs: Option<u64>,

    #[schema(min_length = 1, max_length = 256)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 256))]
    pub welcome_message: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct PatchIntegrationsSettings {
    #[schema(min_length = 1, max_length = 2048)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 2048))]
    pub example_api_base: Option<String>,

    /// 覆盖设置：留空字段表示“不修改”；若提供则必须非空。
    #[schema(format = "password", min_length = 1, max_length = 4096)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 4096))]
    pub example_api_key: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/settings",
    tag = "settings",
    request_body = PatchSettingsRequest,
    responses(
        (status = 200, description = "更新并返回运行期配置", body = SettingsResponse),
        (status = 400, description = "请求参数错误", body = crate::api::openapi::ErrorResponseBody),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn patch_settings_handler(
    State(state): State<AppState>,
    crate::api::validation::ValidatedJson(payload): crate::api::validation::ValidatedJson<
        PatchSettingsRequest,
    >,
) -> Result<Json<SettingsResponse>, AppError> {
    let mut changes: Vec<(String, serde_json::Value)> = Vec::new();

    if let Some(app) = payload.app {
        if let Some(v) = app.check_interval_secs {
            changes.push((
                "app.check_interval_secs".to_string(),
                serde_json::Value::Number(serde_json::Number::from(v)),
            ));
        }
        if let Some(v) = app.welcome_message {
            changes.push((
                "app.welcome_message".to_string(),
                serde_json::Value::String(v),
            ));
        }
    }

    if let Some(integrations) = payload.integrations {
        if let Some(v) = integrations.example_api_base {
            changes.push((
                "integrations.example_api_base".to_string(),
                serde_json::Value::String(v),
            ));
        }
        if let Some(v) = integrations.example_api_key {
            changes.push((
                "integrations.example_api_key".to_string(),
                serde_json::Value::String(v),
            ));
        }
    }

    system_config::upsert_many(&state.db, changes).await?;
    state.reload_runtime().await?;
    get_settings_handler(State(state)).await
}
