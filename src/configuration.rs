use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub app: AppSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(serde::Deserialize)]
pub struct AppSettings {
    port: u16,
    host: String,
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
            .build()?;

        settings.try_deserialize()
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.app.host, self.app.port)
    }
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn no_db_connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}
