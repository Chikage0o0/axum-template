use crate::db::DbPool;
use crate::error::AppError;

/// 批量 upsert system_config。
///
/// - 仅更新 key/value/updated_at，不改动 description
/// - 以事务包裹，保证同一次 PATCH 要么全部成功要么全部失败
pub async fn upsert_many(
    db: &DbPool,
    changes: Vec<(String, serde_json::Value)>,
) -> Result<(), AppError> {
    if changes.is_empty() {
        return Ok(());
    }

    let mut tx = db
        .begin()
        .await
        .map_err(|e| AppError::InternalError(format!("开启事务失败: {e}")))?;

    for (key, value) in changes {
        sqlx::query!(
            r#"
INSERT INTO system_config (key, value)
VALUES ($1, $2)
ON CONFLICT (key) DO UPDATE
SET value = EXCLUDED.value,
    updated_at = NOW()
            "#,
            key,
            value,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::InternalError(format!("写入 system_config 失败: {e}")))?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::InternalError(format!("提交事务失败: {e}")))?;

    Ok(())
}
