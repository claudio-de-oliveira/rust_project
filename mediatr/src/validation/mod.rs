//! Validation framework for automatic request validation.

mod validator;

pub use validator::{Validate, Validator, CompositeValidator, validators};
pub use crate::error::{ValidationError, ValidationErrors};
