mod deserialise;

use deserialise::string_or_seq_string;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[rustfmt::skip]
fn default_empty_list() -> Vec<String> { vec![] }
#[rustfmt::skip]
fn default_file_include_ref() -> String { "HEAD".into() }
#[rustfmt::skip]
fn default_artifact_name() -> String { "artifacts.zip".into() }
#[rustfmt::skip]
fn default_when() -> When { When::OnSuccess }

#[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
struct GitLabConfiguration {
    default: Option<GlobalDefaults>,

    #[serde(skip_serializing_if = "Option::is_none")]
    include: Option<OneOrMoreIncludes>,

    #[serde(default = "default_empty_list", skip_serializing_if = "Vec::is_empty")]
    stages: Vec<String>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    variables: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
struct GlobalDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    after_script: Option<ListOfStrings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifacts: Option<Artifacts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    before_script: Option<ListOfStrings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct ListOfStrings(#[serde(deserialize_with = "string_or_seq_string")] Vec<String>);

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
struct Artifacts {
    #[serde(default = "default_artifact_name")]
    name: String,
    #[serde(default = "default_when")]
    when: When,
    #[serde(default)]
    paths: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
enum When {
    OnSuccess,
    OnFailure,
    Always,
}

impl Default for When {
    fn default() -> Self {
        Self::OnSuccess
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
enum OneOrMoreIncludes {
    Single(Include),
    Multiple(Vec<Include>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
enum Include {
    Local(LocalInclude),
    File(FileInclude),
    Remote(RemoteInclude),
    Template(TemplateInclude),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct LocalInclude {
    local: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct FileInclude {
    project: String,
    #[serde(default = "default_file_include_ref")]
    r#ref: String,
    #[serde(deserialize_with = "string_or_seq_string")]
    file: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct RemoteInclude {
    remote: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TemplateInclude {
    template: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_default {
        use super::*;

        #[test]
        fn deserialises_empty_default_when_missing() {
            let yaml = "";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert!(config.default.is_none());
        }

        #[test]
        fn deserialises_empty_after_script_when_missing() {
            let yaml = "
                default:
                  after_script:
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert!(config.default.unwrap().after_script.is_none());
        }

        #[test]
        fn deserialises_single_after_script_line() {
            let yaml = "
                default:
                  after_script: script.sh
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.default.unwrap().after_script.unwrap().0,
                vec!["script.sh".to_string()]
            );
        }

        #[test]
        fn deserialises_multiple_after_script_line() {
            let yaml = "
                default:
                  after_script:
                    - script-a.sh
                    - script-b.sh
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.default.unwrap().after_script.unwrap().0,
                vec!["script-a.sh".to_string(), "script-b.sh".to_string()]
            );
        }

        #[test]
        fn deserialises_empty_artifacts_when_missing() {
            let yaml = "
                default:
                  artifacts:
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert!(config.default.unwrap().artifacts.is_none());
        }

        #[test]
        fn deserialises_artifacts_paths() {
            let yaml = "
                default:
                  artifacts:
                    paths:
                      - file-a
                      - file-b
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let artifacts = config.default.unwrap().artifacts.unwrap();

            assert_eq!(
                artifacts.paths,
                vec!["file-a".to_string(), "file-b".to_string()]
            );
        }

        #[test]
        fn deserialises_artifacts_with_default_name_when_not_set() {
            let yaml = "
                default:
                  artifacts:
                    paths:
                      - dummy-file-path
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let artifacts = config.default.unwrap().artifacts.unwrap();

            assert_eq!(artifacts.name, "artifacts.zip");
        }

        #[test]
        fn deserialises_artifacts_name() {
            let yaml = "
                default:
                  artifacts:
                    name: the-name.zip
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let artifacts = config.default.unwrap().artifacts.unwrap();

            assert_eq!(artifacts.name, "the-name.zip");
        }

        #[test]
        fn deserialises_artifacts_with_when_on_success_by_default() {
            let yaml = "
                default:
                  artifacts:
                    name: dummy-name
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let artifacts = config.default.unwrap().artifacts.unwrap();

            assert_eq!(artifacts.when, When::OnSuccess);
        }

        #[test]
        fn deserialises_artifacts_with_when_on_success() {
            let yaml = "
                default:
                  artifacts:
                    when: on_success
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let artifacts = config.default.unwrap().artifacts.unwrap();

            assert_eq!(artifacts.when, When::OnSuccess);
        }

        #[test]
        fn deserialises_artifacts_with_when_on_failure() {
            let yaml = "
                default:
                  artifacts:
                    when: on_failure
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let artifacts = config.default.unwrap().artifacts.unwrap();

            assert_eq!(artifacts.when, When::OnFailure);
        }

        #[test]
        fn deserialises_artifacts_with_when_always() {
            let yaml = "
                default:
                  artifacts:
                    when: always
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let artifacts = config.default.unwrap().artifacts.unwrap();

            assert_eq!(artifacts.when, When::Always);
        }

        #[test]
        fn deserialises_empty_before_script_when_missing() {
            let yaml = "
                default:
                  before_script:
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert!(config.default.unwrap().before_script.is_none());
        }

        #[test]
        fn deserialises_single_before_script_line() {
            let yaml = "
                default:
                  before_script: script.sh
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.default.unwrap().before_script.unwrap().0,
                vec!["script.sh".to_string()]
            );
        }

        #[test]
        fn deserialises_multiple_before_script_line() {
            let yaml = "
                default:
                  before_script:
                    - script-a.sh
                    - script-b.sh
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.default.unwrap().before_script.unwrap().0,
                vec!["script-a.sh".to_string(), "script-b.sh".to_string()]
            );
        }

        #[test]
        fn deserialises_empty_image_when_missing() {
            let yaml = "
                default:
                  after_script:
                    - dummy-line
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert!(config.default.unwrap().image.is_none());
        }

        #[test]
        fn deserialises_image() {
            let yaml = "
                default:
                  image: image:name
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.default.unwrap().image.unwrap(),
                "image:name".to_string()
            );
        }
    }

    mod test_include {
        use super::*;

        #[test]
        fn deserialises_empty_include_when_missing() {
            let yaml = "";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert!(config.include.is_none());
        }

        // TODO: doesn't work nicely with the rest of the untagged enum yet
        // #[test]
        // fn deserialises_simple_include() {
        //     let yaml = "
        //         include: 'file.yml'
        //     ";
        //     let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
        //
        //     assert_eq!(
        //         config.include.unwrap(),
        //         OneOrMoreIncludes::Single(Include::Local(LocalInclude {
        //             local: "file.yml".into()
        //         }))
        //     );
        // }

        #[test]
        fn deserialises_single_local_include() {
            let yaml = "
                include:
                  local: 'file.yml'
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.include.unwrap(),
                OneOrMoreIncludes::Single(Include::Local(LocalInclude {
                    local: "file.yml".into()
                }))
            );
        }

        #[test]
        fn deserialises_single_file_include() {
            let yaml = "
                include:
                  project: 'project/group'
                  ref: main
                  file: /path/to/file.yml
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.include.unwrap(),
                OneOrMoreIncludes::Single(Include::File(FileInclude {
                    project: "project/group".into(),
                    r#ref: "main".into(),
                    file: vec!["/path/to/file.yml".into()]
                }))
            );
        }

        #[test]
        fn deserialises_single_file_include_with_multiple_paths() {
            let yaml = "
                include:
                  project: 'project/group'
                  ref: main
                  file:
                    - /path/to/file-a.yml
                    - /path/to/file-b.yml
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.include.unwrap(),
                OneOrMoreIncludes::Single(Include::File(FileInclude {
                    project: "project/group".into(),
                    r#ref: "main".into(),
                    file: vec!["/path/to/file-a.yml".into(), "/path/to/file-b.yml".into()]
                }))
            );
        }

        #[test]
        fn deserialises_single_remote_include() {
            let yaml = "
                include:
                  remote: 'https://external.com/file.yml'
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.include.unwrap(),
                OneOrMoreIncludes::Single(Include::Remote(RemoteInclude {
                    remote: "https://external.com/file.yml".into(),
                }))
            );
        }

        #[test]
        fn deserialises_single_template_include() {
            let yaml = "
                include:
                  template: 'template-file.yml'
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.include.unwrap(),
                OneOrMoreIncludes::Single(Include::Template(TemplateInclude {
                    template: "template-file.yml".into(),
                }))
            );
        }

        #[test]
        fn deserialises_multiple_includes() {
            let yaml = "
                include:
                  - local: 'file.yml'
                  - remote: 'https://external.com/file.yml'
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.include.unwrap(),
                OneOrMoreIncludes::Multiple(vec![
                    Include::Local(LocalInclude {
                        local: "file.yml".into()
                    }),
                    Include::Remote(RemoteInclude {
                        remote: "https://external.com/file.yml".into(),
                    }),
                ])
            );
        }
    }

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
