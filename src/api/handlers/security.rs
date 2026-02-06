use axum::extract::{Extension, State};
use axum::http::StatusCode;
use garde::Validate;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

use argon2::password_hash::rand_core::{OsRng, RngCore};

use crate::api::auth::CurrentUser;
use crate::api::routes::AppState;
use crate::error::AppError;

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct PatchCurrentUserPasswordRequest {
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
    path = "/api/v1/security/password",
    tag = "security",
    request_body = PatchCurrentUserPasswordRequest,
    responses(
        (status = 204, description = "修改当前用户密码成功（无 body）"),
        (status = 400, description = "请求参数错误", body = crate::api::openapi::ErrorResponseBody),
        (status = 401, description = "未登录或 Token 无效 / 当前密码错误", body = crate::api::openapi::ErrorResponseBody),
        (status = 404, description = "当前用户不存在", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn patch_current_user_password_handler(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<AppState>,
    crate::api::validation::ValidatedJson(payload): crate::api::validation::ValidatedJson<
        PatchCurrentUserPasswordRequest,
    >,
) -> Result<StatusCode, AppError> {
    let cfg = state.config.load_full();

    let current_hash = load_current_user_password_hash(&state, current_user.user_id).await?;

    let ok = crate::password::verify_password(&payload.current_password, &current_hash)
        .map_err(|e| AppError::InternalError(format!("当前用户密码校验失败: {e}")))?;
    if !ok {
        return Err(AppError::auth_credential("密码错误"));
    }

    let new_password = payload.new_password.trim();
    let hash = crate::password::hash_password_argon2id(new_password)
        .map_err(|e| AppError::validation(format!("新密码不合法: {e}")))?;

    let new_jwt_secret = generate_jwt_secret_hex()?;

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::InternalError(format!("开启改密事务失败: {e}")))?;

    let updated = sqlx::query(
        r#"
UPDATE users
SET password_hash = $2,
    updated_at = NOW()
WHERE id = $1
        "#,
    )
    .bind(current_user.user_id)
    .bind(hash)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalError(format!("更新当前用户密码失败: {e}")))?;
    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "当前用户不存在: {}",
            current_user.user_id
        )));
    }

    sqlx::query(
        r#"
INSERT INTO system_config (key, value)
VALUES ('security.jwt_secret', $1)
ON CONFLICT (key) DO UPDATE
SET value = EXCLUDED.value,
    updated_at = NOW()
        "#,
    )
    .bind(serde_json::Value::String(new_jwt_secret.clone()))
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalError(format!("写入 security.jwt_secret 失败: {e}")))?;

    tx.commit()
        .await
        .map_err(|e| AppError::InternalError(format!("提交改密事务失败: {e}")))?;

    // 只更新内存中的 jwt_secret，确保密钥轮换立即生效。
    let mut next_cfg = cfg.as_ref().clone();
    next_cfg.security.jwt_secret = new_jwt_secret;
    state.config.store(Arc::new(next_cfg));

    Ok(StatusCode::NO_CONTENT)
}

async fn load_current_user_password_hash(
    state: &AppState,
    user_id: uuid::Uuid,
) -> Result<String, AppError> {
    let hash = sqlx::query_scalar::<_, Option<String>>(
        r#"
SELECT password_hash
FROM users
WHERE id = $1
LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::InternalError(format!("读取当前用户密码失败: {e}")))?
    .flatten()
    .filter(|v| !v.trim().is_empty())
    .ok_or_else(|| AppError::NotFound(format!("当前用户不存在或未设置密码: {user_id}")))?;

    Ok(hash)
}

fn generate_jwt_secret_hex() -> Result<String, AppError> {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    Ok(crate::config::seed::hex_encode(&bytes))
}
