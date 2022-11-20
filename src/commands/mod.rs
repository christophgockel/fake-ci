pub mod image;
pub mod prune;
pub mod run;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("unknown job '{0}'")]
    UnknownJob(String),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}
