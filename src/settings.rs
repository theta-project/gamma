use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use tracing::Level;

/// Gamma configuration
/// By default this is read from `./gamma.toml`, and from environment variables starting with `APP_`
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub db: DatabaseSettings,

    /// The ip address to bind to, defaults to `127.0.0.1`
    /// Environment Variable: `APP__IP`
    #[serde(default = "default_ip")]
    pub ip: String,

    /// The port bind to, defaults to `8080`
    /// Environment Variable: `APP__PORT`
    #[serde(default = "default_port")]
    pub port: u16,

    /// Log level to use
    /// One of `error warn info debug trace`
    /// Environment Variable: `APP__LOG_LEVEL`
    #[serde(default = "default_log_level")]
    log_level: String,

    /// Settings for exporting to OpenTelemetry through OTLP
    /// Defaults to disabled
    #[serde(default)]
    pub telem: Option<TelemSettings>,
}

/// Settings related to redis and mysql
#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    /// Connection URL For Redis, eg `redis://host:port/num`
    /// Environment Variable: `APP__DB__REDIS_URL`
    pub redis_url: String,

    /// Connection URL For MySQL, eg `mysql://user:password@host:port/dbname`
    /// Environment Variable: `APP__DB__MYSQL_URL`
    pub mysql_url: String,
}

/// Settings for exporting to OpenTelemetry through OTLP
#[derive(Debug, Deserialize)]
pub struct TelemSettings {
    /// The GRPC OTLP endpoint
    pub endpoint: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::with_name("gamma.toml").required(false))
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()
    }

    pub fn log_level(&self) -> Level {
        match self.log_level.as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "error" => Level::ERROR,
            "trace" => Level::TRACE,
            "warn" => Level::WARN,
            _ => panic!("invalid log level {}", self.log_level),
        }
    }
}

fn default_ip() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_log_level() -> String {
    "info".to_owned()
}
