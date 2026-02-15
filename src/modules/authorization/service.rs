use chrono::Utc;

use crate::db::DbPool;
use crate::error::AppError;

use super::evaluator::evaluate;
use super::model::{Effect, Subject};
use super::repository::PolicyRepository;

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
        let all_permissions = self.repository.list_permission_codes().await?;
        if all_permissions.is_empty() {
            return Ok(Vec::new());
        }

        let policies = self
            .repository
            .fetch_active_policies_for_subjects(subjects, now)
            .await?;

        let mut allowed = Vec::new();
        for perm_code in all_permissions {
            let result = evaluate(&policies, subjects, &perm_code, now);
            if result.allowed {
                allowed.push(perm_code);
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
}
