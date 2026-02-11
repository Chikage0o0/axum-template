use super::*;

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
