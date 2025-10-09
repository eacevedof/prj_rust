/// Route definition type
/// Equivalent to TypeScript's LzRouteType
#[derive(Debug, Clone)]
pub struct LzRouteType {
    pub method: String,
    pub path: String,
    pub controller: String,
    pub middlewares: Vec<String>,
}

impl LzRouteType {
    pub fn new(method: &str, path: &str, controller: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            controller: controller.to_string(),
            middlewares: Vec::new(),
        }
    }

    pub fn with_middlewares(mut self, middlewares: Vec<String>) -> Self {
        self.middlewares = middlewares;
        self
    }
}
