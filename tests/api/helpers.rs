use email_newsletter::configuration::{get_configuration, DatabaseSettings};
use email_newsletter::startup::{get_connection_pool, Application};
use email_newsletter::telemetry::set_subscriber;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
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
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let mut configuration = get_configuration().expect("Failed to get configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    configuration.application.port = 0;

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
    }

    // tracing::info!("Postgres URL: {:?}", configuration.database.with_db());
    // let connection_pool = configure_database(configuration.database).await;
    //
    // let sender_email = configuration
    //     .email_client
    //     .sender()
    //     .expect("Failed to parse sender email address.");
    // let timeout = configuration.email_client.timeout();
    // let email_client = EmailClient::new(
    //     configuration.email_client.base_url,
    //     sender_email,
    //     configuration.email_client.authorization_token,
    //     timeout,
    // );
    //
    // let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    // let port = listener.local_addr().unwrap().port();
    // let address = format!("http://{}:{}", configuration.application.host, port);
    // tracing::info!("Running server on: http://{}", address);
    //
    // let server =
    //     run(listener, connection_pool.clone(), email_client).expect("Failed to bind address.");
    // tokio::spawn(server);
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // create database
    let mut connection = PgConnection::connect_with(&config.without_db())
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
