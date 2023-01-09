use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

/// Gamma configuration
/// By default this is read from `./gamma.toml`, and from environment variables starting with `APP_`
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub db: DatabaseSettings,

    #[serde(default = "default_ip")]
    pub ip: String,

    #[serde(default = "default_port")]
    pub port: u16,
}

/// Settings related to redis and mysql
#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    /// Connection URL For Redis, eg `redis://host:port/num`
    /// Environment Variable: `APP_DB_REDIS_URL`
    pub redis_url: String,

    /// Connection URL For MySQL, eg `mysql://user:password@host:port/dbname`
    /// Environment Variable: `APP_DB_MYSQL_URL`
    pub mysql_url: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::with_name("gamma.toml").required(false))
            .add_source(Environment::with_prefix("APP"))
            .build()?
            .try_deserialize()
    }
}

fn default_ip() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}
