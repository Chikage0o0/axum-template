use uuid::Uuid;

use super::scope::{ensure_users_write_scope, parse_users_scope_rule, UsersScope};

#[test]
fn parse_scope_should_support_all() {
    let current_user_id = Uuid::new_v4();

    let scope = parse_users_scope_rule(Some("ALL"), current_user_id).expect("ALL 应可解析");

    assert_eq!(scope, UsersScope::All);
}

#[test]
fn parse_scope_should_translate_self_to_current_user_id() {
    let current_user_id = Uuid::new_v4();

    let scope = parse_users_scope_rule(Some("SELF"), current_user_id).expect("SELF 应可解析");

    assert_eq!(scope, UsersScope::Id(current_user_id));
}

#[test]
fn parse_scope_should_support_explicit_id() {
    let current_user_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let scope = parse_users_scope_rule(Some(&format!("ID:{target_id}")), current_user_id)
        .expect("ID:<uuid> 应可解析");

    assert_eq!(scope, UsersScope::Id(target_id));
}

#[test]
fn parse_scope_should_fail_closed_when_scope_invalid() {
    let current_user_id = Uuid::new_v4();

    let err = parse_users_scope_rule(Some("DEPT:SELF"), current_user_id)
        .expect_err("非法 scope 应返回拒绝");

    assert_eq!(err.error_code(), 2002);
}

#[test]
fn write_scope_self_should_only_allow_current_user_target() {
    let current_user_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    let scope = UsersScope::Id(current_user_id);

    ensure_users_write_scope(scope, current_user_id).expect("SELF 应允许操作当前用户");
    let err = ensure_users_write_scope(scope, other_user_id).expect_err("SELF 应拒绝其他用户");

    assert_eq!(err.error_code(), 2002);
}

#[test]
fn write_scope_id_should_only_allow_bound_id_target() {
    let allowed_user_id = Uuid::new_v4();
    let blocked_user_id = Uuid::new_v4();
    let scope = UsersScope::Id(allowed_user_id);

    ensure_users_write_scope(scope, allowed_user_id).expect("ID scope 应允许绑定用户");
    let err =
        ensure_users_write_scope(scope, blocked_user_id).expect_err("ID scope 应拒绝非绑定用户");

    assert_eq!(err.error_code(), 2002);
}
