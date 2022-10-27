use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[rustfmt::skip]
fn default_empty_list() -> Vec<String> { vec![] }

#[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
struct GitLabConfiguration {
    #[serde(default = "default_empty_list", skip_serializing_if = "Vec::is_empty")]
    stages: Vec<String>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    variables: HashMap<String, String>,
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

    mod test_variables {
        use super::*;

        #[test]
        fn deserialises_empty_variables_when_missing() {
            let yaml = "";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(config.variables.len(), 0);
        }

        #[test]
        fn deserialises_variables() {
            let yaml = "
                variables:
                  one: 1
                  two: 2
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.variables,
                HashMap::from([("one".into(), "1".into()), ("two".into(), "2".into())])
            );
        }
    }
}
