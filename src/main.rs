use email_newsletter::configuration::get_configuration;
use email_newsletter::startup::Application;
use email_newsletter::telemetry::{get_subscriber, set_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("email_newsletter".into(), "info".into(), std::io::stdout);
    set_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to get configuration.");
    let server = Application::build(configuration).await?;
    server.run_until_stopped().await?;
    Ok(())
}
