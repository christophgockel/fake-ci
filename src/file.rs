use async_trait::async_trait;
use reqwest::IntoUrl;
#[cfg(test)]
use std::collections::HashMap;
use std::error::Error;
#[cfg(test)]
use std::io::ErrorKind::NotFound;
use std::io::{Cursor, Read};
use std::path::Path;

#[async_trait(?Send)]
pub trait FileAccess {
    fn read_local_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Box<Cursor<Vec<u8>>>, Box<dyn Error>>;

    async fn read_remote_file<URL: IntoUrl>(
        &self,
        url: URL,
    ) -> Result<Box<Cursor<Vec<u8>>>, Box<dyn Error>>;
}

#[derive(Default)]
pub struct RealFileSystem;

#[async_trait(?Send)]
impl FileAccess for RealFileSystem {
    fn read_local_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Box<Cursor<Vec<u8>>>, Box<dyn Error>> {
        let mut file = std::fs::File::open(path)?;
        let mut contents = Vec::new();

        file.read_to_end(&mut contents)?;
        let cursor = Cursor::new(contents);

        Ok(Box::new(cursor))
    }

    async fn read_remote_file<URL: IntoUrl>(
        &self,
        url: URL,
    ) -> Result<Box<Cursor<Vec<u8>>>, Box<dyn Error>> {
        let response = reqwest::get(url).await?.bytes().await?;

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
    ) -> Result<Box<Cursor<Vec<u8>>>, Box<dyn Error>> {
        let file_path = path.as_ref().to_string_lossy().into_owned();
        let content = self.file_contents.get(&file_path).ok_or_else(|| {
            std::io::Error::new(NotFound, format!("File {} not stubbed!", file_path))
        })?;
        let cursor = Cursor::new(content.as_bytes().to_vec());

        Ok(Box::new(cursor))
    }

    async fn read_remote_file<URL: IntoUrl>(
        &self,
        url: URL,
    ) -> Result<Box<Cursor<Vec<u8>>>, Box<dyn Error>> {
        self.read_local_file(url.as_str())
    }
}
