use anyhow::{anyhow, Result};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub fn hash_password_argon2id(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("计算 Argon2id hash 失败: {e}"))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, phc_hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(phc_hash).map_err(|e| anyhow!("解析 PHC hash 失败: {e}"))?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed).is_ok())
}
