use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt as _,
};

use crate::config::Config;

pub fn init(_config: &Config) {
    let subscriber = tracing_subscriber::registry().with(
        fmt::Layer::new()
            .with_writer(std::io::stdout)
            .with_ansi(true)
            .with_span_events(FmtSpan::CLOSE),
    );

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
}
