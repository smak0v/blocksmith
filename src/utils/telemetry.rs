use tracing::{Subscriber, subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

use std::io;

pub fn init(name: impl Into<String>, env_filter: impl Into<String>) {
    init_subscriber(get_subscriber(name, env_filter, io::stdout));
}

fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("failed to set logger");
    subscriber::set_global_default(subscriber).expect("failed to set subscriber");
}

fn get_subscriber<Sink>(
    name: impl Into<String>,
    env_filter: impl Into<String>,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter.into()));
    let json_storage_layer = JsonStorageLayer;
    let formatting_layer = BunyanFormattingLayer::new(name.into(), sink);

    Registry::default()
        .with(env_filter_layer)
        .with(json_storage_layer)
        .with(formatting_layer)
}
