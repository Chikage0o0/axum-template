use chrono::{Duration, Utc};

use super::evaluator::evaluate;
use super::model::{Constraint, Effect, Policy, Subject, SubjectType};

fn subject(subject_type: SubjectType, subject_key: &str) -> Subject {
    Subject {
        subject_type,
        subject_key: subject_key.to_string(),
    }
}

fn policy(
    policy_id: i64,
    subject: Subject,
    perm_code: &str,
    effect: Effect,
    priority: i32,
    expire_at: Option<chrono::DateTime<Utc>>,
    scope_rule: &str,
) -> Policy {
    Policy {
        policy_id,
        subject_type: subject.subject_type,
        subject_key: subject.subject_key,
        perm_code: perm_code.to_string(),
        effect,
        scope_rule: scope_rule.to_string(),
        constraints: Constraint {
            expire_at,
            ip_range: None,
        },
        priority,
    }
}

#[test]
fn evaluator_should_deny_when_deny_and_allow_conflict_in_same_priority() {
    let user_id = "11111111-1111-1111-1111-111111111111";
    let now = Utc::now();
    let subjects = vec![subject(SubjectType::User, user_id)];
    let policies = vec![
        policy(
            2,
            subject(SubjectType::User, user_id),
            "users:update",
            Effect::Allow,
            100,
            None,
            "ALL",
        ),
        policy(
            1,
            subject(SubjectType::User, user_id),
            "users:update",
            Effect::Deny,
            100,
            None,
            "ALL",
        ),
    ];

    let decision = evaluate(&policies, &subjects, "users:update", now);
    assert!(!decision.allowed);
    assert_eq!(decision.effect, Effect::Deny);
    assert_eq!(decision.matched_policy_id, Some(1));
}

#[test]
fn evaluator_should_choose_higher_priority_policy_first() {
    let user_id = "22222222-2222-2222-2222-222222222222";
    let now = Utc::now();
    let subjects = vec![subject(SubjectType::User, user_id)];
    let policies = vec![
        policy(
            10,
            subject(SubjectType::User, user_id),
            "users:update",
            Effect::Deny,
            10,
            None,
            "ALL",
        ),
        policy(
            20,
            subject(SubjectType::User, user_id),
            "users:update",
            Effect::Allow,
            99,
            None,
            "ALL",
        ),
    ];

    let decision = evaluate(&policies, &subjects, "users:update", now);
    assert!(decision.allowed);
    assert_eq!(decision.effect, Effect::Allow);
    assert_eq!(decision.matched_policy_id, Some(20));
}

#[test]
fn evaluator_should_support_permission_wildcards() {
    let role = "user";
    let now = Utc::now();
    let subjects = vec![subject(SubjectType::Role, role)];
    let policies = vec![policy(
        30,
        subject(SubjectType::Role, role),
        "users:*",
        Effect::Allow,
        50,
        None,
        "ALL",
    )];

    let decision = evaluate(&policies, &subjects, "users:update", now);
    assert!(decision.allowed);
    assert_eq!(decision.matched_policy_id, Some(30));
}

#[test]
fn evaluator_should_use_specificity_and_policy_id_as_tie_breaker() {
    let role = "admin";
    let now = Utc::now();
    let subjects = vec![subject(SubjectType::Role, role)];
    let policies = vec![
        policy(
            40,
            subject(SubjectType::Role, role),
            "users:*",
            Effect::Allow,
            50,
            None,
            "ALL",
        ),
        policy(
            50,
            subject(SubjectType::Role, role),
            "users:update",
            Effect::Allow,
            50,
            None,
            "SELF",
        ),
        policy(
            60,
            subject(SubjectType::Role, role),
            "users:update",
            Effect::Allow,
            50,
            None,
            "ID:abc",
        ),
    ];

    let decision = evaluate(&policies, &subjects, "users:update", now);
    assert!(decision.allowed);
    assert_eq!(decision.effect, Effect::Allow);
    assert_eq!(decision.matched_policy_id, Some(50));
    assert_eq!(decision.scope_rule.as_deref(), Some("SELF"));
}

#[test]
fn evaluator_should_skip_expired_policy() {
    let user_id = "33333333-3333-3333-3333-333333333333";
    let now = Utc::now();
    let subjects = vec![subject(SubjectType::User, user_id)];
    let policies = vec![policy(
        70,
        subject(SubjectType::User, user_id),
        "users:update",
        Effect::Allow,
        100,
        Some(now - Duration::minutes(1)),
        "ALL",
    )];

    let decision = evaluate(&policies, &subjects, "users:update", now);
    assert!(!decision.allowed);
    assert_eq!(decision.effect, Effect::Deny);
    assert_eq!(decision.matched_policy_id, None);
}

#[test]
fn evaluator_should_merge_user_and_role_subjects() {
    let user_id = "44444444-4444-4444-4444-444444444444";
    let role = "user";
    let now = Utc::now();
    let subjects = vec![
        subject(SubjectType::User, user_id),
        subject(SubjectType::Role, role),
    ];
    let policies = vec![
        policy(
            80,
            subject(SubjectType::Role, role),
            "users:update",
            Effect::Allow,
            100,
            None,
            "ALL",
        ),
        policy(
            81,
            subject(SubjectType::User, user_id),
            "users:*",
            Effect::Deny,
            100,
            None,
            "ALL",
        ),
    ];

    let decision = evaluate(&policies, &subjects, "users:update", now);
    assert!(!decision.allowed);
    assert_eq!(decision.matched_policy_id, Some(81));
}

#[test]
fn evaluator_should_return_stable_matched_policy_id() {
    let role = "admin";
    let now = Utc::now();
    let subjects = vec![subject(SubjectType::Role, role)];
    let policies = vec![
        policy(
            99,
            subject(SubjectType::Role, role),
            "*",
            Effect::Allow,
            1,
            None,
            "ALL",
        ),
        policy(
            100,
            subject(SubjectType::Role, role),
            "users:update",
            Effect::Allow,
            1,
            None,
            "ALL",
        ),
    ];

    let first = evaluate(&policies, &subjects, "users:update", now);
    let second = evaluate(&policies, &subjects, "users:update", now);

    assert_eq!(first.matched_policy_id, Some(100));
    assert_eq!(second.matched_policy_id, Some(100));
    assert_eq!(first.matched_policy_id, second.matched_policy_id);
}
