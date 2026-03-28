//! Notification traits for pub/sub pattern.
//!
//! Notifications are broadcast to multiple handlers without expecting a response.
//! They are useful for decoupled event handling.

use async_trait::async_trait;

use crate::error::Result;

/// Marker trait for notifications.
///
/// Notifications are broadcast messages that can be handled by multiple handlers.
/// Unlike requests, they don't expect a response.
///
/// # Examples
///
/// ```
/// use mediatr::Notification;
///
/// // Notification when a user is created
/// struct UserCreated {
///     user_id: u64,
///     email: String,
/// }
///
/// impl Notification for UserCreated {}
/// ```
pub trait Notification: Send + Sync + 'static {}

/// Async handler for notifications.
///
/// Multiple handlers can be registered for the same notification type.
/// All handlers will be invoked when a notification is published.
///
/// # Examples
///
/// ```
/// use mediatr::{Notification, NotificationHandler, Result};
/// use async_trait::async_trait;
///
/// struct OrderPlaced {
///     order_id: u64,
///     total: f64,
/// }
///
/// impl Notification for OrderPlaced {}
///
/// // Email notification handler
/// struct EmailNotificationHandler;
///
/// #[async_trait]
/// impl NotificationHandler<OrderPlaced> for EmailNotificationHandler {
///     async fn handle(&self, notification: &OrderPlaced) -> Result<()> {
///         println!("Sending email for order {}", notification.order_id);
///         Ok(())
///     }
/// }
///
/// // Inventory notification handler
/// struct InventoryNotificationHandler;
///
/// #[async_trait]
/// impl NotificationHandler<OrderPlaced> for InventoryNotificationHandler {
///     async fn handle(&self, notification: &OrderPlaced) -> Result<()> {
///         println!("Updating inventory for order {}", notification.order_id);
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait NotificationHandler<N: Notification>: Send + Sync + 'static {
    /// Handles the notification.
    ///
    /// Note: The notification is passed by reference since it may be
    /// handled by multiple handlers.
    ///
    /// # Arguments
    ///
    /// * `notification` - Reference to the notification to handle.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of handling.
    async fn handle(&self, notification: &N) -> Result<()>;
}

/// Synchronous handler for notifications.
pub trait NotificationHandlerSync<N: Notification>: Send + Sync + 'static {
    /// Handles the notification synchronously.
    fn handle(&self, notification: &N) -> Result<()>;
}

/// Blanket implementation: NotificationHandlerSync can be used as async.
#[async_trait]
impl<N, H> NotificationHandler<N> for H
where
    N: Notification,
    H: NotificationHandlerSync<N>,
{
    async fn handle(&self, notification: &N) -> Result<()> {
        NotificationHandlerSync::handle(self, notification)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestNotification {
        message: String,
    }

    impl Notification for TestNotification {}

    struct CountingHandler {
        count: Arc<AtomicUsize>,
    }

    impl NotificationHandlerSync<TestNotification> for CountingHandler {
        fn handle(&self, _notification: &TestNotification) -> Result<()> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_notification_handler() {
        let count = Arc::new(AtomicUsize::new(0));
        let handler = CountingHandler { count: count.clone() };
        let notification = TestNotification {
            message: "test".to_string(),
        };

        NotificationHandler::handle(&handler, &notification).await.unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}
