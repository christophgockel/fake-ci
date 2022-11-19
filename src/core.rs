use crate::error::FakeCiError;
use crate::gitlab;
use crate::gitlab::configuration::{GitLabConfiguration, ListOfStrings, OneOrMoreNeeds};
use std::collections::HashMap;

#[derive(Default)]
pub struct CiDefinition {
    pub jobs: HashMap<String, Job>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Job {
    pub image: String,
    pub script: Vec<String>,
    pub variables: Vec<(String, String)>,
    pub artifacts: Vec<String>,
    pub required_artifacts: HashMap<String, Vec<String>>,
}

pub fn convert_configuration(
    configuration: &GitLabConfiguration,
) -> Result<CiDefinition, FakeCiError> {
    let jobs = configuration
        .jobs
        .iter()
        .map(|(key, value)| Ok((key.clone(), convert_job(value, &configuration.jobs)?)))
        .collect::<Result<HashMap<_, _>, FakeCiError>>()?;

    Ok(CiDefinition { jobs })
}

fn convert_job(
    job: &gitlab::configuration::Job,
    other_jobs: &HashMap<String, gitlab::configuration::Job>,
) -> Result<Job, FakeCiError> {
    let mut final_script = vec![];

    final_script.extend(content_or_default(&job.before_script));
    final_script.extend(content_or_default(&job.script));
    final_script.extend(content_or_default(&job.after_script));

    let mut required: HashMap<String, Vec<String>> = HashMap::new();

    if let Some(OneOrMoreNeeds(needs)) = &job.needs {
        for need in needs.iter() {
            let other_job = other_jobs.get(&need.job).unwrap();

            if let Some(job_artifacts) = &other_job.artifacts {
                required.insert(need.job.clone(), job_artifacts.paths.clone());
            }
        }
    }

    Ok(Job {
        image: job.image.as_ref().cloned().unwrap_or_default(),
        script: final_script,
        variables: job.variables.clone(),
        artifacts: job
            .artifacts
            .as_ref()
            .map(|artifacts| artifacts.paths.clone())
            .unwrap_or_default(),
        required_artifacts: required,
    })
}

fn content_or_default(maybe_list: &Option<ListOfStrings>) -> Vec<String> {
    maybe_list
        .as_ref()
        .map(|list| list.0.clone())
        .unwrap_or_default()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    mod test_gitlab_conversion {
        use super::*;
        use crate::gitlab;
        use crate::gitlab::configuration::{Artifacts, ListOfStrings, Needs, OneOrMoreNeeds};

        #[test]
        fn converts_gitlab_jobs() {
            let gitlab_configuration = GitLabConfiguration {
                jobs: HashMap::from([
                    ("job1".to_string(), gitlab::configuration::Job::default()),
                    ("job2".to_string(), gitlab::configuration::Job::default()),
                ]),
                ..Default::default()
            };

            let definition = convert_configuration(&gitlab_configuration).unwrap();

            assert_eq!(definition.jobs.len(), 2);
        }

        #[test]
        fn copies_job_image() {
            let other_jobs = HashMap::new();
            let gitlab_job = gitlab::configuration::Job {
                image: Some("image:name".into()),
                ..Default::default()
            };

            let job = convert_job(&gitlab_job, &other_jobs).unwrap();

            assert_eq!(job.image, "image:name".to_string());
        }

        #[test]
        fn copies_job_variables() {
            let other_jobs = HashMap::new();
            let gitlab_job = gitlab::configuration::Job {
                variables: vec![("VARIABLE".into(), "value".into())],
                ..Default::default()
            };

            let job = convert_job(&gitlab_job, &other_jobs).unwrap();

            assert_eq!(job.variables, vec![("VARIABLE".into(), "value".into())]);
        }

        #[test]
        fn combines_before_script_main_script_and_after_script_into_one() {
            let other_jobs = HashMap::new();
            let gitlab_job = gitlab::configuration::Job {
                before_script: Some(ListOfStrings(vec!["before-script".into()])),
                script: Some(ListOfStrings(vec!["script".into()])),
                after_script: Some(ListOfStrings(vec!["after-script".into()])),
                ..Default::default()
            };

            let job = convert_job(&gitlab_job, &other_jobs).unwrap();

            assert_eq!(
                job.script,
                vec![
                    "before-script".to_string(),
                    "script".to_string(),
                    "after-script".to_string()
                ]
            );
        }

        #[test]
        fn keeps_list_of_artifacts_to_extract() {
            let other_jobs = HashMap::new();
            let gitlab_job = gitlab::configuration::Job {
                artifacts: Some(gitlab::configuration::Artifacts {
                    paths: vec!["file-1".into(), "file-2".into()],
                    ..Default::default()
                }),
                ..Default::default()
            };

            let job = convert_job(&gitlab_job, &other_jobs).unwrap();

            assert_eq!(
                job.artifacts,
                vec!["file-1".to_string(), "file-2".to_string(),]
            );
        }

        #[test]
        fn knows_which_artifacts_it_needs_from_other_jobs() {
            let other_jobs = HashMap::from([(
                "other-job".to_string(),
                gitlab::configuration::Job {
                    artifacts: Some(Artifacts {
                        paths: vec!["file-1".into(), "file-2".into()],
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )]);
            let gitlab_job = gitlab::configuration::Job {
                needs: Some(OneOrMoreNeeds(vec![Needs {
                    job: "other-job".into(),
                    artifacts: true,
                }])),
                ..Default::default()
            };

            let job = convert_job(&gitlab_job, &other_jobs).unwrap();

            assert_eq!(
                job.required_artifacts,
                HashMap::from([(
                    "other-job".to_string(),
                    vec!["file-1".to_string(), "file-2".to_string()],
                )]),
            );
        }
    }
}
