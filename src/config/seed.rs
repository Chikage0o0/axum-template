use crate::db::DbPool;
use anyhow::{anyhow, Context, Result};
use argon2::password_hash::rand_core::{OsRng, RngCore};

#[derive(Debug, Clone)]
pub struct SeedOptions {
    /// 可选覆盖：初始化管理员用户名，默认 `admin`。
    ///
    /// 约定环境变量：SEED_ADMIN_USERNAME
    pub seed_admin_username: Option<String>,

    /// 可选覆盖：仅在首次初始化管理员密码时使用。
    ///
    /// 约定环境变量：SEED_ADMIN_PASSWORD
    pub seed_admin_password: Option<String>,
}

impl SeedOptions {
    pub fn from_env() -> Self {
        Self {
            seed_admin_username: std::env::var("SEED_ADMIN_USERNAME").ok(),
            seed_admin_password: std::env::var("SEED_ADMIN_PASSWORD").ok(),
        }
    }
}

/// 迁移后/启动时的幂等初始化：
/// - 确保 `security.jwt_secret` 存在
/// - 确保管理员用户（默认 `admin`）存在可用密码哈希
pub async fn seed_if_needed(pool: &DbPool, opts: &SeedOptions) -> Result<()> {
    ensure_jwt_secret_exists(pool).await?;
    ensure_admin_user_password_hash_exists(pool, opts).await?;
    Ok(())
}

async fn ensure_jwt_secret_exists(pool: &DbPool) -> Result<()> {
    let exists_row = sqlx::query!(
        "SELECT EXISTS (SELECT 1 FROM system_config WHERE key = 'security.jwt_secret') AS \"exists!\""
    )
    .fetch_one(pool)
    .await
    .context("检查 security.jwt_secret 是否存在失败")?;
    let exists = exists_row.exists;

    if exists {
        return Ok(());
    }

    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let secret_hex = hex_encode(&bytes);

    let inserted: Option<String> = sqlx::query_scalar!(
        r#"
INSERT INTO system_config (key, value, description)
VALUES ($1, $2, $3)
ON CONFLICT (key) DO NOTHING
RETURNING key
"#,
        "security.jwt_secret",
        serde_json::Value::String(secret_hex),
        "JWT 签名密钥（hex），用于 HS256",
    )
    .fetch_optional(pool)
    .await
    .context("写入 security.jwt_secret 失败")?;

    if inserted.is_some() {
        tracing::info!("已生成并写入 security.jwt_secret");
    }

    Ok(())
}

#[derive(sqlx::FromRow)]
struct AdminUserRow {
    password_hash: Option<String>,
}

async fn ensure_admin_user_password_hash_exists(pool: &DbPool, opts: &SeedOptions) -> Result<()> {
    let username = resolve_admin_username(opts);
    let existing_user = load_admin_user(pool, &username).await?;

    if let Some(user) = &existing_user {
        if user
            .password_hash
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty())
        {
            return Ok(());
        }
    }

    if let Some(legacy_hash) = load_legacy_admin_password_hash(pool).await? {
        upsert_admin_user_password_hash(pool, &username, &legacy_hash)
            .await
            .context("迁移 legacy admin 密码到 users 失败")?;
        tracing::info!("已将 security.admin_password_hash 迁移到用户 {username}");
        return Ok(());
    }

    let (password, should_print) = match opts.seed_admin_password.as_ref() {
        Some(p) if !p.trim().is_empty() => (p.clone(), false),
        _ => (generate_random_password()?, true),
    };
    let password_hash = crate::password::hash_password_argon2id(&password)?;

    upsert_admin_user_password_hash(pool, &username, &password_hash)
        .await
        .context("初始化管理员用户密码失败")?;

    if should_print {
        tracing::warn!(
            "已生成管理员初始密码（用户: {}，仅打印一次，请立即修改/妥善保存）：{}",
            username,
            password
        );
    } else {
        tracing::info!("已使用 SEED_ADMIN_PASSWORD 初始化管理员用户密码（用户: {username}）");
    }

    Ok(())
}

