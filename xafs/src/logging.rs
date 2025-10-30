use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt as _,
};

pub fn init() {
    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout)
                .with_ansi(true)
                .with_span_events(FmtSpan::CLOSE),
        )
        .with(EnvFilter::from_default_env());

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
}
