use interlock::{app::App, models::users};
use insta::{assert_debug_snapshot, with_settings};
use loco_rs::testing::prelude::*;
use rstest::rstest;
use serial_test::serial;

use super::prepare_data;

// TODO: see how to dedup / extract this to app-local test utils
// not to framework, because that would require a runtime dep on insta
macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("auth_request");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn can_register() {
    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        let email = "test@loco.com";
        let payload = serde_json::json!({
            "name": "loco",
            "email": email,
            "password": "12341234"
        });

        let response = request.post("/api/auth/register").json(&payload).await;
        assert_eq!(
            response.status_code(),
            200,
            "Register request should succeed"
        );
        let saved_user = users::Model::find_by_email(&ctx.db, email).await;

        with_settings!({
            filters => cleanup_user_model()
        }, {
            assert_debug_snapshot!(saved_user);
        });

        let deliveries = ctx.mailer.unwrap().deliveries();
        assert_eq!(deliveries.count, 1, "Exactly one email should be sent");

        // with_settings!({
        //     filters => cleanup_email()
        // }, {
        //     assert_debug_snapshot!(ctx.mailer.unwrap().deliveries());
        // });
    })
    .await;
}

#[rstest]
#[case("login_with_valid_password", "12341234")]
#[case("login_with_invalid_password", "invalid-password")]
#[tokio::test]
#[serial]
async fn can_login_with_verify(#[case] test_name: &str, #[case] password: &str) {
    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        let email = "test@loco.com";
        let register_payload = serde_json::json!({
            "name": "loco",
            "email": email,
            "password": "12341234"
        });

        //Creating a new user
        let register_response = request
            .post("/api/auth/register")
            .json(&register_payload)
            .await;

        assert_eq!(
            register_response.status_code(),
            200,
            "Register request should succeed"
        );

        let user = users::Model::find_by_email(&ctx.db, email).await.unwrap();
        let email_verification_token = user
            .email_verification_token
            .expect("Email verification token should be generated");
        request
            .get(&format!("/api/auth/verify/{email_verification_token}"))
            .await;

        //verify user request
        let response = request
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .await;

        // Make sure email_verified_at is set
        let user = users::Model::find_by_email(&ctx.db, email)
            .await
            .expect("Failed to find user by email");

        assert!(
            user.email_verified_at.is_some(),
            "Expected the email to be verified, but it was not. User: {:?}",
            user
        );

        with_settings!({
            filters => cleanup_user_model()
        }, {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()));
        });
    })
    .await;
}

#[tokio::test]
#[serial]
async fn login_with_un_existing_email() {
    configure_insta!();

    request::<App, _, _>(|request, _ctx| async move {

        let login_response = request
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "email": "un_existing@loco.rs",
                "password":  "1234"
            }))
            .await;

        assert_eq!(login_response.status_code(), 401, "Login request should return 401");
        login_response.assert_json(&serde_json::json!({"error": "unauthorized", "description": "You do not have permission to access this resource"}));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_login_without_verify() {
    configure_insta!();

    request::<App, _, _>(|request, _ctx| async move {
        let email = "test@loco.com";
        let password = "12341234";
        let register_payload = serde_json::json!({
            "name": "loco",
            "email": email,
            "password": password
        });

        //Creating a new user
        let register_response = request
            .post("/api/auth/register")
            .json(&register_payload)
            .await;

        assert_eq!(
            register_response.status_code(),
            200,
            "Register request should succeed"
        );

        //verify user request
        let login_response = request
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .await;

        assert_eq!(
            login_response.status_code(),
            200,
            "Login request should succeed"
        );

        with_settings!({
            filters => cleanup_user_model()
        }, {
            assert_debug_snapshot!(login_response.text());
        });
    })
    .await;
}

