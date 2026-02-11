use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{AppendHeaders, IntoResponse};
use axum::Json;
use chrono::{DateTime, Duration, Utc};
use garde::Validate;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::auth::Claims;
use crate::api::auth::CurrentUser;
use crate::error::AppError;
use crate::http::router::AppState;

const ACCESS_TOKEN_EXPIRES_IN_SECS: u64 = 15 * 60;
const REFRESH_TOKEN_EXPIRES_IN_SECS: i64 = 30 * 24 * 60 * 60;
const REFRESH_TOKEN_COOKIE_NAME: &str = "refresh_token";
const REFRESH_TOKEN_COOKIE_PATH: &str = "/api/v1/sessions";

type SessionIssueResponse = (
    AppendHeaders<[(header::HeaderName, HeaderValue); 1]>,
    Json<CreateSessionResponse>,
);

struct SessionIssueContext {
    jwt_secret: String,
    user_id: Uuid,
    auth_version: i32,
    session_id: Uuid,
    username: Option<String>,
    display_name: String,
    email: String,
    refresh_secret: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateSessionRequest {
    #[schema(min_length = 1, max_length = 320)]
    #[serde(deserialize_with = "crate::api::serde_helpers::deserialize_trimmed_string")]
    #[garde(length(min = 1, max = 320))]
    pub identifier: String,

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
) -> Result<impl IntoResponse, AppError> {
    let cfg = state.config.load_full();

    let user = load_login_user(&state, &payload.identifier).await?;
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

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::InternalError(format!("开启登录事务失败: {e}")))?;

    let session_id = Uuid::new_v4();
    let refresh_secret = generate_random_hex(32)?;
    let refresh_secret_hash = crate::password::hash_password_argon2id(&refresh_secret)
        .map_err(|e| AppError::InternalError(format!("refresh token 哈希失败: {e}")))?;
    let refresh_expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_EXPIRES_IN_SECS);

    sqlx::query!(
        r#"
INSERT INTO auth_sessions (
    id,
    user_id,
    refresh_secret_hash,
    expires_at,
    revoked_at,
    revoked_reason
)
VALUES ($1, $2, $3, $4, NULL, NULL)
        "#,
        session_id,
        user.id,
        refresh_secret_hash,
        refresh_expires_at,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalError(format!("创建会话失败: {e}")))?;

    tx.commit()
        .await
        .map_err(|e| AppError::InternalError(format!("提交登录事务失败: {e}")))?;

    let response = build_login_or_refresh_response(SessionIssueContext {
        jwt_secret: cfg.security.jwt_secret.clone(),
        user_id: user.id,
        auth_version: user.auth_version,
        session_id,
        username: user.username,
        display_name: user.display_name,
        email: user.email,
        refresh_secret,
    })?;

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/v1/sessions/refresh",
    tag = "sessions",
    responses(
        (status = 200, description = "刷新成功", body = CreateSessionResponse),
        (status = 401, description = "refresh token 无效或已过期", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    )
)]
pub async fn refresh_session_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let cfg = state.config.load_full();
    let refresh_token = extract_refresh_token_from_headers(&headers)?;
    let (session_id, refresh_secret) = parse_refresh_token(&refresh_token)?;

    let session = load_auth_session(&state, session_id).await?;
    let now = Utc::now();
    if session.revoked_at.is_some() || session.expires_at <= now || !session.user_is_active {
        return Err(AppError::auth_token("Token 无效或已过期"));
    }

    let ok = crate::password::verify_password(&refresh_secret, &session.refresh_secret_hash)
        .map_err(|e| AppError::InternalError(format!("refresh token 校验失败: {e}")))?;
    if !ok {
        return Err(AppError::auth_token("Token 无效或已过期"));
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::InternalError(format!("开启刷新事务失败: {e}")))?;

    let next_refresh_secret = generate_random_hex(32)?;
    let next_refresh_secret_hash = crate::password::hash_password_argon2id(&next_refresh_secret)
        .map_err(|e| AppError::InternalError(format!("refresh token 哈希失败: {e}")))?;
    let next_refresh_expires_at = now + Duration::seconds(REFRESH_TOKEN_EXPIRES_IN_SECS);

    let updated = sqlx::query!(
        r#"
UPDATE auth_sessions
SET refresh_secret_hash = $2,
    expires_at = $3,
    updated_at = NOW()
WHERE id = $1
  AND revoked_at IS NULL
        "#,
        session_id,
        next_refresh_secret_hash,
        next_refresh_expires_at,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalError(format!("轮换会话失败: {e}")))?;
    if updated.rows_affected() == 0 {
        return Err(AppError::auth_token("Token 无效或已过期"));
    }

    tx.commit()
        .await
        .map_err(|e| AppError::InternalError(format!("提交刷新事务失败: {e}")))?;

    let response = build_login_or_refresh_response(SessionIssueContext {
        jwt_secret: cfg.security.jwt_secret.clone(),
        user_id: session.user_id,
        auth_version: session.auth_version,
        session_id,
        username: session.username,
        display_name: session.display_name,
        email: session.email,
        refresh_secret: next_refresh_secret,
    })?;
    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/api/v1/sessions/current",
    tag = "sessions",
    responses(
        (status = 204, description = "当前会话已退出（无 body）"),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_current_session_handler(
    axum::extract::Extension(current_user): axum::extract::Extension<CurrentUser>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        r#"
UPDATE auth_sessions
SET revoked_at = NOW(),
    revoked_reason = COALESCE(revoked_reason, 'manual_logout'),
    updated_at = NOW()
WHERE id = $1
  AND user_id = $2
  AND revoked_at IS NULL
        "#,
        current_user.session_id,
        current_user.user_id,
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::InternalError(format!("退出当前会话失败: {e}")))?;

    let clear_cookie = build_clear_refresh_cookie_value();
    let headers = build_set_cookie_headers(clear_cookie)?;
    Ok((StatusCode::NO_CONTENT, headers))
}

