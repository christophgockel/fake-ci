use duct::cmd;
use regex::Regex;
use std::process::Command;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("cannot execute git to read remote URL: {0}")]
    Execute(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("no git host configured.")]
    NoHostFound(),
    #[error("remote URL is not supported yet (only ssh): {0}")]
    UnsupportedUrl(String),
    #[error("URL for remote git host is invalid: {0}")]
    InvalidUrl(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("unable to get latest SHA {0}")]
    Sha(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl GitError {
    pub fn execute(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        GitError::Execute(error.into())
    }

    pub fn invalid_url(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        GitError::InvalidUrl(error.into())
    }

    pub fn sha(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        GitError::Sha(error.into())
    }
}

#[derive(Default)]
pub struct GitDetails {
    pub host: String,
    pub sha: String,
}

pub fn read_details() -> Result<GitDetails, GitError> {
    let output = Command::new("git")
        .arg("ls-remote")
        .arg("--get-url")
        .output()
        .map_err(GitError::execute)?;
    let content = std::str::from_utf8(&output.stdout).unwrap();
    let mut content = content.trim().to_string();

    if content.is_empty() {
        return Err(GitError::NoHostFound());
    }

    let mut host = None;

    // looks for structure like git@github.com:username/repo.git
    let ssh_url = Regex::new(r"^\S+(@)\S+(:).*$").unwrap();

    if ssh_url.is_match(&content) {
        content = content.replace(':', "/");
        content = content.replace("git@", "https://");

        let url = Url::parse(&content).map_err(GitError::invalid_url)?;

        match url.host() {
            None => {}
            Some(h) => {
                host = Some(format!("{}://{}", url.scheme(), h));
            }
        }
    } else {
        return Err(GitError::UnsupportedUrl(content));
    }
    let sha = cmd!("git", "rev-parse", "--short", "HEAD")
        .read()
        .map_err(GitError::sha)?;

    Ok(GitDetails {
        host: host.unwrap_or_else(|| "".into()),
        sha,
    })
}
