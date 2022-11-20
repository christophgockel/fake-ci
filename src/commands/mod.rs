pub mod image;
pub mod prune;
pub mod run;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("unknown job '{0}'")]
    UnknownJob(String),
    #[error("error executing job {0}")]
    Execution(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

impl CommandError {
    pub fn execution(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        CommandError::Execution(error.into())
    }
}
