use uuid::Uuid;

use super::scope::{ensure_scope_all_only, ensure_scope_target_user, parse_scope_rule, Scope};

#[test]
fn parse_scope_should_support_all() {
    let current_user_id = Uuid::new_v4();

    let scope = parse_scope_rule(Some("ALL"), current_user_id).expect("ALL 应可解析");

    assert_eq!(scope, Scope::All);
}

#[test]
fn parse_scope_should_translate_self_to_current_user_id() {
    let current_user_id = Uuid::new_v4();

    let scope = parse_scope_rule(Some("SELF"), current_user_id).expect("SELF 应可解析");

    assert_eq!(scope, Scope::Id(current_user_id));
}

#[test]
fn parse_scope_should_support_explicit_id() {
    let current_user_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let scope = parse_scope_rule(Some(&format!("ID:{target_id}")), current_user_id)
        .expect("ID:<uuid> 应可解析");

    assert_eq!(scope, Scope::Id(target_id));
}

#[test]
fn parse_scope_should_fail_closed_when_scope_invalid() {
    let current_user_id = Uuid::new_v4();

    let err =
        parse_scope_rule(Some("DEPT:SELF"), current_user_id).expect_err("非法 scope 应返回拒绝");

    assert_eq!(err.error_code(), 2002);
}

#[test]
fn scope_current_user_should_only_allow_current_user_target() {
    let current_user_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    let scope = Scope::Id(current_user_id);

    ensure_scope_target_user(scope, current_user_id).expect("SELF 应允许操作当前用户");
    let err = ensure_scope_target_user(scope, other_user_id).expect_err("SELF 应拒绝其他用户");

    assert_eq!(err.error_code(), 2002);
}

#[test]
fn scope_target_user_should_only_allow_bound_id_target() {
    let allowed_user_id = Uuid::new_v4();
    let blocked_user_id = Uuid::new_v4();
    let scope = Scope::Id(allowed_user_id);

    ensure_scope_target_user(scope, allowed_user_id).expect("ID scope 应允许绑定用户");
    let err =
        ensure_scope_target_user(scope, blocked_user_id).expect_err("ID scope 应拒绝非绑定用户");

    assert_eq!(err.error_code(), 2002);
}

#[test]
fn scope_all_only_should_reject_self_or_id_scope() {
    let current_user_id = Uuid::new_v4();

    ensure_scope_all_only(Scope::All).expect("ALL-only 接口应允许 ALL");
    let err =
        ensure_scope_all_only(Scope::Id(current_user_id)).expect_err("ALL-only 接口应拒绝 SELF/ID");

    assert_eq!(err.error_code(), 2002);
}
