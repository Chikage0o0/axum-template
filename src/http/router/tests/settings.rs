use super::*;

use serde_json::Value;

#[sqlx::test(migrations = "./migrations")]
async fn get_settings_should_require_auth_and_return_runtime_config(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let anonymous_response =
        request_json(&server, Method::GET, "/api/v1/settings", None, None, None).await;
    assert_eq!(anonymous_response.status_code(), StatusCode::UNAUTHORIZED);
    let anonymous_error = anonymous_response.json::<Value>();
    assert_eq!(
        anonymous_error.get("code").and_then(Value::as_u64),
        Some(1001)
    );

    let username = format!("settings_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "SettingsPassword#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    let (token, _) = login_and_get_tokens(&server, &username, password).await;
    let response = request_json(
        &server,
        Method::GET,
        "/api/v1/settings",
        Some(&token),
        None,
        None,
    )
    .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let body = response.json::<Value>();

    assert!(
        body.get("app")
            .and_then(|app| app.get("check_interval_secs"))
            .and_then(Value::as_u64)
            .is_some(),
        "settings.app.check_interval_secs 应返回数字"
    );
    assert!(
        body.get("app")
            .and_then(|app| app.get("welcome_message"))
            .and_then(Value::as_str)
            .is_some(),
        "settings.app.welcome_message 应返回字符串"
    );
    assert!(
        body.get("integrations")
            .and_then(|integrations| integrations.get("example_api_base"))
            .and_then(Value::as_str)
            .is_some(),
        "settings.integrations.example_api_base 应返回字符串"
    );
    assert!(
        body.get("integrations")
            .and_then(|integrations| integrations.get("example_api_key_is_set"))
            .and_then(Value::as_bool)
            .is_some(),
        "settings.integrations.example_api_key_is_set 应返回布尔值"
    );

    cleanup_test_users(&pool, &[user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn non_admin_should_be_forbidden_to_patch_settings(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("settings_patch_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "SettingsPatchPassword#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    let (token, _) = login_and_get_tokens(&server, &username, password).await;
    let response = request_json(
        &server,
        Method::PATCH,
        "/api/v1/settings",
        Some(&token),
        None,
        Some(serde_json::json!({
            "app": {
                "welcome_message": "forbidden-update",
            }
        })),
    )
    .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    let body = response.json::<Value>();
    assert_eq!(body.get("code").and_then(Value::as_u64), Some(2002));

    cleanup_test_users(&pool, &[user_id]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn non_admin_with_settings_update_policy_should_patch_settings(pool: sqlx::PgPool) {
    let server = setup_user_management_test_app(pool.clone()).await;

    let username = format!("settings_policy_user_{}", Uuid::new_v4().simple());
    let email = format!("{username}@example.invalid");
    let password = "SettingsPolicyPassword#A123";
    let user_id = create_or_update_user_with_password(&pool, &username, &email, password).await;

    let _policy_id = insert_test_policy(
        &pool,
        "USER",
        &user_id.to_string(),
        "settings:update",
        "ALLOW",
        "ALL",
        50,
    )
    .await;

    let (token, _) = login_and_get_tokens(&server, &username, password).await;
    let message = format!("policy-updated-{}", Uuid::new_v4().simple());
    let response = request_json(
        &server,
        Method::PATCH,
        "/api/v1/settings",
        Some(&token),
        None,
        Some(serde_json::json!({
            "app": {
                "welcome_message": message,
            }
        })),
    )
    .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body = response.json::<Value>();
    assert!(
        body.get("app")
            .and_then(|app| app.get("welcome_message"))
            .and_then(Value::as_str)
            .is_some(),
        "返回体应包含 app.welcome_message"
    );

    cleanup_test_users(&pool, &[user_id]).await;
}
