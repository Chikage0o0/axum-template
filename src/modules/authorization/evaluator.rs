use chrono::{DateTime, Utc};

use super::model::{Effect, EvaluationResult, Policy, Subject};

pub fn evaluate(
    policies: &[Policy],
    subjects: &[Subject],
    perm_code: &str,
    now: DateTime<Utc>,
) -> EvaluationResult {
    let mut matched: Option<Matched<'_>> = None;

    for policy in policies {
        if policy.constraints.is_expired(now) {
            continue;
        }
        if !subject_matches(policy, subjects) {
            continue;
        }
        let Some(specificity) = permission_specificity(&policy.perm_code, perm_code) else {
            continue;
        };

        let candidate = Matched {
            policy,
            specificity,
        };
        if matched
            .as_ref()
            .is_none_or(|current| candidate.is_better_than(current))
        {
            matched = Some(candidate);
        }
    }

    if let Some(best) = matched {
        return EvaluationResult {
            allowed: best.policy.effect == Effect::Allow,
            scope_rule: Some(best.policy.scope_rule.clone()),
            matched_policy_id: Some(best.policy.policy_id),
            effect: best.policy.effect,
        };
    }

    EvaluationResult {
        allowed: false,
        scope_rule: None,
        matched_policy_id: None,
        effect: Effect::Deny,
    }
}

struct Matched<'a> {
    policy: &'a Policy,
    specificity: u8,
}

impl Matched<'_> {
    fn is_better_than(&self, other: &Matched<'_>) -> bool {
        if self.policy.priority != other.policy.priority {
            return self.policy.priority > other.policy.priority;
        }
        if self.policy.effect != other.policy.effect {
            return self.policy.effect.rank() > other.policy.effect.rank();
        }
        if self.specificity != other.specificity {
            return self.specificity > other.specificity;
        }
        self.policy.policy_id < other.policy.policy_id
    }
}

fn subject_matches(policy: &Policy, subjects: &[Subject]) -> bool {
    subjects.iter().any(|subject| {
        subject.subject_type == policy.subject_type && subject.subject_key == policy.subject_key
    })
}

fn permission_specificity(policy_perm: &str, required_perm: &str) -> Option<u8> {
    if policy_perm == required_perm {
        return Some(3);
    }
    if policy_perm == "*" {
        return Some(1);
    }

    let (namespace, wildcard) = policy_perm.split_once(':')?;
    if wildcard != "*" {
        return None;
    }
    if required_perm
        .strip_prefix(namespace)
        .is_some_and(|suffix| suffix.starts_with(':'))
    {
        return Some(2);
    }

    None
}
