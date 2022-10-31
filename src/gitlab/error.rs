use crate::file::FileAccessError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitLabError {
    #[error("Cannot parse configuration: {0}")]
    Parse(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("Cannot adjust URL")]
    AdjustUrl(),
    #[error("Cannot create URL: {0}")]
    CreateUrl(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error(transparent)]
    File(#[from] FileAccessError),
}

impl GitLabError {
    pub fn parse(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        GitLabError::Parse(error.into())
    }
    pub fn adjust_url(_: ()) -> Self {
        GitLabError::AdjustUrl()
    }

    pub fn create_url(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        GitLabError::CreateUrl(error.into())
    }

    pub fn file(error: FileAccessError) -> Self {
        GitLabError::File(error)
    }
}
