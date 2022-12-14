use crate::gitlab::deserialise::{
    hashmap_of_jobs, hashmap_of_templates, list_of_string_tuples_to_map,
    map_to_list_of_string_tuples, seq_string_or_struct, str_or_map_to_list_of_maps,
    string_or_seq_string,
};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::str::FromStr;

#[rustfmt::skip]
fn default_empty_list() -> Vec<String> { vec![] }
#[rustfmt::skip]
fn default_empty_include_list() -> Vec<Include> { vec![] }
#[rustfmt::skip]
fn default_file_include_ref() -> String { "HEAD".into() }
#[rustfmt::skip]
fn default_artifact_name() -> String { "artifacts.zip".into() }
#[rustfmt::skip]
fn default_when() -> When { When::OnSuccess }
#[rustfmt::skip]
fn default_true() -> bool { true }

// Keyword reference: https://docs.gitlab.com/ee/ci/yaml/

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Default)]
pub struct GitLabConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<GlobalDefaults>,

    #[serde(
        default = "default_empty_include_list",
        deserialize_with = "str_or_map_to_list_of_maps",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub include: Vec<Include>,

    #[serde(default = "default_empty_list", skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<String>,

    #[serde(
        default,
        deserialize_with = "map_to_list_of_string_tuples",
        serialize_with = "list_of_string_tuples_to_map",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub variables: Vec<(String, String)>,

    // `workflow` is a global keyword, but we don't do anything with it (yet).
    // It's defined in here, so that it doesn't get picked up as a regular job
    // in the jobs map below.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<Value>,

    #[serde(deserialize_with = "hashmap_of_jobs")]
    #[serde(flatten)]
    pub jobs: HashMap<String, Job>,

    #[serde(deserialize_with = "hashmap_of_templates")]
    #[serde(flatten)]
    pub templates: HashMap<String, Job>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
