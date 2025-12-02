use model_provider::ProviderError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("Context too long: {0} messages")]
    ContextTooLong(usize),

    #[error("Invalid model: {0}")]
    InvalidModel(String),

    #[error("Tool execution failed: {0}")]
    ToolError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
