use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use tracing::{info, info_span, subscriber::set_global_default};
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::*, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

use std::env;

mod server;

#[derive(Clone)]
pub struct Databases {
    redis: redis::Client,
    mysql: mysql::Pool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    setup_tracing();

    info!("theta! Gamma Server. Ctrl+C to exit");

    HttpServer::new(|| {
        let _span = info_span!("worker_thread_init").entered();

        info!("connecting to redis");
        let redis_client = redis::Client::open(
            env::var_os("REDIS_URL")
                .expect("Please add the URL to your redis instance")
                .to_str()
                .unwrap(),
        )
        .unwrap();

        info!("connecting to mysql");
        let mysql_client = mysql::Pool::new(
            env::var_os("MYSQL_URL")
                .expect("Please add the URL to your mysql instance")
                .to_str()
                .unwrap(),
        )
        .unwrap();

        let databases = Databases {
            redis: redis_client,
            mysql: mysql_client,
        };

        App::new()
            .app_data(web::Data::new(databases))
            .wrap(TracingLogger::default())
            .service(server::index)
            .service(server::bancho_server)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn setup_tracing() {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(
        Registry::default()
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
            .with(HierarchicalLayer::new(2)),
    )
    .expect("failed to setup tracing");
}
