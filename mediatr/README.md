# MediatR for Rust

A Rust implementation of the Mediator pattern with CQRS support, pipeline behaviors, and dependency injection.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- 🎯 **CQRS Support**: Separate Commands (write) and Queries (read) with distinct traits
- 🔗 **Pipeline Behaviors**: Middleware for cross-cutting concerns (logging, validation, timing)
- ⚡ **Async/Sync Handlers**: Support for both async and blocking request handling
- ✅ **Automatic Validation**: Validate requests before handling with the `Validate` trait
- 💉 **Dependency Injection**: Simple DI container with transient and singleton lifetimes
- 🔒 **Type-Safe**: Compile-time verification of request-handler mappings
- 📢 **Notifications**: Pub/sub pattern for event broadcasting

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mediatr = { path = "." }
async-trait = "0.1"
tokio = { version = "1.0", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use mediatr::prelude::*;

// Define a request
struct Ping;

impl Request for Ping {
    type Response = String;
}

// Define a handler
struct PingHandler;

#[async_trait]
impl RequestHandler<Ping> for PingHandler {
    async fn handle(&self, _request: Ping) -> Result<String> {
        Ok("Pong".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Build the mediator
    let mediator = MediatorBuilder::new()
        .register_handler(PingHandler)
        .build();

    // Send a request
    let response = mediator.send(Ping).await?;
    println!("Response: {}", response); // "Pong"
    
    Ok(())
}
```

## CQRS Example

```rust
use mediatr::prelude::*;

// Commands modify state
struct CreateUser {
    name: String,
    email: String,
}

impl Request for CreateUser {
    type Response = u64; // Returns user ID
}

impl Command for CreateUser {}

// Queries read state
struct GetUserById {
    id: u64,
}

impl Request for GetUserById {
    type Response = Option<User>;
}

impl Query for GetUserById {}
```

## Pipeline Behaviors

Add cross-cutting concerns through the pipeline:

```rust
use mediatr::prelude::*;

struct LoggingBehavior;

#[async_trait]
impl<R: Request> PipelineBehavior<R> for LoggingBehavior {
    async fn handle<'a>(
        &'a self,
        request: R,
        next: RequestDelegate<'a, R>,
    ) -> Result<R::Response> {
        println!("Before handling");
        let result = next(request).await;
        println!("After handling");
        result
    }
}

// Use with Pipeline
let pipeline = Pipeline::new()
    .add_behavior(LoggingBehavior)
    .add_behavior(TimingBehavior::new())
    .add_behavior(ValidationBehavior::new());
```

## Validation

Implement the `Validate` trait for automatic validation:

```rust
use mediatr::prelude::*;

struct CreateUser {
    name: String,
    email: String,
}

impl Validate for CreateUser {
    fn validate(&self) -> std::result::Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        
        if self.name.is_empty() {
            errors.add_field_error("name", "Name is required");
        }
        
        if !self.email.contains('@') {
            errors.add_field_error("email", "Invalid email format");
        }
        
        errors.into_result()
    }
}
```

## Notifications

Broadcast events to multiple handlers:

```rust
use mediatr::prelude::*;

struct OrderPlaced {
    order_id: u64,
}

impl Notification for OrderPlaced {}

struct EmailHandler;

#[async_trait]
impl NotificationHandler<OrderPlaced> for EmailHandler {
    async fn handle(&self, notification: &OrderPlaced) -> Result<()> {
        println!("Sending email for order {}", notification.order_id);
        Ok(())
    }
}

// Register multiple handlers
let mediator = MediatorBuilder::new()
    .register_notification_handler(EmailHandler)
    .register_notification_handler(InventoryHandler)
    .build();

// Publish notification
mediator.publish(OrderPlaced { order_id: 1 }).await?;
```

## Dependency Injection

```rust
use mediatr::di::{Container, ServiceLifetime};

let mut container = Container::new();

// Register singleton
container.register_singleton::<MyService, _>(|_| MyService::new());

// Register transient
container.register_transient::<RequestHandler, _>(|c| {
    let service = c.resolve::<MyService>().unwrap();
    RequestHandler::new(service)
});

// Resolve
let handler: Arc<RequestHandler> = container.resolve()?;
```

## Examples

Run the examples to see MediatR in action:

```bash
# Basic request/response
cargo run --example basic_usage

# CQRS with commands and queries
cargo run --example cqrs_example

# Pipeline behaviors (middleware)
cargo run --example pipeline_behaviors

# Automatic validation
cargo run --example validation_example
```

## License

MIT License - see [LICENSE](LICENSE) file for details.
