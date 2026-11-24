//! Outbound notifications: SMS/push/email digests and dispatcher alerts.
//!
//! Services depend on the [`Notifier`] trait only. The scaffold logs
//! notifications; a real deployment swaps in an SMS/push provider or a NATS
//! publisher without touching service code.

use tracing::info;

/// What happened, in a channel-agnostic shape.
#[derive(Debug, Clone)]
pub struct Notification {
    /// e.g. `HOUSEHOLD-0421`, `DRIVER-07`, `council@kinondoni.go.tz`
    pub recipient: String,
    pub title: String,
    pub body: String,
}

impl Notification {
    pub fn new(recipient: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            recipient: recipient.into(),
            title: title.into(),
            body: body.into(),
        }
    }

    /// Convenience constructor for pickup lifecycle events.
    pub fn pickup_status(recipient: impl Into<String>, pickup_id: &str, status: &str) -> Self {
        Self::new(
            recipient,
            "Pickup update",
            format!("Pickup {pickup_id} is now {status}"),
        )
    }
}

/// Delivery backend. Implementations must not block the caller.
pub trait Notifier: Send + Sync {
    fn notify(&self, notification: &Notification);
}

/// Default implementation: writes the notification to the tracing pipeline.
#[derive(Debug, Default, Clone, Copy)]
pub struct LogNotifier;

impl Notifier for LogNotifier {
    fn notify(&self, notification: &Notification) {
        info!(
            recipient = %notification.recipient,
            title = %notification.title,
            "notification: {}", notification.body
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pickup_status_notification_mentions_id_and_status() {
        let n = Notification::pickup_status("HOUSEHOLD-1", "PICKUP-00001", "ASSIGNED");
        assert!(n.body.contains("PICKUP-00001"));
        assert!(n.body.contains("ASSIGNED"));
    }

    #[test]
    fn log_notifier_does_not_panic() {
        LogNotifier.notify(&Notification::new("x", "y", "z"));
    }
}
