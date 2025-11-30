use thiserror::Error;

/// Error type returned by provider implementations.
#[derive(Debug, Error)]
pub enum ProviderError {
    /// Generic provider-side error with textual description.
    #[error("provider error: {0}")]
    Provider(String),

    /// HTTP / transport errors.
    #[error("http error: {0}")]
    Http(String),

    /// Other errors.
    #[error("other error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for ProviderError {
    fn from(e: anyhow::Error) -> Self {
        ProviderError::Other(e.to_string())
    }
}
