use crate::simulator::router::RequestRouter;

/// Abstraction over route registration operations.
pub trait RouteRegistry {
    /// Register routes for a service using its base path.
    fn register_service(&mut self, service_name: &str, base_path: &str);
    /// Remove all routes for a service.
    fn unregister_service(&mut self, service_name: &str);
    /// Remove all registered routes.
    fn clear_all(&mut self);
    /// Return total number of registered routes.
    fn routes_count(&self) -> usize;
}

impl RouteRegistry for RequestRouter {
    fn register_service(&mut self, service_name: &str, base_path: &str) {
        self.register_service_routes(service_name, base_path);
    }

    fn unregister_service(&mut self, service_name: &str) {
        self.unregister_service_routes(service_name);
    }

    fn clear_all(&mut self) {
        self.clear_all_routes();
    }

    fn routes_count(&self) -> usize {
        RequestRouter::routes_count(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_clear_routes() {
        let mut registry = RequestRouter::new();
        registry.register_service("svc", "/api");
        assert_eq!(registry.routes_count(), 1);
        registry.clear_all();
        assert_eq!(registry.routes_count(), 0);
    }
}
