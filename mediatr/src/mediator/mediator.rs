//! Core Mediator implementation.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

use crate::error::{Error, Result};
use crate::traits::{Request, RequestHandler, Notification, NotificationHandler};
use crate::pipeline::{Pipeline, PipelineBehavior};

// Removed ErasedHandler trait and wrappers

/// The core Mediator that dispatches requests to handlers.
///
/// The Mediator is the central component that routes requests to their
/// appropriate handlers, executing any registered pipeline behaviors
/// before and after handling.
// ... (omitted doc comments)
pub struct Mediator {
    /// Registered request handlers.
    handlers: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    /// Registered notification handlers.
    notification_handlers: HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>,
    /// Global pipeline behaviors that apply to all requests.
    global_behaviors: Vec<Arc<dyn Any + Send + Sync>>,
}

impl Mediator {
    /// Creates a new mediator builder.
    pub fn builder() -> super::MediatorBuilder {
        super::MediatorBuilder::new()
    }

    /// Internal constructor - use MediatorBuilder instead.
    /// pub(crate): público dentro do mesmo crate
    pub(crate) fn new(
        handlers: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
        notification_handlers: HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>,
        global_behaviors: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Self {
        Self {
            handlers,
            notification_handlers,
            global_behaviors,
        }
    }

    pub async fn send<R: Request>(&self, request: R) -> Result<R::Response> {
        let type_id = TypeId::of::<R>(); // Essa declaração é a forma como o Rust 
        // lida com a identificação de tipos em tempo de execução (ou 
        // Runtime Type Identification - RTTI). Basicamente, você está pedindo ao 
        // compilador para gerar um "RG" único para o tipo R.
        // * TypeId: É uma struct da biblioteca padrão (std::any::TypeId) que 
        //   representa um identificador globalmente único para um tipo.
        // * ::of::<R>(): É um método genérico. O ::<R> (chamado de turbofish) 
        //   especifica para qual tipo você quer gerar o ID.
        // ---------------
        // R precisa seguir algumas regras:
        // * 'static: O tipo deve ter uma vida útil estática. Isso significa que 
        //   ele não pode conter referências temporárias (como &str ou &u32), a 
        //   menos que sejam referências estáticas.
        // * Determinismo: O ID é garantido como sendo o mesmo durante toda a 
        //   execução de um mesmo binário, mas pode mudar entre diferentes versões 
        //   do compilador ou em diferentes compilações.
        // * Opacidade: Você não consegue "reconstruir" o tipo a partir do ID. 
        //   O TypeId serve apenas para comparação (==, !=).
        // ---------------
        // É a base do trait Any. Se você já viu um Box<dyn Any> e usou o método 
        // .is::<T>() ou .downcast_ref::<T>(), saiba que, por baixo dos panos, o 
        // Rust está usando o TypeId para verificar se o tipo guardado no Box é o 
        // mesmo que você está tentando recuperar.
        // ---------------
        // O Rust não permite que você simplesmente "adivinhe" o tipo de um 
        // ponteiro genérico. O processo segue este fluxo:
        // 1. Você tem um objeto que implementa o trait Any.
        // 2. Você pergunta: "O TypeId deste objeto é igual ao TypeId do tipo T 
        //    que eu quero?".
        // 3. Se sim, você faz um cast de ponteiro (geralmente usando unsafe) para 
        //    o tipo original.
        // 4. Se não, você retorna um erro ou None.
        //
        // O Jeito "Maneira de Rust" (Idiomático)
        //   is::<T>(): Retorna true se o objeto for do tipo T.
        //   downcast_ref::<T>(): Retorna Option<&T>.
        //   downcast_mut::<T>(): Retorna Option<&mut T>.

        // Get the handler
        let erased_handler = self.handlers.get(&type_id)
            .ok_or_else(|| {
                Error::handler_not_found::<R>()
            })?;

        let handler = erased_handler
            .downcast_ref::<Arc<dyn RequestHandler<R>>>()
            .ok_or_else(|| {
                Error::handler_not_found::<R>()
            })?
            .clone();

        // Build pipeline with compatible behaviors
        let mut pipeline = Pipeline::new();
        
        for behavior in &self.global_behaviors {
            if let Some(b) = behavior.downcast_ref::<Arc<dyn PipelineBehavior<R>>>() {
                pipeline = pipeline.add_behavior_arc(b.clone());
            }
        }

        // Execute through pipeline
        if pipeline.is_empty() {
            handler.handle(request).await
        } else {
            pipeline.execute(request, handler).await
        }
    }

    pub async fn publish<N: Notification>(&self, notification: N) -> Result<()> {
        let type_id = TypeId::of::<N>();

        let handlers = self.notification_handlers.get(&type_id);
        
        if handlers.is_none() || handlers.unwrap().is_empty() {
            // No handlers registered - this is not an error for notifications
            return Ok(());
        }

        let handlers = handlers.unwrap();
        let mut errors = Vec::new();
        let mut success_count = 0;

        for erased_handler in handlers {
            if let Some(handler_arc) = erased_handler
                .downcast_ref::<Arc<dyn NotificationHandler<N>>>()
            {
                match handler_arc.handle(&notification).await {
                    Ok(()) => success_count += 1,
                    Err(e) => errors.push(e),
                }
            }
        }

        if success_count > 0 {
            Ok(())
        } else if !errors.is_empty() {
            Err(errors.remove(0))
        } else {
            Ok(())
        }
    }

    /// Checks if a handler is registered for the given request type.
    pub fn has_handler<R: Request>(&self) -> bool {
        self.handlers.contains_key(&TypeId::of::<R>())
    }

    /// Returns the number of registered request handlers.
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

// Mediator is thread-safe
unsafe impl Send for Mediator {}
unsafe impl Sync for Mediator {}

/// Trait for types that can send requests through a mediator.
#[async_trait]
pub trait Sender: Send + Sync {
    /// Sends a request and returns the response.
    async fn send<R: Request>(&self, request: R) -> Result<R::Response>;
}

#[async_trait]
impl Sender for Mediator {
    async fn send<R: Request>(&self, request: R) -> Result<R::Response> {
        Mediator::send(self, request).await
    }
}

/// Trait for types that can publish notifications.
#[async_trait]
pub trait Publisher: Send + Sync {
    /// Publishes a notification to all handlers.
    async fn publish<N: Notification>(&self, notification: N) -> Result<()>;
}

#[async_trait]
impl Publisher for Mediator {
    async fn publish<N: Notification>(&self, notification: N) -> Result<()> {
        Mediator::publish(self, notification).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::RequestHandlerSync;
    // use std::any::TypeId; // Unused

    struct TestRequest {
        value: i32,
    }

    impl Request for TestRequest {
        type Response = i32;
    }

    struct TestHandler;

    impl RequestHandlerSync<TestRequest> for TestHandler {
        fn handle(&self, request: TestRequest) -> Result<i32> {
            Ok(request.value * 2)
        }
    }



    struct TestNotification {
        message: String,
    }

    impl Notification for TestNotification {}

    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingNotificationHandler {
        count: Arc<AtomicUsize>,
    }

    impl crate::traits::NotificationHandlerSync<TestNotification> for CountingNotificationHandler {
        fn handle(&self, _notification: &TestNotification) -> Result<()> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_send_request() {
        let mediator = Mediator::builder()
            .register_sync_handler(TestHandler)
            .build();

        let result = mediator.send(TestRequest { value: 21 }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_handler_not_found() {
        let mediator = Mediator::builder().build();

        let result = mediator.send(TestRequest { value: 1 }).await;
        assert!(matches!(result.unwrap_err(), Error::HandlerNotFound { .. }));
    }

    #[tokio::test]
    async fn test_publish_notification() {
        let count = Arc::new(AtomicUsize::new(0));
        
        let mediator = Mediator::builder()
            .register_notification_handler(CountingNotificationHandler { count: count.clone() })
            .register_notification_handler(CountingNotificationHandler { count: count.clone() })
            .build();

        mediator.publish(TestNotification { message: "test".into() }).await.unwrap();
        
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }
}
