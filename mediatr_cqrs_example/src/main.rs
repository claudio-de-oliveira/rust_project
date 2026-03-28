//! CQRS example demonstrating Command/Query separation.

use mediatr::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ============================================================================
// Domain Model
// ============================================================================

#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
    email: String,
}

// In-memory user repository
type UserRepository = Arc<RwLock<HashMap<u64, User>>>;

// ============================================================================
// Commands (Write Operations)
// ============================================================================

// Create User Command
struct CreateUserCommand {
    name: String,
    email: String,
}

impl Request for CreateUserCommand {
    type Response = u64; // Returns the new user ID
}

impl Command for CreateUserCommand {}

struct CreateUserHandler {
    repo: UserRepository,
    next_id: Arc<RwLock<u64>>,
}

#[async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserHandler {
    async fn handle(&self, command: CreateUserCommand) -> Result<u64> {
        let mut next_id = self.next_id.write().unwrap();
        let id = *next_id;
        *next_id += 1;
        drop(next_id);

        let user = User {
            id,
            name: command.name,
            email: command.email,
        };

        self.repo.write().unwrap().insert(id, user);
        println!("  [CreateUserHandler] Created user with ID: {}", id);

        Ok(id)
    }
}

// Update User Command
struct UpdateUserCommand {
    id: u64,
    name: Option<String>,
    email: Option<String>,
}

impl Request for UpdateUserCommand {
    type Response = bool; // Returns true if updated
}

impl Command for UpdateUserCommand {}

struct UpdateUserHandler {
    repo: UserRepository,
}

#[async_trait]
impl CommandHandler<UpdateUserCommand> for UpdateUserHandler {
    async fn handle(&self, command: UpdateUserCommand) -> Result<bool> {
        let mut repo = self.repo.write().unwrap();
        
        if let Some(user) = repo.get_mut(&command.id) {
            if let Some(name) = command.name {
                user.name = name;
            }
            if let Some(email) = command.email {
                user.email = email;
            }
            println!("  [UpdateUserHandler] Updated user {}", command.id);
            Ok(true)
        } else {
            println!("  [UpdateUserHandler] User {} not found", command.id);
            Ok(false)
        }
    }
}

// Delete User Command
struct DeleteUserCommand {
    id: u64,
}

impl Request for DeleteUserCommand {
    type Response = bool;
}

impl Command for DeleteUserCommand {}

struct DeleteUserHandler {
    repo: UserRepository,
}

#[async_trait]
impl CommandHandler<DeleteUserCommand> for DeleteUserHandler {
    async fn handle(&self, command: DeleteUserCommand) -> Result<bool> {
        let removed = self.repo.write().unwrap().remove(&command.id).is_some();
        if removed {
            println!("  [DeleteUserHandler] Deleted user {}", command.id);
        } else {
            println!("  [DeleteUserHandler] User {} not found", command.id);
        }
        Ok(removed)
    }
}

// ============================================================================
// Queries (Read Operations)
// ============================================================================

// Get User By ID Query
struct GetUserByIdQuery {
    id: u64,
}

impl Request for GetUserByIdQuery {
    type Response = Option<User>;
}

impl Query for GetUserByIdQuery {}

struct GetUserByIdHandler {
    repo: UserRepository,
}

#[async_trait]
impl QueryHandler<GetUserByIdQuery> for GetUserByIdHandler {
    async fn handle(&self, query: GetUserByIdQuery) -> Result<Option<User>> {
        let user = self.repo.read().unwrap().get(&query.id).cloned();
        println!("  [GetUserByIdHandler] Fetching user {}: {:?}", query.id, user.is_some());
        Ok(user)
    }
}

// Get All Users Query
struct GetAllUsersQuery;

impl Request for GetAllUsersQuery {
    type Response = Vec<User>;
}

impl Query for GetAllUsersQuery {}

struct GetAllUsersHandler {
    repo: UserRepository,
}

#[async_trait]
impl QueryHandler<GetAllUsersQuery> for GetAllUsersHandler {
    async fn handle(&self, _query: GetAllUsersQuery) -> Result<Vec<User>> {
        let users: Vec<User> = self.repo.read().unwrap().values().cloned().collect();
        println!("  [GetAllUsersHandler] Fetching all users: {} found", users.len());
        Ok(users)
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== MediatR CQRS Example ===\n");

    // Shared state
    let repo: UserRepository = Arc::new(RwLock::new(HashMap::new()));
    let next_id = Arc::new(RwLock::new(1u64));

    // Build the mediator with all handlers
    let mediator = MediatorBuilder::new()
        // Commands
        .register_command_handler(CreateUserHandler {
            repo: repo.clone(),
            next_id: next_id.clone(),
        })
        .register_command_handler(UpdateUserHandler { repo: repo.clone() })
        .register_command_handler(DeleteUserHandler { repo: repo.clone() })
        // Queries
        .register_query_handler(GetUserByIdHandler { repo: repo.clone() })
        .register_query_handler(GetAllUsersHandler { repo: repo.clone() })
        .build();

    // ========================================
    // Execute Commands (Write Operations)
    // ========================================
    println!("--- Creating Users (Commands) ---");
    
    let user1_id = mediator.send(CreateUserCommand {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }).await?;

    let user2_id = mediator.send(CreateUserCommand {
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    }).await?;

    let user3_id = mediator.send(CreateUserCommand {
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
    }).await?;

    println!("\nCreated users with IDs: {}, {}, {}\n", user1_id, user2_id, user3_id);

    // ========================================
    // Execute Queries (Read Operations)
    // ========================================
    println!("--- Querying Users ---");

    let all_users = mediator.send(GetAllUsersQuery).await?;
    println!("\nAll users:");
    for user in &all_users {
        println!("  - {} (ID: {}): {}", user.name, user.id, user.email);
    }

    println!("\n--- Updating User (Command) ---");
    mediator.send(UpdateUserCommand {
        id: user1_id,
        name: Some("Alice Smith".to_string()),
        email: None,
    }).await?;

    println!("\n--- Querying Updated User ---");
    let alice = mediator.send(GetUserByIdQuery { id: user1_id }).await?;
    if let Some(user) = alice {
        println!("  Updated user: {} ({}) - {}", user.name, user.id, user.email);
    }

    println!("\n--- Deleting User (Command) ---");
    mediator.send(DeleteUserCommand { id: user2_id }).await?;

    println!("\n--- Final User List ---");
    let final_users = mediator.send(GetAllUsersQuery).await?;
    for user in &final_users {
        println!("  - {} (ID: {}): {}", user.name, user.id, user.email);
    }

    println!("\n=== CQRS Example Complete ===");
    Ok(())
}
