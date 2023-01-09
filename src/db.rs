use anyhow::Result;
use redis::{aio::ConnectionManager as RedisConnectionManager, AsyncCommands};
use tracing::{info, instrument};

use crate::settings::DatabaseSettings;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};

pub struct Databases {
    redis: RedisConnectionManager,
    pub mysql: MySqlPool,
}

impl Databases {
    #[instrument(name = "connect_databases", skip(settings))]
    pub async fn new(settings: &DatabaseSettings) -> Result<Self> {
        info!("connecting to redis");
        let mut redis =
            RedisConnectionManager::new(redis::Client::open(settings.redis_url.as_str())?).await?;

        // test to make sure the connection actually works
        let _: () = redis.acl_whoami().await?;

        info!("connecting to mysql");
        let mysql = MySqlPoolOptions::new()
            .connect(settings.mysql_url.as_str())
            .await?;

        Ok(Databases { redis, mysql })
    }

    pub fn redis(&self) -> RedisConnectionManager {
        // TODO: This is inefficient, we should really use a connection pool
        // like deadpool, mobc, etc
        self.redis.clone()
    }
}
