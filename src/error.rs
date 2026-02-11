use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use garde::Report;
use serde::Serialize;
use thiserror::Error;

use crate::api::request_id::current_request_id;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("身份验证失败: {0}")]
    AuthTokenError(String),
    #[error("身份验证失败: {0}")]
    AuthCredentialError(String),
    #[error("权限不足: {0}")]
    PermissionDenied(String),
    #[error("验证失败: {message}")]
    ValidationError {
        message: String,
        details: Option<serde_json::Value>,
    },
    #[error("未找到资源: {0}")]
    NotFound(String),
    #[error("服务器内部错误: {0}")]
    InternalError(String),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl AppError {
    pub fn auth_token(message: impl Into<String>) -> Self {
        Self::AuthTokenError(message.into())
    }

    pub fn auth_credential(message: impl Into<String>) -> Self {
        Self::AuthCredentialError(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            details: None,
        }
    }

    pub fn validation_with_details(
        message: impl Into<String>,
        details: Option<serde_json::Value>,
    ) -> Self {
        Self::ValidationError {
            message: message.into(),
            details,
        }
    }

    pub fn from_garde_report(prefix: &str, report: Report) -> Self {
        let mut map: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        for (path, error) in report.iter() {
            map.entry(path.to_string())
                .or_default()
                .push(error.to_string());
        }
        let details = serde_json::to_value(map).ok();
        Self::validation_with_details(prefix.to_string(), details)
    }

    pub fn error_code(&self) -> u16 {
        match self {
            AppError::ValidationError { .. } => 1000,
            AppError::AuthTokenError(_) => 1001,
            AppError::AuthCredentialError(_) => 1002,
            AppError::PermissionDenied(_) => 2002,
            AppError::NotFound(_) => 2000,
            AppError::InternalError(_) => 5000,
            AppError::Unknown(_) => 5000,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::AuthTokenError(_) => StatusCode::UNAUTHORIZED,
            AppError::AuthCredentialError(_) => StatusCode::UNAUTHORIZED,
            AppError::PermissionDenied(_) => StatusCode::FORBIDDEN,
            AppError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
    request_id: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();
        let raw_message = self.to_string();
        let message = match &self {
            AppError::InternalError(_) | AppError::Unknown(_) => "服务器内部错误".to_string(),
            _ => raw_message.clone(),
        };
        let details = match &self {
            AppError::ValidationError { details, .. } => details.clone(),
            _ => None,
        };
        let request_id = current_request_id().unwrap_or_else(|| "req_unknown".to_string());

        if matches!(self, AppError::InternalError(_) | AppError::Unknown(_)) {
            tracing::error!(%request_id, code, status = %status, error = %raw_message, "internal app error");
        }

        let body = Json(ErrorResponse {
            code,
            message,
            details,
            request_id,
        });

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn token_auth_error_should_use_1001() {
        assert_eq!(AppError::auth_token("token 无效").error_code(), 1001);
    }

    #[test]
    fn credential_auth_error_should_use_1002() {
        assert_eq!(AppError::auth_credential("密码错误").error_code(), 1002);
    }
}
