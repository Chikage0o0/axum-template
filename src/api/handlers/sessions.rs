use axum::extract::State;
use axum::Json;
use chrono::{Duration, Utc};
use garde::Validate;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::auth::Claims;
use crate::api::routes::AppState;
use crate::error::AppError;

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateSessionRequest {
    #[schema(min_length = 1, max_length = 64)]
    #[serde(deserialize_with = "crate::api::serde_helpers::deserialize_trimmed_string")]
    #[garde(length(min = 1, max = 64))]
    pub username: String,

    #[schema(format = "password", min_length = 1, max_length = 256)]
    #[serde(deserialize_with = "crate::api::serde_helpers::deserialize_trimmed_string")]
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
        (status = 401, description = "未授权（用户名或密码错误）", body = crate::api::openapi::ErrorResponseBody),
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

    let user = load_login_user(&state, &payload.username).await?;
    if !user.is_active {
        return Err(AppError::auth_credential("用户名或密码错误"));
    }
    let Some(password_hash) = user.password_hash.as_deref() else {
        return Err(AppError::auth_credential("用户名或密码错误"));
    };
    if password_hash.trim().is_empty() {
        return Err(AppError::auth_credential("用户名或密码错误"));
    }

    let ok = crate::password::verify_password(&payload.password, password_hash)
        .map_err(|e| AppError::InternalError(format!("用户密码校验失败: {e}")))?;
    if !ok {
        return Err(AppError::auth_credential("用户名或密码错误"));
    }

    let now = Utc::now();
    let expires_in = 24 * 60 * 60u64;
    let exp = (now + Duration::seconds(expires_in as i64)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        exp,
        iat,
        sub: user.id.to_string(),
        username: Some(payload.username.clone()),
        display_name: Some(user.display_name.clone()),
        email: Some(user.email.clone()),
        role: if payload.username == "admin" {
            "admin".to_string()
        } else {
            "user".to_string()
        },
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.security.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("JWT 生成失败: {e}")))?;

    Ok(Json(CreateSessionResponse { token, expires_in }))
}

#[derive(Debug, FromRow)]
struct LoginUserRow {
    id: Uuid,
    display_name: String,
    email: String,
    password_hash: Option<String>,
    is_active: bool,
}

async fn load_login_user(state: &AppState, username: &str) -> Result<LoginUserRow, AppError> {
    let user = sqlx::query_as::<_, LoginUserRow>(
        r#"
SELECT id, display_name, email, password_hash, is_active
FROM users
WHERE username = $1
LIMIT 1
        "#,
    )
    .bind(username)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::InternalError(format!("查询登录用户失败: {e}")))?;

    user.ok_or_else(|| AppError::auth_credential("用户名或密码错误"))
}
