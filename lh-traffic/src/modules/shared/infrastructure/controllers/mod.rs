use async_trait::async_trait;
use crate::modules::shared::infrastructure::components::http::{LzRequest, LzResponse};

/// Base controller interface
#[async_trait]
pub trait ControllerBase {
    async fn invoke(&self, request: LzRequest) -> LzResponse;
}
