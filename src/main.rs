use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use opentelemetry::sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::{info, subscriber::set_global_default};
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_subscriber::{filter::LevelFilter, prelude::*, Registry};
use tracing_tree::HierarchicalLayer;

use std::{sync::Arc, time::Duration};

use crate::{db::Databases, settings::Settings};

mod db;
mod server;
mod settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let settings = Arc::new(Settings::new().unwrap());
    setup_tracing(&settings);

    info!("theta! Gamma Server. Ctrl+C to exit");

    let databases = Arc::new(Databases::new(&settings.db).await);
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

fn setup_tracing(settings: &Settings) {
    LogTracer::init().expect("Failed to set logger");
    let registry = Registry::default()
        .with(LevelFilter::from_level(settings.log_level()))
        .with(HierarchicalLayer::new(2));

    if let Some(telem_settings) = &settings.telem {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint("http://localhost:4317")
                    .with_timeout(Duration::from_secs(3)),
            )
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_max_events_per_span(64)
                    .with_max_attributes_per_span(16)
                    .with_max_events_per_span(16),
            )
            .install_batch(opentelemetry::runtime::Tokio)
            .unwrap();

        set_global_default(registry.with(tracing_opentelemetry::layer().with_tracer(tracer)))
    } else {
        set_global_default(registry.with(HierarchicalLayer::new(2)))
    }
    .expect("failed to setup tracing");
}
