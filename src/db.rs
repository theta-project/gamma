use anyhow::Result;
use redis::aio::ConnectionManager as RedisConnectionManager;
use tracing::{info, instrument};

use crate::settings::DatabaseSettings;

pub struct Databases {
    redis: RedisConnectionManager,
    pub mysql: mysql::Pool,
}

impl Databases {
    #[instrument(name = "connect_databases", skip(settings))]
    pub async fn new(settings: &DatabaseSettings) -> Result<Self> {
        dbg!(settings);
        info!("connecting to redis");
        let redis =
            RedisConnectionManager::new(redis::Client::open(settings.redis_url.as_str())?).await?;

        info!("connecting to mysql");
        let mysql = mysql::Pool::new(settings.mysql_url.as_str())?;

        Ok(Databases { redis, mysql })
    }

    pub fn redis(&self) -> RedisConnectionManager {
        // TODO: This is inefficient, we should really use a connection pool
        self.redis.clone()
    }
}
