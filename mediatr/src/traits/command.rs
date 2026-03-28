//! CQRS Command traits.
//!
//! Commands represent write operations that modify state.
//! They are part of the Command Query Responsibility Segregation (CQRS) pattern.

use async_trait::async_trait;

use crate::error::Result;
use super::request::Request;

/// Marker trait for CQRS commands.
///
/// Commands represent intent to change the state of the system.
/// They typically don't return data (or return minimal acknowledgment).
///
/// # Examples
///
/// ```
/// use mediatr::{Command, Request};
///
/// // Command that creates a user
/// struct CreateUser {
///     name: String,
///     email: String,
/// }
///
/// // Commands typically return an ID or acknowledgment
/// struct CreateUserResult {
///     id: u64,
/// }
///
/// impl Request for CreateUser {
///     type Response = CreateUserResult;
/// }
///
/// impl Command for CreateUser {}
/// ```
pub trait Command: Request {}

/// Async handler for CQRS commands.
///
/// This is a specialized version of `RequestHandler` for commands.
/// It provides semantic clarity and allows for command-specific behaviors.
///
/// # Examples
///
/// ```
/// use mediatr::{Command, CommandHandler, Request, Result};
/// use async_trait::async_trait;
///
/// struct DeleteUser { id: u64 }
/// impl Request for DeleteUser { type Response = (); }
/// impl Command for DeleteUser {}
///
/// struct DeleteUserHandler;
///
/// #[async_trait]
/// impl CommandHandler<DeleteUser> for DeleteUserHandler {
///     async fn handle(&self, command: DeleteUser) -> Result<()> {
///         // Delete user from database
///         println!("Deleting user {}", command.id);
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync + 'static {
    /// Handles the command and returns the response.
    async fn handle(&self, command: C) -> Result<C::Response>;
}

/// Synchronous handler for CQRS commands.
pub trait CommandHandlerSync<C: Command>: Send + Sync + 'static {
    /// Handles the command synchronously.
    fn handle(&self, command: C) -> Result<C::Response>;
}

/// Blanket implementation: CommandHandlerSync can be used as async CommandHandler.
#[async_trait]
impl<C, H> CommandHandler<C> for H
where
    C: Command,
    H: CommandHandlerSync<C>,
{
    async fn handle(&self, command: C) -> Result<C::Response> {
        CommandHandlerSync::handle(self, command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand {
        value: String,
    }

    impl Request for TestCommand {
        type Response = u64;
    }

    impl Command for TestCommand {}

    struct TestCommandHandler;

    impl CommandHandlerSync<TestCommand> for TestCommandHandler {
        fn handle(&self, command: TestCommand) -> Result<u64> {
            Ok(command.value.len() as u64)
        }
    }

    #[tokio::test]
    async fn test_command_handler() {
        let handler = TestCommandHandler;
        let command = TestCommand {
            value: "hello".to_string(),
        };

        let result = CommandHandler::handle(&handler, command).await;
        assert_eq!(result.unwrap(), 5);
    }
}
