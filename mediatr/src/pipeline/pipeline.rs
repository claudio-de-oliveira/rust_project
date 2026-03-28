//! Pipeline execution logic.

use std::sync::Arc;

use crate::error::Result;
use crate::traits::{Request, RequestHandler};
use super::behavior::{PipelineBehavior, RequestDelegate};

/// Pipeline that wraps a handler with behaviors.
///
/// The pipeline executes behaviors in order, with each behavior able to
/// run code before/after the next behavior or handler.
pub struct Pipeline<R: Request> {
    behaviors: Vec<Arc<dyn PipelineBehavior<R>>>,
}

impl<R: Request> Pipeline<R> {
    /// Creates a new empty pipeline.
    pub fn new() -> Self {
        Self {
            behaviors: Vec::new(),
        }
    }

    /// Adds a behavior to the pipeline.
    ///
    /// Behaviors are executed in the order they are added.
    pub fn add_behavior<B: PipelineBehavior<R>>(mut self, behavior: B) -> Self {
        self.behaviors.push(Arc::new(behavior));
        self
    }

    /// Adds an already-Arc'd behavior to the pipeline.
    pub fn add_behavior_arc(mut self, behavior: Arc<dyn PipelineBehavior<R>>) -> Self {
        self.behaviors.push(behavior);
        self
    }

    /// Returns the number of behaviors in the pipeline.
    pub fn len(&self) -> usize {
        self.behaviors.len()
    }

    /// Returns true if the pipeline has no behaviors.
    pub fn is_empty(&self) -> bool {
        self.behaviors.is_empty()
    }

    /// Executes the pipeline with the given request and handler.
    pub fn execute<'a>(
        &'a self,
        request: R,
        handler: Arc<dyn RequestHandler<R>>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R::Response>> + Send + 'a>>
    where
        R: 'a,
    {
        Box::pin(async move {
            self.execute_behavior_chain(request, handler, 0).await
        })
    }

    /// Execute behaviors in a chain.
    fn execute_behavior_chain<'a>(
        &'a self,
        request: R,
        handler: Arc<dyn RequestHandler<R>>,
        start_index: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R::Response>> + Send + 'a>>
    where
        R: 'a,
    {
        Box::pin(async move {
            if start_index >= self.behaviors.len() {
                return handler.handle(request).await;
            }

            // Build the chain from the end backwards
            let behaviors = &self.behaviors[start_index..];
            
            if behaviors.is_empty() {
                return handler.handle(request).await;
            }

            // Execute first behavior with a next that chains to remaining
            let behavior = behaviors[0].clone();
            let remaining_behaviors: Vec<_> = behaviors[1..].iter().cloned().collect();
            let handler_clone = handler.clone();

            // Create next delegate
            let next: RequestDelegate<'_, R> = if remaining_behaviors.is_empty() {
                Box::new(move |req| {
                    let h = handler_clone;
                    Box::pin(async move { h.handle(req).await })
                })
            } else {
                let inner_pipeline = Pipeline {
                    behaviors: remaining_behaviors,
                };
                Box::new(move |req| {
                    let h = handler_clone;
                    Box::pin(async move { 
                        inner_pipeline.execute(req, h).await 
                    })
                })
            };

            behavior.handle(request, next).await
        })
    }
}

impl<R: Request> Default for Pipeline<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: Request> Clone for Pipeline<R> {
    fn clone(&self) -> Self {
        Self {
            behaviors: self.behaviors.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::traits::RequestHandlerSync;
    use async_trait::async_trait;

    struct TestRequest {
        value: i32,
    }

    impl Request for TestRequest {
        type Response = i32;
    }

    struct DoubleHandler;

    #[async_trait]
    impl RequestHandler<TestRequest> for DoubleHandler {
        async fn handle(&self, request: TestRequest) -> Result<i32> {
            Ok(request.value * 2)
        }
    }

    struct AddOneBehavior;

    #[async_trait]
    impl PipelineBehavior<TestRequest> for AddOneBehavior {
        async fn handle<'a>(
            &'a self,
            mut request: TestRequest,
            next: RequestDelegate<'a, TestRequest>,
        ) -> Result<i32> {
            request.value += 1;
            next(request).await
        }
    }

    struct MultiplyByThreeBehavior;

    #[async_trait]
    impl PipelineBehavior<TestRequest> for MultiplyByThreeBehavior {
        async fn handle<'a>(
            &'a self,
            mut request: TestRequest,
            next: RequestDelegate<'a, TestRequest>,
        ) -> Result<i32> {
            request.value *= 3;
            next(request).await
        }
    }

    #[tokio::test]
    async fn test_pipeline_no_behaviors() {
        let pipeline: Pipeline<TestRequest> = Pipeline::new();
        let handler: Arc<dyn RequestHandler<TestRequest>> = Arc::new(DoubleHandler);
        let request = TestRequest { value: 10 };

        let result = pipeline.execute(request, handler).await;
        assert_eq!(result.unwrap(), 20);
    }

    #[tokio::test]
    async fn test_pipeline_single_behavior() {
        let pipeline = Pipeline::new().add_behavior(AddOneBehavior);
        let handler: Arc<dyn RequestHandler<TestRequest>> = Arc::new(DoubleHandler);
        let request = TestRequest { value: 10 };

        // (10 + 1) * 2 = 22
        let result = pipeline.execute(request, handler).await;
        assert_eq!(result.unwrap(), 22);
    }

    #[tokio::test]
    async fn test_pipeline_multiple_behaviors() {
        let pipeline = Pipeline::new()
            .add_behavior(AddOneBehavior)
            .add_behavior(MultiplyByThreeBehavior);
        let handler: Arc<dyn RequestHandler<TestRequest>> = Arc::new(DoubleHandler);
        let request = TestRequest { value: 10 };

        // ((10 + 1) * 3) * 2 = 66
        let result = pipeline.execute(request, handler).await;
        assert_eq!(result.unwrap(), 66);
    }
}
