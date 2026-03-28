//! Validator trait and implementations.

use crate::error::ValidationErrors;

/// Trait for self-validating types.
///
/// Implement this trait on request types to enable automatic validation
/// through the [`ValidationBehavior`](crate::pipeline::behaviors::ValidationBehavior).
///
/// # Examples
///
/// ```
/// use mediatr::{Validate, ValidationErrors, ValidationError};
///
/// struct CreateUserCommand {
///     name: String,
///     email: String,
///     age: u8,
/// }
///
/// impl Validate for CreateUserCommand {
///     fn validate(&self) -> std::result::Result<(), ValidationErrors> {
///         let mut errors = ValidationErrors::new();
///
///         // Name validation
///         if self.name.is_empty() {
///             errors.add_field_error("name", "Name is required");
///         } else if self.name.len() < 2 {
///             errors.add_field_error("name", "Name must be at least 2 characters");
///         }
///
///         // Email validation
///         if !self.email.contains('@') {
///             errors.add_field_error("email", "Invalid email format");
///         }
///
///         // Age validation
///         if self.age < 18 {
///             errors.add_field_error("age", "Must be at least 18 years old");
///         }
///
///         errors.into_result()
///     }
/// }
/// ```
pub trait Validate {
    /// Validates the instance.
    ///
    /// Returns `Ok(())` if all validations pass, or `Err(ValidationErrors)`
    /// containing all validation failures.
    fn validate(&self) -> std::result::Result<(), ValidationErrors>;
}

/// Trait for external validators.
///
/// Use this when you want to separate validation logic from the request type,
/// or when you need access to external resources for validation.
///
/// # Examples
///
/// ```
/// use mediatr::validation::{Validator, ValidationErrors};
///
/// struct User {
///     email: String,
/// }
///
/// struct UniqueEmailValidator {
///     existing_emails: Vec<String>,
/// }
///
/// impl Validator<User> for UniqueEmailValidator {
///     fn validate(&self, value: &User) -> std::result::Result<(), ValidationErrors> {
///         if self.existing_emails.contains(&value.email) {
///             let mut errors = ValidationErrors::new();
///             errors.add_field_error("email", "Email already exists");
///             Err(errors)
///         } else {
///             Ok(())
///         }
///     }
/// }
/// ```
pub trait Validator<T>: Send + Sync {
    /// Validates the given value.
    fn validate(&self, value: &T) -> std::result::Result<(), ValidationErrors>;
}

/// A composite validator that runs multiple validators.
pub struct CompositeValidator<T> {
    validators: Vec<Box<dyn Validator<T>>>,
}

impl<T> CompositeValidator<T> {
    /// Creates a new empty composite validator.
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Adds a validator to the composite.
    pub fn add<V: Validator<T> + 'static>(mut self, validator: V) -> Self {
        self.validators.push(Box::new(validator));
        self
    }
}

impl<T> Default for CompositeValidator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + Sync> Validator<T> for CompositeValidator<T> {
    fn validate(&self, value: &T) -> std::result::Result<(), ValidationErrors> {
        let mut all_errors = ValidationErrors::new();

        for validator in &self.validators {
            if let Err(errors) = validator.validate(value) {
                all_errors.merge(errors);
            }
        }

        all_errors.into_result()
    }
}

/// Common validation functions.
pub mod validators {
    use crate::error::ValidationError;

    /// Validates that a string is not empty.
    pub fn not_empty(value: &str, field: &str) -> std::result::Result<(), ValidationError> {
        if value.is_empty() {
            Err(ValidationError::field(field, format!("{} is required", field)))
        } else {
            Ok(())
        }
    }

    /// Validates that a string has a minimum length.
    pub fn min_length(value: &str, min: usize, field: &str) -> std::result::Result<(), ValidationError> {
        if value.len() < min {
            Err(ValidationError::field(
                field,
                format!("{} must be at least {} characters", field, min),
            ))
        } else {
            Ok(())
        }
    }

    /// Validates that a string has a maximum length.
    pub fn max_length(value: &str, max: usize, field: &str) -> std::result::Result<(), ValidationError> {
        if value.len() > max {
            Err(ValidationError::field(
                field,
                format!("{} must be at most {} characters", field, max),
            ))
        } else {
            Ok(())
        }
    }

    /// Validates that a value is within a range (inclusive).
    pub fn in_range<T: PartialOrd + std::fmt::Display>(
        value: T,
        min: T,
        max: T,
        field: &str,
    ) -> std::result::Result<(), ValidationError> {
        if value < min || value > max {
            Err(ValidationError::field(
                field,
                format!("{} must be between {} and {}", field, min, max),
            ))
        } else {
            Ok(())
        }
    }

    /// Validates that a string looks like an email (contains @).
    pub fn email(value: &str, field: &str) -> std::result::Result<(), ValidationError> {
        if !value.contains('@') || !value.contains('.') {
            Err(ValidationError::field(field, "Invalid email format"))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ValidationError;

    struct TestData {
        name: String,
        age: u32,
    }

    impl Validate for TestData {
        fn validate(&self) -> std::result::Result<(), ValidationErrors> {
            let mut errors = ValidationErrors::new();

            if self.name.is_empty() {
                errors.add(ValidationError::field("name", "Name is required"));
            }

            if self.age < 18 {
                errors.add(ValidationError::field("age", "Must be at least 18"));
            }

            errors.into_result()
        }
    }

    #[test]
    fn test_validate_success() {
        let data = TestData {
            name: "Alice".to_string(),
            age: 25,
        };
        assert!(data.validate().is_ok());
    }

    #[test]
    fn test_validate_failure() {
        let data = TestData {
            name: "".to_string(),
            age: 15,
        };
        let result = data.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_validators_not_empty() {
        assert!(validators::not_empty("hello", "field").is_ok());
        assert!(validators::not_empty("", "field").is_err());
    }

    #[test]
    fn test_validators_min_length() {
        assert!(validators::min_length("hello", 3, "field").is_ok());
        assert!(validators::min_length("hi", 3, "field").is_err());
    }

    #[test]
    fn test_validators_email() {
        assert!(validators::email("test@example.com", "email").is_ok());
        assert!(validators::email("invalid", "email").is_err());
    }
}
