use email_newsletter::configuration::get_configuration;
use email_newsletter::startup::run;
use email_newsletter::telemetry::{get_subscriber, set_subscriber};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("email_newsletter".into(), "info".into(), std::io::stdout);
    set_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to get configuration.");
    tracing::info!(
        "Postgres URL: {}",
        configuration.database.connection_string().expose_secret()
    );
    // let connection_pool = PgPoolOptions::new()
    //     .acquire_timeout(std::time::Duration::from_secs(2))
    //     .connect_lazy(configuration.database.connection_string().expose_secret())
    //     .expect("Failed to connect to Postgres.");
    let connection_pool =
        PgPool::connect_lazy(configuration.database.connection_string().expose_secret())
            .expect("Failed to create Postgres connection pool.");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)
        .unwrap_or_else(|_| panic!("Failed to bind to port {}.", configuration.application.port));
    run(listener, connection_pool)?.await
}
