//! Pipeline behavior trait definition.

use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

use crate::error::Result;
use crate::traits::Request;

/// Type alias for the next delegate in the pipeline.
pub type RequestDelegate<'a, R> = Box<
    dyn FnOnce(R) -> Pin<Box<dyn Future<Output = Result<<R as Request>::Response>> + Send + 'a>>
        + Send
        + 'a,
>;

/// Pipeline behavior for cross-cutting concerns.
///
/// Behaviors are executed in order before the handler is invoked.
/// Each behavior can:
/// - Execute code before calling the next behavior/handler
/// - Modify the request before passing it on
/// - Execute code after the handler returns
/// - Short-circuit the pipeline by not calling next
/// - Handle or transform errors
///
/// # Examples
///
/// ```
/// use mediatr::{PipelineBehavior, Request, RequestDelegate, Result};
/// use async_trait::async_trait;
///
/// struct LoggingBehavior;
///
/// #[async_trait]
/// impl<R: Request> PipelineBehavior<R> for LoggingBehavior {
///     async fn handle<'a>(
///         &'a self,
///         request: R,
///         next: RequestDelegate<'a, R>,
///     ) -> Result<R::Response> {
///         println!("Before handling request");
///         let result = next(request).await;
///         println!("After handling request");
///         result
///     }
/// }
/// ```
#[async_trait]
pub trait PipelineBehavior<R: Request>: Send + Sync + 'static {
    /// Handles the request with access to the next delegate.
    ///
    /// # Arguments
    ///
    /// * `request` - The request being processed.
    /// * `next` - The next delegate in the pipeline (either another behavior or the handler).
    ///
    /// # Returns
    ///
    /// The result of processing the request.
    async fn handle<'a>(
        &'a self,
        request: R,
        next: RequestDelegate<'a, R>,
    ) -> Result<R::Response>;
}

/// Trait for behaviors that can handle any request type.
///
/// Implement this trait when you want a behavior that works with all requests,
/// regardless of their specific type.
#[async_trait]
pub trait GenericPipelineBehavior: Send + Sync + 'static {
    /// Handles any request type with access to the next delegate.
    async fn handle<'a, R: Request>(
        &'a self,
        request: R,
        next: RequestDelegate<'a, R>,
    ) -> Result<R::Response>;
}

/// Blanket implementation: GenericPipelineBehavior is a PipelineBehavior for any R.
#[async_trait]
impl<R: Request, T: GenericPipelineBehavior> PipelineBehavior<R> for T {
    async fn handle<'a>(
        &'a self,
        request: R,
        next: RequestDelegate<'a, R>,
    ) -> Result<R::Response> {
        GenericPipelineBehavior::handle(self, request, next).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestRequest {
        value: i32,
    }

    impl Request for TestRequest {
        type Response = i32;
    }

    struct CountingBehavior {
        before_count: Arc<AtomicUsize>,
        after_count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl<R: Request> PipelineBehavior<R> for CountingBehavior {
        async fn handle<'a>(
            &'a self,
            request: R,
            next: RequestDelegate<'a, R>,
        ) -> Result<R::Response> {
            self.before_count.fetch_add(1, Ordering::SeqCst);
            let result = next(request).await;
            self.after_count.fetch_add(1, Ordering::SeqCst);
            result
        }
    }

    #[tokio::test]
    async fn test_behavior_execution_order() {
        let before = Arc::new(AtomicUsize::new(0));
        let after = Arc::new(AtomicUsize::new(0));

        let behavior = CountingBehavior {
            before_count: before.clone(),
            after_count: after.clone(),
        };

        let request = TestRequest { value: 42 };
        let next: RequestDelegate<'_, TestRequest> = Box::new(|req| {
            Box::pin(async move { Ok(req.value * 2) })
        });

        let result = behavior.handle(request, next).await;

        assert_eq!(result.unwrap(), 84);
        assert_eq!(before.load(Ordering::SeqCst), 1);
        assert_eq!(after.load(Ordering::SeqCst), 1);
    }
}
