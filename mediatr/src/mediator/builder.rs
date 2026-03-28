//! Fluent builder for constructing a Mediator.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::{Request, RequestHandler, RequestHandlerSync, Notification, NotificationHandler};
use crate::pipeline::PipelineBehavior;
use super::Mediator;

/// A fluent builder for creating [`Mediator`] instances.
///
/// The builder pattern allows for clean, readable configuration of the mediator
/// with handlers, behaviors, and other settings.
///
/// # Examples
///
/// ```
/// use mediatr::{MediatorBuilder, Request, RequestHandler, Result};
/// use mediatr::pipeline::behaviors::{LoggingBehavior, TimingBehavior};
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
///     async fn handle(&self, _: Ping) -> Result<String> {
///         Ok("Pong".to_string())
///     }
/// }
///
/// let mediator = MediatorBuilder::new()
///     .register_handler(PingHandler)
///     // .add_behavior::<Ping, _>(LoggingBehavior::new())
///     // .add_behavior::<Ping, _>(TimingBehavior::new())
///     .build();
/// ```
pub struct MediatorBuilder {
    handlers: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    notification_handlers: HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>,
    global_behaviors: Vec<Arc<dyn Any + Send + Sync>>,
}

impl MediatorBuilder {
    /// Creates a new empty mediator builder.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            notification_handlers: HashMap::new(),
            global_behaviors: Vec::new(),
        }
    }

    /// Registers a request handler.
    ///
    /// Only one handler can be registered per request type.
    /// Registering a second handler for the same type will replace the first.
    ///
    /// # Type Parameters
    ///
    /// * `R` - The request type (inferred from the handler).
    /// * `H` - The handler type.
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler instance.
    pub fn register_handler<R, H>(mut self, handler: H) -> Self
    where
        R: Request,
        H: RequestHandler<R>,
    {
        let type_id = TypeId::of::<R>();
        //println!("Registering handler for type {:?} (Box<Arc>)", type_id);
        let wrapped: Arc<dyn RequestHandler<R>> = Arc::new(handler);
        // Store as Box<dyn Any + Send + Sync> containing Arc<dyn RequestHandler<R>>
        self.handlers.insert(type_id, Box::new(wrapped) as Box<dyn Any + Send + Sync>);
        self
    }
    
    /// Registers a synchronous request handler.
    ///
    /// This wraps the synchronous handler to be used in the async pipeline.
    pub fn register_sync_handler<R, H>(self, handler: H) -> Self
    where
        R: Request,
        H: RequestHandlerSync<R>,
    {
        // let wrapped = SyncHandlerAdapter(handler, std::marker::PhantomData);
        // self.register_handler(wrapped)
        // Wait, I need SyncHandlerAdapter! 
        // SyncHandlerAdapter is defined later in the file.
        // It wraps H.
        let wrapped = SyncHandlerAdapter(handler, std::marker::PhantomData);
        self.register_handler(wrapped)
    }

    /// Registers a notification handler.
    ///
    /// Multiple handlers can be registered for the same notification type.
    /// All registered handlers will be invoked when the notification is published.
    ///
    /// # Type Parameters
    ///
    /// * `N` - The notification type (inferred from the handler).
    /// * `H` - The handler type.
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler instance.
    pub fn register_notification_handler<N, H>(mut self, handler: H) -> Self
    where
        N: Notification,
        H: NotificationHandler<N>,
    {
        let type_id = TypeId::of::<N>();
        let wrapped: Arc<dyn NotificationHandler<N>> = Arc::new(handler);
        
        self.notification_handlers
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(Box::new(wrapped) as Box<dyn Any + Send + Sync>);
        
        self
    }

    /// Registers a command handler.
    ///
    /// This adapter allows registering a `CommandHandler` as a `RequestHandler`.
    pub fn register_command_handler<C, H>(self, handler: H) -> Self
    where
        C: Command,
        H: CommandHandler<C>,
    {
        let wrapped = CommandHandlerAdapter(handler, std::marker::PhantomData);
        self.register_handler(wrapped)
    }

    /// Registers a query handler.
    ///
    /// This adapter allows registering a `QueryHandler` as a `RequestHandler`.
    pub fn register_query_handler<Q, H>(self, handler: H) -> Self
    where
        Q: Query,
        H: QueryHandler<Q>,
    {
        let wrapped = QueryHandlerAdapter(handler, std::marker::PhantomData);
        self.register_handler(wrapped)
    }

    /// Adds a pipeline behavior for a specific request type.
    ///
    /// Behaviors are executed in the order they are added, wrapping the handler.
    ///
    /// # Type Parameters
    ///
    /// * `R` - The request type this behavior applies to.
    /// * `B` - The behavior type.
    ///
    /// # Arguments
    ///
    /// * `behavior` - The behavior instance.
    pub fn add_behavior<R, B>(mut self, behavior: B) -> Self
    where
        R: Request,
        B: PipelineBehavior<R>,
    {
        let wrapped: Arc<dyn PipelineBehavior<R>> = Arc::new(behavior);
        self.global_behaviors.push(Arc::new(wrapped) as Arc<dyn Any + Send + Sync>);
        self
    }

    /// Builds the [`Mediator`] with all registered handlers and behaviors.
    pub fn build(self) -> Mediator {
        Mediator::new(
            self.handlers,
            self.notification_handlers,
            self.global_behaviors,
        )
    }
}

impl Default for MediatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Adapters

use async_trait::async_trait;
use crate::traits::{Command, CommandHandler, Query, QueryHandler};
use crate::error::Result;

struct CommandHandlerAdapter<C, H>(H, std::marker::PhantomData<C>);

#[async_trait]
impl<C, H> RequestHandler<C> for CommandHandlerAdapter<C, H>
where
    C: Command,
    H: CommandHandler<C>,
{
    async fn handle(&self, request: C) -> Result<C::Response> {
        self.0.handle(request).await
    }
}

struct QueryHandlerAdapter<Q, H>(H, std::marker::PhantomData<Q>);

#[async_trait]
impl<Q, H> RequestHandler<Q> for QueryHandlerAdapter<Q, H>
where
    Q: Query,
    H: QueryHandler<Q>,
{
    async fn handle(&self, request: Q) -> Result<Q::Response> {
        self.0.handle(request).await
    }
}

struct SyncHandlerAdapter<R, H>(H, std::marker::PhantomData<R>);

#[async_trait]
impl<R, H> RequestHandler<R> for SyncHandlerAdapter<R, H>
where
    R: Request,
    H: RequestHandlerSync<R>,
{
    async fn handle(&self, request: R) -> Result<R::Response> {
        RequestHandlerSync::handle(&self.0, request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::RequestHandlerSync;
    use crate::error::Result;

    struct TestRequest;

    impl Request for TestRequest {
        type Response = String;
    }

    struct TestHandler;

    impl RequestHandlerSync<TestRequest> for TestHandler {
        fn handle(&self, _: TestRequest) -> Result<String> {
            Ok("handled".to_string())
        }
    }

    #[test]
    fn test_builder_creates_mediator() {
        let mediator = MediatorBuilder::new()
            .register_sync_handler(TestHandler)
            .build();

        assert!(mediator.has_handler::<TestRequest>());
    }

    #[test]
    fn test_builder_default() {
        let builder = MediatorBuilder::default();
        let mediator = builder.build();
        
        assert_eq!(mediator.handler_count(), 0);
    }
}
