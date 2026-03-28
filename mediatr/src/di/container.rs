//! Simple dependency injection container.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::error::{Error, Result};

/// Service lifetime configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceLifetime {
    /// A new instance is created each time the service is requested.
    Transient,
    /// A single instance is shared across all requests.
    Singleton,
}

/// Internal service descriptor.
pub struct ServiceDescriptor {
    /// The lifetime of the service.
    pub lifetime: ServiceLifetime,
    /// Factory function to create the service.
    factory: Box<dyn Fn(&Container) -> Arc<dyn Any + Send + Sync> + Send + Sync>,
    /// Cached singleton instance (if applicable).
    instance: RwLock<Option<Arc<dyn Any + Send + Sync>>>,
}

impl ServiceDescriptor {
    /// Creates a new transient service descriptor.
    pub fn transient<T, F>(factory: F) -> Self
    where
        T: Send + Sync + 'static,
        F: Fn(&Container) -> T + Send + Sync + 'static,
    {
        Self {
            lifetime: ServiceLifetime::Transient,
            factory: Box::new(move |c| Arc::new(factory(c)) as Arc<dyn Any + Send + Sync>),
            instance: RwLock::new(None),
        }
    }

    /// Creates a new singleton service descriptor.
    pub fn singleton<T, F>(factory: F) -> Self
    where
        T: Send + Sync + 'static,
        F: Fn(&Container) -> T + Send + Sync + 'static,
    {
        Self {
            lifetime: ServiceLifetime::Singleton,
            factory: Box::new(move |c| Arc::new(factory(c)) as Arc<dyn Any + Send + Sync>),
            instance: RwLock::new(None),
        }
    }

    /// Creates a singleton from an existing instance.
    pub fn instance<T: Send + Sync + 'static>(value: T) -> Self {
        Self {
            lifetime: ServiceLifetime::Singleton,
            factory: Box::new(|_| panic!("Instance already set")),
            instance: RwLock::new(Some(Arc::new(value) as Arc<dyn Any + Send + Sync>)),
        }
    }
}

/// A simple dependency injection container.
///
/// Provides service registration and resolution with support for
/// transient and singleton lifetimes.
///
/// # Examples
///
/// ```
/// use mediatr::di::{Container, ServiceLifetime};
/// use std::sync::Arc;
///
/// // Define a service trait
/// trait GreetingService: Send + Sync {
///     fn greet(&self, name: &str) -> String;
/// }
///
/// // Implement the service
/// struct EnglishGreeting;
///
/// impl GreetingService for EnglishGreeting {
///     fn greet(&self, name: &str) -> String {
///         format!("Hello, {}!", name)
///     }
/// }
///
/// // Create and configure container
/// let mut container = Container::new();
/// container.register_singleton::<EnglishGreeting, _>(|_| EnglishGreeting);
///
/// // Resolve the service
/// let service = container.resolve::<EnglishGreeting>().unwrap();
/// assert_eq!(service.greet("World"), "Hello, World!");
/// ```
pub struct Container {
    services: HashMap<TypeId, ServiceDescriptor>,
}

impl Container {
    /// Creates a new empty container.
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Registers a transient service.
    ///
    /// A new instance will be created each time the service is requested.
    pub fn register_transient<T, F>(&mut self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn(&Container) -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.services.insert(type_id, ServiceDescriptor::transient(factory));
    }

    /// Registers a singleton service.
    ///
    /// The first time the service is requested, the factory is called.
    /// Subsequent requests return the same instance.
    pub fn register_singleton<T, F>(&mut self, factory: F)
    where
        T: Send + Sync + 'static,
        F: Fn(&Container) -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.services.insert(type_id, ServiceDescriptor::singleton(factory));
    }

    /// Registers an existing instance as a singleton.
    pub fn register_instance<T: Send + Sync + 'static>(&mut self, instance: T) {
        let type_id = TypeId::of::<T>();
        self.services.insert(type_id, ServiceDescriptor::instance(instance));
    }

    /// Resolves a service by type.
    ///
    /// Returns `Ok(Arc<T>)` if the service is registered, or an error otherwise.
    pub fn resolve<T: Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        let type_id = TypeId::of::<T>();
        
        let descriptor = self.services.get(&type_id)
            .ok_or_else(|| Error::dependency_error(format!(
                "Service not registered: {}",
                std::any::type_name::<T>()
            )))?;

        match descriptor.lifetime {
            ServiceLifetime::Singleton => {
                // Check if we have a cached instance
                {
                    let read_guard = descriptor.instance.read();
                    if let Some(instance) = read_guard.as_ref() {
                        return instance.clone()
                            .downcast::<T>()
                            .map_err(|_| Error::dependency_error("Type mismatch"));
                    }
                }

                // Create and cache the instance
                {
                    let mut write_guard = descriptor.instance.write();
                    // Double-check pattern
                    if write_guard.is_none() {
                        let instance = (descriptor.factory)(self);
                        *write_guard = Some(instance);
                    }
                    write_guard.as_ref().unwrap().clone()
                        .downcast::<T>()
                        .map_err(|_| Error::dependency_error("Type mismatch"))
                }
            }
            ServiceLifetime::Transient => {
                let instance = (descriptor.factory)(self);
                instance.downcast::<T>()
                    .map_err(|_| Error::dependency_error("Type mismatch"))
            }
        }
    }

    /// Checks if a service is registered.
    pub fn is_registered<T: 'static>(&self) -> bool {
        self.services.contains_key(&TypeId::of::<T>())
    }

    /// Returns the number of registered services.
    pub fn len(&self) -> usize {
        self.services.len()
    }

    /// Returns true if no services are registered.
    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

// Container is Send + Sync because all internal data is protected
unsafe impl Send for Container {}
unsafe impl Sync for Container {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_resolve_transient() {
        let mut container = Container::new();
        
        container.register_transient::<i32, _>(|_| 42);
        
        let value: Arc<i32> = container.resolve().unwrap();
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_register_and_resolve_singleton() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let mut container = Container::new();
        container.register_singleton::<String, _>(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            "singleton".to_string()
        });

        // Resolve multiple times
        let _: Arc<String> = container.resolve().unwrap();
        let _: Arc<String> = container.resolve().unwrap();
        let _: Arc<String> = container.resolve().unwrap();

        // Factory should only be called once
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_register_instance() {
        let mut container = Container::new();
        container.register_instance("hello".to_string());

        let value: Arc<String> = container.resolve().unwrap();
        assert_eq!(*value, "hello");
    }

    #[test]
    fn test_resolve_not_registered() {
        let container = Container::new();
        let result: Result<Arc<i32>> = container.resolve();
        assert!(result.is_err());
    }

    #[test]
    fn test_is_registered() {
        let mut container = Container::new();
        container.register_instance(42i32);

        assert!(container.is_registered::<i32>());
        assert!(!container.is_registered::<String>());
    }
}
