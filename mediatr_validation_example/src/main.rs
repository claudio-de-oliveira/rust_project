//! Validation example demonstrating automatic request validation.

use mediatr::prelude::*;

// ============================================================================
// Domain Types
// ============================================================================

#[derive(Debug)]
struct UserCreated {
    id: u64,
    name: String,
    email: String,
}

// ============================================================================
// Request with Validation
// ============================================================================

struct RegisterUser {
    name: String,
    email: String,
    password: String,
    age: u8,
}

impl Request for RegisterUser {
    type Response = UserCreated;
}

impl Command for RegisterUser {}

// Implement validation for the request
impl Validate for RegisterUser {
    fn validate(&self) -> std::result::Result<(), ValidationErrors> {
        use mediatr::validation::validators;
        
        let mut errors = ValidationErrors::new();

        // Name validation
        if let Err(e) = validators::not_empty(&self.name, "name") {
            errors.add(e);
        } else if let Err(e) = validators::min_length(&self.name, 2, "name") {
            errors.add(e);
        } else if let Err(e) = validators::max_length(&self.name, 50, "name") {
            errors.add(e);
        }

        // Email validation
        if let Err(e) = validators::not_empty(&self.email, "email") {
            errors.add(e);
        } else if let Err(e) = validators::email(&self.email, "email") {
            errors.add(e);
        }

        // Password validation
        if let Err(e) = validators::not_empty(&self.password, "password") {
            errors.add(e);
        } else if let Err(e) = validators::min_length(&self.password, 8, "password") {
            errors.add(e);
        }

        // Age validation
        if let Err(e) = validators::in_range(self.age, 18, 120, "age") {
            errors.add(e);
        }

        errors.into_result()
    }
}

// ============================================================================
// Handler
// ============================================================================

struct RegisterUserHandler {
    next_id: std::sync::atomic::AtomicU64,
}

impl RegisterUserHandler {
    fn new() -> Self {
        Self {
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }
}

#[async_trait]
impl RequestHandler<RegisterUser> for RegisterUserHandler {
    async fn handle(&self, command: RegisterUser) -> Result<UserCreated> {
        // At this point, validation has already passed
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        println!("  [Handler] Creating user: {} ({})", command.name, command.email);
        
        Ok(UserCreated {
            id,
            name: command.name,
            email: command.email,
        })
    }
}

// ============================================================================
// Custom Validation Behavior
// ============================================================================

/// A validation behavior that validates requests implementing Validate trait.
struct CustomValidationBehavior;

#[async_trait]
impl PipelineBehavior<RegisterUser> for CustomValidationBehavior {
    async fn handle<'a>(
        &'a self,
        request: RegisterUser,
        next: RequestDelegate<'a, RegisterUser>,
    ) -> Result<UserCreated> {
        println!("[Validation] Validating request...");
        
        // Validate the request
        if let Err(errors) = request.validate() {
            println!("[Validation] Failed with {} errors:", errors.len());
            for error in errors.iter() {
                println!("  - {}", error);
            }
            return Err(Error::ValidationError(errors));
        }
        
        println!("[Validation] Request is valid");
        next(request).await
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== MediatR Validation Example ===\n");

    // Create pipeline with validation behavior
    let pipeline = Pipeline::new()
        .add_behavior(CustomValidationBehavior);
    
    let handler = std::sync::Arc::new(RegisterUserHandler::new());

    // ========================================
    // Valid Registration
    // ========================================
    println!("--- Valid User Registration ---\n");
    
    let valid_request = RegisterUser {
        name: "Alice Smith".to_string(),
        email: "alice@example.com".to_string(),
        password: "securepassword123".to_string(),
        age: 25,
    };

    match pipeline.execute(valid_request, handler.clone()).await {
        Ok(user) => println!("\nSuccess! Created user: {:?}\n", user),
        Err(e) => println!("\nError: {}\n", e),
    }

    // ========================================
    // Invalid: Empty name
    // ========================================
    println!("--- Invalid: Empty Name ---\n");
    
    let invalid_name = RegisterUser {
        name: "".to_string(),
        email: "bob@example.com".to_string(),
        password: "password123".to_string(),
        age: 30,
    };

    match pipeline.execute(invalid_name, handler.clone()).await {
        Ok(user) => println!("\nSuccess! Created user: {:?}\n", user),
        Err(e) => println!("\nRejected: {}\n", e),
    }

    // ========================================
    // Invalid: Bad email format
    // ========================================
    println!("--- Invalid: Bad Email ---\n");
    
    let invalid_email = RegisterUser {
        name: "Charlie".to_string(),
        email: "not-an-email".to_string(),
        password: "password123".to_string(),
        age: 28,
    };

    match pipeline.execute(invalid_email, handler.clone()).await {
        Ok(user) => println!("\nSuccess! Created user: {:?}\n", user),
        Err(e) => println!("\nRejected: {}\n", e),
    }

    // ========================================
    // Invalid: Password too short
    // ========================================
    println!("--- Invalid: Short Password ---\n");
    
    let invalid_password = RegisterUser {
        name: "David".to_string(),
        email: "david@example.com".to_string(),
        password: "short".to_string(),
        age: 35,
    };

    match pipeline.execute(invalid_password, handler.clone()).await {
        Ok(user) => println!("\nSuccess! Created user: {:?}\n", user),
        Err(e) => println!("\nRejected: {}\n", e),
    }

    // ========================================
    // Invalid: Underage
    // ========================================
    println!("--- Invalid: Underage User ---\n");
    
    let underage = RegisterUser {
        name: "Young User".to_string(),
        email: "young@example.com".to_string(),
        password: "password123".to_string(),
        age: 15,
    };

    match pipeline.execute(underage, handler.clone()).await {
        Ok(user) => println!("\nSuccess! Created user: {:?}\n", user),
        Err(e) => println!("\nRejected: {}\n", e),
    }

    // ========================================
    // Invalid: Multiple errors
    // ========================================
    println!("--- Invalid: Multiple Errors ---\n");
    
    let multiple_errors = RegisterUser {
        name: "".to_string(),
        email: "bad".to_string(),
        password: "123".to_string(),
        age: 10,
    };

    match pipeline.execute(multiple_errors, handler.clone()).await {
        Ok(user) => println!("\nSuccess! Created user: {:?}\n", user),
        Err(e) => println!("\nRejected: {}\n", e),
    }

    println!("=== Validation Example Complete ===");
    Ok(())
}
