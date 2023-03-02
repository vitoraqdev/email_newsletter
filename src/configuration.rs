use secrecy::{ExposeSecret, Secret};
use sqlx::ConnectOptions;
// use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub require_ssl: bool,
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    // #[serde(deserialize_with="deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    // #[serde(deserialize_with="deserialize_number_from_string")]
    pub port: u16,
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    fn as_str(&self) -> &str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory.");
    let configuration_directory = base_path.join("configuration");

    // Check environment variable "APP_ENVIRONMENT"
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        // If not set, then default as local
        .unwrap_or_else(|_| "local".into())
        .try_into()
        // If mistyped, panic
        .expect("Failed to parse APP_ENVIRONMENT.");

    // Check base.yaml then
    // If environment is local, then check local.yaml, if production, then check production.yaml
    let settings = config::Config::builder()
        .add_source(config::File::from(configuration_directory.join("base")))
        .add_source(config::File::from(
            configuration_directory.join(environment.as_str()),
        ))
        // Override with environment variables
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()
        .expect("Failed to load configuration files.");

    settings.try_deserialize()
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = match self.require_ssl {
            true => PgSslMode::Require,
            false => PgSslMode::Prefer,
        };

        PgConnectOptions::new()
            .ssl_mode(ssl_mode)
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}
