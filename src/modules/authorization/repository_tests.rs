use chrono::{Duration, Utc};
use uuid::Uuid;

use super::model::{Subject, SubjectType};
use super::repository::PolicyRepository;

#[sqlx::test(migrations = "./migrations")]
async fn repository_should_load_policies_by_subject_and_fixed_order(pool: sqlx::PgPool) {
    let repo = PolicyRepository::new(pool.clone());
    let user_id = Uuid::new_v4();
    let role_key = "repo_test_role";
    let user_key = user_id.to_string();

    let high_allow = insert_policy(
        &pool,
        InsertPolicyParams {
            subject_type: "USER",
            subject_key: &user_key,
            perm_code: "users:update",
            effect: "ALLOW",
            scope_rule: "ALL",
            constraints: serde_json::json!({}),
            priority: 90,
        },
    )
    .await;

    let deny_namespace = insert_policy(
        &pool,
        InsertPolicyParams {
            subject_type: "ROLE",
            subject_key: role_key,
            perm_code: "users:*",
            effect: "DENY",
            scope_rule: "ALL",
            constraints: serde_json::json!({}),
            priority: 80,
        },
    )
    .await;

    let allow_exact_large_id = insert_policy(
        &pool,
        InsertPolicyParams {
            subject_type: "ROLE",
            subject_key: role_key,
            perm_code: "users:update",
            effect: "ALLOW",
            scope_rule: "ALL",
            constraints: serde_json::json!({}),
            priority: 80,
        },
    )
    .await;

    let allow_exact_small_id = insert_policy(
        &pool,
        InsertPolicyParams {
            subject_type: "ROLE",
            subject_key: role_key,
            perm_code: "users:update",
            effect: "ALLOW",
            scope_rule: "SELF",
            constraints: serde_json::json!({}),
            priority: 80,
        },
    )
    .await;

    let allow_global = insert_policy(
        &pool,
        InsertPolicyParams {
            subject_type: "ROLE",
            subject_key: role_key,
            perm_code: "*",
            effect: "ALLOW",
            scope_rule: "ALL",
            constraints: serde_json::json!({}),
            priority: 80,
        },
    )
    .await;

    let _ignored_other_subject = insert_policy(
        &pool,
        InsertPolicyParams {
            subject_type: "ROLE",
            subject_key: "guest",
            perm_code: "users:update",
            effect: "DENY",
            scope_rule: "ALL",
            constraints: serde_json::json!({}),
            priority: 999,
        },
    )
    .await;

    let subjects = vec![
        Subject {
            subject_type: SubjectType::User,
            subject_key: user_key,
        },
        Subject {
            subject_type: SubjectType::Role,
            subject_key: role_key.to_string(),
        },
    ];

    let rows = repo
        .fetch_candidate_policies(&subjects, "users:update", Utc::now())
        .await
        .expect("按主体读取策略应成功");

    let exact_ids: Vec<i64> = rows
        .iter()
        .filter(|p| p.perm_code == "users:update" && p.priority == 80)
        .map(|p| p.policy_id)
        .collect();

    let (exact_first, exact_second) = if allow_exact_small_id < allow_exact_large_id {
        (allow_exact_small_id, allow_exact_large_id)
    } else {
        (allow_exact_large_id, allow_exact_small_id)
    };
    assert_eq!(exact_ids, vec![exact_first, exact_second]);

    let actual: Vec<i64> = rows.into_iter().map(|p| p.policy_id).collect();
    assert_eq!(
        actual,
        vec![
            high_allow,
            deny_namespace,
            exact_first,
            exact_second,
            allow_global,
        ]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn repository_should_filter_expired_and_permission_candidates(pool: sqlx::PgPool) {
    let repo = PolicyRepository::new(pool.clone());
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    let user_key = user_id.to_string();

    let valid_exact = insert_policy(
        &pool,
        InsertPolicyParams {
            subject_type: "USER",
            subject_key: &user_key,
            perm_code: "users:update",
            effect: "ALLOW",
            scope_rule: "ALL",
            constraints: serde_json::json!({}),
            priority: 10,
        },
    )
    .await;

    let valid_namespace = insert_policy(
        &pool,
        InsertPolicyParams {
            subject_type: "ROLE",
            subject_key: "user",
            perm_code: "users:*",
            effect: "ALLOW",
            scope_rule: "ALL",
            constraints: serde_json::json!({}),
            priority: 9,
        },
    )
    .await;

    let valid_global = insert_policy(
        &pool,
        InsertPolicyParams {
            subject_type: "ROLE",
            subject_key: "user",
            perm_code: "*",
            effect: "ALLOW",
            scope_rule: "ALL",
            constraints: serde_json::json!({}),
            priority: 8,
        },
    )
    .await;

    let _expired_exact = insert_policy(
        &pool,
        InsertPolicyParams {
            subject_type: "USER",
            subject_key: &user_key,
            perm_code: "users:update",
            effect: "ALLOW",
            scope_rule: "ALL",
            constraints: serde_json::json!({
                "expire_at": (now - Duration::minutes(5)).to_rfc3339(),
            }),
            priority: 100,
        },
    )
    .await;

    let _unrelated_permission = insert_policy(
        &pool,
        InsertPolicyParams {
            subject_type: "ROLE",
            subject_key: "user",
            perm_code: "settings:view",
            effect: "ALLOW",
            scope_rule: "ALL",
            constraints: serde_json::json!({}),
            priority: 999,
        },
    )
    .await;

    let subjects = vec![
        Subject {
            subject_type: SubjectType::User,
            subject_key: user_key,
        },
        Subject {
            subject_type: SubjectType::Role,
            subject_key: "user".to_string(),
        },
    ];

    let rows = repo
        .fetch_candidate_policies(&subjects, "users:update", now)
        .await
        .expect("读取策略应成功");

    let actual: Vec<i64> = rows.into_iter().map(|p| p.policy_id).collect();
    assert_eq!(actual, vec![valid_exact, valid_namespace, valid_global]);
}

async fn insert_policy(pool: &sqlx::PgPool, params: InsertPolicyParams<'_>) -> i64 {
    let expire_at = params
        .constraints
        .get("expire_at")
        .and_then(serde_json::Value::as_str)
        .and_then(|raw| chrono::DateTime::parse_from_rfc3339(raw).ok())
        .map(|dt| dt.with_timezone(&Utc));

    sqlx::query_scalar!(
        r#"
INSERT INTO sys_policy (
    subject_type,
    subject_key,
    perm_code,
    effect,
    scope_rule,
    constraints,
    expire_at,
    priority
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
RETURNING policy_id
        "#,
        params.subject_type,
        params.subject_key,
        params.perm_code,
        params.effect,
        params.scope_rule,
        params.constraints,
        expire_at,
        params.priority,
    )
    .fetch_one(pool)
    .await
    .expect("插入策略应成功")
}

struct InsertPolicyParams<'a> {
    subject_type: &'a str,
    subject_key: &'a str,
    perm_code: &'a str,
    effect: &'a str,
    scope_rule: &'a str,
    constraints: serde_json::Value,
    priority: i32,
}
