use crate::commands::Command;
use crate::data_types::DataType;
use crate::error::Result;
use crate::protocol::Response;
use crate::storage::Storage;
use async_trait::async_trait;
use std::sync::Arc;

pub struct SetCommand {
    key: String,
    value: String,
}

impl SetCommand {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

#[async_trait]
impl Command for SetCommand {
    async fn execute(&self, storage: Arc<dyn Storage>) -> Result<Response> {
        storage.set(&self.key, DataType::String(self.value.clone())).await?;
        Ok(Response::Ok)
    }
}