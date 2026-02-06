use anyhow::{anyhow, Context, Result};
use argon2::password_hash::rand_core::{OsRng, RngCore};

use crate::db::DbPool;

#[derive(Debug, Clone)]
pub struct SeedOptions {
    /// 可选覆盖：仅在首次初始化管理员密码时使用。
    ///
    /// 约定环境变量：SEED_ADMIN_PASSWORD
    pub seed_admin_password: Option<String>,
}

impl SeedOptions {
    pub fn from_env() -> Self {
        Self {
            seed_admin_password: std::env::var("SEED_ADMIN_PASSWORD").ok(),
        }
    }
}

/// 迁移后/启动时的幂等初始化：
/// - 确保 `security.jwt_secret` 存在
/// - 确保 `security.admin_password_hash` 存在（Argon2id PHC）
pub async fn seed_if_needed(pool: &DbPool, opts: &SeedOptions) -> Result<()> {
    ensure_jwt_secret_exists(pool).await?;
    ensure_admin_password_hash_exists(pool, opts).await?;
    Ok(())
}

async fn ensure_jwt_secret_exists(pool: &DbPool) -> Result<()> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM system_config WHERE key = 'security.jwt_secret')",
    )
    .fetch_one(pool)
    .await
    .context("检查 security.jwt_secret 是否存在失败")?;

    if exists {
        return Ok(());
    }

    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let secret_hex = hex_encode(&bytes);

    let inserted: Option<String> = sqlx::query_scalar(
        r#"
INSERT INTO system_config (key, value, description)
VALUES ($1, $2, $3)
ON CONFLICT (key) DO NOTHING
RETURNING key
"#,
    )
    .bind("security.jwt_secret")
    .bind(serde_json::Value::String(secret_hex))
    .bind("JWT 签名密钥（hex），用于 HS256")
    .fetch_optional(pool)
    .await
    .context("写入 security.jwt_secret 失败")?;

    if inserted.is_some() {
        tracing::info!("已生成并写入 security.jwt_secret");
    }

    Ok(())
}

async fn ensure_admin_password_hash_exists(pool: &DbPool, opts: &SeedOptions) -> Result<()> {
    let existing_value: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT value FROM system_config WHERE key = 'security.admin_password_hash'",
    )
    .fetch_optional(pool)
    .await
    .context("读取 security.admin_password_hash 失败")?;

    enum SeedMode {
        Insert,
        UpdateEmpty,
    }

    let mode = match existing_value {
        None => SeedMode::Insert,
        Some(v) => {
            if v.is_null() {
                SeedMode::UpdateEmpty
            } else if let Some(s) = v.as_str() {
                if s.trim().is_empty() {
                    SeedMode::UpdateEmpty
                } else {
                    return Ok(());
                }
            } else {
                return Err(anyhow!(
                    "security.admin_password_hash 类型错误：期望 string（Argon2id PHC）"
                ));
            }
        }
    };

    let (password, should_print) = match opts.seed_admin_password.as_ref() {
        Some(p) if !p.trim().is_empty() => (p.clone(), false),
        _ => (generate_random_password()?, true),
    };

    let password_hash = crate::password::hash_password_argon2id(&password)?;

    let changed: Option<String> = match mode {
        SeedMode::Insert => sqlx::query_scalar(
            r#"
INSERT INTO system_config (key, value, description)
VALUES ($1, $2, $3)
ON CONFLICT (key) DO NOTHING
RETURNING key
"#,
        )
        .bind("security.admin_password_hash")
        .bind(serde_json::Value::String(password_hash))
        .bind("管理员密码 Argon2id 哈希（PHC 格式）")
        .fetch_optional(pool)
        .await
        .context("写入 security.admin_password_hash 失败")?,
        SeedMode::UpdateEmpty => sqlx::query_scalar(
            r#"
UPDATE system_config
SET value = $2,
    updated_at = NOW()
WHERE key = $1
  AND (
       (jsonb_typeof(value) = 'string' AND trim(btrim(value::text, '"')) = '')
       OR jsonb_typeof(value) = 'null'
  )
RETURNING key
"#,
        )
        .bind("security.admin_password_hash")
        .bind(serde_json::Value::String(password_hash))
        .fetch_optional(pool)
        .await
        .context("修复空的 security.admin_password_hash 失败")?,
    };

    if changed.is_some() {
        if should_print {
            tracing::warn!(
                "已生成管理员初始密码（仅打印一次，请立即修改/妥善保存）：{}",
                password
            );
        } else {
            tracing::info!("已使用 SEED_ADMIN_PASSWORD 初始化管理员密码（未打印明文）");
        }
    }

    Ok(())
}

fn generate_random_password() -> Result<String> {
    let mut bytes = [0u8; 16];
    OsRng.fill_bytes(&mut bytes);
    Ok(hex_encode(&bytes))
}

pub fn hex_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = Vec::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(TABLE[(b >> 4) as usize]);
        out.push(TABLE[(b & 0x0f) as usize]);
    }
    // 安全：TABLE 只包含 ASCII。
    String::from_utf8(out).unwrap_or_default()
}
