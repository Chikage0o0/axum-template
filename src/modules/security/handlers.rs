use axum::extract::{Extension, State};
use axum::http::StatusCode;
use garde::Validate;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::api::auth::CurrentUser;
use crate::error::AppError;
use crate::http::router::AppState;

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
    let current_hash = load_current_user_password_hash(&state, current_user.user_id).await?;

    let ok = crate::password::verify_password(&payload.current_password, &current_hash)
        .map_err(|e| AppError::InternalError(format!("当前用户密码校验失败: {e}")))?;
    if !ok {
        return Err(AppError::auth_credential("密码错误"));
    }

    let new_password = payload.new_password.trim();
    let hash = crate::password::hash_password_argon2id(new_password)
        .map_err(|e| AppError::validation(format!("新密码不合法: {e}")))?;

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::InternalError(format!("开启改密事务失败: {e}")))?;

    let updated = sqlx::query!(
        r#"
UPDATE users
SET password_hash = $2,
    auth_version = auth_version + 1,
    updated_at = NOW()
WHERE id = $1
        "#,
        current_user.user_id,
        hash,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalError(format!("更新当前用户密码失败: {e}")))?;
    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "当前用户不存在: {}",
            current_user.user_id
        )));
    }

    sqlx::query!(
        r#"
UPDATE auth_sessions
SET revoked_at = NOW(),
    revoked_reason = 'password_changed',
    updated_at = NOW()
WHERE user_id = $1
  AND revoked_at IS NULL
        "#,
        current_user.user_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalError(format!("撤销用户会话失败: {e}")))?;

    tx.commit()
        .await
        .map_err(|e| AppError::InternalError(format!("提交改密事务失败: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn load_current_user_password_hash(
    state: &AppState,
    user_id: uuid::Uuid,
) -> Result<String, AppError> {
    let hash = sqlx::query_scalar!(
        r#"
SELECT password_hash
FROM users
WHERE id = $1
LIMIT 1
        "#,
        user_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::InternalError(format!("读取当前用户密码失败: {e}")))?
    .flatten()
    .filter(|v| !v.trim().is_empty())
    .ok_or_else(|| AppError::NotFound(format!("当前用户不存在或未设置密码: {user_id}")))?;

    Ok(hash)
}
