use std::time::Duration;

use opentelemetry::{
    sdk::{
        trace::{self, RandomIdGenerator, Sampler},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::{filter::LevelFilter, prelude::*, Registry};
use tracing_tree::HierarchicalLayer;

use crate::settings::Settings;

pub fn setup_tracing(settings: &Settings) {
    LogTracer::init().expect("Failed to set logger"); // for crates stil using `log`

    let registry = Registry::default()
        .with(LevelFilter::from_level(settings.log_level()))
        .with(HierarchicalLayer::new(2)); // human friendly output

    if let Some(telem_settings) = &settings.telem {
        // also output to OTLP
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(&telem_settings.endpoint)
                    .with_timeout(Duration::from_secs(3)),
            )
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_max_events_per_span(64)
                    .with_max_attributes_per_span(16)
                    .with_resource(Resource::new(vec![KeyValue::new("service.name", "gamma")])),
            )
            .install_batch(opentelemetry::runtime::Tokio)
            .unwrap();

        set_global_default(registry.with(tracing_opentelemetry::layer().with_tracer(tracer)))
    } else {
        set_global_default(registry)
    }
    .expect("failed to setup tracing");
}
