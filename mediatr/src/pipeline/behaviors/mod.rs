//! Built-in pipeline behaviors.

mod logging;
mod timing;
mod validation;

pub use logging::LoggingBehavior;
pub use timing::TimingBehavior;
pub use validation::ValidationBehavior;
