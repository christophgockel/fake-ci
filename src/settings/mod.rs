use crate::{FileAccess, Settings};
use std::path::Path;
use thiserror::Error;

pub mod structure;

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("syntax error {0}")]
    Syntax(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl SettingsError {
    pub fn syntax(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        SettingsError::Syntax(error.into())
    }
}

pub enum LoadedSettings {
    FromFile(Settings),
    Default(Settings),
}

pub async fn load_settings<P: AsRef<Path>>(
    path: P,
    file_access: &impl FileAccess,
) -> Result<LoadedSettings, SettingsError> {
    let settings = match file_access.read_local_file(path) {
        Ok(file) => {
            let configuration: Settings =
                serde_yaml::from_reader(file).map_err(SettingsError::syntax)?;

            LoadedSettings::FromFile(configuration)
        }
        Err(_) => LoadedSettings::Default(Settings::default()),
    };

    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::StubFiles;

    #[tokio::test]
    async fn returns_default_settings_when_reading_file_does_not_exist() {
        let file_access = StubFiles::default();
        let settings = load_settings("unknown.file", &file_access).await.unwrap();

        assert!(matches!(settings, LoadedSettings::Default(..)));
    }

    #[tokio::test]
    async fn returns_settings_from_file() {
        let file_access = StubFiles::with_file("valid.file", "");
        let settings = load_settings("valid.file", &file_access).await.unwrap();

        assert!(matches!(settings, LoadedSettings::FromFile(..)));
    }

    #[tokio::test]
    async fn returns_error_on_syntax_errors() {
        let file_access = StubFiles::with_file("invalid.file", "invalid-yaml-content");
        let result = load_settings("invalid.file", &file_access).await;

        assert!(result.is_err());
    }
}
