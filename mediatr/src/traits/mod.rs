//! Core traits for the MediatR pattern.
//!
//! This module contains all the fundamental traits that define the MediatR pattern:
//! - [`Request`] - Base trait for all requests
//! - [`RequestHandler`] - Async handler for requests
//! - [`RequestHandlerSync`] - Sync handler for requests
//! - [`Command`] - CQRS command marker (write operations)
//! - [`CommandHandler`] - Handler for commands
//! - [`Query`] - CQRS query marker (read operations)
//! - [`QueryHandler`] - Handler for queries
//! - [`Notification`] - Pub/sub notification
//! - [`NotificationHandler`] - Handler for notifications

mod request;
mod handler;
mod command;
mod query;
mod notification;

pub use request::Request;
pub use handler::{RequestHandler, RequestHandlerSync};
pub use command::{Command, CommandHandler, CommandHandlerSync};
pub use query::{Query, QueryHandler, QueryHandlerSync};
pub use notification::{Notification, NotificationHandler, NotificationHandlerSync};