fn build_login_or_refresh_response(
    ctx: SessionIssueContext,
) -> Result<SessionIssueResponse, AppError> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(ACCESS_TOKEN_EXPIRES_IN_SECS as i64)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        exp,
        iat,
        sub: ctx.user_id.to_string(),
        sid: ctx.session_id.to_string(),
        ver: ctx.auth_version,
        username: ctx.username.clone(),
        display_name: Some(ctx.display_name),
        email: Some(ctx.email),
        role: if ctx.username.as_deref() == Some("admin") {
            "admin".to_string()
        } else {
            "user".to_string()
        },
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(ctx.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("JWT 生成失败: {e}")))?;

    let refresh_token = build_refresh_token(ctx.session_id, &ctx.refresh_secret);
    let set_cookie = build_refresh_set_cookie_value(&refresh_token);
    let headers = build_set_cookie_headers(set_cookie)?;

    Ok((
        headers,
        Json(CreateSessionResponse {
            token,
            expires_in: ACCESS_TOKEN_EXPIRES_IN_SECS,
        }),
    ))
}

fn build_set_cookie_headers(
    set_cookie_value: String,
) -> Result<AppendHeaders<[(header::HeaderName, HeaderValue); 1]>, AppError> {
    let value = HeaderValue::from_str(&set_cookie_value)
        .map_err(|e| AppError::InternalError(format!("构造 Set-Cookie 失败: {e}")))?;
    Ok(AppendHeaders([(header::SET_COOKIE, value)]))
}

fn build_refresh_set_cookie_value(refresh_token: &str) -> String {
    let mut segments = vec![
        format!("{REFRESH_TOKEN_COOKIE_NAME}={refresh_token}"),
        format!("Path={REFRESH_TOKEN_COOKIE_PATH}"),
        "HttpOnly".to_string(),
        "SameSite=Lax".to_string(),
        format!("Max-Age={REFRESH_TOKEN_EXPIRES_IN_SECS}"),
    ];
    if !cfg!(debug_assertions) {
        segments.push("Secure".to_string());
    }
    segments.join("; ")
}

fn build_clear_refresh_cookie_value() -> String {
    let mut segments = vec![
        format!("{REFRESH_TOKEN_COOKIE_NAME}="),
        format!("Path={REFRESH_TOKEN_COOKIE_PATH}"),
        "HttpOnly".to_string(),
        "SameSite=Lax".to_string(),
        "Max-Age=0".to_string(),
    ];
    if !cfg!(debug_assertions) {
        segments.push("Secure".to_string());
    }
    segments.join("; ")
}

