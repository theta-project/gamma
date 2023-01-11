use std::{str::FromStr, time::Duration};

use crate::errors::InternalError;
use anyhow::Result;
use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use tracing::{debug_span, info, info_span, instrument, Instrument};

use crate::settings::DatabaseSettings;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    ConnectOptions, MySqlPool,
};

pub type PoolConnection = sqlx::pool::PoolConnection<sqlx::MySql>;

pub struct Databases {
    redis: RedisPool,
    mysql: MySqlPool,
}

impl Databases {
    #[instrument(name = "connect_databases", skip(settings))]
    pub async fn new(settings: &DatabaseSettings) -> Result<Self> {
        info!("connecting to redis");
        let redis = {
            let cfg = deadpool_redis::Config::from_url(settings.redis_url.as_str());
            cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?
        };

        // test to make sure the connection actually works
        redis
            .get()
            .await?
            .acl_whoami()
            .instrument(info_span!("test_redis_conn"))
            .await?;

        info!("connecting to mysql");
        let mut mysql = MySqlConnectOptions::from_str(&settings.mysql_url)?;
        mysql
            .log_statements(tracing::log::LevelFilter::Debug)
            .log_slow_statements(tracing::log::LevelFilter::Info, Duration::from_secs(1));
        let mysql = MySqlPoolOptions::new().connect_with(mysql).await?;

        Ok(Databases { redis, mysql })
    }

    /// Get a redis connection
    pub async fn redis(&self) -> Result<impl AsyncCommands, InternalError> {
        self.redis
            .get()
            .instrument(debug_span!("get_redis_conn"))
            .await
            .map_err(|e| InternalError::RedisPoolError(e))
    }

    /// Get a mysql connection
    pub async fn mysql(&self) -> Result<PoolConnection, InternalError> {
        self.mysql
            .acquire()
            .instrument(debug_span!("get_mysql_conn"))
            .await
            .map_err(|e| InternalError::SqlPoolError(e))
    }
}
