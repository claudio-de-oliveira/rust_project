//! Handler traits for processing requests.

use async_trait::async_trait;

use crate::error::Result;
use super::request::Request;

/// Async handler for processing requests.
///
/// Implement this trait to handle a specific request type asynchronously.
/// The handler receives the request and returns the associated response type.
///
/// # Type Parameters
///
/// * `R` - The request type this handler processes.
///
/// # Examples
///
/// ```
/// use mediatr::{Request, RequestHandler, Result};
/// use async_trait::async_trait;
///
/// struct Ping;
/// impl Request for Ping {
///     type Response = String;
/// }
///
/// struct PingHandler;
///
/// #[async_trait]
/// impl RequestHandler<Ping> for PingHandler {
///     async fn handle(&self, _request: Ping) -> Result<String> {
///         Ok("Pong".to_string())
///     }
/// }
/// ```
#[async_trait]
pub trait RequestHandler<R: Request>: Send + Sync + 'static {
    /// Handles the request and returns the response.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to handle.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response on success, or an error on failure.
    async fn handle(&self, request: R) -> Result<R::Response>;
}

/// Synchronous handler for processing requests.
///
/// Implement this trait to handle a specific request type synchronously.
/// Use this when async is not needed or not desirable.
///
/// # Type Parameters
///
/// * `R` - The request type this handler processes.
///
/// # Examples
///
/// ```
/// use mediatr::{Request, RequestHandlerSync, Result};
///
/// struct Add {
///     a: i32,
///     b: i32,
/// }
///
/// impl Request for Add {
///     type Response = i32;
/// }
///
/// struct AddHandler;
///
/// impl RequestHandlerSync<Add> for AddHandler {
///     fn handle(&self, request: Add) -> Result<i32> {
///         Ok(request.a + request.b)
///     }
/// }
/// ```
pub trait RequestHandlerSync<R: Request>: Send + Sync + 'static {
    /// Handles the request synchronously and returns the response.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to handle.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response on success, or an error on failure.
    fn handle(&self, request: R) -> Result<R::Response>;
}

// Blanket implementation removed to avoid conflicts with other adapters.
// Use SyncHandlerAdapter or register_sync_handler in MediatorBuilder instead.

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRequest {
        value: i32,
    }

    impl Request for TestRequest {
        type Response = i32;
    }

    struct TestHandler;

    impl RequestHandlerSync<TestRequest> for TestHandler {
        fn handle(&self, request: TestRequest) -> Result<i32> {
            Ok(request.value * 2)
        }
    }

    // Test removed as blanket implementation was removed.
    // Sync handlers are now adapted via SyncHandlerAdapter in the builder.
}
