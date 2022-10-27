use serde::{Deserialize, Serialize};

#[rustfmt::skip]
fn default_empty_list() -> Vec<String> { vec![] }

#[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
struct GitLabConfiguration {
    #[serde(default = "default_empty_list", skip_serializing_if = "Vec::is_empty")]
    stages: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_stages {
        use super::*;

        #[test]
        fn deserialises_stages() {
            let yaml = "
                stages:
                  - a
                  - b
                  - c
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(config.stages, vec!["a", "b", "c"]);
        }

        #[test]
        fn deserialises_empty_stages_when_missing() {
            let yaml = "";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(config.stages.len(), 0);
        }
    }
}
