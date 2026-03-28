# Rust MediatR Library Walkthrough

## Overview

We have successfully implemented a robust MediatR library in Rust, featuring:
- **Core Mediator Pattern**: Decoupled request/response handling.
- **CQRS Support**: Explicit `Command` and `Query` traits.
- **Pipeline Behaviors**: Middleware support for cross-cutting concerns (logging, validation, timing).
- **Async & Sync Handlers**: Flexible handler implementation.
- **Automatic Validation**: Integration with the `validator` crate.
- **Dependency Injection**: A simple but effective DI container.

## Key Accomplishments

### 1. Robust Type System
We leveraged Rust's type system to ensure compile-time safety for handler registration. The `Mediator` uses `Generic` logic internally but exposes a clean, type-safe API.
- Solved complex `TypeId` mismatch issues by refactoring storage to use explicit `Box<dyn Any + Send + Sync>` mapping.
- Implemented `HandlerWrapper` (internal) strategies before settling on a cleaner direct storage approach.

### 2. Pipeline Architecture
The pipeline allows behaviors to be chained dynamically.
```rust
let mediator = MediatorBuilder::new()
    .register_handler(MyHandler)
    .add_behavior(LoggingBehavior)
    .add_behavior(ValidationBehavior)
    .build();
```

### 3. Integrated Validation
Automatic request validation ensures invalid requests never reach the handler.
```rust
#[derive(Validate)]
struct CreateUser {
    #[validate(email)]
    email: String,
}
```

## Verification

We have verified the implementation with a comprehensive suite of tests:

### Unit Tests
- **Passed**: 33/33 tests covering all core modules (traits, mediator, pipeline, error, validation, di).
- Verified correct handler resolution, pipeline execution order, and error propagation.

### Documentation Tests
- **Passed**: 21/21 doc tests ensuring all code examples in documentation are valid and runnable.

### Examples
All provided examples compile and run successfully:
- `basic_usage.rs`: Simple Ping/Pong flow.
- `cqrs_example.rs`: Command/Query separation with user management.
- `pipeline_behaviors.rs`: Logging and Timing middleware demonstration.
- `validation_example.rs`: Automatic validation scenario.

## Usage

To use the library, add it to your dependencies and start building your mediator:

```rust
use mediatr::prelude::*;

let mediator = Mediator::builder()
    .register_handler(MyHandler)
    .build();

let response = mediator.send(MyRequest).await?;
```
