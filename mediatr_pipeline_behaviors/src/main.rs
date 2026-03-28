//! Pipeline behaviors example demonstrating middleware pattern.

use mediatr::prelude::*;
use std::time::Instant;

// ============================================================================
// Request Types
// ============================================================================

struct ProcessOrder {
    order_id: u64,
    items: Vec<String>,
    total: f64,
}

impl Request for ProcessOrder {
    type Response = String;
}

struct ProcessOrderHandler;

#[async_trait]
impl RequestHandler<ProcessOrder> for ProcessOrderHandler {
    async fn handle(&self, request: ProcessOrder) -> Result<String> {
        // Simulate some processing time
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        
        Ok(format!(
            "Order {} processed: {} items, total: ${:.2}",
            request.order_id,
            request.items.len(),
            request.total
        ))
    }
}

// ============================================================================
// Custom Pipeline Behaviors
// ============================================================================

/// Behavior that logs request details before and after handling.
struct DetailedLoggingBehavior {
    prefix: String,
}

impl DetailedLoggingBehavior {
    fn new(prefix: impl Into<String>) -> Self {
        Self { prefix: prefix.into() }
    }
}

#[async_trait]
impl PipelineBehavior<ProcessOrder> for DetailedLoggingBehavior {
    async fn handle<'a>(
        &'a self,
        request: ProcessOrder,
        next: RequestDelegate<'a, ProcessOrder>,
    ) -> Result<String> {
        println!("[{}] Processing order {} with {} items", 
            self.prefix, request.order_id, request.items.len());
        println!("[{}] Items: {:?}", self.prefix, request.items);
        
        let result = next(request).await;
        
        match &result {
            Ok(response) => println!("[{}] Success: {}", self.prefix, response),
            Err(e) => println!("[{}] Error: {}", self.prefix, e),
        }
        
        result
    }
}

/// Behavior that measures execution time.
struct PerformanceBehavior;

#[async_trait]
impl PipelineBehavior<ProcessOrder> for PerformanceBehavior {
    async fn handle<'a>(
        &'a self,
        request: ProcessOrder,
        next: RequestDelegate<'a, ProcessOrder>,
    ) -> Result<String> {
        let start = Instant::now();
        
        let result = next(request).await;
        
        let elapsed = start.elapsed();
        println!("[PERF] Request completed in {:?}", elapsed);
        
        result
    }
}

/// Behavior that validates the order before processing.
struct OrderValidationBehavior {
    max_items: usize,
    max_total: f64,
}

impl OrderValidationBehavior {
    fn new(max_items: usize, max_total: f64) -> Self {
        Self { max_items, max_total }
    }
}

#[async_trait]
impl PipelineBehavior<ProcessOrder> for OrderValidationBehavior {
    async fn handle<'a>(
        &'a self,
        request: ProcessOrder,
        next: RequestDelegate<'a, ProcessOrder>,
    ) -> Result<String> {
        // Validate the order
        let mut errors = ValidationErrors::new();
        
        if request.items.is_empty() {
            errors.add_field_error("items", "Order must have at least one item");
        }
        
        if request.items.len() > self.max_items {
            errors.add_field_error("items", format!(
                "Order cannot have more than {} items", self.max_items
            ));
        }
        
        if request.total <= 0.0 {
            errors.add_field_error("total", "Order total must be positive");
        }
        
        if request.total > self.max_total {
            errors.add_field_error("total", format!(
                "Order total cannot exceed ${:.2}", self.max_total
            ));
        }
        
        // If validation fails, return early
        if !errors.is_empty() {
            println!("[VALIDATION] Failed: {}", errors);
            return Err(Error::ValidationError(errors));
        }
        
        println!("[VALIDATION] Order {} passed validation", request.order_id);
        
        // Continue to next behavior/handler
        next(request).await
    }
}

/// Behavior that wraps errors with additional context.
struct ErrorHandlingBehavior;

#[async_trait]
impl PipelineBehavior<ProcessOrder> for ErrorHandlingBehavior {
    async fn handle<'a>(
        &'a self,
        request: ProcessOrder,
        next: RequestDelegate<'a, ProcessOrder>,
    ) -> Result<String> {
        let order_id = request.order_id;
        
        match next(request).await {
            Ok(response) => Ok(response),
            Err(e) => {
                println!("[ERROR] Order {} failed: {}", order_id, e);
                // Could retry, alert, or transform the error here
                Err(e)
            }
        }
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== MediatR Pipeline Behaviors Example ===\n");

    // Build pipeline with multiple behaviors
    // Order: ErrorHandling -> Performance -> Validation -> Logging -> Handler
    let pipeline = Pipeline::new()
        .add_behavior(ErrorHandlingBehavior)
        .add_behavior(PerformanceBehavior)
        .add_behavior(OrderValidationBehavior::new(10, 10000.0))
        .add_behavior(DetailedLoggingBehavior::new("ORDER"));

    let handler = std::sync::Arc::new(ProcessOrderHandler);

    // ========================================
    // Valid Order
    // ========================================
    println!("--- Processing Valid Order ---\n");
    
    let valid_order = ProcessOrder {
        order_id: 1001,
        items: vec![
            "Widget A".to_string(),
            "Widget B".to_string(),
            "Gadget X".to_string(),
        ],
        total: 299.99,
    };

    match pipeline.execute(valid_order, handler.clone()).await {
        Ok(result) => println!("\nResult: {}\n", result),
        Err(e) => println!("\nError: {}\n", e),
    }

    // ========================================
    // Invalid Order (too many items)
    // ========================================
    println!("--- Processing Invalid Order (too many items) ---\n");
    
    let invalid_order = ProcessOrder {
        order_id: 1002,
        items: (1..=15).map(|i| format!("Item {}", i)).collect(),
        total: 1500.00,
    };

    match pipeline.execute(invalid_order, handler.clone()).await {
        Ok(result) => println!("\nResult: {}\n", result),
        Err(e) => println!("\nError: {}\n", e),
    }

    // ========================================
    // Invalid Order (empty)
    // ========================================
    println!("--- Processing Invalid Order (empty) ---\n");
    
    let empty_order = ProcessOrder {
        order_id: 1003,
        items: vec![],
        total: 0.0,
    };

    match pipeline.execute(empty_order, handler.clone()).await {
        Ok(result) => println!("\nResult: {}\n", result),
        Err(e) => println!("\nError: {}\n", e),
    }

    println!("=== Pipeline Behaviors Example Complete ===");
    Ok(())
}
