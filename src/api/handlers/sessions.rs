use axum::extract::State;
use axum::Json;
use chrono::{Duration, Utc};
use garde::Validate;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::auth::Claims;
use crate::api::routes::AppState;
use crate::error::AppError;

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateSessionRequest {
    #[schema(format = "password", min_length = 1, max_length = 256)]
    #[garde(length(min = 1, max = 256))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateSessionResponse {
    pub token: String,
    pub expires_in: u64,
}

#[utoipa::path(
    post,
    path = "/api/v1/sessions",
    tag = "sessions",
    request_body = CreateSessionRequest,
    responses(
        (status = 200, description = "登录成功", body = CreateSessionResponse),
        (status = 401, description = "未授权（密码错误）", body = crate::api::openapi::ErrorResponseBody),
        (status = 400, description = "请求参数错误", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    )
)]
pub async fn create_session_handler(
    State(state): State<AppState>,
    crate::api::validation::ValidatedJson(payload): crate::api::validation::ValidatedJson<
        CreateSessionRequest,
    >,
) -> Result<Json<CreateSessionResponse>, AppError> {
    let cfg = state.config.load_full();

    let ok = crate::password::verify_password(&payload.password, &cfg.security.admin_password_hash)
        .map_err(|e| AppError::InternalError(format!("管理员密码校验失败: {e}")))?;
    if !ok {
        return Err(AppError::AuthError("密码错误".to_string()));
    }

    let now = Utc::now();
    let expires_in = 24 * 60 * 60u64;
    let exp = (now + Duration::seconds(expires_in as i64)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        exp,
        iat,
        role: "admin".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.security.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("JWT 生成失败: {e}")))?;

    Ok(Json(CreateSessionResponse { token, expires_in }))
}
