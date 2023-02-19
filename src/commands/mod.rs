pub mod image;
pub mod print;
pub mod prune;
pub mod run;

use crate::gitlab::error::GitLabError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("unknown job '{0}'")]
    UnknownJob(String),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    GitLab(#[from] GitLabError),
}