fn resolve_admin_username(opts: &SeedOptions) -> String {
    opts.seed_admin_username
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("admin")
        .to_string()
}

async fn load_admin_user(pool: &DbPool, username: &str) -> Result<Option<AdminUserRow>> {
    let user = sqlx::query_as!(
        AdminUserRow,
        r#"
SELECT password_hash
FROM users
WHERE username = $1
LIMIT 1
        "#,
        username,
    )
    .fetch_optional(pool)
    .await
    .context("查询管理员用户失败")?;

    Ok(user)
}

async fn load_legacy_admin_password_hash(pool: &DbPool) -> Result<Option<String>> {
    let value: Option<serde_json::Value> = sqlx::query_scalar!(
        "SELECT value FROM system_config WHERE key = 'security.admin_password_hash'",
    )
    .fetch_optional(pool)
    .await
    .context("读取 security.admin_password_hash 失败")?;

    let Some(value) = value else {
        return Ok(None);
    };
    let Some(hash) = value.as_str() else {
        return Err(anyhow!(
            "security.admin_password_hash 类型错误：期望 string（Argon2id PHC）"
        ));
    };

    let hash = hash.trim();
    if hash.is_empty() {
        return Ok(None);
    }

    Ok(Some(hash.to_string()))
}

async fn upsert_admin_user_password_hash(
    pool: &DbPool,
    username: &str,
    password_hash: &str,
) -> Result<()> {
    let default_email = format!("{username}@local.invalid");
    if try_upsert_admin_user(pool, username, &default_email, password_hash)
        .await
        .is_ok()
    {
        return Ok(());
    }

    let mut suffix = [0u8; 4];
    OsRng.fill_bytes(&mut suffix);
    let fallback_email = format!("{username}+{}@local.invalid", hex_encode(&suffix));
    try_upsert_admin_user(pool, username, &fallback_email, password_hash)
        .await
        .context("创建管理员用户失败（默认与回退邮箱均不可用）")?;
    Ok(())
}

async fn try_upsert_admin_user(
    pool: &DbPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
INSERT INTO users (
    username,
    display_name,
    email,
    role,
    is_active,
    metadata,
    password_hash
)
VALUES ($1, $2, $3, 'admin', TRUE, '{}'::jsonb, $4)
ON CONFLICT (username) WHERE deleted_at IS NULL AND username IS NOT NULL DO UPDATE
SET password_hash = COALESCE(NULLIF(users.password_hash, ''), EXCLUDED.password_hash),
    role = 'admin',
    is_active = TRUE,
    updated_at = NOW()
        "#,
        username,
        "Administrator",
        email,
        password_hash,
    )
    .execute(pool)
    .await
    .context("upsert 管理员用户失败")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[sqlx::test(migrations = "./migrations")]
    async fn seed_should_create_admin_user_with_password_when_missing(pool: sqlx::PgPool) {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("执行迁移失败");

        let username = format!("seed-admin-{}", Uuid::new_v4().simple());
        let password = "SeedPassword#A123".to_string();

        sqlx::query!("DELETE FROM users WHERE username = $1", username)
            .execute(&pool)
            .await
            .expect("清理测试管理员失败");

        let opts = SeedOptions {
            seed_admin_username: Some(username.clone()),
            seed_admin_password: Some(password),
        };

        let result = seed_if_needed(&pool, &opts).await;
        assert!(
            result.is_ok(),
            "seed 应能成功创建管理员用户，实际错误: {result:?}"
        );

        let hash = sqlx::query_scalar!(
            r#"
SELECT password_hash
FROM users
WHERE username = $1
LIMIT 1
            "#,
            username,
        )
        .fetch_optional(&pool)
        .await
        .expect("查询测试管理员失败")
        .flatten();

        assert!(
            hash.as_deref().is_some_and(|v| !v.trim().is_empty()),
            "seed 创建后管理员密码哈希不应为空"
        );

        sqlx::query!("DELETE FROM users WHERE username = $1", username)
            .execute(&pool)
            .await
            .expect("清理测试管理员失败");
    }
}
