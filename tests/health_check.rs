use email_newsletter::configuration::{get_configuration, DatabaseSettings};
use email_newsletter::startup::run;
use email_newsletter::telemetry::set_subscriber;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber_name = "test".into();
    let default_filter_level = "info".into();
    match std::env::var("TEST_LOG").is_ok() {
        true => {
            let subscriber = email_newsletter::telemetry::get_subscriber(
                subscriber_name,
                default_filter_level,
                std::io::stdout,
            );
            set_subscriber(subscriber);
        }
        false => {
            let subscriber = email_newsletter::telemetry::get_subscriber(
                subscriber_name,
                default_filter_level,
                std::io::sink,
            );
            set_subscriber(subscriber);
        }
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// Launch the application in the background
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let mut configuration = get_configuration().expect("Failed to get configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    tracing::info!(
        "Postgres URL: {:?}",
        configuration.database.with_db()
    );
    let connection_pool = configure_database(configuration.database).await;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();
    let address = format!(
        "http://{}:{}",
        configuration.application.host, port
    );
    tracing::info!("Running server on: http://{}", address);

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address.");
    tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_database(config: DatabaseSettings) -> PgPool {
    // create database
    let mut connection =
        PgConnection::connect_with(&config.without_db())
            .await
            .expect("Could not connect to Postgres.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // migrate
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Could not connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database.");

    connection_pool
}

#[actix_web::test]
async fn health_check_works() {
    // Arrange
    let test_app = spawn_app().await;

    // Act
    let response = reqwest::get(format!("{}/health_check", test_app.address))
        .await
        .unwrap();

    // Assert
    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

#[actix_web::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Act
    let response = client
        .post(format!("{}/subscriptions", test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = reqwest::Client::new()
            .post(format!("{}/subscriptions", test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {error_message}.",
        );
    }
}
