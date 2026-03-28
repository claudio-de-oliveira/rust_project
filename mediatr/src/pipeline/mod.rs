//! Pipeline behaviors for cross-cutting concerns.
//!
//! Pipeline behaviors are middleware that wrap request handling,
//! allowing for pre/post processing, validation, logging, etc.

mod behavior;
mod pipeline;
pub mod behaviors;

pub use behavior::{PipelineBehavior, RequestDelegate};
pub use pipeline::Pipeline;
