use anyhow::Result;
use tracing::{info, instrument};

use crate::settings::DatabaseSettings;

#[derive(Debug)]
pub struct Databases {
    pub redis: redis::Client,
    pub mysql: mysql::Pool,
}

impl Databases {
    #[instrument(name = "connect_databases", skip(settings))]
    pub fn new(settings: &DatabaseSettings) -> Result<Self> {
        dbg!(settings);
        info!("connecting to redis");
        let redis = redis::Client::open(settings.redis_url.as_str())?;

        info!("connecting to mysql");
        let mysql = mysql::Pool::new(settings.mysql_url.as_str())?;

        Ok(Databases { redis, mysql })
    }
}
