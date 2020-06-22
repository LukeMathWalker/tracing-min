use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{Registry};
use tracing_error::{ErrorLayer, SpanTrace};
use tracing_subscriber::layer::SubscriberExt;
use opentelemetry::api::{Provider, Tracer};
use opentelemetry::{sdk, global};
use opentelemetry::sdk::{Sampler};

pub fn get_tracer() -> thrift::Result<sdk::Tracer> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_agent_endpoint("127.0.0.1:6831".parse().unwrap())
        .with_process(opentelemetry_jaeger::Process {
            service_name: "trace-demo".to_string(),
            tags: vec![
            ],
        })
        .init()?;
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::Always),
            ..Default::default()
        })
        .build();

    Ok(provider.get_tracer("test"))
}

fn init_telemetry() {
    let tracer =
        get_tracer().expect("Failed to build tracer.");
    let opentracing_layer = OpenTelemetryLayer::new(tracer, Sampler::Always);

    let subscriber =
        Registry::default()
            .with(opentracing_layer)
            .with(ErrorLayer::default());

    tracing::subscriber::set_global_default(subscriber).unwrap()
}

#[derive(thiserror::Error, Debug)]
#[error("Test error: {0}")]
pub struct DummyError(SpanTrace);

#[tracing::instrument]
pub fn test() -> Result<(), DummyError> {
    let error = DummyError(SpanTrace::capture());
    tracing::warn!("Something went wrong: {:?}", error);
    Err(error)
}

fn main() {
    init_telemetry();
    let tracer = global::tracer("request");
    tracer.in_span("middleware", move |_cx| {
        let _ = test();
    });
}
