use super::LzRouteType;
use std::collections::HashMap;

/// Router for managing HTTP routes
/// Equivalent to TypeScript's LzRouter
pub struct LzRouter {
    routes: HashMap<String, LzRouteType>,
}

impl LzRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Register a route
    pub fn register(&mut self, route: LzRouteType) {
        let key = format!("{}:{}", route.method, route.path);
        self.routes.insert(key, route);
    }

    /// Get a route by method and path
    pub fn get_route(&self, method: &str, path: &str) -> Option<&LzRouteType> {
        let key = format!("{}:{}", method, path);
        self.routes.get(&key)
    }

    /// Get all routes
    pub fn get_routes(&self) -> &HashMap<String, LzRouteType> {
        &self.routes
    }
}

impl Default for LzRouter {
    fn default() -> Self {
        Self::new()
    }
}
