use super::*;

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::Value;

#[sqlx::test(migrations = "./migrations")]
async fn session_refresh_and_logout_should_revoke_current_session(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("session_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "SessionPassword#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    let (access_token, refresh_cookie) = login_and_get_tokens(&server, &username, password).await;

    let refresh_response = request_json(
        &server,
        Method::POST,
        "/api/v1/sessions/refresh",
        None,
        Some(&refresh_cookie),
        None,
    )
    .await;
    assert_eq!(refresh_response.status_code(), StatusCode::OK);
    let rotated_refresh_cookie = refresh_response.cookie("refresh_token");
    let rotated_refresh_pair = format!("refresh_token={}", rotated_refresh_cookie.value());

    let logout_response = request_json(
        &server,
        Method::DELETE,
        "/api/v1/sessions/current",
        Some(&access_token),
        None,
        None,
    )
    .await;
    assert_eq!(logout_response.status_code(), StatusCode::NO_CONTENT);
    let cleared_cookie = logout_response.cookie("refresh_token");
    assert!(
        cleared_cookie.value().is_empty(),
        "退出后 refresh_token cookie 应被清空"
    );

    let me_response = request_json(
        &server,
        Method::GET,
        "/api/v1/users/me",
        Some(&access_token),
        None,
        None,
    )
    .await;
    assert_eq!(me_response.status_code(), StatusCode::UNAUTHORIZED);
    let me_error = me_response.json::<Value>();
    assert_eq!(me_error.get("code").and_then(Value::as_u64), Some(1001));

    let refresh_after_logout = request_json(
        &server,
        Method::POST,
        "/api/v1/sessions/refresh",
        None,
        Some(&rotated_refresh_pair),
        None,
    )
    .await;
    assert_eq!(refresh_after_logout.status_code(), StatusCode::UNAUTHORIZED);
    let refresh_error = refresh_after_logout.json::<Value>();
    assert_eq!(
        refresh_error.get("code").and_then(Value::as_u64),
        Some(1001)
    );

    cleanup_test_users(&pool, &[user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn session_login_should_accept_username_email_and_phone_identifier(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("multi_login_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let phone = format!("138{:08}", Uuid::new_v4().as_u128() % 100_000_000);
    let password = "IdentifierPassword#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    sqlx::query!("UPDATE users SET phone = $2 WHERE id = $1", user_id, phone)
        .execute(&pool)
        .await
        .expect("写入测试手机号失败");

    for identifier in [&username, &email, &phone] {
        let login_response = request_json(
            &server,
            Method::POST,
            "/api/v1/sessions",
            None,
            None,
            Some(serde_json::json!({
                "identifier": identifier,
                "password": password,
            })),
        )
        .await;

        assert_eq!(
            login_response.status_code(),
            StatusCode::OK,
            "identifier={identifier} 应可登录"
        );

        let body = login_response.json::<Value>();
        assert!(
            body.get("token")
                .and_then(Value::as_str)
                .map(|v| !v.is_empty())
                .unwrap_or(false),
            "登录响应应包含 token"
        );
    }

    cleanup_test_users(&pool, &[user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn login_should_issue_role_from_db_not_username(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("admin_like_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "RoleByDbPassword#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    sqlx::query!("UPDATE users SET role = 'user' WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("设置测试用户角色失败");

    let (access_token, _) = login_and_get_tokens(&server, &username, password).await;
    let token_data = decode::<crate::api::auth::Claims>(
        &access_token,
        &DecodingKey::from_secret(E2E_JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .expect("解码 access token 失败");

    assert_eq!(token_data.claims.role, "user");

    cleanup_test_users(&pool, &[user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn role_change_should_invalidate_old_access_token(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("role_change_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "RoleChangePassword#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    sqlx::query!("UPDATE users SET role = 'user' WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("设置初始 role 失败");

    let (access_token, _) = login_and_get_tokens(&server, &username, password).await;

    sqlx::query!(
        "UPDATE users SET role = 'admin', auth_version = auth_version + 1 WHERE id = $1",
        user_id
    )
    .execute(&pool)
    .await
    .expect("更新 role 并 bump auth_version 失败");

    let me_response = request_json(
        &server,
        Method::GET,
        "/api/v1/users/me",
        Some(&access_token),
        None,
        None,
    )
    .await;
    assert_eq!(me_response.status_code(), StatusCode::UNAUTHORIZED);
    let body = me_response.json::<Value>();
    assert_eq!(body.get("code").and_then(Value::as_u64), Some(1001));

    cleanup_test_users(&pool, &[user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_refresh_with_same_cookie_should_only_succeed_once(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("refresh_race_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "RefreshRacePassword#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    let (_, refresh_cookie) = login_and_get_tokens(&server, &username, password).await;

    let req_a = request_json(
        &server,
        Method::POST,
        "/api/v1/sessions/refresh",
        None,
        Some(&refresh_cookie),
        None,
    );
    let req_b = request_json(
        &server,
        Method::POST,
        "/api/v1/sessions/refresh",
        None,
        Some(&refresh_cookie),
        None,
    );
    let (resp_a, resp_b) = tokio::join!(req_a, req_b);

    let statuses = [resp_a.status_code(), resp_b.status_code()];
    let success_count = statuses.iter().filter(|s| **s == StatusCode::OK).count();
    let unauthorized_count = statuses
        .iter()
        .filter(|s| **s == StatusCode::UNAUTHORIZED)
        .count();

    assert_eq!(success_count, 1, "并发刷新应仅成功一次");
    assert_eq!(unauthorized_count, 1, "并发刷新应有一次返回 401");

    cleanup_test_users(&pool, &[user_id]).await;
}
