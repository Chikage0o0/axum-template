use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectType {
    User,
    Role,
}

impl SubjectType {
    pub fn as_str(self) -> &'static str {
        match self {
            SubjectType::User => "USER",
            SubjectType::Role => "ROLE",
        }
    }
}

impl std::str::FromStr for SubjectType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "USER" => Ok(SubjectType::User),
            "ROLE" => Ok(SubjectType::Role),
            _ => Err(format!("不支持的 subject_type: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subject {
    pub subject_type: SubjectType,
    pub subject_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Allow,
    Deny,
}

impl Effect {
    pub fn as_str(self) -> &'static str {
        match self {
            Effect::Allow => "ALLOW",
            Effect::Deny => "DENY",
        }
    }

    pub fn rank(self) -> u8 {
        match self {
            Effect::Deny => 2,
            Effect::Allow => 1,
        }
    }
}

impl std::str::FromStr for Effect {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ALLOW" => Ok(Effect::Allow),
            "DENY" => Ok(Effect::Deny),
            _ => Err(format!("不支持的 effect: {value}")),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Constraint {
    pub expire_at: Option<DateTime<Utc>>,
    pub ip_range: Option<String>,
}

impl Constraint {
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expire_at.is_some_and(|expire_at| expire_at <= now)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub policy_id: i64,
    pub subject_type: SubjectType,
    pub subject_key: String,
    pub perm_code: String,
    pub effect: Effect,
    pub scope_rule: String,
    pub constraints: Constraint,
    pub priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationResult {
    pub allowed: bool,
    pub scope_rule: Option<String>,
    pub matched_policy_id: Option<i64>,
    pub effect: Effect,
}
