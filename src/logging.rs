use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[must_use]
pub fn setup_logging() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    let targets = filter::Targets::new()
        .with_target("tower_http::trace::on_request", Level::DEBUG)
        .with_target("tower_http::trace::make_span", Level::DEBUG)
        .with_target("jira_tracker", Level::DEBUG)
        .with_default(Level::INFO);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(targets)
        .init();

    TraceLayer::new_for_http()
}