#[tokio::test]
#[serial]
async fn invalid_verification_token() {
    configure_insta!();

    request::<App, _, _>(|request, _ctx| async move {
        let response = request.get("/api/auth/verify/invalid-token").await;

        assert_eq!(response.status_code(), 401, "Verify request should reject");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_reset_password() {
    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        let login_data = prepare_data::init_user_login(&request, &ctx).await;

        let forgot_payload = serde_json::json!({
            "email": login_data.user.email,
        });
        let forget_response = request.post("/api/auth/forgot").json(&forgot_payload).await;
        assert_eq!(
            forget_response.status_code(),
            200,
            "Forget request should succeed"
        );

        let user = users::Model::find_by_email(&ctx.db, &login_data.user.email)
            .await
            .expect("Failed to find user by email");

        assert!(
            user.reset_token.is_some(),
            "Expected reset_token to be set, but it was None. User: {user:?}"
        );
        assert!(
            user.reset_sent_at.is_some(),
            "Expected reset_sent_at to be set, but it was None. User: {user:?}"
        );

        let new_password = "new-password";
        let reset_payload = serde_json::json!({
            "token": user.reset_token,
            "password": new_password,
        });

        let reset_response = request.post("/api/auth/reset").json(&reset_payload).await;
        assert_eq!(
            reset_response.status_code(),
            200,
            "Reset password request should succeed"
        );

        let user = users::Model::find_by_email(&ctx.db, &user.email)
            .await
            .unwrap();

        assert!(user.reset_token.is_none());
        assert!(user.reset_sent_at.is_none());

        assert_debug_snapshot!(reset_response.text());

        let login_response = request
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "email": user.email,
                "password": new_password
            }))
            .await;

        assert_eq!(
            login_response.status_code(),
            200,
            "Login request should succeed"
        );

        let deliveries = ctx.mailer.unwrap().deliveries();
        assert_eq!(deliveries.count, 2, "Exactly one email should be sent");
        // with_settings!({
        //     filters => cleanup_email()
        // }, {
        //     assert_debug_snapshot!(deliveries.messages);
        // });
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_get_current_user() {
    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .get("/api/auth/current")
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "Current request should succeed"
        );

        with_settings!({
            filters => cleanup_user_model()
        }, {
            assert_debug_snapshot!((response.status_code(), response.text()));
        });
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_auth_with_magic_link() {
    configure_insta!();
    request::<App, _, _>(|request, ctx| async move {
        seed::<App>(&ctx).await.unwrap();

        let payload = serde_json::json!({
            "email": "user1@example.com",
        });
        let response = request.post("/api/auth/magic-link").json(&payload).await;
        assert_eq!(
            response.status_code(),
            200,
            "Magic link request should succeed"
        );

        let deliveries = ctx.mailer.unwrap().deliveries();
        assert_eq!(deliveries.count, 1, "Exactly one email should be sent");

        // let redact_token = format!("[a-zA-Z0-9]{{{}}}", users::MAGIC_LINK_LENGTH);
        // with_settings!({
        //      filters => {
        //          let mut combined_filters = cleanup_email().clone();
        //         combined_filters.extend(vec![(r"(\\r\\n|=\\r\\n)", ""), (redact_token.as_str(), "[REDACT_TOKEN]") ]);
        //         combined_filters
        //     }
        // }, {
        //     assert_debug_snapshot!(deliveries.messages);
        // });

        let user = users::Model::find_by_email(&ctx.db, "user1@example.com")
            .await
            .expect("User should be found");

        let magic_link_token = user
            .magic_link_token
            .expect("Magic link token should be generated");
        let magic_link_response = request
            .get(&format!("/api/auth/magic-link/{magic_link_token}"))
            .await;
        assert_eq!(
            magic_link_response.status_code(),
            200,
            "Magic link authentication should succeed"
        );

        with_settings!({
            filters => cleanup_user_model()
        }, {
            assert_debug_snapshot!(magic_link_response.text());
        });
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_reject_invalid_email() {
    configure_insta!();
    request::<App, _, _>(|request, _ctx| async move {
        let invalid_email = "user1@temp-mail.com";
        let payload = serde_json::json!({
            "email": invalid_email,
        });
        let response = request.post("/api/auth/magic-link").json(&payload).await;
        assert_eq!(
            response.status_code(),
            400,
            "Expected request with invalid email '{invalid_email}' to be blocked, but it was allowed."
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_reject_invalid_magic_link_token() {
    configure_insta!();
    request::<App, _, _>(|request, ctx| async move {
        seed::<App>(&ctx).await.unwrap();

        let magic_link_response = request.get("/api/auth/magic-link/invalid-token").await;
        assert_eq!(
            magic_link_response.status_code(),
            401,
            "Magic link authentication should be rejected"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_resend_verification_email() {
    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        let email = "test@loco.com";
        let payload = serde_json::json!({
            "name": "loco",
            "email": email,
            "password": "12341234"
        });

        let response = request.post("/api/auth/register").json(&payload).await;
        assert_eq!(
            response.status_code(),
            200,
            "Register request should succeed"
        );

        let resend_payload = serde_json::json!({ "email": email });

        let resend_response = request
            .post("/api/auth/resend-verification-mail")
            .json(&resend_payload)
            .await;

        assert_eq!(
            resend_response.status_code(),
            200,
            "Resend verification email should succeed"
        );

        let deliveries = ctx.mailer.unwrap().deliveries();

        assert_eq!(
            deliveries.count, 2,
            "Two emails should have been sent: welcome and re-verification"
        );

        let user = users::Model::find_by_email(&ctx.db, email)
            .await
            .expect("User should exist");

        with_settings!({
            filters => cleanup_user_model()
        }, {
            assert_debug_snapshot!("resend_verification_user", user);
        });
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_resend_email_if_already_verified() {
    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        let email = "verified@loco.com";
        let payload = serde_json::json!({
            "name": "verified",
            "email": email,
            "password": "12341234"
        });

        request.post("/api/auth/register").json(&payload).await;

        // Verify user
        let user = users::Model::find_by_email(&ctx.db, email).await.unwrap();
        if let Some(token) = user.email_verification_token.clone() {
            request.get(&format!("/api/auth/verify/{token}")).await;
        }

        // Try resending verification email
        let resend_payload = serde_json::json!({ "email": email });

        let resend_response = request
            .post("/api/auth/resend-verification-mail")
            .json(&resend_payload)
            .await;

        assert_eq!(
            resend_response.status_code(),
            200,
            "Should return 200 even if already verified"
        );

        let deliveries = ctx.mailer.unwrap().deliveries();
        assert_eq!(
            deliveries.count, 1,
            "Only the original welcome email should be sent"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_auth_with_api_key_and_regenerate() {
    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        let api_key = user.user.api_key;

        // 1. Access current user with API Key
        let response = request
            .get("/api/auth/current")
            .add_header("X-API-Key", &api_key)
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "Current request with API Key should succeed"
        );

        // 2. Regenerate API Key (using existing JWT or API Key)
        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let regen_response = request
            .post("/api/auth/api-key")
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(
            regen_response.status_code(),
            200,
            "Regenerate API Key request should succeed"
        );

        let json: serde_json::Value = regen_response.json();
        let new_api_key = json["api_key"].as_str().unwrap().to_string();

        assert_ne!(api_key, new_api_key, "New API Key should be different");

        // 3. Verify old key fails
        let fail_response = request
            .get("/api/auth/current")
            .add_header("X-API-Key", &api_key)
            .await;

        assert_eq!(
            fail_response.status_code(),
            401,
            "Old API Key should fail"
        );

        // 4. Verify new key works
        let success_response = request
            .get("/api/auth/current")
            .add_header("X-API-Key", &new_api_key)
            .await;

        assert_eq!(
            success_response.status_code(),
            200,
            "New API Key should succeed"
        );

        with_settings!({
            filters => cleanup_user_model()
        }, {
            assert_debug_snapshot!((success_response.status_code(), success_response.text()));
        });
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_change_password() {
    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // 1. Setup user
        let user = prepare_data::init_user_login(&request, &ctx).await;

        // 2. Change password
        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);

        let change_password_payload = serde_json::json!({
            "old_password": "1234",
            "new_password": "new-password-123",
        });

        let response = request
            .post("/api/auth/change-password")
            .add_header(auth_key.clone(), auth_value.clone())
            .json(&change_password_payload)
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "Change password request should succeed"
        );

        // 3. Verify old password fails
        let login_old = request
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "email": user.user.email,
                "password": "1234"
            }))
            .await;

        assert_eq!(
            login_old.status_code(),
            401,
            "Login with old password should fail"
        );

        // 4. Verify new password works
        let login_new = request
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "email": user.user.email,
                "password": "new-password-123"
            }))
            .await;

        assert_eq!(
            login_new.status_code(),
            200,
            "Login with new password should succeed"
        );

        // 5. Test invalid old password
        // Login again to get token (or reuse old one if still valid - JWT usually is)
        let token = login_new.json::<serde_json::Value>()["token"].as_str().unwrap().to_string();
        let (auth_key, auth_value) = prepare_data::auth_header(&token);

        let invalid_payload = serde_json::json!({
            "old_password": "wrong-password",
            "new_password": "another-password"
        });

        let invalid_response = request
            .post("/api/auth/change-password")
            .add_header(auth_key, auth_value)
            .json(&invalid_payload)
            .await;

        assert_eq!(
            invalid_response.status_code(),
            401,
            "Change password with invalid old password should fail"
        );

        with_settings!({
            filters => cleanup_user_model()
        }, {
             assert_debug_snapshot!((response.status_code(), response.text()));
        });
    })
    .await;
}
