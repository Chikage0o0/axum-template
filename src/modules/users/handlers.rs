use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::auth::{authorize, CurrentUser};
use crate::db::DbPool;
use crate::error::AppError;
use crate::http::router::AppState;

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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateUserRequest {
    #[schema(min_length = 1, max_length = 64)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(custom(crate::api::garde_helpers::opt_username_format))]
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
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct PatchUserRequest {
    #[schema(min_length = 1, max_length = 64)]
    #[serde(
        default,
        deserialize_with = "crate::api::serde_helpers::deserialize_opt_trimmed_string"
    )]
    #[garde(custom(crate::api::garde_helpers::opt_string_trim_non_empty))]
    #[garde(custom(crate::api::garde_helpers::opt_username_format))]
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

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct PatchCurrentUserRequest {
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
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListUsersQuery {
    #[serde(default)]
    pub include_deleted: bool,
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
    let resource_hint = current_user.user_id.to_string();
    authorize(
        &state,
        &current_user,
        "users:me:view",
        Some(resource_hint.as_str()),
    )
    .await?;

    let user = get_user_by_id(&state.db, current_user.user_id).await?;
    Ok(Json(user))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/me",
    tag = "users",
    request_body = PatchCurrentUserRequest,
    responses(
        (status = 200, description = "更新当前用户", body = UserResponse),
        (status = 400, description = "请求参数错误", body = crate::api::openapi::ErrorResponseBody),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 404, description = "当前用户不存在", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn patch_current_user_handler(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<AppState>,
    crate::api::validation::ValidatedJson(payload): crate::api::validation::ValidatedJson<
        PatchCurrentUserRequest,
    >,
) -> Result<Json<UserResponse>, AppError> {
    let resource_hint = current_user.user_id.to_string();
    authorize(
        &state,
        &current_user,
        "users:me:update",
        Some(resource_hint.as_str()),
    )
    .await?;

    let user = patch_current_user(&state.db, current_user.user_id, payload).await?;
    Ok(Json(user))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "users",
    params(
        ("include_deleted" = Option<bool>, Query, description = "是否包含已逻辑删除用户")
    ),
    responses(
        (status = 200, description = "获取用户列表", body = [UserResponse]),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_users_handler(
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ListUsersQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    authorize(&state, &current_user, "users:list", None).await?;
    let users = list_users(&state.db, query.include_deleted).await?;
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
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<AppState>,
    crate::api::validation::ValidatedJson(payload): crate::api::validation::ValidatedJson<
        CreateUserRequest,
    >,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    authorize(&state, &current_user, "users:create", None).await?;
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
    Extension(current_user): Extension<CurrentUser>,
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
    crate::api::validation::ValidatedJson(payload): crate::api::validation::ValidatedJson<
        PatchUserRequest,
    >,
) -> Result<Json<UserResponse>, AppError> {
    let resource_hint = user_id.to_string();
    authorize(
        &state,
        &current_user,
        "users:update",
        Some(resource_hint.as_str()),
    )
    .await?;

    if current_user.user_id == user_id && payload.is_active == Some(false) {
        return Err(AppError::validation("管理员不能停用自己的账号"));
    }
    let user = patch_user(&state.db, user_id, payload).await?;
    Ok(Json(user))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}",
    tag = "users",
    params(("user_id" = Uuid, Path, description = "用户 ID")),
    responses(
        (status = 204, description = "逻辑删除用户"),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 404, description = "用户不存在", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_user_handler(
    Extension(current_user): Extension<CurrentUser>,
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let resource_hint = user_id.to_string();
    authorize(
        &state,
        &current_user,
        "users:delete",
        Some(resource_hint.as_str()),
    )
    .await?;

    if current_user.user_id == user_id {
        return Err(AppError::validation("管理员不能删除自己的账号"));
    }
    soft_delete_user(&state.db, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/users/{user_id}/restore",
    tag = "users",
    params(("user_id" = Uuid, Path, description = "用户 ID")),
    responses(
        (status = 200, description = "恢复逻辑删除用户", body = UserResponse),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 404, description = "用户不存在", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn restore_user_handler(
    Extension(current_user): Extension<CurrentUser>,
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, AppError> {
    let resource_hint = user_id.to_string();
    authorize(
        &state,
        &current_user,
        "users:restore",
        Some(resource_hint.as_str()),
    )
    .await?;

    let user = restore_user(&state.db, user_id).await?;
    Ok(Json(user))
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

async fn list_users(db: &DbPool, include_deleted: bool) -> Result<Vec<UserResponse>, AppError> {
    let users: Vec<UserRow> = sqlx::query_as!(
        UserRow,
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
WHERE ($1::bool = TRUE OR deleted_at IS NULL)
ORDER BY created_at DESC
        "#,
        include_deleted,
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::InternalError(format!("查询用户列表失败: {e}")))?;

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
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect())
}

async fn soft_delete_user(db: &DbPool, user_id: Uuid) -> Result<(), AppError> {
    let mut tx = db
        .begin()
        .await
        .map_err(|e| AppError::InternalError(format!("开启删除用户事务失败: {e}")))?;

    let result = sqlx::query!(
        r#"
UPDATE users
SET
    deleted_at = NOW(),
    is_active = FALSE,
    auth_version = auth_version + 1,
    updated_at = NOW()
WHERE id = $1
  AND deleted_at IS NULL
        "#,
        user_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalError(format!("逻辑删除用户失败: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("用户不存在或已删除: {user_id}")));
    }

    sqlx::query!(
        r#"
UPDATE auth_sessions
SET revoked_at = NOW(),
    revoked_reason = COALESCE(revoked_reason, 'user_soft_deleted'),
    updated_at = NOW()
WHERE user_id = $1
  AND revoked_at IS NULL
        "#,
        user_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalError(format!("删除用户后吊销会话失败: {e}")))?;

    tx.commit()
        .await
        .map_err(|e| AppError::InternalError(format!("提交删除用户事务失败: {e}")))?;

    Ok(())
}

async fn restore_user(db: &DbPool, user_id: Uuid) -> Result<UserResponse, AppError> {
    let mut tx = db
        .begin()
        .await
        .map_err(|e| AppError::InternalError(format!("开启恢复用户事务失败: {e}")))?;

    let row = sqlx::query_as!(
        UserRow,
        r#"
UPDATE users
SET
    deleted_at = NULL,
    is_active = TRUE,
    auth_version = auth_version + 1,
    updated_at = NOW()
WHERE id = $1
  AND deleted_at IS NOT NULL
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
        user_id,
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| map_user_db_error("恢复用户失败", e))?
    .ok_or_else(|| AppError::NotFound(format!("用户不存在或未删除: {user_id}")))?;

    sqlx::query!(
        r#"
UPDATE auth_sessions
SET revoked_at = NOW(),
    revoked_reason = COALESCE(revoked_reason, 'user_restored_reauth_required'),
    updated_at = NOW()
WHERE user_id = $1
  AND revoked_at IS NULL
        "#,
        user_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalError(format!("恢复用户后吊销旧会话失败: {e}")))?;

    tx.commit()
        .await
        .map_err(|e| AppError::InternalError(format!("提交恢复用户事务失败: {e}")))?;

    Ok(UserResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        email: row.email,
        phone: row.phone,
        avatar_url: row.avatar_url,
        is_active: row.is_active,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn get_user_by_id(db: &DbPool, user_id: Uuid) -> Result<UserResponse, AppError> {
    let row = sqlx::query_as!(
        UserRow,
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
        user_id,
    )
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::InternalError(format!("查询用户失败: {e}")))?
    .ok_or_else(|| AppError::NotFound(format!("用户不存在: {user_id}")))?;

    Ok(UserResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        email: row.email,
        phone: row.phone,
        avatar_url: row.avatar_url,
        is_active: row.is_active,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn create_user(db: &DbPool, payload: CreateUserRequest) -> Result<UserResponse, AppError> {
    if let Some(username) = payload.username.as_deref() {
        ensure_username_not_conflicts_with_other_user_contacts(db, username, None).await?;
    }

    let row: UserRow = sqlx::query_as!(
        UserRow,
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
        payload.username,
        payload.display_name,
        payload.email,
        payload.phone,
        payload.avatar_url,
        payload.metadata.unwrap_or_else(|| serde_json::json!({})),
    )
    .fetch_one(db)
    .await
    .map_err(|e| map_user_db_error("创建用户失败", e))?;

    Ok(UserResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        email: row.email,
        phone: row.phone,
        avatar_url: row.avatar_url,
        is_active: row.is_active,
        metadata: row.metadata,
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

    if let Some(username) = payload.username.as_deref() {
        ensure_username_not_conflicts_with_other_user_contacts(db, username, Some(user_id)).await?;
    }

    let row = sqlx::query_as!(
        UserRow,
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
        user_id,
        payload.username,
        payload.display_name,
        payload.email,
        payload.phone,
        payload.avatar_url,
        payload.is_active,
        payload.metadata,
    )
    .fetch_optional(db)
    .await
    .map_err(|e| map_user_db_error("更新用户失败", e))?
    .ok_or_else(|| AppError::NotFound(format!("用户不存在: {user_id}")))?;

    Ok(UserResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        email: row.email,
        phone: row.phone,
        avatar_url: row.avatar_url,
        is_active: row.is_active,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn patch_current_user(
    db: &DbPool,
    user_id: Uuid,
    payload: PatchCurrentUserRequest,
) -> Result<UserResponse, AppError> {
    if payload.display_name.is_none()
        && payload.email.is_none()
        && payload.phone.is_none()
        && payload.avatar_url.is_none()
    {
        return Err(AppError::validation("至少需要提供一个可更新字段"));
    }

    let row = sqlx::query_as!(
        UserRow,
        r#"
UPDATE users
SET
    display_name = COALESCE($2, display_name),
    email = COALESCE($3, email),
    phone = COALESCE($4, phone),
    avatar_url = COALESCE($5, avatar_url),
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
        user_id,
        payload.display_name,
        payload.email,
        payload.phone,
        payload.avatar_url,
    )
    .fetch_optional(db)
    .await
    .map_err(|e| map_user_db_error("更新当前用户失败", e))?
    .ok_or_else(|| AppError::NotFound(format!("用户不存在: {user_id}")))?;

    Ok(UserResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        email: row.email,
        phone: row.phone,
        avatar_url: row.avatar_url,
        is_active: row.is_active,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn ensure_username_not_conflicts_with_other_user_contacts(
    db: &DbPool,
    username: &str,
    exclude_user_id: Option<Uuid>,
) -> Result<(), AppError> {
    let conflict = sqlx::query!(
        r#"
SELECT EXISTS (
    SELECT 1
    FROM users
    WHERE deleted_at IS NULL
      AND ($2::uuid IS NULL OR id <> $2)
      AND (email = $1 OR phone = $1)
) AS "exists!"
        "#,
        username,
        exclude_user_id,
    )
    .fetch_one(db)
    .await
    .map_err(|e| AppError::InternalError(format!("检查用户名冲突失败: {e}")))?;

    if conflict.exists {
        return Err(AppError::validation(
            "用户名不能与其他用户的邮箱或手机号相同",
        ));
    }

    Ok(())
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
