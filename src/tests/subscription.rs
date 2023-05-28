use crate::configuration::Settings;
use crate::domain::Subscriber;
use crate::tests::*;

use sqlx::PgPool;

const EMAIL: &str = "email=ursula_le_guin%40gmail.com";
const NAME: &str = "name=le%20guin";
const NAME_AND_EMAIL: &str = "name=le%20guin&email=ursula_le_guin%40gmail.com";

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = NAME_AND_EMAIL;
    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subsribe_returns_400_when_data_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        (NAME, "missing the email"),
        (EMAIL, "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(400, response.status().as_u16(), "{}", error_message)
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_when_fields_present_but_empty() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=le_guin%40gmail.com", "empty name"),
        ("name=le&email=", "empty email"),
        ("name=Ursula&email=Not-An-Email", "Invalid Email"),
    ];

    for (body, descr) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(400, response.status().as_u16(), "{}", descr)
    }
}
