pub mod prune;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("executing command failed {0}")]
    Unknown(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl CommandError {
    pub fn unknown(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        CommandError::Unknown(error.into())
    }
}
