use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub app: AppSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(serde::Deserialize)]
pub struct AppSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    port: u16,
    host: String,
}

#[derive(serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub auth_token: Secret<String>,
}

pub enum Environment {
    Local,
    Production,
}
impl Environment {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => return Ok(Self::Local),
            "production" => return Ok(Self::Production),
            other => {
                return Err(format!(
                    "{} is not a supported environment\nUse either `local` or `production`",
                    other
                ))
            }
        }
    }
}

impl Settings {
    pub fn get() -> Result<Self, config::ConfigError> {
        let base = std::env::current_dir().expect("Failed to determine current Directory");
        let config_path = base.join("config");
        let app_env: Environment = std::env::var("APP_ENV")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse APP_ENV");
        let env_file_name = format!("{}.toml", app_env.as_str());

        let settings = config::Config::builder()
            .add_source(config::File::from(config_path.join("base.toml")))
            .add_source(config::File::from(config_path.join(env_file_name)))
            .add_source(
                config::Environment::with_prefix("App")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        settings.try_deserialize()
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.app.host, self.app.port)
    }
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, ()> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}
