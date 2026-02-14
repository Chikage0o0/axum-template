use uuid::Uuid;

use crate::api::request_id::current_request_id;
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsersScope {
    All,
    Id(Uuid),
}

impl UsersScope {
    pub fn list_filter_user_id(self) -> Option<Uuid> {
        match self {
            UsersScope::All => None,
            UsersScope::Id(user_id) => Some(user_id),
        }
    }
}

pub fn parse_users_scope_rule(
    scope_rule: Option<&str>,
    current_user_id: Uuid,
) -> Result<UsersScope, AppError> {
    let Some(raw) = scope_rule.map(str::trim).filter(|v| !v.is_empty()) else {
        return Err(scope_config_forbidden("missing_scope_rule", scope_rule));
    };

    if raw == "ALL" {
        return Ok(UsersScope::All);
    }

    if raw == "SELF" {
        return Ok(UsersScope::Id(current_user_id));
    }

    if let Some(value) = raw.strip_prefix("ID:") {
        return Uuid::parse_str(value)
            .map(UsersScope::Id)
            .map_err(|_| scope_config_forbidden("invalid_scope_id", scope_rule));
    }

    Err(scope_config_forbidden("invalid_scope_rule", scope_rule))
}

pub fn ensure_users_write_scope(scope: UsersScope, target_user_id: Uuid) -> Result<(), AppError> {
    match scope {
        UsersScope::All => Ok(()),
        UsersScope::Id(allowed_user_id) if allowed_user_id == target_user_id => Ok(()),
        UsersScope::Id(_) => Err(AppError::PermissionDenied("权限不足".to_string())),
    }
}

fn scope_config_forbidden(reason: &str, scope_rule: Option<&str>) -> AppError {
    let request_id = current_request_id().unwrap_or_else(|| "req_unknown".to_string());
    tracing::warn!(
        %request_id,
        scope_config_error = %reason,
        scope_rule = scope_rule.unwrap_or("<none>"),
        "scope rule rejected by fail-closed"
    );
    AppError::PermissionDenied("权限不足".to_string())
}
