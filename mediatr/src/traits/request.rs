//! Request trait definition.

/// Base trait for all requests in the MediatR pattern.
///
/// A request represents an intent to perform an action and expects a response.
/// The associated `Response` type defines what the handler will return.
///
/// # Type Parameters
///
/// * `Response` - The type returned when this request is handled.
///
/// # Examples
///
/// ```
/// use mediatr::Request;
///
/// // Define a request with its response type
/// struct GetUserById {
///     id: u64,
/// }
///
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// impl Request for GetUserById {
///     type Response = User;
/// }
/// ```
pub trait Request: Send + Sync + 'static {
    /// The type returned when this request is handled.
    type Response: Send + Sync + 'static;
}

/// Marker trait for unit responses (commands that return nothing).
impl Request for () {
    type Response = ();
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRequest {
        value: i32,
    }

    struct TestResponse {
        result: String,
    }

    impl Request for TestRequest {
        type Response = TestResponse;
    }

    #[test]
    fn test_request_trait_bounds() {
        fn assert_request<R: Request>() {}
        assert_request::<TestRequest>();
    }

    #[test]
    fn test_unit_request() {
        fn assert_request<R: Request>() {}
        assert_request::<()>();
    }
}
