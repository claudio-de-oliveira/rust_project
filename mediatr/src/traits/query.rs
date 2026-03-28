//! CQRS Query traits.
//!
//! Queries represent read operations that retrieve data without modifying state.
//! They are part of the Command Query Responsibility Segregation (CQRS) pattern.

use async_trait::async_trait;

use crate::error::Result;
use super::request::Request;

/// Marker trait for CQRS queries.
///
/// Queries represent intent to read data without changing state.
/// They should be idempotent and side-effect free.
///
/// # Examples
///
/// ```
/// use mediatr::{Query, Request};
///
/// // Query to get a user by ID
/// struct GetUserById {
///     id: u64,
/// }
///
/// struct User {
///     id: u64,
///     name: String,
///     email: String,
/// }
///
/// impl Request for GetUserById {
///     type Response = Option<User>;
/// }
///
/// impl Query for GetUserById {}
/// ```
pub trait Query: Request {}

/// Async handler for CQRS queries.
///
/// This is a specialized version of `RequestHandler` for queries.
/// It provides semantic clarity and allows for query-specific behaviors
/// (like caching or read replicas).
///
/// # Examples
///
/// ```
/// use mediatr::{Query, QueryHandler, Request, Result};
/// use async_trait::async_trait;
///
/// struct GetAllUsers;
/// impl Request for GetAllUsers { type Response = Vec<String>; }
/// impl Query for GetAllUsers {}
///
/// struct GetAllUsersHandler;
///
/// #[async_trait]
/// impl QueryHandler<GetAllUsers> for GetAllUsersHandler {
///     async fn handle(&self, _query: GetAllUsers) -> Result<Vec<String>> {
///         // Fetch users from read replica
///         Ok(vec!["Alice".into(), "Bob".into()])
///     }
/// }
/// ```
#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync + 'static {
    /// Handles the query and returns the response.
    async fn handle(&self, query: Q) -> Result<Q::Response>;
}

/// Synchronous handler for CQRS queries.
pub trait QueryHandlerSync<Q: Query>: Send + Sync + 'static {
    /// Handles the query synchronously.
    fn handle(&self, query: Q) -> Result<Q::Response>;
}

/// Blanket implementation: QueryHandlerSync can be used as async QueryHandler.
#[async_trait]
impl<Q, H> QueryHandler<Q> for H
where
    Q: Query,
    H: QueryHandlerSync<Q>,
{
    async fn handle(&self, query: Q) -> Result<Q::Response> {
        QueryHandlerSync::handle(self, query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestQuery {
        filter: String,
    }

    impl Request for TestQuery {
        type Response = Vec<String>;
    }

    impl Query for TestQuery {}

    struct TestQueryHandler;

    impl QueryHandlerSync<TestQuery> for TestQueryHandler {
        fn handle(&self, query: TestQuery) -> Result<Vec<String>> {
            Ok(vec![format!("Result for: {}", query.filter)])
        }
    }

    #[tokio::test]
    async fn test_query_handler() {
        let handler = TestQueryHandler;
        let query = TestQuery {
            filter: "test".to_string(),
        };

        let result = QueryHandler::handle(&handler, query).await;
        assert_eq!(result.unwrap(), vec!["Result for: test".to_string()]);
    }
}
