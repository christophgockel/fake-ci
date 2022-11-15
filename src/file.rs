use async_trait::async_trait;
use reqwest::IntoUrl;
#[cfg(test)]
use std::collections::HashMap;
use std::env::current_dir;
use std::io::{Cursor, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileAccessError {
    #[error("cannot read file {0} ({1})")]
    CannotRead(String, #[source] Box<dyn std::error::Error + Send + Sync>),
    #[cfg(test)]
    #[error("file {0} has not been stubbed")]
    NotStubbed(String),
}

fn file_path<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().to_string()
}

fn full_url<URL: IntoUrl>(url: &URL) -> String {
    url.as_str().to_string()
}

impl FileAccessError {
    pub fn cannot_read<P: AsRef<Path>>(
        path: P,
        error: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        FileAccessError::CannotRead(file_path(path), error.into())
    }

    pub fn cannot_read_remote<URL: IntoUrl>(
        url: &URL,
        error: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        FileAccessError::CannotRead(full_url(url), error.into())
    }
}

#[async_trait(?Send)]
pub trait FileAccess {
    fn read_local_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Box<Cursor<Vec<u8>>>, FileAccessError>;

    async fn read_remote_file<URL: IntoUrl>(
        &self,
        url: URL,
    ) -> Result<Box<Cursor<Vec<u8>>>, FileAccessError>;

    fn read_current_directory(&self) -> Result<String, FileAccessError>;
}

#[derive(Default)]
pub struct RealFileSystem;

#[async_trait(?Send)]
impl FileAccess for RealFileSystem {
    fn read_local_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Box<Cursor<Vec<u8>>>, FileAccessError> {
        let mut file = std::fs::File::open(&path)
            .map_err(|e| FileAccessError::cannot_read(path.as_ref(), e))?;
        let mut contents = Vec::new();

        file.read_to_end(&mut contents)
            .map_err(|e| FileAccessError::cannot_read(&path, e))?;
        let cursor = Cursor::new(contents);

        Ok(Box::new(cursor))
    }

    async fn read_remote_file<URL: IntoUrl>(
        &self,
        url: URL,
    ) -> Result<Box<Cursor<Vec<u8>>>, FileAccessError> {
        let response = reqwest::get(url.as_str())
            .await
            .map_err(|e| FileAccessError::cannot_read_remote(&url, e))?
            .bytes()
            .await
            .map_err(|e| FileAccessError::cannot_read_remote(&url, e))?;

        Ok(Box::new(Cursor::new(response.to_vec())))
    }

    fn read_current_directory(&self) -> Result<String, FileAccessError> {
        let current_path = current_dir().map_err(|e| FileAccessError::cannot_read(".", e))?;

        Ok(current_path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct StubFiles {
    file_contents: HashMap<String, String>,
}

#[cfg(test)]
impl StubFiles {
    pub fn with_file(file_name: &str, content: &str) -> Self {
        Self {
            file_contents: HashMap::from([(file_name.into(), content.into())]),
        }
    }

    pub fn add_file(&mut self, file_name: &str, content: &str) {
        self.file_contents.insert(file_name.into(), content.into());
    }

    pub fn add_remote_file(&mut self, url: &str, content: &str) {
        self.add_file(url, content);
    }
}

#[cfg(test)]
#[async_trait(?Send)]
impl FileAccess for StubFiles {
    fn read_local_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Box<Cursor<Vec<u8>>>, FileAccessError> {
        let file_path = path.as_ref().to_string_lossy().into_owned();
        let content = self
            .file_contents
            .get(&file_path)
            .ok_or(FileAccessError::NotStubbed(file_path))?;
        let cursor = Cursor::new(content.as_bytes().to_vec());

        Ok(Box::new(cursor))
    }

    async fn read_remote_file<URL: IntoUrl>(
        &self,
        url: URL,
    ) -> Result<Box<Cursor<Vec<u8>>>, FileAccessError> {
        self.read_local_file(url.as_str())
    }

    fn read_current_directory(&self) -> Result<String, FileAccessError> {
        Ok(".".into())
    }
}
