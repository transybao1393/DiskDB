use crate::commands::Command;
use crate::data_types::DataType;
use crate::error::{DiskDBError, Result};
use crate::protocol::Response;
use crate::storage::Storage;
use async_trait::async_trait;
use std::sync::Arc;

pub struct GetCommand {
    key: String,
}

impl GetCommand {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

#[async_trait]
impl Command for GetCommand {
    async fn execute(&self, storage: Arc<dyn Storage>) -> Result<Response> {
        match storage.get(&self.key).await? {
            Some(DataType::String(value)) => Ok(Response::String(Some(value))),
            Some(_) => Err(DiskDBError::Protocol("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
            None => Err(DiskDBError::KeyNotFound(self.key.clone())),
        }
    }
}