use duct::cmd;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("unable to get current branch name {0}")]
    Branch(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("unable to get latest SHA {0}")]
    Sha(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl GitError {
    pub fn branch(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        GitError::Branch(error.into())
    }

    pub fn sha(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        GitError::Sha(error.into())
    }
}

#[derive(Default)]
pub struct GitDetails {
    pub branch_name: String,
    pub sha: String,
    pub short_sha: String,
}

pub fn read_details() -> Result<GitDetails, GitError> {
    let sha = cmd!("git", "rev-parse", "HEAD")
        .read()
        .map_err(GitError::sha)?;

    let short_sha = cmd!("git", "rev-parse", "--short", "HEAD")
        .read()
        .map_err(GitError::sha)?;

    let branch_name = cmd!("git", "rev-parse", "--abbrev-ref", "HEAD")
        .read()
        .map_err(GitError::branch)?;

    Ok(GitDetails {
        branch_name,
        sha,
        short_sha,
    })
}
