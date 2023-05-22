use tracing::subscriber::{ Subscriber, set_global_default };
use tracing_subscriber::{ fmt::MakeWriter, EnvFilter, Registry, layer::SubscriberExt };
use tracing_bunyan_formatter::{ JsonStorageLayer, BunyanFormattingLayer };

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync 
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static
{
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(
        name.into(),
        sink,
    );

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(sub: impl Subscriber + Send + Sync) {
    tracing_log::LogTracer::init().expect("Failed to init LogTracer");
    set_global_default(sub).expect("Failed to set subscriber");
}
