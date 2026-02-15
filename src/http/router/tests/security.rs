use super::*;

use axum::response::IntoResponse;
use serde_json::{json, Value};

#[sqlx::test(migrations = "./migrations")]
async fn password_change_should_only_invalidate_current_user_sessions(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username_a = format!("e2e_user_a_{}", Uuid::new_v4().simple());
    let username_b = format!("e2e_user_b_{}", Uuid::new_v4().simple());
    let password_a_old = "OldPassword#A123";
    let password_a_new = "NewPassword#A123";
    let password_b = "StablePassword#B123";

    let user_a_id = create_user_with_password(&pool, &username_a, password_a_old).await;
    let user_b_id = create_user_with_password(&pool, &username_b, password_b).await;

    let (access_a_old, refresh_a_old) =
        login_and_get_tokens(&server, &username_a, password_a_old).await;
    let (access_b_old, refresh_b_old) =
        login_and_get_tokens(&server, &username_b, password_b).await;

    let patch_response = request_json(
        &server,
        Method::PATCH,
        "/api/v1/security/password",
        Some(&access_a_old),
        None,
        Some(json!({
            "current_password": password_a_old,
            "new_password": password_a_new,
        })),
    )
    .await;
    assert_eq!(patch_response.status_code(), StatusCode::NO_CONTENT);

    let me_a_old = request_json(
        &server,
        Method::GET,
        "/api/v1/users/me",
        Some(&access_a_old),
        None,
        None,
    )
    .await;
    assert_eq!(me_a_old.status_code(), StatusCode::UNAUTHORIZED);
    let me_a_old_body = me_a_old.json::<Value>();
    assert_eq!(
        me_a_old_body.get("code").and_then(Value::as_u64),
        Some(1001)
    );

    let refresh_a_old_response = request_json(
        &server,
        Method::POST,
        "/api/v1/sessions/refresh",
        None,
        Some(&refresh_a_old),
        None,
    )
    .await;
    assert_eq!(
        refresh_a_old_response.status_code(),
        StatusCode::UNAUTHORIZED
    );
    let refresh_a_old_body = refresh_a_old_response.json::<Value>();
    assert_eq!(
        refresh_a_old_body.get("code").and_then(Value::as_u64),
        Some(1001)
    );

    let me_b_old = request_json(
        &server,
        Method::GET,
        "/api/v1/users/me",
        Some(&access_b_old),
        None,
        None,
    )
    .await;
    assert_eq!(me_b_old.status_code(), StatusCode::OK);

    let refresh_b_old_response = request_json(
        &server,
        Method::POST,
        "/api/v1/sessions/refresh",
        None,
        Some(&refresh_b_old),
        None,
    )
    .await;
    assert_eq!(refresh_b_old_response.status_code(), StatusCode::OK);

    cleanup_test_users(&pool, &[user_a_id, user_b_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn internal_error_should_not_expose_raw_database_message(_pool: sqlx::PgPool) {
    let response = AppError::InternalError(
        "数据库执行失败: duplicate key value violates unique constraint users_email_key"
            .to_string(),
    )
    .into_response();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("读取错误响应 body 失败");
    let body: Value = serde_json::from_slice(&body_bytes).expect("解析错误响应 JSON 失败");

    assert_eq!(body.get("code").and_then(Value::as_u64), Some(5000));
    let message = body
        .get("message")
        .and_then(Value::as_str)
        .expect("错误响应缺少 message");
    let message_lower = message.to_ascii_lowercase();
    assert!(!message_lower.contains("sql"));
    assert!(!message_lower.contains("duplicate"));
    assert!(!message_lower.contains("constraint"));
}

#[sqlx::test(migrations = "./migrations")]
async fn auth_failures_should_keep_request_id_contract(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let invalid_token_response = request_json(
        &server,
        Method::GET,
        "/api/v1/users/me",
        Some("invalid.token.value"),
        None,
        None,
    )
    .await;

    assert_eq!(
        invalid_token_response.status_code(),
        StatusCode::UNAUTHORIZED
    );
    let invalid_token_header = invalid_token_response
        .header("x-request-id")
        .to_str()
        .expect("401 响应 x-request-id 应为有效字符串")
        .to_string();
    let invalid_token_body = invalid_token_response.json::<Value>();
    assert_eq!(
        invalid_token_body.get("code").and_then(Value::as_u64),
        Some(1001)
    );
    assert_eq!(
        invalid_token_body
            .get("request_id")
            .and_then(Value::as_str)
            .expect("401 错误体必须包含 request_id"),
        invalid_token_header
    );

    let username = format!("req_id_user_{}", Uuid::new_v4().simple());
    let password = "ReqIdPassword#A123";
    let user_id = create_user_with_password(&pool, &username, password).await;

    let (token, _) = login_and_get_tokens(&server, &username, password).await;
    let forbidden_response = request_json(
        &server,
        Method::PATCH,
        "/api/v1/settings",
        Some(&token),
        None,
        Some(json!({
            "app": {
                "welcome_message": "forbidden"
            }
        })),
    )
    .await;

    assert_eq!(forbidden_response.status_code(), StatusCode::FORBIDDEN);
    let forbidden_header = forbidden_response
        .header("x-request-id")
        .to_str()
        .expect("403 响应 x-request-id 应为有效字符串")
        .to_string();
    let forbidden_body = forbidden_response.json::<Value>();
    assert_eq!(
        forbidden_body.get("code").and_then(Value::as_u64),
        Some(2002)
    );
    assert_eq!(
        forbidden_body
            .get("request_id")
            .and_then(Value::as_str)
            .expect("403 错误体必须包含 request_id"),
        forbidden_header
    );

    cleanup_test_users(&pool, &[user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn client_request_id_should_be_passthrough_in_error_response(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("passthrough_user_{}", Uuid::new_v4().simple());
    let password = "PassThroughPassword#A123";
    let user_id = create_user_with_password(&pool, &username, password).await;
    let (token, _) = login_and_get_tokens(&server, &username, password).await;

    let request_id = format!("client-rid-{}", Uuid::new_v4().simple());
    let response = server
        .method(Method::PATCH, "/api/v1/settings")
        .add_header("accept", "application/json")
        .add_header("x-request-id", &request_id)
        .authorization_bearer(&token)
        .json(&json!({
            "app": {
                "welcome_message": "forbidden"
            }
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    assert_eq!(
        response.header("x-request-id").to_str().ok(),
        Some(request_id.as_str())
    );
    let body = response.json::<Value>();
    assert_eq!(body.get("code").and_then(Value::as_u64), Some(2002));
    assert_eq!(
        body.get("request_id").and_then(Value::as_str),
        Some(request_id.as_str())
    );

    cleanup_test_users(&pool, &[user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn explicit_user_deny_security_password_update_should_forbid_password_change(
    pool: sqlx::PgPool,
) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("pwd_deny_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let old_password = "OldPassword#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, old_password).await;

    let _policy_id = insert_test_policy(
        &pool,
        "USER",
        &user_id.to_string(),
        "users:update",
        "DENY",
        "SELF",
        100,
    )
    .await;

    let (token, _) = login_and_get_tokens(&server, &username, old_password).await;
    let response = request_json(
        &server,
        Method::PATCH,
        "/api/v1/security/password",
        Some(&token),
        None,
        Some(json!({
            "current_password": old_password,
            "new_password": "NewPassword#A123",
        })),
    )
    .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    let body = response.json::<Value>();
    assert_eq!(body.get("code").and_then(Value::as_u64), Some(2002));

    cleanup_test_users(&pool, &[user_id]).await;
}
