//! Error types for the MediatR library.

use std::any::TypeId;
use std::fmt;
use thiserror::Error;

/// Main error type for MediatR operations.
#[derive(Error, Debug)]
pub enum Error {
    /// No handler registered for the given request type.
    #[error("No handler registered for request type: {type_name}")]
    HandlerNotFound { type_name: String, type_id: TypeId },

    /// Handler execution failed.
    #[error("Handler execution failed: {0}")]
    HandlerError(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Validation failed.
    #[error("Validation failed: {0}")]
    ValidationError(#[from] ValidationErrors),

    /// Pipeline behavior error.
    #[error("Pipeline behavior error: {0}")]
    PipelineError(String),

    /// Dependency injection error.
    #[error("Dependency resolution error: {0}")]
    DependencyError(String),

    /// Generic internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Creates a new handler not found error.
    pub fn handler_not_found<T: 'static>() -> Self {
        Self::HandlerNotFound {
            type_name: std::any::type_name::<T>().to_string(),
            type_id: TypeId::of::<T>(),
        }
    }

    /// Creates a new handler error from any error type.
    pub fn handler_error<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        Self::HandlerError(Box::new(err))
    }

    /// Creates a new pipeline error.
    pub fn pipeline_error(msg: impl Into<String>) -> Self {
        Self::PipelineError(msg.into())
    }

    /// Creates a new dependency error.
    pub fn dependency_error(msg: impl Into<String>) -> Self {
        Self::DependencyError(msg.into())
    }
}

/// Collection of validation errors.
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    /// Creates a new empty validation errors collection.
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Adds a validation error.
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Adds a validation error for a specific field.
    pub fn add_field_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationError::field(field, message));
    }

    /// Returns true if there are no validation errors.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the number of validation errors.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Returns an iterator over the validation errors.
    pub fn iter(&self) -> impl Iterator<Item = &ValidationError> {
        self.errors.iter()
    }

    /// Merges another ValidationErrors into this one.
    pub fn merge(&mut self, other: ValidationErrors) {
        self.errors.extend(other.errors);
    }

    /// Converts to Result - Ok if no errors, Err with self otherwise.
    pub fn into_result(self) -> std::result::Result<(), Self> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let messages: Vec<String> = self.errors.iter().map(|e| e.to_string()).collect();
        write!(f, "{}", messages.join("; "))
    }
}

impl std::error::Error for ValidationErrors {}

impl IntoIterator for ValidationErrors {
    type Item = ValidationError;
    type IntoIter = std::vec::IntoIter<ValidationError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl FromIterator<ValidationError> for ValidationErrors {
    fn from_iter<I: IntoIterator<Item = ValidationError>>(iter: I) -> Self {
        Self {
            errors: iter.into_iter().collect(),
        }
    }
}

/// A single validation error.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// The field that failed validation (if applicable).
    pub field: Option<String>,
    /// The validation error message.
    pub message: String,
    /// Optional error code for programmatic handling.
    pub code: Option<String>,
}

impl ValidationError {
    /// Creates a new validation error with just a message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            field: None,
            message: message.into(),
            code: None,
        }
    }

    /// Creates a new validation error for a specific field.
    pub fn field(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: Some(field.into()),
            message: message.into(),
            code: None,
        }
    }

    /// Creates a validation error with a code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.field {
            Some(field) => write!(f, "{}: {}", field, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Result type alias for MediatR operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_errors_empty() {
        let errors = ValidationErrors::new();
        assert!(errors.is_empty());
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_validation_errors_add() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error("email", "Invalid email format");
        errors.add_field_error("name", "Name is required");

        assert!(!errors.is_empty());
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_validation_errors_into_result() {
        let empty = ValidationErrors::new();
        assert!(empty.into_result().is_ok());

        let mut with_errors = ValidationErrors::new();
        with_errors.add_field_error("field", "error");
        assert!(with_errors.into_result().is_err());
    }

    #[test]
    fn test_error_handler_not_found() {
        let err = Error::handler_not_found::<String>();
        assert!(matches!(err, Error::HandlerNotFound { .. }));
    }
}
