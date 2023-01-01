use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Default)]
pub struct Settings {
    #[serde(default)]
    pub gitlab: GitlabSettings,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
pub struct GitlabSettings {
    pub host: String,
}

impl Default for GitlabSettings {
    fn default() -> Self {
        Self {
            host: "https://gitlab.com".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_gitlab {
        use super::*;

        #[test]
        fn deserialises_host() {
            let yaml = "
                gitlab:
                  host: https://example.com
            ";
            let config = serde_yaml::from_str::<Settings>(yaml).unwrap();

            assert_eq!(config.gitlab.host, "https://example.com".to_string());
        }

        #[test]
        fn deserialises_default_host_when_missing() {
            let yaml = "";
            let config = serde_yaml::from_str::<Settings>(yaml).unwrap();

            assert_eq!(config.gitlab.host, "https://gitlab.com".to_string());
        }
    }
}
