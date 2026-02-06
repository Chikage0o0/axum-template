use axum::extract::{FromRequest, Request};
use axum::Json;
use garde::Validate;
use serde::de::DeserializeOwned;

use crate::error::AppError;

/// JSON body + garde 校验的统一 extractor。
///
/// 设计目的：
/// - 让“字段有效性验证”集中在结构体上（garde attributes），而不是散落在 handler 里。
/// - 失败时返回统一错误体，并携带字段级 details。
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate<Context = ()> + Send,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await.map_err(|e| {
            // 避免泄漏 body 内容，只返回 extractor 的简要错误文本。
            AppError::validation_with_details(
                "请求 JSON 不合法".to_string(),
                Some(serde_json::json!({ "json": [e.to_string()] })),
            )
        })?;

        value
            .validate()
            .map_err(|report| AppError::from_garde_report("字段校验失败", report))?;

        Ok(Self(value))
    }
}
