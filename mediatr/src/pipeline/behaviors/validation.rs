//! Validation behavior for automatic request validation.

use async_trait::async_trait;

use crate::error::{Error, Result, ValidationErrors};
use crate::pipeline::{PipelineBehavior, RequestDelegate};
use crate::traits::Request;

// Re-export the Validate trait from the validation module
pub use crate::validation::Validate;

/// Behavior that validates requests before handling.
///
/// Only works with requests that implement the [`Validate`] trait.
/// Add this behavior to your pipeline for automatic request validation.
///
/// # Examples
///
/// ```
/// use mediatr::pipeline::behaviors::ValidationBehavior;
///
/// let behavior = ValidationBehavior::new();
/// ```
pub struct ValidationBehavior {
    _private: (),
}

impl ValidationBehavior {
    /// Creates a new validation behavior.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for ValidationBehavior {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<R: Request + Validate> PipelineBehavior<R> for ValidationBehavior {
    async fn handle<'a>(
        &'a self,
        request: R,
        next: RequestDelegate<'a, R>,
    ) -> Result<R::Response> {
        // Validate the request
        request.validate().map_err(Error::ValidationError)?;
        
        // Continue to next behavior/handler
        next(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Request;
    use crate::error::ValidationError;

    struct ValidRequest {
        value: i32,
    }

    impl Request for ValidRequest {
        type Response = i32;
    }

    impl Validate for ValidRequest {
        fn validate(&self) -> std::result::Result<(), ValidationErrors> {
            if self.value > 0 {
                Ok(())
            } else {
                let mut errors = ValidationErrors::new();
                errors.add(ValidationError::field("value", "Value must be positive"));
                Err(errors)
            }
        }
    }

    #[tokio::test]
    async fn test_validation_passes() {
        let behavior = ValidationBehavior::new();
        let request = ValidRequest { value: 42 };

        let next: RequestDelegate<'_, ValidRequest> = Box::new(|req| {
            Box::pin(async move { Ok(req.value) })
        });

        let result = behavior.handle(request, next).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_validation_fails() {
        let behavior = ValidationBehavior::new();
        let request = ValidRequest { value: -1 };

        let next: RequestDelegate<'_, ValidRequest> = Box::new(|req| {
            Box::pin(async move { Ok(req.value) })
        });

        let result = behavior.handle(request, next).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ValidationError(_)));
    }
}
