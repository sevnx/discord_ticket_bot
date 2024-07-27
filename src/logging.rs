use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub fn setup() -> Result<(), String> {
    let fmt_layer = tracing_subscriber::fmt::layer().compact();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|error| error.to_string())?;

    Registry::default()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    Ok(())
}
