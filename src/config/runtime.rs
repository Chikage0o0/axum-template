use anyhow::{anyhow, Context, Result};

use crate::db::DbPool;

/// 运行期（Runtime）配置：全部从数据库 `system_config` 读取。
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub security: SecurityRuntimeConfig,
    pub app: AppRuntimeConfig,
    pub integrations: IntegrationsRuntimeConfig,
}

#[derive(Debug, Clone)]
pub struct SecurityRuntimeConfig {
    pub jwt_secret: String,
}

#[derive(Debug, Clone)]
pub struct AppRuntimeConfig {
    pub check_interval_secs: u64,
    pub welcome_message: String,
}

#[derive(Debug, Clone)]
pub struct IntegrationsRuntimeConfig {
    pub example_api_base: String,
    pub example_api_key: String,
}

impl RuntimeConfig {
    pub async fn load_from_db(pool: &DbPool) -> Result<Self> {
        let jwt_secret = get_required_string(pool, "security.jwt_secret")
            .await
            .context("加载 security.jwt_secret 失败")?;

        let check_interval_secs = get_u64_with_default(pool, "app.check_interval_secs", 3600)
            .await
            .context("加载 app.check_interval_secs 失败")?;
        let welcome_message = get_string_with_default(
            pool,
            "app.welcome_message",
            "Hello from PROJECT_NAME".to_string(),
        )
        .await
        .context("加载 app.welcome_message 失败")?;

        let example_api_base = get_string_with_default(
            pool,
            "integrations.example_api_base",
            "https://example.com/api".to_string(),
        )
        .await
        .context("加载 integrations.example_api_base 失败")?;
        let example_api_key =
            get_string_with_default(pool, "integrations.example_api_key", "".into())
                .await
                .context("加载 integrations.example_api_key 失败")?;

        Ok(Self {
            security: SecurityRuntimeConfig { jwt_secret },
            app: AppRuntimeConfig {
                check_interval_secs,
                welcome_message,
            },
            integrations: IntegrationsRuntimeConfig {
                example_api_base,
                example_api_key,
            },
        })
    }
}

async fn get_value(pool: &DbPool, key: &str) -> Result<Option<serde_json::Value>> {
    let value = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT value FROM system_config WHERE key = $1",
    )
    .bind(key)
    .fetch_optional(pool)
    .await
    .context("查询 system_config 失败")?;

    Ok(value)
}

async fn get_required_string(pool: &DbPool, key: &str) -> Result<String> {
    let value = get_value(pool, key).await?;
    let Some(value) = value else {
        return Err(anyhow!("缺少配置项: {key}"));
    };
    value
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("配置项 {key} 类型错误：期望 string"))
}

async fn get_string_with_default(pool: &DbPool, key: &str, default: String) -> Result<String> {
    let value = get_value(pool, key).await?;
    let Some(value) = value else {
        return Ok(default);
    };

    value
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("配置项 {key} 类型错误：期望 string"))
}

async fn get_u64_with_default(pool: &DbPool, key: &str, default: u64) -> Result<u64> {
    let value = get_value(pool, key).await?;
    let Some(value) = value else {
        return Ok(default);
    };
    if let Some(v) = value.as_u64() {
        return Ok(v);
    }
    if let Some(v) = value.as_i64().filter(|v| *v >= 0) {
        return Ok(v as u64);
    }
    Err(anyhow!("配置项 {key} 类型错误：期望 non-negative integer"))
}
