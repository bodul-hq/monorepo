//! Query bus module for routing queries to handlers.
//!
//! The QueryBus manages query dispatch and handler registration.
//! Each query type can have exactly one handler registered.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::query_handler::QueryHandler;

/// Errors that can occur when using the QueryBus.
#[derive(Debug, Clone)]
pub enum QueryBusError {
    /// No handler registered for the query type.
    NoHandlerRegistered(String),
    /// Handler already registered for the query type.
    DuplicateHandler(String),
    /// Handler execution failed.
    HandlerFailed(String),
}

impl std::fmt::Display for QueryBusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryBusError::NoHandlerRegistered(query) => {
                write!(f, "No handler registered for query: {}", query)
            }
            QueryBusError::DuplicateHandler(query) => {
                write!(f, "Handler already registered for query: {}", query)
            }
            QueryBusError::HandlerFailed(msg) => {
                write!(f, "Handler execution failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for QueryBusError {}

/// A boxed handler that can handle a specific query type.
trait BoxedHandler {
    fn as_any(&self) -> &dyn Any;
}

/// Wrapper to store a handler in the registry.
struct HandlerWrapper<H> {
    handler: H,
}

impl<H: 'static> BoxedHandler for HandlerWrapper<H> {
    fn as_any(&self) -> &dyn Any {
        &self.handler
    }
}

/// QueryBus manages query dispatch and handler registration.
///
/// Handlers are registered by query type and looked up at execution time.
/// Each query type can have exactly one handler.
pub struct QueryBus {
    handlers: HashMap<TypeId, Box<dyn BoxedHandler>>,
    type_names: HashMap<TypeId, &'static str>,
}

impl QueryBus {
    /// Creates a new empty QueryBus.
    pub fn new() -> Self {
        QueryBus {
            handlers: HashMap::new(),
            type_names: HashMap::new(),
        }
    }

    /// Registers a handler for a query type.
    ///
    /// # Type Parameters
    /// - `Q`: The query type this handler processes
    /// - `H`: The handler type implementing `QueryHandler<Q>`
    ///
    /// # Errors
    /// Returns `QueryBusError::DuplicateHandler` if a handler is already
    /// registered for query type `Q`.
    pub fn register<Q, H>(&mut self, handler: H) -> Result<(), QueryBusError>
    where
        Q: 'static,
        H: QueryHandler<Q> + 'static,
    {
        let type_id = TypeId::of::<Q>();
        let type_name = std::any::type_name::<Q>();

        if self.handlers.contains_key(&type_id) {
            return Err(QueryBusError::DuplicateHandler(type_name.to_string()));
        }

        self.handlers
            .insert(type_id, Box::new(HandlerWrapper { handler }));
        self.type_names.insert(type_id, type_name);
        Ok(())
    }

    /// Executes a query through its registered handler.
    ///
    /// # Type Parameters
    /// - `Q`: The query type to execute
    /// - `H`: The expected handler type (must match what was registered)
    ///
    /// # Errors
    /// - `QueryBusError::NoHandlerRegistered` if no handler exists for query type `Q`
    /// - `QueryBusError::HandlerFailed` if the handler returns an error
    pub fn execute<Q, H>(&self, query: Q) -> Result<H::Result, QueryBusError>
    where
        Q: 'static,
        H: QueryHandler<Q> + 'static,
        H::Error: Debug,
    {
        let type_id = TypeId::of::<Q>();
        let type_name = std::any::type_name::<Q>();

        let boxed = self
            .handlers
            .get(&type_id)
            .ok_or_else(|| QueryBusError::NoHandlerRegistered(type_name.to_string()))?;

        let handler = boxed
            .as_any()
            .downcast_ref::<H>()
            .expect("Handler type mismatch - this should not happen if register and execute use the same types");

        handler
            .handle(query)
            .map_err(|e| QueryBusError::HandlerFailed(format!("{:?}", e)))
    }

    /// Returns true if a handler is registered for query type Q.
    pub fn has_handler<Q: 'static>(&self) -> bool {
        self.handlers.contains_key(&TypeId::of::<Q>())
    }

    /// Returns the number of registered handlers.
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for QueryBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test query types
    #[derive(Debug)]
    struct GetItemById {
        id: i32,
    }

    #[derive(Debug)]
    struct ListItems {
        limit: usize,
    }

    // Test result types
    #[derive(Debug, PartialEq)]
    struct Item {
        id: i32,
        name: String,
    }

    // Test handler for GetItemById
    struct GetItemHandler;

    impl QueryHandler<GetItemById> for GetItemHandler {
        type Result = Item;
        type Error = String;

        fn handle(&self, query: GetItemById) -> Result<Item, String> {
            Ok(Item {
                id: query.id,
                name: format!("Item {}", query.id),
            })
        }
    }

    // Test handler for ListItems
    struct ListItemsHandler;

    impl QueryHandler<ListItems> for ListItemsHandler {
        type Result = Vec<Item>;
        type Error = String;

        fn handle(&self, query: ListItems) -> Result<Vec<Item>, String> {
            Ok((0..query.limit)
                .map(|i| Item {
                    id: i as i32,
                    name: format!("Item {}", i),
                })
                .collect())
        }
    }

    // Failing handler for testing error propagation
    struct FailingHandler;

    impl QueryHandler<GetItemById> for FailingHandler {
        type Result = Item;
        type Error = String;

        fn handle(&self, _query: GetItemById) -> Result<Item, String> {
            Err("Item not found".to_string())
        }
    }

    #[test]
    fn test_new_bus_is_empty() {
        let bus = QueryBus::new();
        assert_eq!(bus.handler_count(), 0);
    }

    #[test]
    fn test_register_handler() {
        let mut bus = QueryBus::new();
        let result = bus.register::<GetItemById, _>(GetItemHandler);
        assert!(result.is_ok());
        assert!(bus.has_handler::<GetItemById>());
        assert_eq!(bus.handler_count(), 1);
    }

    #[test]
    fn test_register_multiple_handlers() {
        let mut bus = QueryBus::new();
        bus.register::<GetItemById, _>(GetItemHandler).unwrap();
        bus.register::<ListItems, _>(ListItemsHandler).unwrap();

        assert!(bus.has_handler::<GetItemById>());
        assert!(bus.has_handler::<ListItems>());
        assert_eq!(bus.handler_count(), 2);
    }

    #[test]
    fn test_duplicate_handler_fails() {
        let mut bus = QueryBus::new();
        bus.register::<GetItemById, _>(GetItemHandler).unwrap();

        let result = bus.register::<GetItemById, _>(GetItemHandler);
        assert!(result.is_err());
        match result.unwrap_err() {
            QueryBusError::DuplicateHandler(name) => {
                assert!(name.contains("GetItemById"));
            }
            _ => panic!("Expected DuplicateHandler error"),
        }
    }

    #[test]
    fn test_execute_succeeds() {
        let mut bus = QueryBus::new();
        bus.register::<GetItemById, _>(GetItemHandler).unwrap();

        let query = GetItemById { id: 42 };
        let result = bus.execute::<GetItemById, GetItemHandler>(query);

        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.id, 42);
        assert_eq!(item.name, "Item 42");
    }

    #[test]
    fn test_execute_no_handler() {
        let bus = QueryBus::new();
        let query = GetItemById { id: 1 };

        let result = bus.execute::<GetItemById, GetItemHandler>(query);

        assert!(result.is_err());
        match result.unwrap_err() {
            QueryBusError::NoHandlerRegistered(name) => {
                assert!(name.contains("GetItemById"));
            }
            _ => panic!("Expected NoHandlerRegistered error"),
        }
    }

    #[test]
    fn test_execute_handler_error() {
        let mut bus = QueryBus::new();
        bus.register::<GetItemById, _>(FailingHandler).unwrap();

        let query = GetItemById { id: 1 };
        let result = bus.execute::<GetItemById, FailingHandler>(query);

        assert!(result.is_err());
        match result.unwrap_err() {
            QueryBusError::HandlerFailed(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected HandlerFailed error"),
        }
    }

    #[test]
    fn test_execute_correct_handler() {
        let mut bus = QueryBus::new();
        bus.register::<GetItemById, _>(GetItemHandler).unwrap();
        bus.register::<ListItems, _>(ListItemsHandler).unwrap();

        let result_get = bus.execute::<GetItemById, GetItemHandler>(GetItemById { id: 5 });
        let result_list = bus.execute::<ListItems, ListItemsHandler>(ListItems { limit: 3 });

        assert_eq!(result_get.unwrap().id, 5);
        assert_eq!(result_list.unwrap().len(), 3);
    }
}
