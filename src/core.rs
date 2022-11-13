use crate::error::FakeCiError;
use crate::gitlab;
use crate::gitlab::configuration::{GitLabConfiguration, ListOfStrings};
use std::collections::HashMap;

#[derive(Default)]
pub struct CiDefinition {
    pub jobs: HashMap<String, Job>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Job {
    pub script: Vec<String>,
    pub variables: Vec<(String, String)>,
    pub artifacts: Vec<String>,
}

pub fn convert_configuration(
    configuration: &GitLabConfiguration,
) -> Result<CiDefinition, FakeCiError> {
    let jobs = configuration
        .jobs
        .iter()
        .map(|(key, value)| Ok((key.clone(), convert_job(value)?)))
        .collect::<Result<HashMap<_, _>, FakeCiError>>()?;

    Ok(CiDefinition { jobs })
}

fn convert_job(job: &gitlab::configuration::Job) -> Result<Job, FakeCiError> {
    let mut final_script = vec![];

    final_script.extend(content_or_default(&job.before_script));
    final_script.extend(content_or_default(&job.script));
    final_script.extend(content_or_default(&job.after_script));

    Ok(Job {
        script: final_script,
        variables: job.variables.clone(),
        artifacts: job
            .artifacts
            .as_ref()
            .map(|artifacts| artifacts.paths.clone())
            .unwrap_or_default(),
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
        use crate::gitlab::configuration::ListOfStrings;

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
        fn copies_job_variables() {
            let gitlab_job = gitlab::configuration::Job {
                variables: vec![("VARIABLE".into(), "value".into())],
                ..Default::default()
            };

            let job = convert_job(&gitlab_job).unwrap();

            assert_eq!(job.variables, vec![("VARIABLE".into(), "value".into())]);
        }

        #[test]
        fn combines_before_script_main_script_and_after_script_into_one() {
            let gitlab_job = gitlab::configuration::Job {
                before_script: Some(ListOfStrings(vec!["before-script".into()])),
                script: Some(ListOfStrings(vec!["script".into()])),
                after_script: Some(ListOfStrings(vec!["after-script".into()])),
                ..Default::default()
            };

            let job = convert_job(&gitlab_job).unwrap();

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
            let gitlab_job = gitlab::configuration::Job {
                artifacts: Some(gitlab::configuration::Artifacts {
                    paths: vec!["file-1".into(), "file-2".into()],
                    ..Default::default()
                }),
                ..Default::default()
            };

            let job = convert_job(&gitlab_job).unwrap();

            assert_eq!(
                job.artifacts,
                vec!["file-1".to_string(), "file-2".to_string(),]
            );
        }
    }
}
