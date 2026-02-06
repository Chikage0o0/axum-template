use std::collections::{HashMap, HashSet};

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::auth::CurrentUser;
use crate::api::routes::AppState;
use crate::db::DbPool;
use crate::error::AppError;

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: Option<String>,
    pub display_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub identities: Vec<UserIdentityResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct UserIdentityResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider_kind: String,
    pub provider_name: String,
    pub provider_user_id: String,
    pub provider_username: Option<String>,
    pub provider_email: Option<String>,
    pub oidc_issuer: Option<String>,
    pub oidc_subject: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateUserRequest {
    #[schema(min_length = 1, max_length = 64)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 64))]
    pub username: Option<String>,

    #[schema(min_length = 1, max_length = 128)]
    #[serde(deserialize_with = "crate::api::serde_helpers::deserialize_trimmed_string")]
    #[garde(length(min = 1, max = 128))]
    pub display_name: String,

    #[schema(min_length = 3, max_length = 320)]
    #[serde(deserialize_with = "crate::api::serde_helpers::deserialize_trimmed_string")]
    #[garde(custom(crate::api::garde_helpers::string_basic_email))]
    #[garde(length(max = 320))]
    pub email: String,

    #[schema(min_length = 1, max_length = 32)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 32))]
    pub phone: Option<String>,

    #[schema(min_length = 1, max_length = 2048)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 2048))]
    pub avatar_url: Option<String>,

    #[serde(default)]
    #[garde(skip)]
    pub metadata: Option<serde_json::Value>,

    #[serde(default)]
    #[garde(dive)]
    pub identities: Vec<CreateUserIdentityRequest>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct PatchUserRequest {
    #[schema(min_length = 1, max_length = 64)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 64))]
    pub username: Option<String>,

    #[schema(min_length = 1, max_length = 128)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 128))]
    pub display_name: Option<String>,

    #[schema(min_length = 3, max_length = 320)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_basic_email))]
    #[garde(length(max = 320))]
    pub email: Option<String>,

    #[schema(min_length = 1, max_length = 32)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 32))]
    pub phone: Option<String>,

    #[schema(min_length = 1, max_length = 2048)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 2048))]
    pub avatar_url: Option<String>,

    #[garde(skip)]
    pub is_active: Option<bool>,

    #[garde(skip)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema, Validate, Clone)]
pub struct CreateUserIdentityRequest {
    #[schema(min_length = 1, max_length = 32, example = "oidc")]
    #[serde(deserialize_with = "crate::api::serde_helpers::deserialize_trimmed_string")]
    #[garde(length(min = 1, max = 32))]
    pub provider_kind: String,

    #[schema(min_length = 1, max_length = 64, example = "google")]
    #[serde(deserialize_with = "crate::api::serde_helpers::deserialize_trimmed_string")]
    #[garde(length(min = 1, max = 64))]
    pub provider_name: String,

    #[schema(min_length = 1, max_length = 256)]
    #[serde(deserialize_with = "crate::api::serde_helpers::deserialize_trimmed_string")]
    #[garde(length(min = 1, max = 256))]
    pub provider_user_id: String,

    #[schema(min_length = 1, max_length = 256)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 256))]
    pub provider_username: Option<String>,

    #[schema(min_length = 1, max_length = 320)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_basic_email))]
    #[garde(length(max = 320))]
    pub provider_email: Option<String>,

    #[schema(min_length = 1, max_length = 2048)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 2048))]
    pub oidc_issuer: Option<String>,

    #[schema(min_length = 1, max_length = 256)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(length(max = 256))]
    pub oidc_subject: Option<String>,

    #[serde(default)]
    #[garde(skip)]
    pub metadata: Option<serde_json::Value>,
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "users",
    responses(
        (status = 200, description = "获取当前用户", body = UserResponse),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 404, description = "当前用户不存在", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_current_user_handler(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, AppError> {
    let user = get_user_by_id(&state.db, current_user.user_id).await?;
    Ok(Json(user))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "users",
    responses(
        (status = 200, description = "获取用户列表", body = [UserResponse]),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_users_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = list_users(&state.db).await?;
    Ok(Json(users))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "创建用户", body = UserResponse),
        (status = 400, description = "请求参数错误", body = crate::api::openapi::ErrorResponseBody),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_user_handler(
    State(state): State<AppState>,
    crate::api::validation::ValidatedJson(payload): crate::api::validation::ValidatedJson<
        CreateUserRequest,
    >,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    ensure_no_duplicate_provider_bindings(&payload.identities)?;
    for identity in &payload.identities {
        validate_identity_semantics(identity)?;
    }

    let user = create_user(&state.db, payload).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/{user_id}",
    tag = "users",
    params(("user_id" = Uuid, Path, description = "用户 ID")),
    request_body = PatchUserRequest,
    responses(
        (status = 200, description = "更新用户", body = UserResponse),
        (status = 400, description = "请求参数错误", body = crate::api::openapi::ErrorResponseBody),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 404, description = "用户不存在", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn patch_user_handler(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
    crate::api::validation::ValidatedJson(payload): crate::api::validation::ValidatedJson<
        PatchUserRequest,
    >,
) -> Result<Json<UserResponse>, AppError> {
    let user = patch_user(&state.db, user_id, payload).await?;
    Ok(Json(user))
}

#[utoipa::path(
    post,
    path = "/api/v1/users/{user_id}/identities",
    tag = "users",
    params(("user_id" = Uuid, Path, description = "用户 ID")),
    request_body = CreateUserIdentityRequest,
    responses(
        (status = 201, description = "绑定外部账号", body = UserIdentityResponse),
        (status = 400, description = "请求参数错误", body = crate::api::openapi::ErrorResponseBody),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 404, description = "用户不存在", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_user_identity_handler(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
    crate::api::validation::ValidatedJson(payload): crate::api::validation::ValidatedJson<
        CreateUserIdentityRequest,
    >,
) -> Result<(StatusCode, Json<UserIdentityResponse>), AppError> {
    validate_identity_semantics(&payload)?;
    let identity = create_user_identity(&state.db, user_id, payload).await?;
    Ok((StatusCode::CREATED, Json(identity)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}/identities/{identity_id}",
    tag = "users",
    params(
        ("user_id" = Uuid, Path, description = "用户 ID"),
        ("identity_id" = Uuid, Path, description = "身份连接 ID")
    ),
    responses(
        (status = 204, description = "删除成功"),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 404, description = "身份连接不存在", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_user_identity_handler(
    Path((user_id, identity_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    delete_user_identity(&state.db, user_id, identity_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    username: Option<String>,
    display_name: String,
    email: String,
    phone: Option<String>,
    avatar_url: Option<String>,
    is_active: bool,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct UserIdentityRow {
    id: Uuid,
    user_id: Uuid,
    provider_kind: String,
    provider_name: String,
    provider_user_id: String,
    provider_username: Option<String>,
    provider_email: Option<String>,
    oidc_issuer: Option<String>,
    oidc_subject: Option<String>,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    last_login_at: Option<DateTime<Utc>>,
}

async fn list_users(db: &DbPool) -> Result<Vec<UserResponse>, AppError> {
    let users: Vec<UserRow> = sqlx::query_as(
        r#"
SELECT
    id,
    username,
    display_name,
    email,
    phone,
    avatar_url,
    is_active,
    metadata,
    created_at,
    updated_at
FROM users
ORDER BY created_at DESC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::InternalError(format!("查询用户列表失败: {e}")))?;

    if users.is_empty() {
        return Ok(Vec::new());
    }

    let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();
    let identities = list_identities_by_user_ids(db, &user_ids).await?;

    Ok(users
        .into_iter()
        .map(|row| UserResponse {
            id: row.id,
            username: row.username,
            display_name: row.display_name,
            email: row.email,
            phone: row.phone,
            avatar_url: row.avatar_url,
            is_active: row.is_active,
            metadata: row.metadata,
            identities: identities.get(&row.id).cloned().unwrap_or_default(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect())
}

async fn get_user_by_id(db: &DbPool, user_id: Uuid) -> Result<UserResponse, AppError> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
SELECT
    id,
    username,
    display_name,
    email,
    phone,
    avatar_url,
    is_active,
    metadata,
    created_at,
    updated_at
FROM users
WHERE id = $1
LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::InternalError(format!("查询用户失败: {e}")))?
    .ok_or_else(|| AppError::NotFound(format!("用户不存在: {user_id}")))?;

    let identities = list_identities_by_user_id(db, user_id).await?;

    Ok(UserResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        email: row.email,
        phone: row.phone,
        avatar_url: row.avatar_url,
        is_active: row.is_active,
        metadata: row.metadata,
        identities,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn create_user(db: &DbPool, payload: CreateUserRequest) -> Result<UserResponse, AppError> {
    let mut tx = db
        .begin()
        .await
        .map_err(|e| AppError::InternalError(format!("开启用户创建事务失败: {e}")))?;

    let row: UserRow = sqlx::query_as(
        r#"
INSERT INTO users (
    username,
    display_name,
    email,
    phone,
    avatar_url,
    metadata
)
VALUES ($1, $2, $3, $4, $5, $6)
RETURNING
    id,
    username,
    display_name,
    email,
    phone,
    avatar_url,
    is_active,
    metadata,
    created_at,
    updated_at
        "#,
    )
    .bind(payload.username)
    .bind(payload.display_name)
    .bind(payload.email)
    .bind(payload.phone)
    .bind(payload.avatar_url)
    .bind(payload.metadata.unwrap_or_else(|| serde_json::json!({})))
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| map_user_db_error("创建用户失败", e))?;

    let mut identity_responses: Vec<UserIdentityResponse> = Vec::new();
    for identity in payload.identities {
        let inserted = insert_identity_row(&mut tx, row.id, identity).await?;
        identity_responses.push(map_identity_row(inserted));
    }

    tx.commit()
        .await
        .map_err(|e| AppError::InternalError(format!("提交用户创建事务失败: {e}")))?;

    Ok(UserResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        email: row.email,
        phone: row.phone,
        avatar_url: row.avatar_url,
        is_active: row.is_active,
        metadata: row.metadata,
        identities: identity_responses,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn patch_user(
    db: &DbPool,
    user_id: Uuid,
    payload: PatchUserRequest,
) -> Result<UserResponse, AppError> {
    if payload.username.is_none()
        && payload.display_name.is_none()
        && payload.email.is_none()
        && payload.phone.is_none()
        && payload.avatar_url.is_none()
        && payload.is_active.is_none()
        && payload.metadata.is_none()
    {
        return Err(AppError::validation("至少需要提供一个可更新字段"));
    }

    let row = sqlx::query_as::<_, UserRow>(
        r#"
UPDATE users
SET
    username = COALESCE($2, username),
    display_name = COALESCE($3, display_name),
    email = COALESCE($4, email),
    phone = COALESCE($5, phone),
    avatar_url = COALESCE($6, avatar_url),
    is_active = COALESCE($7, is_active),
    metadata = COALESCE($8, metadata),
    updated_at = NOW()
WHERE id = $1
RETURNING
    id,
    username,
    display_name,
    email,
    phone,
    avatar_url,
    is_active,
    metadata,
    created_at,
    updated_at
        "#,
    )
    .bind(user_id)
    .bind(payload.username)
    .bind(payload.display_name)
    .bind(payload.email)
    .bind(payload.phone)
    .bind(payload.avatar_url)
    .bind(payload.is_active)
    .bind(payload.metadata)
    .fetch_optional(db)
    .await
    .map_err(|e| map_user_db_error("更新用户失败", e))?
    .ok_or_else(|| AppError::NotFound(format!("用户不存在: {user_id}")))?;

    let identities = list_identities_by_user_id(db, user_id).await?;

    Ok(UserResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        email: row.email,
        phone: row.phone,
        avatar_url: row.avatar_url,
        is_active: row.is_active,
        metadata: row.metadata,
        identities,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn create_user_identity(
    db: &DbPool,
    user_id: Uuid,
    payload: CreateUserIdentityRequest,
) -> Result<UserIdentityResponse, AppError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)")
        .bind(user_id)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::InternalError(format!("检查用户存在性失败: {e}")))?;
    if !exists {
        return Err(AppError::NotFound(format!("用户不存在: {user_id}")));
    }

    let row = sqlx::query_as::<_, UserIdentityRow>(
        r#"
INSERT INTO user_identities (
    user_id,
    provider_kind,
    provider_name,
    provider_user_id,
    provider_username,
    provider_email,
    oidc_issuer,
    oidc_subject,
    metadata
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
RETURNING
    id,
    user_id,
    provider_kind,
    provider_name,
    provider_user_id,
    provider_username,
    provider_email,
    oidc_issuer,
    oidc_subject,
    metadata,
    created_at,
    updated_at,
    last_login_at
        "#,
    )
    .bind(user_id)
    .bind(normalize_provider_segment(&payload.provider_kind))
    .bind(normalize_provider_segment(&payload.provider_name))
    .bind(payload.provider_user_id)
    .bind(payload.provider_username)
    .bind(payload.provider_email)
    .bind(payload.oidc_issuer)
    .bind(payload.oidc_subject)
    .bind(payload.metadata.unwrap_or_else(|| serde_json::json!({})))
    .fetch_one(db)
    .await
    .map_err(|e| map_user_db_error("绑定外部账号失败", e))?;

    Ok(map_identity_row(row))
}

async fn delete_user_identity(
    db: &DbPool,
    user_id: Uuid,
    identity_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM user_identities WHERE id = $1 AND user_id = $2")
        .bind(identity_id)
        .bind(user_id)
        .execute(db)
        .await
        .map_err(|e| AppError::InternalError(format!("删除外部账号连接失败: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "身份连接不存在: user_id={user_id}, identity_id={identity_id}"
        )));
    }

    Ok(())
}

async fn insert_identity_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    payload: CreateUserIdentityRequest,
) -> Result<UserIdentityRow, AppError> {
    sqlx::query_as::<_, UserIdentityRow>(
        r#"
INSERT INTO user_identities (
    user_id,
    provider_kind,
    provider_name,
    provider_user_id,
    provider_username,
    provider_email,
    oidc_issuer,
    oidc_subject,
    metadata
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
RETURNING
    id,
    user_id,
    provider_kind,
    provider_name,
    provider_user_id,
    provider_username,
    provider_email,
    oidc_issuer,
    oidc_subject,
    metadata,
    created_at,
    updated_at,
    last_login_at
        "#,
    )
    .bind(user_id)
    .bind(normalize_provider_segment(&payload.provider_kind))
    .bind(normalize_provider_segment(&payload.provider_name))
    .bind(payload.provider_user_id)
    .bind(payload.provider_username)
    .bind(payload.provider_email)
    .bind(payload.oidc_issuer)
    .bind(payload.oidc_subject)
    .bind(payload.metadata.unwrap_or_else(|| serde_json::json!({})))
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| map_user_db_error("创建用户身份连接失败", e))
}

async fn list_identities_by_user_id(
    db: &DbPool,
    user_id: Uuid,
) -> Result<Vec<UserIdentityResponse>, AppError> {
    let rows = sqlx::query_as::<_, UserIdentityRow>(
        r#"
SELECT
    id,
    user_id,
    provider_kind,
    provider_name,
    provider_user_id,
    provider_username,
    provider_email,
    oidc_issuer,
    oidc_subject,
    metadata,
    created_at,
    updated_at,
    last_login_at
FROM user_identities
WHERE user_id = $1
ORDER BY created_at ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::InternalError(format!("查询用户身份连接失败: {e}")))?;

    Ok(rows.into_iter().map(map_identity_row).collect())
}

async fn list_identities_by_user_ids(
    db: &DbPool,
    user_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<UserIdentityResponse>>, AppError> {
    let rows = sqlx::query_as::<_, UserIdentityRow>(
        r#"
SELECT
    id,
    user_id,
    provider_kind,
    provider_name,
    provider_user_id,
    provider_username,
    provider_email,
    oidc_issuer,
    oidc_subject,
    metadata,
    created_at,
    updated_at,
    last_login_at
FROM user_identities
WHERE user_id = ANY($1)
ORDER BY created_at ASC
        "#,
    )
    .bind(user_ids)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::InternalError(format!("批量查询用户身份连接失败: {e}")))?;

    let mut grouped: HashMap<Uuid, Vec<UserIdentityResponse>> = HashMap::new();
    for row in rows {
        grouped
            .entry(row.user_id)
            .or_default()
            .push(map_identity_row(row));
    }

    Ok(grouped)
}

fn map_identity_row(row: UserIdentityRow) -> UserIdentityResponse {
    UserIdentityResponse {
        id: row.id,
        user_id: row.user_id,
        provider_kind: row.provider_kind,
        provider_name: row.provider_name,
        provider_user_id: row.provider_user_id,
        provider_username: row.provider_username,
        provider_email: row.provider_email,
        oidc_issuer: row.oidc_issuer,
        oidc_subject: row.oidc_subject,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
        last_login_at: row.last_login_at,
    }
}

fn ensure_no_duplicate_provider_bindings(
    identities: &[CreateUserIdentityRequest],
) -> Result<(), AppError> {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for identity in identities {
        let kind = normalize_provider_segment(&identity.provider_kind);
        let name = normalize_provider_segment(&identity.provider_name);
        if !seen.insert((kind.clone(), name.clone())) {
            return Err(AppError::validation(format!(
                "同一用户不能重复绑定 provider: {kind}/{name}"
            )));
        }
    }
    Ok(())
}

fn validate_identity_semantics(identity: &CreateUserIdentityRequest) -> Result<(), AppError> {
    if identity.oidc_subject.is_some() && identity.oidc_issuer.is_none() {
        return Err(AppError::validation(
            "当提供 oidc_subject 时必须同时提供 oidc_issuer",
        ));
    }
    Ok(())
}

fn normalize_provider_segment(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

fn map_user_db_error(prefix: &str, err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        if let Some(code) = db_err.code().as_deref() {
            return match code {
                "23505" => AppError::validation(format!("{prefix}: 数据已存在或重复绑定")),
                "23514" => AppError::validation(format!("{prefix}: 字段约束校验失败")),
                _ => AppError::InternalError(format!("{prefix}: {err}")),
            };
        }
    }

    AppError::InternalError(format!("{prefix}: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_no_duplicate_provider_bindings, validate_identity_semantics,
        CreateUserIdentityRequest,
    };

    #[test]
    fn should_reject_duplicate_provider_bindings() {
        let identities = vec![
            CreateUserIdentityRequest {
                provider_kind: "oidc".to_string(),
                provider_name: "Google".to_string(),
                provider_user_id: "u_1".to_string(),
                provider_username: None,
                provider_email: None,
                oidc_issuer: Some("https://accounts.google.com".to_string()),
                oidc_subject: Some("sub_1".to_string()),
                metadata: None,
            },
            CreateUserIdentityRequest {
                provider_kind: "OIDC".to_string(),
                provider_name: "google".to_string(),
                provider_user_id: "u_2".to_string(),
                provider_username: None,
                provider_email: None,
                oidc_issuer: Some("https://accounts.google.com".to_string()),
                oidc_subject: Some("sub_2".to_string()),
                metadata: None,
            },
        ];

        let result = ensure_no_duplicate_provider_bindings(&identities);
        assert!(result.is_err());
    }

    #[test]
    fn should_reject_oidc_subject_without_issuer() {
        let identity = CreateUserIdentityRequest {
            provider_kind: "oidc".to_string(),
            provider_name: "google".to_string(),
            provider_user_id: "u_1".to_string(),
            provider_username: None,
            provider_email: None,
            oidc_issuer: None,
            oidc_subject: Some("subject".to_string()),
            metadata: None,
        };

        let result = validate_identity_semantics(&identity);
        assert!(result.is_err());
    }
}