pub struct GlobalDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_script: Option<ListOfStrings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Artifacts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_script: Option<ListOfStrings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ListOfStrings(#[serde(deserialize_with = "string_or_seq_string")] pub Vec<String>);

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
pub struct Artifacts {
    #[serde(default = "default_artifact_name")]
    pub name: String,
    #[serde(default = "default_when")]
    pub when: When,
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum When {
    OnSuccess,
    OnFailure,
    Always,
}

impl Default for When {
    fn default() -> Self {
        Self::OnSuccess
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(untagged)]
pub enum Include {
    Local(LocalInclude),
    File(FileInclude),
    Remote(RemoteInclude),
    Template(TemplateInclude),
}

impl FromStr for Include {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Include::Local(LocalInclude {
            local: s.to_string(),
        }))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct LocalInclude {
    pub local: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct FileInclude {
    pub project: String,
    #[serde(default = "default_file_include_ref")]
    pub r#ref: String,
    #[serde(deserialize_with = "string_or_seq_string")]
    pub file: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct RemoteInclude {
    pub remote: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TemplateInclude {
    pub template: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
pub struct Job {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_script: Option<ListOfStrings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Artifacts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_script: Option<ListOfStrings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<ListOfStrings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needs: Option<OneOrMoreNeeds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<ListOfStrings>,
    #[serde(
        default,
        deserialize_with = "map_to_list_of_string_tuples",
        serialize_with = "list_of_string_tuples_to_map",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub variables: Vec<(String, String)>,
}

// Wrapping was necessary to get the custom deserializer work with an `Option`
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct OneOrMoreNeeds(#[serde(deserialize_with = "seq_string_or_struct")] pub Vec<Needs>);

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
pub struct Needs {
    pub job: String,
    #[serde(default = "default_true")]
    pub artifacts: bool,
}

impl FromStr for Needs {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Needs {
            job: s.to_string(),
            artifacts: true,
        })
    }
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

            assert_eq!(config.include.len(), 0);
        }

        #[test]
        fn deserialises_simple_include() {
            let yaml = "
                include: 'file.yml'
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.include,
                vec![Include::Local(LocalInclude {
                    local: "file.yml".into()
                })]
            );
        }

        #[test]
        fn deserialises_single_local_include() {
            let yaml = "
                include:
                  local: 'file.yml'
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.include,
                vec![Include::Local(LocalInclude {
                    local: "file.yml".into()
                })]
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
                config.include,
                vec![Include::File(FileInclude {
                    project: "project/group".into(),
                    r#ref: "main".into(),
                    file: vec!["/path/to/file.yml".into()]
                })]
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
                config.include,
                vec![Include::File(FileInclude {
                    project: "project/group".into(),
                    r#ref: "main".into(),
                    file: vec!["/path/to/file-a.yml".into(), "/path/to/file-b.yml".into()]
                })]
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
                config.include,
                vec![Include::Remote(RemoteInclude {
                    remote: "https://external.com/file.yml".into(),
                })]
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
                config.include,
                vec![Include::Template(TemplateInclude {
                    template: "template-file.yml".into(),
                })]
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
                config.include,
                vec![
                    Include::Local(LocalInclude {
                        local: "file.yml".into()
                    }),
                    Include::Remote(RemoteInclude {
                        remote: "https://external.com/file.yml".into(),
                    }),
                ]
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
                vec![("one".into(), "1".into()), ("two".into(), "2".into())]
            );
        }

        #[test]
        fn deserialises_variables_in_stable_key_order() {
            let yaml = "
                variables:
                  z: 0
                  '1': 1
                  a: 2
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(
                config.variables,
                vec![
                    ("z".into(), "0".into()),
                    ("1".into(), "1".into()),
                    ("a".into(), "2".into())
                ]
            );
        }
    }

    mod test_jobs {
        use super::*;

        #[test]
        fn deserialises_no_jobs_when_none_defined() {
            let yaml = "";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(config.jobs.len(), 0);
        }

        #[test]
        fn deserialises_all_non_global_keyword_not_starting_with_a_dot_as_jobs() {
            let yaml = "
                job-name:
                  image: dummy:name
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(config.jobs.len(), 1);
        }

        #[test]
        fn does_not_deserialises_jobs_that_start_with_a_dot() {
            let yaml = "
                .template-name:
                  image: dummy:name
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(config.jobs.len(), 0);
        }

        #[test]
        fn deserialises_empty_after_script_when_missing() {
            let yaml = "
                job-name:
                  after_script:
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert!(job.after_script.is_none());
        }

        #[test]
        fn deserialises_after_script_lines() {
            let yaml = "
                job-name:
                  after_script:
                    - script.sh
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert_eq!(
                job.after_script.as_ref().unwrap().0,
                vec!["script.sh".to_string()]
            );
        }

        #[test]
        fn deserialises_empty_artifacts_when_missing() {
            let yaml = "
                job-name:
                  image: dummy:name
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert!(job.artifacts.is_none());
        }

        #[test]
        fn deserialises_artifacts() {
            let yaml = "
                job-name:
                  artifacts:
                    paths:
                      - file
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert_eq!(
                job.artifacts.as_ref().unwrap().paths,
                vec!["file".to_string()]
            );
        }

        #[test]
        fn deserialises_empty_before_script_when_missing() {
            let yaml = "
                job-name:
                  before_script:
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert!(job.before_script.is_none());
        }

        #[test]
        fn deserialises_before_script_lines() {
            let yaml = "
                job-name:
                  before_script:
                    - script.sh
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert_eq!(
                job.before_script.as_ref().unwrap().0,
                vec!["script.sh".to_string()]
            );
        }

        #[test]
        fn deserialises_empty_extends_when_missing() {
            let yaml = "
                job-name:
                  extends:
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert!(job.extends.is_none());
        }

        #[test]
        fn deserialises_extend_lines() {
            let yaml = "
                job-name:
                  extends:
                    - .some-template
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert_eq!(
                job.extends.as_ref().unwrap().0,
                vec![".some-template".to_string()]
            );
        }

        #[test]
        fn deserialises_empty_image_when_missing() {
            let yaml = "
                job-name:
                  after_script:
                    - dummy.sh
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert!(job.image.is_none());
        }

        #[test]
        fn deserialises_job_image_names() {
            let yaml = "
                job-name:
                  image: image:name
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert_eq!(job.image.to_owned().unwrap(), "image:name".to_string());
        }

        #[test]
        fn deserialises_empty_needs_when_missing() {
            let yaml = "
                job-name:
                  image: dummy:name
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert!(job.needs.is_none());
        }

        #[test]
        fn deserialises_empty_needs() {
            let yaml = "
                job-name:
                  needs: []
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert_eq!(job.needs.as_ref().unwrap().0, vec![]);
        }

        #[test]
        fn deserialises_needs_with_list_of_job_names() {
            let yaml = "
                job-name:
                  needs:
                    - job-a
                    - job-b
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert_eq!(
                job.needs.as_ref().unwrap().0,
                vec![
                    Needs {
                        job: "job-a".to_string(),
                        artifacts: true
                    },
                    Needs {
                        job: "job-b".to_string(),
                        artifacts: true
                    },
                ]
            );
        }

        #[test]
        fn deserialises_needs_with_explicit_job_definition() {
            let yaml = "
                job-name:
                  needs:
                    - job: name-a
                      artifacts: false
                    - job: name-b
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert_eq!(
                job.needs.as_ref().unwrap().0,
                vec![
                    Needs {
                        job: "name-a".to_string(),
                        artifacts: false
                    },
                    Needs {
                        job: "name-b".to_string(),
                        artifacts: true
                    },
                ]
            );
        }

        #[test]
        fn deserialises_empty_script_when_missing() {
            let yaml = "
                job-name:
                  script:
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert!(job.script.is_none());
        }

        #[test]
        fn deserialises_script_lines() {
            let yaml = "
                job-name:
                  script:
                    - script.sh
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert_eq!(
                job.script.as_ref().unwrap().0,
                vec!["script.sh".to_string()]
            );
        }

        #[test]
        fn deserialises_empty_variables_when_missing() {
            let yaml = "
              job-name:
                image: dummy:name
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert!(job.script.is_none());
        }

        #[test]
        fn deserialises_variables() {
            let yaml = "
                job-name:
                  variables:
                    one: 1
                    two: 2
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert_eq!(
                job.variables,
                vec![("one".into(), "1".into()), ("two".into(), "2".into())]
            );
        }

        #[test]
        fn deserialises_variables_in_stable_key_order() {
            let yaml = "
                job-name:
                  variables:
                    z: 0
                    '1': 1
                    a: 2
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();
            let job = config.jobs.get("job-name").unwrap();

            assert_eq!(
                job.variables,
                vec![
                    ("z".into(), "0".into()),
                    ("1".into(), "1".into()),
                    ("a".into(), "2".into())
                ]
            );
        }
    }

    mod test_templates {
        use super::*;

        #[test]
        fn deserialises_all_non_global_keyword_starting_with_a_dot_as_templates() {
            let yaml = "
                .template-name:
                  image: dummy:name
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(config.templates.len(), 1);
        }

        #[test]
        fn does_not_deserialises_templates_that_do_not_start_with_a_dot() {
            let yaml = "
                job-name:
                  image: dummy:name
            ";
            let config = serde_yaml::from_str::<GitLabConfiguration>(yaml).unwrap();

            assert_eq!(config.templates.len(), 0);
        }
    }
}
