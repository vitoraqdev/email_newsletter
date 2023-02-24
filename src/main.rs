use email_newsletter::configuration::Settings;
use email_newsletter::startup::run;
use email_newsletter::telemetry::{get_subscriber, set_subscriber};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("email_newsletter".into(), "info".into(), std::io::stdout);
    set_subscriber(subscriber);
    let configuration = Settings::default();
    let connection_pool =
        PgPool::connect(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind to port 8000.");
    run(listener, connection_pool)?.await
}
