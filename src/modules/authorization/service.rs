use chrono::Utc;

use crate::db::DbPool;
use crate::error::AppError;

use super::evaluator::evaluate;
use super::model::{Effect, Subject};
use super::permission::PermissionNode;
use super::repository::PolicyRepository;

#[derive(Debug, Clone)]
pub struct PermissionNodeDictionaryItem {
    pub code: PermissionNode,
    pub name: String,
    pub description: String,
    pub module: String,
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub allowed: bool,
    pub scope_rule: Option<String>,
    pub matched_policy_id: Option<i64>,
    pub effect: Effect,
}

#[derive(Clone)]
pub struct AuthorizationService {
    repository: PolicyRepository,
}

impl AuthorizationService {
    pub fn new(db: DbPool) -> Self {
        Self {
            repository: PolicyRepository::new(db),
        }
    }

    pub async fn authorize(
        &self,
        subjects: &[Subject],
        perm_code: &str,
        resource_hint: Option<&str>,
        request_id: &str,
    ) -> Result<Decision, AppError> {
        let now = Utc::now();
        let policies = self
            .repository
            .fetch_candidate_policies(subjects, perm_code, now)
            .await?;
        let result = evaluate(&policies, subjects, perm_code, now);

        let decision = Decision {
            allowed: result.allowed,
            scope_rule: result.scope_rule,
            matched_policy_id: result.matched_policy_id,
            effect: result.effect,
        };

        tracing::info!(
            request_id = %request_id,
            perm_code = %perm_code,
            resource_hint = resource_hint.unwrap_or("-"),
            matched_policy_id = decision.matched_policy_id,
            effect = %decision.effect.as_str(),
            allowed = decision.allowed,
            "authorization decision"
        );

        Ok(decision)
    }

    pub async fn list_allowed_permissions(
        &self,
        subjects: &[Subject],
        request_id: &str,
    ) -> Result<Vec<String>, AppError> {
        let now = Utc::now();
        let policies = self
            .repository
            .fetch_active_policies_for_subjects(subjects, now)
            .await?;

        let mut allowed = Vec::new();
        for permission in PermissionNode::ALL {
            let perm_code = permission.as_str();
            let result = evaluate(&policies, subjects, perm_code, now);
            if result.allowed {
                allowed.push(perm_code.to_string());
            }
        }

        tracing::info!(
            request_id = %request_id,
            subjects_count = subjects.len(),
            permissions_count = allowed.len(),
            "authorization permissions snapshot"
        );

        Ok(allowed)
    }

    pub async fn list_permission_nodes_from_db(
        &self,
        request_id: &str,
    ) -> Result<Vec<PermissionNodeDictionaryItem>, AppError> {
        let rows = self.repository.list_permission_dictionary_rows().await?;
        let mut nodes = Vec::with_capacity(rows.len());
        let mut unknown_count = 0usize;
        let mut unknown_samples: Vec<String> = Vec::new();

        for row in rows {
            match PermissionNode::try_from_code(row.perm_code.as_str()) {
                Some(node) => nodes.push(PermissionNodeDictionaryItem {
                    code: node,
                    name: row.perm_name,
                    description: row.description.unwrap_or_default(),
                    module: permission_module(node.as_str()),
                }),
                None => {
                    unknown_count += 1;
                    if unknown_samples.len() < 5 {
                        unknown_samples.push(row.perm_code);
                    }
                }
            }
        }

        if unknown_count > 0 {
            tracing::warn!(
                request_id = %request_id,
                unknown_count,
                unknown_samples = ?unknown_samples,
                "ignore unknown permission codes from sys_permission"
            );
        }

        tracing::info!(
            request_id = %request_id,
            permissions_count = nodes.len(),
            "authorization permission dictionary from db"
        );

        Ok(nodes)
    }
}

fn permission_module(perm_code: &str) -> String {
    if perm_code == "*" {
        return "global".to_string();
    }

    perm_code
        .split_once(':')
        .map(|(module, _)| module.to_string())
        .unwrap_or_else(|| "global".to_string())
}
