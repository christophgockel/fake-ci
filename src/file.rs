use async_trait::async_trait;
use reqwest::IntoUrl;
#[cfg(test)]
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileAccessError {
    #[error("Cannot read file: {0}")]
    CannotRead(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[cfg(test)]
    #[error("File has not been stubbed: {0}")]
    NotStubbed(String),
}

impl FileAccessError {
    pub fn cannot_read(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        FileAccessError::CannotRead(error.into())
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
}

#[derive(Default)]
pub struct RealFileSystem;

#[async_trait(?Send)]
impl FileAccess for RealFileSystem {
    fn read_local_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Box<Cursor<Vec<u8>>>, FileAccessError> {
        let mut file = std::fs::File::open(path).map_err(FileAccessError::cannot_read)?;
        let mut contents = Vec::new();

        file.read_to_end(&mut contents)
            .map_err(FileAccessError::cannot_read)?;
        let cursor = Cursor::new(contents);

        Ok(Box::new(cursor))
    }

    async fn read_remote_file<URL: IntoUrl>(
        &self,
        url: URL,
    ) -> Result<Box<Cursor<Vec<u8>>>, FileAccessError> {
        let response = reqwest::get(url)
            .await
            .map_err(FileAccessError::cannot_read)?
            .bytes()
            .await
            .map_err(FileAccessError::cannot_read)?;

        Ok(Box::new(Cursor::new(response.to_vec())))
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
}
