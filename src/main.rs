use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::{db::Databases, settings::Settings, telem::setup_tracing};

mod db;
mod errors;
mod server;
mod settings;
mod telem;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let settings = Arc::new(Settings::new().unwrap());
    setup_tracing(&settings);

    info!("theta! Gamma Server. Ctrl+C to exit");

    let databases = Arc::new(Databases::new(&settings.db).await.unwrap());
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
