//! Timing behavior for performance measurement.

use async_trait::async_trait;
use std::time::Instant;

use crate::error::Result;
use crate::pipeline::{PipelineBehavior, RequestDelegate};
use crate::traits::Request;

/// Behavior that measures request handling time.
///
/// # Examples
///
/// ```
/// use mediatr::pipeline::behaviors::TimingBehavior;
///
/// let behavior = TimingBehavior::new();
/// // or with custom threshold for slow request warnings
/// let behavior = TimingBehavior::with_threshold(std::time::Duration::from_millis(100));
/// ```
pub struct TimingBehavior {
    slow_threshold: std::time::Duration,
}

impl TimingBehavior {
    /// Creates a new timing behavior with default threshold (500ms).
    pub fn new() -> Self {
        Self {
            slow_threshold: std::time::Duration::from_millis(500),
        }
    }

    /// Creates a timing behavior with a custom slow request threshold.
    pub fn with_threshold(threshold: std::time::Duration) -> Self {
        Self {
            slow_threshold: threshold,
        }
    }
}

impl Default for TimingBehavior {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<R: Request> PipelineBehavior<R> for TimingBehavior {
    async fn handle<'a>(
        &'a self,
        request: R,
        next: RequestDelegate<'a, R>,
    ) -> Result<R::Response> {
        let type_name = std::any::type_name::<R>();
        let start = Instant::now();

        let result = next(request).await;

        let elapsed = start.elapsed();

        if elapsed >= self.slow_threshold {
            #[cfg(feature = "tracing")]
            {
                tracing::warn!(
                    target: "mediatr",
                    request_type = %type_name,
                    elapsed_ms = %elapsed.as_millis(),
                    threshold_ms = %self.slow_threshold.as_millis(),
                    "Slow request detected"
                );
            }

            #[cfg(not(feature = "tracing"))]
            {
                println!(
                    "[MediatR] SLOW REQUEST: {} took {}ms (threshold: {}ms)",
                    type_name,
                    elapsed.as_millis(),
                    self.slow_threshold.as_millis()
                );
            }
        } else {
            #[cfg(feature = "tracing")]
            {
                tracing::debug!(
                    target: "mediatr",
                    request_type = %type_name,
                    elapsed_ms = %elapsed.as_millis(),
                    "Request completed"
                );
            }

            #[cfg(not(feature = "tracing"))]
            {
                // Only log in debug builds to avoid spam
                #[cfg(debug_assertions)]
                println!(
                    "[MediatR] Request {} completed in {}ms",
                    type_name,
                    elapsed.as_millis()
                );
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Request;

    struct TestRequest;

    impl Request for TestRequest {
        type Response = ();
    }

    #[tokio::test]
    async fn test_timing_behavior() {
        let behavior = TimingBehavior::new();
        let request = TestRequest;

        let next: RequestDelegate<'_, TestRequest> = Box::new(|_req| {
            Box::pin(async { Ok(()) })
        });

        let result = behavior.handle(request, next).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timing_with_custom_threshold() {
        let behavior = TimingBehavior::with_threshold(std::time::Duration::from_millis(1));
        let request = TestRequest;

        let next: RequestDelegate<'_, TestRequest> = Box::new(|_req| {
            Box::pin(async {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                Ok(())
            })
        });

        let result = behavior.handle(request, next).await;
        assert!(result.is_ok());
    }
}
