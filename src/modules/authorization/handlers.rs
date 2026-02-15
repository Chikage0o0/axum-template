use axum::{extract::Extension, extract::State, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::api::auth::{authorize_scoped, CurrentUser};
use crate::api::request_id::current_request_id;
use crate::error::AppError;
use crate::http::router::AppState;
use crate::modules::authorization::scope::ensure_scope_all_only;

use super::permission::{permission_catalog_version_for_codes, PermissionNode};

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionNodeItemResponse {
    pub code: PermissionNode,
    pub name: String,
    pub description: String,
    pub module: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionNodeDictionaryResponse {
    pub version: String,
    pub items: Vec<PermissionNodeItemResponse>,
}

#[utoipa::path(
    get,
    path = "/api/v1/authorization/permission-nodes",
    tag = "authorization",
    responses(
        (status = 200, description = "获取权限节点字典", body = PermissionNodeDictionaryResponse),
        (status = 401, description = "未登录或 Token 无效", body = crate::api::openapi::ErrorResponseBody),
        (status = 403, description = "没有读取权限节点字典的权限", body = crate::api::openapi::ErrorResponseBody),
        (status = 500, description = "服务器内部错误", body = crate::api::openapi::ErrorResponseBody)
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_permission_nodes_handler(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<AppState>,
) -> Result<Json<PermissionNodeDictionaryResponse>, AppError> {
    let scope = authorize_scoped(
        &state,
        &current_user,
        PermissionNode::AuthorizationPermissionNodesView,
        None,
    )
    .await?;
    ensure_scope_all_only(scope)?;

    let request_id = current_request_id().unwrap_or_else(|| "req_unknown".to_string());
    let items = state
        .authorization_service
        .list_permission_nodes_from_db(&request_id)
        .await?;

    let version = permission_catalog_version_for_codes(items.iter().map(|item| item.code.as_str()));
    let items = items
        .into_iter()
        .map(|item| PermissionNodeItemResponse {
            code: item.code,
            name: item.name,
            description: item.description,
            module: item.module,
        })
        .collect();

    Ok(Json(PermissionNodeDictionaryResponse { version, items }))
}
