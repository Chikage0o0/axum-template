use super::*;

use serde_json::{json, Value};

#[sqlx::test(migrations = "./migrations")]
async fn delete_user_should_soft_delete_and_hide_from_default_list(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let admin_password = "AdminPassword#A123";
    ensure_admin_user_with_password(&pool, admin_password).await;

    let victim_username = format!("soft_delete_target_{}", Uuid::new_v4().simple());
    let victim_email = format!("{victim_username}@example.invalid");
    let victim_id = create_or_update_user_with_password(
        &pool,
        &victim_username,
        &victim_email,
        "TargetPassword#A123",
    )
    .await;

    let (admin_token, _) = login_and_get_tokens(&server, "admin", admin_password).await;

    let delete_response = request_json(
        &server,
        Method::DELETE,
        &format!("/api/v1/users/{victim_id}"),
        Some(&admin_token),
        None,
        None,
    )
    .await;
    assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);

    let list_response = request_json(
        &server,
        Method::GET,
        "/api/v1/users",
        Some(&admin_token),
        None,
        None,
    )
    .await;
    assert_eq!(list_response.status_code(), StatusCode::OK);
    let users = list_response.json::<Value>();
    let users = users.as_array().expect("用户列表响应体应为 JSON 数组");
    let victim_visible = users
        .iter()
        .any(|user| user.get("id").and_then(Value::as_str) == Some(victim_id.to_string().as_str()));
    assert!(!victim_visible, "默认列表不应包含已逻辑删除用户");

    cleanup_test_users(&pool, &[victim_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn deleted_user_email_should_be_reusable(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let admin_password = "AdminPassword#A123";
    ensure_admin_user_with_password(&pool, admin_password).await;

    let reusable_email = format!("reusable_{}@example.invalid", Uuid::new_v4().simple());
    let user_a_username = format!("reuse_user_a_{}", Uuid::new_v4().simple());
    let user_a_id = create_or_update_user_with_password(
        &pool,
        &user_a_username,
        &reusable_email,
        "UserAPassword#A123",
    )
    .await;

    let (admin_token, _) = login_and_get_tokens(&server, "admin", admin_password).await;

    let delete_response = request_json(
        &server,
        Method::DELETE,
        &format!("/api/v1/users/{user_a_id}"),
        Some(&admin_token),
        None,
        None,
    )
    .await;
    assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);

    let user_b_username = format!("reuse_user_b_{}", Uuid::new_v4().simple());
    let create_response = request_json(
        &server,
        Method::POST,
        "/api/v1/users",
        Some(&admin_token),
        None,
        Some(json!({
            "username": user_b_username,
            "display_name": "Reusable Email User",
            "email": reusable_email,
        })),
    )
    .await;
    assert_eq!(create_response.status_code(), StatusCode::CREATED);
    let created = create_response.json::<Value>();
    let user_b_id = created
        .get("id")
        .and_then(Value::as_str)
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("创建用户响应应包含合法 id");

    cleanup_test_users(&pool, &[user_a_id, user_b_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn non_admin_should_forbidden_on_user_management_routes(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("normal_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "NormalUserPassword#A123";
    let normal_user_id =
        create_or_update_user_with_password(&pool, &username, &email, password).await;

    let target_username = format!("target_user_{}", Uuid::new_v4().simple());
    let target_email = format!("{target_username}@example.invalid");
    let target_user_id = create_or_update_user_with_password(
        &pool,
        &target_username,
        &target_email,
        "TargetPassword#A123",
    )
    .await;

    let (token, _) = login_and_get_tokens(&server, &username, password).await;

    let routes = vec![
        (Method::GET, "/api/v1/users".to_string(), None),
        (
            Method::POST,
            "/api/v1/users".to_string(),
            Some(json!({
                "username": format!("created_by_normal_{}", Uuid::new_v4().simple()),
                "display_name": "should fail",
                "email": format!("create_by_normal_{}@example.invalid", Uuid::new_v4().simple()),
            })),
        ),
        (
            Method::PATCH,
            format!("/api/v1/users/{target_user_id}"),
            Some(json!({
                "display_name": "updated by normal",
            })),
        ),
        (
            Method::DELETE,
            format!("/api/v1/users/{target_user_id}"),
            None,
        ),
        (
            Method::POST,
            format!("/api/v1/users/{target_user_id}/restore"),
            None,
        ),
    ];

    for (method, uri, body) in routes {
        let response = request_json(&server, method, &uri, Some(&token), None, body).await;
        assert_eq!(
            response.status_code(),
            StatusCode::FORBIDDEN,
            "非管理员访问 {uri} 应返回 403"
        );
        let body = response.json::<Value>();
        assert_eq!(body.get("code").and_then(Value::as_u64), Some(2002));
    }

    cleanup_test_users(&pool, &[normal_user_id, target_user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn restore_user_should_reactivate_soft_deleted_user(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let admin_password = "AdminPassword#A123";
    ensure_admin_user_with_password(&pool, admin_password).await;

    let username = format!("restore_target_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let user_id =
        create_or_update_user_with_password(&pool, &username, &email, "TargetPassword#A123").await;

    let (admin_token, _) = login_and_get_tokens(&server, "admin", admin_password).await;

    let delete_response = request_json(
        &server,
        Method::DELETE,
        &format!("/api/v1/users/{user_id}"),
        Some(&admin_token),
        None,
        None,
    )
    .await;
    assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);

    let restore_response = request_json(
        &server,
        Method::POST,
        &format!("/api/v1/users/{user_id}/restore"),
        Some(&admin_token),
        None,
        None,
    )
    .await;
    assert_eq!(restore_response.status_code(), StatusCode::OK);
    let restored = restore_response.json::<Value>();
    assert_eq!(
        restored.get("id").and_then(Value::as_str),
        Some(user_id.to_string().as_str())
    );

    let list_response = request_json(
        &server,
        Method::GET,
        "/api/v1/users?include_deleted=true",
        Some(&admin_token),
        None,
        None,
    )
    .await;
    assert_eq!(list_response.status_code(), StatusCode::OK);
    let users = list_response.json::<Value>();
    let users = users.as_array().expect("用户列表响应体应为 JSON 数组");
    let restored_user = users
        .iter()
        .find(|user| user.get("id").and_then(Value::as_str) == Some(user_id.to_string().as_str()))
        .expect("include_deleted 列表应包含恢复后的用户");
    assert_eq!(
        restored_user.get("is_active").and_then(Value::as_bool),
        Some(true)
    );

    cleanup_test_users(&pool, &[user_id]).await;
}
