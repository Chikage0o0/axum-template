use axum::extract::State;
use axum::http::StatusCode;
use garde::Validate;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

use argon2::password_hash::rand_core::{OsRng, RngCore};

use crate::api::routes::AppState;
use crate::error::AppError;
use crate::services::system_config;

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct PatchAdminPasswordRequest {
    #[schema(format = "password", min_length = 1, max_length = 256)]
    #[garde(length(min = 1, max = 256))]
    pub current_password: String,

    #[schema(format = "password", min_length = 8, max_length = 256)]
    #[garde(custom(crate::api::garde_helpers::string_trim_min_len_8))]
    #[garde(length(max = 256))]
    pub new_password: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/security/admin-password",
    tag = "security",
    request_body = PatchAdminPasswordRequest,
    responses(
        (status = 204, description = "修改成功（无 body）"),
        (status = 400, description = "请求参数错误", body = crate::api::openapi::ErrorResponseBody),
        (status = 401, description = "未登录或 Token 无效 / 当前密码错误", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn patch_admin_password_handler(
    State(state): State<AppState>,
    crate::api::validation::ValidatedJson(payload): crate::api::validation::ValidatedJson<
        PatchAdminPasswordRequest,
    >,
) -> Result<StatusCode, AppError> {
    let cfg = state.config.load_full();

    let ok = crate::password::verify_password(
        &payload.current_password,
        &cfg.security.admin_password_hash,
    )
    .map_err(|e| AppError::InternalError(format!("管理员密码校验失败: {e}")))?;
    if !ok {
        return Err(AppError::AuthError("密码错误".to_string()));
    }

    let new_password = payload.new_password.trim();
    let hash = crate::password::hash_password_argon2id(new_password)
        .map_err(|e| AppError::validation(format!("新密码不合法: {e}")))?;

    let new_jwt_secret = generate_jwt_secret_hex()?;

    system_config::upsert_many(
        &state.db,
        vec![
            (
                "security.admin_password_hash".to_string(),
                serde_json::Value::String(hash.clone()),
            ),
            (
                "security.jwt_secret".to_string(),
                serde_json::Value::String(new_jwt_secret.clone()),
            ),
        ],
    )
    .await?;

    // 只更新内存中的 security 字段，确保密钥轮换立即生效。
    let mut next_cfg = cfg.as_ref().clone();
    next_cfg.security.admin_password_hash = hash;
    next_cfg.security.jwt_secret = new_jwt_secret;
    state.config.store(Arc::new(next_cfg));

    Ok(StatusCode::NO_CONTENT)
}

fn generate_jwt_secret_hex() -> Result<String, AppError> {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    Ok(crate::config::seed::hex_encode(&bytes))
}
