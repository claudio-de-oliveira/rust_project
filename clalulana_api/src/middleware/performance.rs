use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

/// Middleware that logs request duration for performance monitoring.
pub struct PerformanceMonitor;

impl<S, B> Transform<S, ServiceRequest> for PerformanceMonitor
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = PerformanceMonitorMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(PerformanceMonitorMiddleware { service })
    }
}

pub struct PerformanceMonitorMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for PerformanceMonitorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start.elapsed();
            let status = res.status().as_u16();

            tracing::info!(
                method = %method,
                path = %path,
                status = status,
                duration_ms = duration.as_millis() as u64,
                "Request completed"
            );

            // Warn on slow requests (> 500ms)
            if duration.as_millis() > 500 {
                tracing::warn!(
                    method = %method,
                    path = %path,
                    duration_ms = duration.as_millis() as u64,
                    "Slow request detected"
                );
            }

            Ok(res)
        })
    }
}
