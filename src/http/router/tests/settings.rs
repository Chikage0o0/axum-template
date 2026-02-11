use super::*;

use serde_json::Value;

#[tokio::test]
async fn get_settings_should_require_auth_and_return_runtime_config() {
    let Some((pool, server)) = setup_user_management_test_app().await else {
        return;
    };

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
