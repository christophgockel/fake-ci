mod configuration;
mod deserialise;
mod merge;

use crate::gitlab::configuration::GitLabConfiguration;
use crate::gitlab::merge::{merge_image, merge_variables};

pub fn parse<R>(reader: R) -> Result<GitLabConfiguration, Box<dyn std::error::Error>>
where
    R: std::io::Read,
{
    let mut configuration: GitLabConfiguration = serde_yaml::from_reader(reader)?;

    for (_name, job) in configuration.jobs.iter_mut() {
        merge_variables(&configuration.variables, &mut job.variables);

        if let Some(defaults) = &configuration.default {
            merge_image(&defaults.image, &mut job.image);
        }
    }

    Ok(configuration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_yaml_configuration() {
        let content = "
            stages:
              - test
        ";

        let configuration = parse(content.as_bytes()).unwrap();

        assert_eq!(configuration.stages, vec!["test".to_string(),]);
    }

    mod test_merge_precedence_of_variables {
        use super::*;

        #[test]
        fn prepends_global_variables_into_jobs() {
            let content = "
                variables:
                  VARIABLE_A: 1

                job:
                  variables:
                    VARIABLE_B: 2
            ";

            let configuration = parse(content.as_bytes()).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.variables,
                vec![
                    ("VARIABLE_A".into(), "1".into()),
                    ("VARIABLE_B".into(), "2".into())
                ]
            );
        }
    }

    mod test_merge_precedence_of_image_names {
        use super::*;

        #[test]
        fn uses_global_image_when_job_does_not_define_one() {
            let content = "
                default:
                  image: default:image

                job:
                  variables:
                    DUMMY: true
            ";

            let configuration = parse(content.as_bytes()).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(job.image, Some("default:image".into()));
        }

        #[test]
        fn uses_job_image_when_job_does_define_one() {
            let content = "
                default:
                  image: default:image

                job:
                  image: job:image
            ";

            let configuration = parse(content.as_bytes()).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(job.image, Some("job:image".into()));
        }
    }
}
