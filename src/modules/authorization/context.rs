use crate::api::auth::CurrentUser;

use super::model::{Subject, SubjectType};

#[derive(Debug, Clone)]
pub struct AuthorizationContext {
    subjects: Vec<Subject>,
}

impl AuthorizationContext {
    pub fn from_current_user(current_user: &CurrentUser) -> Self {
        Self {
            subjects: vec![
                Subject {
                    subject_type: SubjectType::User,
                    subject_key: current_user.user_id.to_string(),
                },
                Subject {
                    subject_type: SubjectType::Role,
                    subject_key: current_user.role.clone(),
                },
            ],
        }
    }

    pub fn subjects(&self) -> &[Subject] {
        &self.subjects
    }

    pub fn into_subjects(self) -> Vec<Subject> {
        self.subjects
    }
}
