use uuid::Uuid;

use crate::api::request_id::current_request_id;
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    All,
    Id(Uuid),
}

impl Scope {
    pub fn list_filter_user_id(self) -> Option<Uuid> {
        match self {
            Scope::All => None,
            Scope::Id(user_id) => Some(user_id),
        }
    }
}

pub fn parse_scope_rule(
    scope_rule: Option<&str>,
    current_user_id: Uuid,
) -> Result<Scope, AppError> {
    let Some(raw) = scope_rule.map(str::trim).filter(|v| !v.is_empty()) else {
        return Err(scope_config_forbidden("missing_scope_rule", scope_rule));
    };

    if raw == "ALL" {
        return Ok(Scope::All);
    }

    if raw == "SELF" {
        return Ok(Scope::Id(current_user_id));
    }

    if let Some(value) = raw.strip_prefix("ID:") {
        return Uuid::parse_str(value)
            .map(Scope::Id)
            .map_err(|_| scope_config_forbidden("invalid_scope_id", scope_rule));
    }

    Err(scope_config_forbidden("invalid_scope_rule", scope_rule))
}

pub fn ensure_scope_all_only(scope: Scope) -> Result<(), AppError> {
    match scope {
        Scope::All => Ok(()),
        Scope::Id(_) => Err(scope_mismatch_forbidden("scope_requires_all", scope)),
    }
}

pub fn ensure_scope_target_user(scope: Scope, target_user_id: Uuid) -> Result<(), AppError> {
    match scope {
        Scope::All => Ok(()),
        Scope::Id(allowed_user_id) if allowed_user_id == target_user_id => Ok(()),
        Scope::Id(_) => Err(AppError::PermissionDenied("权限不足".to_string())),
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

fn scope_mismatch_forbidden(reason: &str, scope: Scope) -> AppError {
    let request_id = current_request_id().unwrap_or_else(|| "req_unknown".to_string());
    let scope = match scope {
        Scope::All => "ALL".to_string(),
        Scope::Id(user_id) => format!("ID:{user_id}"),
    };
    tracing::warn!(
        %request_id,
        scope_constraint_error = %reason,
        scope = %scope,
        "scope rejected by endpoint constraint"
    );
    AppError::PermissionDenied("权限不足".to_string())
}
