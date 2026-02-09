use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::http::router::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub sub: String,
    pub sid: String,
    pub ver: i32,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: Uuid,
    pub session_id: Uuid,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .unwrap_or("");

    let token = auth
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::auth_token("缺少 Authorization: Bearer token"))?;

    let cfg = state.config.load_full();
    let secret = cfg.security.jwt_secret.as_bytes();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .map_err(|_| AppError::auth_token("Token 无效或已过期"))?;

    let user_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| AppError::auth_token("Token 无效或已过期"))?;
    let session_id = Uuid::parse_str(&token_data.claims.sid)
        .map_err(|_| AppError::auth_token("Token 无效或已过期"))?;

    let auth_row = sqlx::query_as!(
        AuthCheckRow,
        r#"
SELECT
    u.auth_version,
    s.expires_at AS session_expires_at,
    s.revoked_at AS session_revoked_at
FROM users u
LEFT JOIN auth_sessions s
       ON s.id = $2
      AND s.user_id = u.id
WHERE u.id = $1
  AND u.is_active = TRUE
LIMIT 1
        "#,
        user_id,
        session_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::InternalError(format!("查询鉴权状态失败: {e}")))?
    .ok_or_else(|| AppError::auth_token("Token 无效或已过期"))?;

    if auth_row.auth_version != token_data.claims.ver {
        return Err(AppError::auth_token("Token 无效或已过期"));
    }

    if auth_row.session_revoked_at.is_some() {
        return Err(AppError::auth_token("Token 无效或已过期"));
    }
    let Some(session_expires_at) = auth_row.session_expires_at else {
        return Err(AppError::auth_token("Token 无效或已过期"));
    };
    if session_expires_at <= Utc::now() {
        return Err(AppError::auth_token("Token 无效或已过期"));
    }

    req.extensions_mut().insert(CurrentUser {
        user_id,
        session_id,
    });

    Ok(next.run(req).await)
}

#[derive(Debug, sqlx::FromRow)]
struct AuthCheckRow {
    auth_version: i32,
    session_expires_at: Option<chrono::DateTime<Utc>>,
    session_revoked_at: Option<chrono::DateTime<Utc>>,
}
