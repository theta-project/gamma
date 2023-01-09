use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use tracing::{info, subscriber::set_global_default};
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::*, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

use std::sync::Arc;

use crate::{db::Databases, settings::Settings};

mod db;
mod server;
mod settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    setup_tracing();

    info!("theta! Gamma Server. Ctrl+C to exit");

    let settings = Arc::new(Settings::new().unwrap());
    let databases = Arc::new(Databases::new(&settings.db));
    let bind_info = (settings.ip.clone(), settings.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(settings.clone()))
            .app_data(web::Data::new(databases.clone()))
            .wrap(TracingLogger::default())
            .service(server::index)
            .service(server::bancho_server)
    })
    .bind(bind_info)?
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
