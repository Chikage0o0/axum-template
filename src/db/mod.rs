use crate::error::AppError;

pub type DbPool = sqlx::PgPool;

pub async fn connect(database_url: &str) -> Result<DbPool, AppError> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| AppError::InternalError(format!("数据库连接失败: {e}")))?;

    sqlx::query_scalar!("SELECT 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::InternalError(format!("数据库探测失败: {e}")))?;

    Ok(pool)
}
