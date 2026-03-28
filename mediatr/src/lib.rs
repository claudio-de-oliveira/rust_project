//! # MediatR for Rust
//!
//! A Rust implementation of the Mediator pattern with CQRS support,
//! pipeline behaviors, and dependency injection.
//!
//! ## Features
//!
//! - **CQRS Support**: Separate Commands (write) and Queries (read)
//! - **Pipeline Behaviors**: Middleware for cross-cutting concerns
//! - **Async/Sync Handlers**: Support for both async and sync request handling
//! - **Automatic Validation**: Validate requests before handling
//! - **Dependency Injection**: Simple DI container integration
//! - **Type-Safe**: Compile-time verification of request-handler mappings
//!
//! ## Quick Start
//!
//! ```rust
//! use mediatr::{Mediator, MediatorBuilder, Request, RequestHandler, Result};
//! use async_trait::async_trait;
//!
//! // Define a request
//! struct Ping;
//!
//! impl Request for Ping {
//!     type Response = String;
//! }
//!
//! // Define a handler
//! struct PingHandler;
//!
//! #[async_trait]
//! impl RequestHandler<Ping> for PingHandler {
//!     async fn handle(&self, _request: Ping) -> Result<String> {
//!         Ok("Pong".to_string())
//!     }
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<()> {
//! // Build and use the mediator
//! let mediator = MediatorBuilder::new()
//!     .register_handler(PingHandler)
//!     .build();
//!
//! let response = mediator.send(Ping).await?;
//! assert_eq!(response, "Pong");
//! # Ok(())
//! # }
//! ```
//!
//! ## CQRS Example
//!
//! ```rust
//! use mediatr::{Command, Query, CommandHandler, QueryHandler, Request, Result};
//! use async_trait::async_trait;
//!
//! // Commands modify state
//! struct CreateUser {
//!     name: String,
//!     email: String,
//! }
//!
//! impl Request for CreateUser {
//!     type Response = u64; // Returns the new user ID
//! }
//!
//! impl Command for CreateUser {}
//!
//! // Queries read state
//! struct GetUserById {
//!     id: u64,
//! }
//!
//! impl Request for GetUserById {
//!     type Response = Option<String>; // Returns user name if found
//! }
//!
//! impl Query for GetUserById {}
//! ```
//!
//! ## Pipeline Behaviors
//!
//! ```rust
//! use mediatr::{PipelineBehavior, Request, RequestDelegate, Result};
//! use async_trait::async_trait;
//!
//! struct LoggingBehavior;
//!
//! #[async_trait]
//! impl<R: Request> PipelineBehavior<R> for LoggingBehavior {
//!     async fn handle<'a>(
//!         &'a self,
//!         request: R,
//!         next: RequestDelegate<'a, R>,
//!     ) -> Result<R::Response> {
//!         println!("Before handling");
//!         let result = next(request).await;
//!         println!("After handling");
//!         result
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

// Core modules
pub mod error;
pub mod traits;
pub mod mediator;
pub mod pipeline;
pub mod validation;
pub mod di;

// Re-export commonly used types at crate root
pub use error::{Error, Result, ValidationError, ValidationErrors};
pub use traits::{
    Request,
    RequestHandler,
    RequestHandlerSync,
    Command,
    CommandHandler,
    CommandHandlerSync,
    Query,
    QueryHandler,
    QueryHandlerSync,
    Notification,
    NotificationHandler,
};
pub use mediator::{Mediator, MediatorBuilder};
pub use pipeline::{PipelineBehavior, RequestDelegate, Pipeline};
pub use validation::Validate;

/// Prelude module for convenient imports.
///
/// ```rust
/// use mediatr::prelude::*;
/// ```
pub mod prelude {
    pub use crate::error::{Error, Result, ValidationError, ValidationErrors};
    pub use crate::traits::{
        Request,
        RequestHandler,
        RequestHandlerSync,
        Command,
        CommandHandler,
        CommandHandlerSync,
        Query,
        QueryHandler,
        QueryHandlerSync,
        Notification,
        NotificationHandler,
    };
    pub use crate::mediator::{Mediator, MediatorBuilder};
    pub use crate::pipeline::{PipelineBehavior, RequestDelegate, Pipeline};
    pub use crate::pipeline::behaviors::{LoggingBehavior, TimingBehavior, ValidationBehavior};
    pub use crate::validation::Validate;
    pub use crate::di::{Container, ServiceLifetime};
    pub use async_trait::async_trait;
}
