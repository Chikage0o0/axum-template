use super::permission::{permission_catalog, permission_catalog_version, PermissionNode};
use super::repository::PolicyRepository;

#[test]
fn permission_node_should_parse_from_code() {
    assert_eq!(
        PermissionNode::try_from_code("users:list"),
        Some(PermissionNode::UsersList)
    );
    assert_eq!(PermissionNode::try_from_code("users:password:update"), None);
    assert_eq!(PermissionNode::try_from_code("users:me:view"), None);
    assert_eq!(PermissionNode::try_from_code("users:me:update"), None);
    assert_eq!(PermissionNode::try_from_code("unknown:perm"), None);
}

#[test]
fn permission_catalog_should_be_stable_and_include_builtin_codes() {
    let codes: Vec<&'static str> = permission_catalog()
        .iter()
        .map(|item| item.as_str())
        .collect();

    assert_eq!(
        codes,
        vec![
            "*",
            "users:*",
            "users:list",
            "users:create",
            "users:update",
            "users:delete",
            "users:restore",
            "settings:view",
            "settings:update",
            "sessions:delete",
            "authorization:permission-nodes:view",
        ]
    );
}

#[test]
fn permission_catalog_should_not_contain_duplicate_codes() {
    let mut codes: Vec<&'static str> = permission_catalog()
        .iter()
        .map(|item| item.as_str())
        .collect();
    codes.sort_unstable();
    codes.dedup();

    assert_eq!(codes.len(), permission_catalog().len());
}

#[test]
fn permission_catalog_version_should_be_stable_hex_hash() {
    let first = permission_catalog_version();
    let second = permission_catalog_version();

    assert_eq!(first, second);
    assert_eq!(first.len(), 16);
    assert!(first.chars().all(|ch| ch.is_ascii_hexdigit()));
}

#[test]
fn permission_catalog_version_should_track_catalog_content() {
    let entries = permission_catalog()
        .iter()
        .map(|item| item.as_str())
        .collect::<Vec<_>>();
    let expected = test_catalog_fingerprint(&entries);

    assert_eq!(permission_catalog_version(), expected);

    let mut mutated = entries.clone();
    mutated[0] = "all:permissions:changed";
    assert_ne!(test_catalog_fingerprint(&mutated), expected);
}

fn test_catalog_fingerprint(entries: &[&str]) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for code in entries {
        hash = fnv1a_update(hash, code.as_bytes(), FNV_PRIME);
        hash = fnv1a_update(hash, b"\x1e", FNV_PRIME);
    }

    format!("{hash:016x}")
}

fn fnv1a_update(mut hash: u64, bytes: &[u8], prime: u64) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(prime);
    }
    hash
}

#[sqlx::test(migrations = "./migrations")]
async fn permission_table_after_migrations_should_match_permission_node_enum(pool: sqlx::PgPool) {
    let repository = PolicyRepository::new(pool);

    let mut from_db = repository
        .list_permission_codes()
        .await
        .expect("读取 sys_permission 失败");
    from_db.sort_unstable();

    let mut from_enum: Vec<String> = PermissionNode::ALL
        .into_iter()
        .map(|item| item.as_str().to_string())
        .collect();
    from_enum.sort_unstable();

    assert_eq!(
        from_db, from_enum,
        "迁移中的 sys_permission 与 PermissionNode::ALL 不一致"
    );
}
