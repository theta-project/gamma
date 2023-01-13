use std::{str::FromStr, time::Duration};

use crate::errors::{InternalError, RequestError};
use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use tracing::{info_span, instrument, Instrument, Level};

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
    #[instrument(name = "connect_databases", skip_all)]
    pub async fn new(settings: &DatabaseSettings) -> Self {
        let redis = {
            let cfg = deadpool_redis::Config::from_url(settings.redis_url.as_str());
            cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
                .unwrap()
        };

        // test to make sure the connection actually works
        redis
            .get()
            .instrument(info_span!("get_redis_conn"))
            .await
            .unwrap()
            .acl_whoami::<()>()
            .instrument(info_span!("test_redis_conn"))
            .await
            .unwrap();

        let mut mysql = MySqlConnectOptions::from_str(&settings.mysql_url).unwrap();
        mysql
            .log_statements(tracing::log::LevelFilter::Debug)
            .log_slow_statements(tracing::log::LevelFilter::Info, Duration::from_secs(1));
        let mysql = MySqlPoolOptions::new()
            .connect_with(mysql)
            .instrument(info_span!("get_mysql_conn"))
            .await
            .unwrap();

        Databases { redis, mysql }
    }

    /// Get a redis connection
    #[instrument(level = Level::DEBUG, name = "get_redis_conn", skip_all)]
    pub async fn redis(&self) -> Result<impl AsyncCommands, RequestError> {
        self.redis
            .get()
            .await
            .map_err(|e| InternalError::RedisPool(e).into())
    }

    /// Get a mysql connection
    #[instrument(level = Level::DEBUG, name = "get_mysql_conn", skip_all)]
    pub async fn mysql(&self) -> Result<PoolConnection, RequestError> {
        self.mysql
            .acquire()
            .await
            .map_err(|e| InternalError::SqlPool(e).into())
    }
}
