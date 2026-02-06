use anyhow::{anyhow, Context, Result};

/// 启动期（Bootstrap）配置：只允许从环境变量读取。
///
/// 约定：
/// - `DATABASE_URL`：数据库连接（必填）
/// - `SERVER__HOST` / `SERVER__PORT`：服务绑定地址（可选，有默认值）
/// - `RUST_LOG`：日志过滤（仅影响日志系统，不在这里解析）
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
}

impl BootstrapConfig {
    pub fn load_from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .context("缺少环境变量 DATABASE_URL（仅此项用于数据库连接引导）")?;

        let server_host = std::env::var("SERVER__HOST").unwrap_or_else(|_| "0.0.0.0".into());

        let server_port = match std::env::var("SERVER__PORT") {
            Ok(v) => v
                .parse::<u16>()
                .map_err(|e| anyhow!("SERVER__PORT 解析失败: {e}"))?,
            Err(_) => 8080,
        };

        Ok(Self {
            database_url,
            server_host,
            server_port,
        })
    }
}
