//! Basic usage example demonstrating request/response pattern.

use mediatr::prelude::*;

// Define a simple request
struct Ping {
    message: String,
}

impl Request for Ping {
    type Response = String;
}

// Define its handler
struct PingHandler;

#[async_trait]
impl RequestHandler<Ping> for PingHandler {
    async fn handle(&self, request: Ping) -> Result<String> {
        Ok(format!("Pong: {}", request.message))
    }
}

// Define an addition request (sync handler example)
struct Add {
    a: i32,
    b: i32,
}

impl Request for Add {
    type Response = i32;
}

struct AddHandler;

impl RequestHandlerSync<Add> for AddHandler {
    fn handle(&self, request: Add) -> Result<i32> {
        Ok(request.a + request.b)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== MediatR Basic Usage Example ===\n");

    // Build the mediator with handlers
    let mediator = MediatorBuilder::new()
        .register_handler(PingHandler)
        .register_sync_handler(AddHandler)
        .build();

    // Send an async request
    println!("Sending Ping request...");
    let ping_response = mediator.send(Ping {
        message: "Hello, MediatR!".to_string(),
    }).await?;
    println!("Ping response: {}\n", ping_response);

    // Send a sync request (used as async through blanket impl)
    println!("Sending Add request...");
    let add_response = mediator.send(Add { a: 21, b: 21 }).await?;
    println!("Add response: {} + {} = {}\n", 21, 21, add_response);

    println!("=== Example Complete ===");
    Ok(())
}
