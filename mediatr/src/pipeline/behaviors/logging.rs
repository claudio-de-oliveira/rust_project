//! Logging behavior for request/response logging.

use async_trait::async_trait;
use std::fmt::Debug;

use crate::error::Result;
use crate::pipeline::{PipelineBehavior, RequestDelegate};
use crate::traits::Request;

/// Behavior that logs request handling.
///
/// When the `tracing` feature is enabled, this uses the `tracing` crate.
/// Otherwise, it uses `println!` for basic logging.
///
/// # Examples
///
/// ```
/// use mediatr::pipeline::behaviors::LoggingBehavior;
///
/// let behavior = LoggingBehavior::new();
/// // or with custom prefix
/// let behavior = LoggingBehavior::with_prefix("MyApp");
/// ```
pub struct LoggingBehavior {
    prefix: String,
}

impl LoggingBehavior {
    /// Creates a new logging behavior with default prefix.
    pub fn new() -> Self {
        Self {
            prefix: "MediatR".to_string(),
        }
    }

    /// Creates a logging behavior with a custom prefix.
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl Default for LoggingBehavior {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<R: Request> PipelineBehavior<R> for LoggingBehavior {
    async fn handle<'a>(
        &'a self,
        request: R,
        next: RequestDelegate<'a, R>,
    ) -> Result<R::Response> {
        let type_name = std::any::type_name::<R>();
        
        #[cfg(feature = "tracing")]
        {
            tracing::info!(
                target: "mediatr",
                prefix = %self.prefix,
                request_type = %type_name,
                "Handling request"
            );
        }

        #[cfg(not(feature = "tracing"))]
        {
            println!("[{}] Handling request: {}", self.prefix, type_name);
        }

        let result = next(request).await;

        match &result {
            Ok(_) => {
                #[cfg(feature = "tracing")]
                {
                    tracing::info!(
                        target: "mediatr",
                        prefix = %self.prefix,
                        request_type = %type_name,
                        "Request handled successfully"
                    );
                }

                #[cfg(not(feature = "tracing"))]
                {
                    println!("[{}] Request handled successfully: {}", self.prefix, type_name);
                }
            }
            Err(e) => {
                #[cfg(feature = "tracing")]
                {
                    tracing::error!(
                        target: "mediatr",
                        prefix = %self.prefix,
                        request_type = %type_name,
                        error = %e,
                        "Request handling failed"
                    );
                }

                #[cfg(not(feature = "tracing"))]
                {
                    println!("[{}] Request handling failed: {} - {}", self.prefix, type_name, e);
                }
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
        type Response = String;
    }

    #[tokio::test]
    async fn test_logging_behavior() {
        let behavior = LoggingBehavior::new();
        let request = TestRequest;

        let next: RequestDelegate<'_, TestRequest> = Box::new(|_req| {
            Box::pin(async { Ok("success".to_string()) })
        });

        let result = behavior.handle(request, next).await;
        assert!(result.is_ok());
    }
}
