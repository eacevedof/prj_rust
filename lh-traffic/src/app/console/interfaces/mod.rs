use async_trait::async_trait;

/// Console interface trait
/// Equivalent to TypeScript's ConsoleInterface
#[async_trait]
pub trait ConsoleInterface {
    async fn invoke(&self);
}
