use anyhow::Context;
use chrono::{DateTime, Utc};

use crate::db::DbPool;
use crate::error::AppError;

use super::model::{Constraint, Effect, Policy, Subject, SubjectType};

#[derive(Clone)]
pub struct PolicyRepository {
    db: DbPool,
}

impl PolicyRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub async fn fetch_candidate_policies(
        &self,
        subjects: &[Subject],
        perm_code: &str,
        now: DateTime<Utc>,
    ) -> Result<Vec<Policy>, AppError> {
        if subjects.is_empty() {
            return Ok(Vec::new());
        }

        let subject_types: Vec<&str> = subjects.iter().map(|s| s.subject_type.as_str()).collect();
        let subject_keys: Vec<&str> = subjects.iter().map(|s| s.subject_key.as_str()).collect();
        let perm_candidates = permission_candidates(perm_code);
        let namespace_wildcard =
            namespace_wildcard(perm_code).unwrap_or_else(|| perm_code.to_string());

        let rows = sqlx::query_as_unchecked!(
            PolicyRow,
            r#"
SELECT
    policy_id,
    subject_type,
    subject_key,
    perm_code,
    effect,
    scope_rule,
    constraints,
    priority
FROM sys_policy
WHERE (subject_type, subject_key) IN (
    SELECT *
    FROM UNNEST($1::text[], $2::text[])
)
  AND perm_code = ANY($3::text[])
  AND (expire_at IS NULL OR expire_at > $4)
ORDER BY
    priority DESC,
    CASE WHEN effect = 'DENY' THEN 0 ELSE 1 END ASC,
    CASE
        WHEN perm_code = $5 THEN 0
        WHEN perm_code = $6 THEN 1
        ELSE 2
    END ASC,
    policy_id ASC
            "#,
            &subject_types,
            &subject_keys,
            &perm_candidates,
            now,
            perm_code,
            &namespace_wildcard,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalError(format!("查询授权策略失败: {e}")))?;

        rows.into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e: anyhow::Error| AppError::InternalError(format!("解析授权策略失败: {e}")))
    }

    pub async fn fetch_active_policies_for_subjects(
        &self,
        subjects: &[Subject],
        now: DateTime<Utc>,
    ) -> Result<Vec<Policy>, AppError> {
        if subjects.is_empty() {
            return Ok(Vec::new());
        }

        let subject_types: Vec<&str> = subjects.iter().map(|s| s.subject_type.as_str()).collect();
        let subject_keys: Vec<&str> = subjects.iter().map(|s| s.subject_key.as_str()).collect();

        let rows = sqlx::query_as_unchecked!(
            PolicyRow,
            r#"
SELECT
    policy_id,
    subject_type,
    subject_key,
    perm_code,
    effect,
    scope_rule,
    constraints,
    priority
FROM sys_policy
WHERE (subject_type, subject_key) IN (
    SELECT *
    FROM UNNEST($1::text[], $2::text[])
)
  AND (expire_at IS NULL OR expire_at > $3)
ORDER BY
    priority DESC,
    CASE WHEN effect = 'DENY' THEN 0 ELSE 1 END ASC,
    perm_code ASC,
    policy_id ASC
            "#,
            &subject_types,
            &subject_keys,
            now,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalError(format!("查询授权策略失败: {e}")))?;

        rows.into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e: anyhow::Error| AppError::InternalError(format!("解析授权策略失败: {e}")))
    }

    pub async fn list_permission_codes(&self) -> Result<Vec<String>, AppError> {
        sqlx::query_scalar!(
            r#"
SELECT perm_code
FROM sys_permission
ORDER BY perm_code ASC
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::InternalError(format!("读取权限码失败: {e}")))
    }
}

pub fn permission_candidates(perm_code: &str) -> Vec<String> {
    let mut candidates = Vec::with_capacity(3);
    candidates.push(perm_code.to_string());

    if let Some(namespace_wildcard) = namespace_wildcard(perm_code) {
        candidates.push(namespace_wildcard);
    }

    candidates.push("*".to_string());
    candidates.dedup();
    candidates
}

fn namespace_wildcard(perm_code: &str) -> Option<String> {
    let (namespace, _) = perm_code.split_once(':')?;
    if namespace.is_empty() {
        return None;
    }
    if namespace == "*" {
        return Some("*".to_string());
    }
    Some(format!("{namespace}:*"))
}

#[derive(Debug, sqlx::FromRow)]
struct PolicyRow {
    policy_id: i64,
    subject_type: String,
    subject_key: String,
    perm_code: String,
    effect: String,
    scope_rule: String,
    constraints: serde_json::Value,
    priority: i32,
}

impl TryFrom<PolicyRow> for Policy {
    type Error = anyhow::Error;

    fn try_from(row: PolicyRow) -> Result<Self, Self::Error> {
        let subject_type = row
            .subject_type
            .parse::<SubjectType>()
            .map_err(|e| anyhow::anyhow!("无效 subject_type: {} ({e})", row.subject_type))?;
        let effect = row
            .effect
            .parse::<Effect>()
            .map_err(|e| anyhow::anyhow!("无效 effect: {} ({e})", row.effect))?;
        let constraints = parse_constraints(&row.constraints)?;

        Ok(Policy {
            policy_id: row.policy_id,
            subject_type,
            subject_key: row.subject_key,
            perm_code: row.perm_code,
            effect,
            scope_rule: row.scope_rule,
            constraints,
            priority: row.priority,
        })
    }
}

fn parse_constraints(value: &serde_json::Value) -> Result<Constraint, anyhow::Error> {
    let Some(object) = value.as_object() else {
        anyhow::bail!("constraints 必须为 JSON object");
    };

    let expire_at = match object.get("expire_at") {
        Some(v) => {
            let Some(raw) = v.as_str() else {
                anyhow::bail!("constraints.expire_at 必须为字符串");
            };
            let parsed = chrono::DateTime::parse_from_rfc3339(raw)
                .with_context(|| format!("constraints.expire_at 不是 RFC3339: {raw}"))?;
            Some(parsed.with_timezone(&Utc))
        }
        None => None,
    };

    let ip_range = match object.get("ip_range") {
        Some(v) => {
            let Some(raw) = v.as_str() else {
                anyhow::bail!("constraints.ip_range 必须为字符串");
            };
            Some(raw.to_string())
        }
        None => None,
    };

    Ok(Constraint {
        expire_at,
        ip_range,
    })
}
