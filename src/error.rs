use crate::file::FileAccessError;
use crate::git::GitError;
use crate::gitlab::error::GitLabError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FakeCiError {
    #[error(transparent)]
    File(#[from] FileAccessError),
    #[error(transparent)]
    Git(#[from] GitError),
    #[error(transparent)]
    GitLab(#[from] GitLabError),
    #[error("unknown error: {0}")]
    Other(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl FakeCiError {
    pub fn other(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        FakeCiError::Other(error.into())
    }
}
