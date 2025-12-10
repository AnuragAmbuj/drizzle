use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{Event, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

pub struct BroadcastLayer {
    tx: broadcast::Sender<String>,
}

impl BroadcastLayer {
    pub fn new(tx: broadcast::Sender<String>) -> Self {
        Self { tx }
    }
}

impl<S> Layer<S> for BroadcastLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Format the event as a string
        let mut visitor = StringVisitor::new();
        event.record(&mut visitor);
        let msg = format!(
            "{} [{}] {}",
            chrono::Utc::now().to_rfc3339(),
            event.metadata().level(),
            visitor.message
        );
        let _ = self.tx.send(msg); // Ignore error if no receivers
    }
}

struct StringVisitor {
    message: String,
}

impl StringVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
}

impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            // Append other fields if needed, simplified for MVP
            // self.message.push_str(&format!(" {}={:?}", field.name(), value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}