fn generate_random_hex(byte_len: usize) -> Result<String, AppError> {
    use argon2::password_hash::rand_core::{OsRng, RngCore};

    let mut bytes = vec![0u8; byte_len];
    OsRng.fill_bytes(&mut bytes);
    Ok(crate::config::seed::hex_encode(&bytes))
}

fn build_refresh_token(session_id: Uuid, refresh_secret: &str) -> String {
    format!("{session_id}.{refresh_secret}")
}

fn parse_refresh_token(token: &str) -> Result<(Uuid, String), AppError> {
    let token = token.trim();
    let (session_id, secret) = token
        .split_once('.')
        .ok_or_else(|| AppError::auth_token("Token 无效或已过期"))?;
    let session_id =
        Uuid::parse_str(session_id).map_err(|_| AppError::auth_token("Token 无效或已过期"))?;
    let secret = secret.trim();
    if secret.is_empty() {
        return Err(AppError::auth_token("Token 无效或已过期"));
    }
    Ok((session_id, secret.to_string()))
}

fn extract_refresh_token_from_headers(headers: &HeaderMap) -> Result<String, AppError> {
    let cookie_header = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::auth_token("缺少 refresh token"))?;

    extract_cookie_value(cookie_header, REFRESH_TOKEN_COOKIE_NAME)
        .ok_or_else(|| AppError::auth_token("缺少 refresh token"))
}

fn extract_cookie_value(cookie_header: &str, name: &str) -> Option<String> {
    for segment in cookie_header.split(';') {
        let Some((key, value)) = segment.trim().split_once('=') else {
            continue;
        };
        if key.trim() == name {
            let v = value.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

#[derive(Debug, FromRow)]
struct LoginUserRow {
    id: Uuid,
    username: Option<String>,
    display_name: String,
    email: String,
    password_hash: Option<String>,
    is_active: bool,
    auth_version: i32,
}

async fn load_login_user(state: &AppState, identifier: &str) -> Result<LoginUserRow, AppError> {
    let mut users = sqlx::query_as!(
        LoginUserRow,
        r#"
SELECT id, username, display_name, email, password_hash, is_active, auth_version
FROM users
WHERE deleted_at IS NULL
  AND (
    username = $1
    OR email = $1
    OR phone = $1
  )
LIMIT 2
        "#,
        identifier,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(format!("查询登录用户失败: {e}")))?;

    if users.len() != 1 {
        return Err(AppError::auth_credential("用户名或密码错误"));
    }

    Ok(users.remove(0))
}

#[derive(Debug, FromRow)]
struct AuthSessionRow {
    user_id: Uuid,
    username: Option<String>,
    display_name: String,
    email: String,
    user_is_active: bool,
    auth_version: i32,
    refresh_secret_hash: String,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

async fn load_auth_session(state: &AppState, session_id: Uuid) -> Result<AuthSessionRow, AppError> {
    let row = sqlx::query_as!(
        AuthSessionRow,
        r#"
SELECT
    s.user_id,
    u.username,
    u.display_name,
    u.email,
    u.is_active AS user_is_active,
    u.auth_version,
    s.refresh_secret_hash,
    s.expires_at,
    s.revoked_at
FROM auth_sessions s
INNER JOIN users u ON u.id = s.user_id
WHERE s.id = $1
LIMIT 1
        "#,
        session_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::InternalError(format!("查询会话失败: {e}")))?;

    row.ok_or_else(|| AppError::auth_token("Token 无效或已过期"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_extract_refresh_token_from_cookie_header() {
        let cookie_header = "foo=1; refresh_token=abc.def; bar=2";
        let actual = extract_cookie_value(cookie_header, "refresh_token");
        assert_eq!(actual.as_deref(), Some("abc.def"));
    }

    #[test]
    fn should_parse_refresh_token_payload() {
        let session_id = Uuid::new_v4();
        let token = format!("{session_id}.deadbeef");
        let parsed = parse_refresh_token(&token).expect("应成功解析 refresh token");

        assert_eq!(parsed.0, session_id);
        assert_eq!(parsed.1, "deadbeef");
    }
}
