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
}
