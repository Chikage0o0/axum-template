use super::*;

use serde_json::Value;

use crate::modules::authorization::permission::permission_catalog_version_for_codes;

#[sqlx::test(migrations = "./migrations")]
async fn permission_nodes_dictionary_should_require_auth(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool).await;

    let response = request_json(
        &server,
        Method::GET,
        "/api/v1/authorization/permission-nodes",
        None,
        None,
        None,
    )
    .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    let body = response.json::<Value>();
    assert_eq!(body.get("code").and_then(Value::as_u64), Some(1001));
}

#[sqlx::test(migrations = "./migrations")]
async fn permission_nodes_dictionary_should_require_permission(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("perm_dict_denied_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "PermDictDeniedPwd#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    let (token, _) = login_and_get_tokens(&server, &username, password).await;
    let response = request_json(
        &server,
        Method::GET,
        "/api/v1/authorization/permission-nodes",
        Some(&token),
        None,
        None,
    )
    .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    let body = response.json::<Value>();
    assert_eq!(body.get("code").and_then(Value::as_u64), Some(2002));

    cleanup_test_users(&pool, &[user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn permission_nodes_dictionary_should_return_builtin_nodes_when_policy_allows(
    pool: sqlx::PgPool,
) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("perm_dict_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "PermDictPwd#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    let _policy_id = insert_test_policy(
        &pool,
        "USER",
        &user_id.to_string(),
        "authorization:permission-nodes:view",
        "ALLOW",
        "ALL",
        50,
    )
    .await;

    let (token, _) = login_and_get_tokens(&server, &username, password).await;
    let response = request_json(
        &server,
        Method::GET,
        "/api/v1/authorization/permission-nodes",
        Some(&token),
        None,
        None,
    )
    .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body = response.json::<Value>();

    let items = body
        .get("items")
        .and_then(Value::as_array)
        .expect("字典接口应返回 items 数组");
    assert!(!items.is_empty(), "权限字典不应为空");

    let users_list_item = items
        .iter()
        .find(|item| item.get("code").and_then(Value::as_str) == Some("users:list"));
    let users_list_item = users_list_item.expect("应包含 users:list 节点");
    assert_eq!(
        users_list_item.get("name").and_then(Value::as_str),
        Some("List Users")
    );
    assert_eq!(
        users_list_item.get("description").and_then(Value::as_str),
        Some("查看用户列表")
    );
    assert_eq!(
        users_list_item.get("module").and_then(Value::as_str),
        Some("users")
    );

    let version = body
        .get("version")
        .and_then(Value::as_str)
        .expect("字典接口应返回 version 字段");
    let expected_version = permission_catalog_version_for_codes(
        items
            .iter()
            .filter_map(|item| item.get("code").and_then(Value::as_str)),
    );
    assert_eq!(version, expected_version);

    cleanup_test_users(&pool, &[user_id]).await;
}
