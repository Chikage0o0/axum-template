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
        let request_id = response
            .header("x-request-id")
            .to_str()
            .expect("403 响应应包含有效 x-request-id")
            .to_string();
        let body = response.json::<Value>();
        assert_eq!(body.get("code").and_then(Value::as_u64), Some(2002));
        assert_eq!(
            body.get("request_id")
                .and_then(Value::as_str)
                .expect("403 错误体应包含 request_id"),
            request_id
        );
    }

    cleanup_test_users(&pool, &[normal_user_id, target_user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn non_admin_with_users_list_policy_should_access_user_list(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("users_list_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "UsersListPassword#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    let _policy_id = insert_test_policy(
        &pool,
        "USER",
        &user_id.to_string(),
        "users:list",
        "ALLOW",
        "ALL",
        50,
    )
    .await;

    let (token, _) = login_and_get_tokens(&server, &username, password).await;
    let response = request_json(
        &server,
        Method::GET,
        "/api/v1/users",
        Some(&token),
        None,
        None,
    )
    .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body = response.json::<Value>();
    assert!(body.is_array(), "users:list 应返回数组");

    cleanup_test_users(&pool, &[user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn explicit_user_deny_users_wildcard_should_override_admin_role(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let admin_password = "AdminPassword#A123";
    let admin_id = ensure_admin_user_with_password(&pool, admin_password).await;

    let _policy_id = insert_test_policy(
        &pool,
        "USER",
        &admin_id.to_string(),
        "users:*",
        "DENY",
        "ALL",
        100,
    )
    .await;

    let (admin_token, _) = login_and_get_tokens(&server, "admin", admin_password).await;
    let response = request_json(
        &server,
        Method::GET,
        "/api/v1/users",
        Some(&admin_token),
        None,
        None,
    )
    .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    let request_id = response
        .header("x-request-id")
        .to_str()
        .expect("403 响应应包含有效 x-request-id")
        .to_string();
    let body = response.json::<Value>();
    assert_eq!(body.get("code").and_then(Value::as_u64), Some(2002));
    assert_eq!(
        body.get("request_id")
            .and_then(Value::as_str)
            .expect("403 错误体应包含 request_id"),
        request_id
    );
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

#[sqlx::test(migrations = "./migrations")]
async fn admin_should_not_be_able_to_deactivate_or_delete_self(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let admin_password = "AdminPassword#A123";
    let admin_id = ensure_admin_user_with_password(&pool, admin_password).await;
    let (admin_token, _) = login_and_get_tokens(&server, "admin", admin_password).await;

    let deactivate_response = request_json(
        &server,
        Method::PATCH,
        &format!("/api/v1/users/{admin_id}"),
        Some(&admin_token),
        None,
        Some(json!({
            "is_active": false,
        })),
    )
    .await;
    assert_eq!(deactivate_response.status_code(), StatusCode::BAD_REQUEST);
    let deactivate_error = deactivate_response.json::<Value>();
    assert_eq!(
        deactivate_error.get("code").and_then(Value::as_u64),
        Some(1000)
    );

    let delete_response = request_json(
        &server,
        Method::DELETE,
        &format!("/api/v1/users/{admin_id}"),
        Some(&admin_token),
        None,
        None,
    )
    .await;
    assert_eq!(delete_response.status_code(), StatusCode::BAD_REQUEST);
    let delete_error = delete_response.json::<Value>();
    assert_eq!(delete_error.get("code").and_then(Value::as_u64), Some(1000));
}

#[sqlx::test(migrations = "./migrations")]
async fn restore_user_should_not_revive_old_session(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let admin_password = "AdminPassword#A123";
    ensure_admin_user_with_password(&pool, admin_password).await;

    let username = format!("restore_session_target_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "TargetPassword#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    let (old_access_token, old_refresh_cookie) =
        login_and_get_tokens(&server, &username, password).await;
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

    let me_response = request_json(
        &server,
        Method::GET,
        "/api/v1/users/me",
        Some(&old_access_token),
        None,
        None,
    )
    .await;
    assert_eq!(me_response.status_code(), StatusCode::UNAUTHORIZED);
    let me_error = me_response.json::<Value>();
    assert_eq!(me_error.get("code").and_then(Value::as_u64), Some(1001));

    let refresh_response = request_json(
        &server,
        Method::POST,
        "/api/v1/sessions/refresh",
        None,
        Some(&old_refresh_cookie),
        None,
    )
    .await;
    assert_eq!(refresh_response.status_code(), StatusCode::UNAUTHORIZED);
    let refresh_error = refresh_response.json::<Value>();
    assert_eq!(
        refresh_error.get("code").and_then(Value::as_u64),
        Some(1001)
    );

    cleanup_test_users(&pool, &[user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn username_should_not_conflict_with_other_user_email_or_phone(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let admin_password = "AdminPassword#A123";
    ensure_admin_user_with_password(&pool, admin_password).await;

    let owner_username = format!("owner_user_{}", Uuid::new_v4().simple());
    let owner_email = format!("{owner_username}@example.invalid");
    let owner_phone = format!("139{:08}", Uuid::new_v4().as_u128() % 100_000_000);
    let owner_id =
        create_or_update_user_with_password(&pool, &owner_username, &owner_email, "OwnerPwd#A123")
            .await;
    sqlx::query!(
        "UPDATE users SET phone = $2 WHERE id = $1",
        owner_id,
        owner_phone
    )
    .execute(&pool)
    .await
    .expect("写入 owner 手机号失败");

    let patch_target_username = format!("patch_target_{}", Uuid::new_v4().simple());
    let patch_target_email = format!("{patch_target_username}@example.invalid");
    let patch_target_id = create_or_update_user_with_password(
        &pool,
        &patch_target_username,
        &patch_target_email,
        "PatchTargetPwd#A123",
    )
    .await;

    let (admin_token, _) = login_and_get_tokens(&server, "admin", admin_password).await;

    let create_with_email_username = request_json(
        &server,
        Method::POST,
        "/api/v1/users",
        Some(&admin_token),
        None,
        Some(json!({
            "username": owner_email,
            "display_name": "conflict-email-username",
            "email": format!("new_email_{}@example.invalid", Uuid::new_v4().simple()),
        })),
    )
    .await;
    assert_eq!(
        create_with_email_username.status_code(),
        StatusCode::BAD_REQUEST
    );
    let create_email_error = create_with_email_username.json::<Value>();
    assert_eq!(
        create_email_error.get("code").and_then(Value::as_u64),
        Some(1000)
    );

    let create_with_phone_username = request_json(
        &server,
        Method::POST,
        "/api/v1/users",
        Some(&admin_token),
        None,
        Some(json!({
            "username": owner_phone,
            "display_name": "conflict-phone-username",
            "email": format!("new_phone_email_{}@example.invalid", Uuid::new_v4().simple()),
        })),
    )
    .await;
    assert_eq!(
        create_with_phone_username.status_code(),
        StatusCode::BAD_REQUEST
    );
    let create_phone_error = create_with_phone_username.json::<Value>();
    assert_eq!(
        create_phone_error.get("code").and_then(Value::as_u64),
        Some(1000)
    );

    let patch_conflict = request_json(
        &server,
        Method::PATCH,
        &format!("/api/v1/users/{patch_target_id}"),
        Some(&admin_token),
        None,
        Some(json!({
            "username": owner_email,
        })),
    )
    .await;
    assert_eq!(patch_conflict.status_code(), StatusCode::BAD_REQUEST);
    let patch_error = patch_conflict.json::<Value>();
    assert_eq!(patch_error.get("code").and_then(Value::as_u64), Some(1000));

    cleanup_test_users(&pool, &[owner_id, patch_target_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn username_should_reject_special_symbols_and_require_letters(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let admin_password = "AdminPassword#A123";
    ensure_admin_user_with_password(&pool, admin_password).await;
    let (admin_token, _) = login_and_get_tokens(&server, "admin", admin_password).await;

    let invalid_usernames = ["12345678", "alice@demo", "alice-001", "alice#001"];
    let mut created_ids = Vec::new();

    for username in invalid_usernames {
        let response = request_json(
            &server,
            Method::POST,
            "/api/v1/users",
            Some(&admin_token),
            None,
            Some(json!({
                "username": username,
                "display_name": format!("display-{username}"),
                "email": format!("u_{}@example.invalid", Uuid::new_v4().simple()),
            })),
        )
        .await;

        assert_eq!(
            response.status_code(),
            StatusCode::BAD_REQUEST,
            "username={username} 应被拒绝"
        );
        let body = response.json::<Value>();
        assert_eq!(body.get("code").and_then(Value::as_u64), Some(1000));
    }

    let valid_response = request_json(
        &server,
        Method::POST,
        "/api/v1/users",
        Some(&admin_token),
        None,
        Some(json!({
            "username": "alice_001",
            "display_name": "valid-user",
            "email": format!("valid_{}@example.invalid", Uuid::new_v4().simple()),
        })),
    )
    .await;
    assert_eq!(valid_response.status_code(), StatusCode::CREATED);
    let valid_body = valid_response.json::<Value>();
    if let Some(id) = valid_body
        .get("id")
        .and_then(Value::as_str)
        .and_then(|id| Uuid::parse_str(id).ok())
    {
        created_ids.push(id);
    }

    cleanup_test_users(&pool, &created_ids).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn user_can_patch_me_but_cannot_patch_others(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let user_a_name = format!("self_patch_user_a_{}", Uuid::new_v4().simple());
    let user_a_email = format!("{user_a_name}@example.invalid");
    let user_a_password = "SelfPatchPassword#A123";
    let user_a_id =
        create_or_update_user_with_password(&pool, &user_a_name, &user_a_email, user_a_password)
            .await;

    let user_b_name = format!("self_patch_user_b_{}", Uuid::new_v4().simple());
    let user_b_email = format!("{user_b_name}@example.invalid");
    let user_b_id =
        create_or_update_user_with_password(&pool, &user_b_name, &user_b_email, "UserBPwd#A123")
            .await;

    let (user_a_token, _) = login_and_get_tokens(&server, &user_a_name, user_a_password).await;

    let patch_me_response = request_json(
        &server,
        Method::PATCH,
        "/api/v1/users/me",
        Some(&user_a_token),
        None,
        Some(json!({
            "display_name": "Self Updated",
            "email": format!("me_updated_{}@example.invalid", Uuid::new_v4().simple()),
        })),
    )
    .await;
    assert_eq!(patch_me_response.status_code(), StatusCode::OK);
    let patch_me_body = patch_me_response.json::<Value>();
    assert_eq!(
        patch_me_body.get("id").and_then(Value::as_str),
        Some(user_a_id.to_string().as_str())
    );

    let patch_other_response = request_json(
        &server,
        Method::PATCH,
        &format!("/api/v1/users/{user_b_id}"),
        Some(&user_a_token),
        None,
        Some(json!({
            "display_name": "Should Not Update",
        })),
    )
    .await;
    assert_eq!(patch_other_response.status_code(), StatusCode::FORBIDDEN);
    let patch_other_body = patch_other_response.json::<Value>();
    assert_eq!(
        patch_other_body.get("code").and_then(Value::as_u64),
        Some(2002)
    );

    cleanup_test_users(&pool, &[user_a_id, user_b_id]).await;
}
