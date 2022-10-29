mod configuration;
mod deserialise;
mod merge;

use crate::gitlab::configuration::GitLabConfiguration;
use crate::gitlab::merge::{collect_template_names, merge_image, merge_script, merge_variables};
use std::io::ErrorKind;

pub fn parse<R>(reader: R) -> Result<GitLabConfiguration, Box<dyn std::error::Error>>
where
    R: std::io::Read,
{
    let mut configuration: GitLabConfiguration = serde_yaml::from_reader(reader)?;

    for (_name, job) in configuration.jobs.iter_mut() {
        let required_template_names = collect_template_names(job, &configuration.templates)?;

        for template_name in required_template_names {
            let template = configuration
                .templates
                .get(&template_name)
                .ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, "template not found"))?;

            merge_variables(&template.variables, &mut job.variables);
            merge_script(&template.after_script, &mut job.after_script);
        }

        merge_variables(&configuration.variables, &mut job.variables);

        if let Some(defaults) = &configuration.default {
            merge_script(&defaults.after_script, &mut job.after_script);
            merge_image(&defaults.image, &mut job.image);
            merge_script(&defaults.before_script, &mut job.before_script);
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

        #[test]
        fn insert_template_variables_between_global_and_job_ones() {
            let content = "
                variables:
                  VARIABLE_A: 1

                .template:
                  variables:
                    VARIABLE_B: 2

                job:
                  extends:
                    - .template
                  variables:
                    VARIABLE_C: 3
            ";

            let configuration = parse(content.as_bytes()).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.variables,
                vec![
                    ("VARIABLE_A".into(), "1".into()),
                    ("VARIABLE_B".into(), "2".into()),
                    ("VARIABLE_C".into(), "3".into()),
                ]
            );
        }
    }

    mod test_merge_precedence_of_after_scripts {
        use super::*;
        use crate::gitlab::configuration::ListOfStrings;

        #[test]
        fn uses_global_after_script_when_job_does_not_define_one() {
            let content = "
                default:
                  after_script:
                    - default_command.sh

                job:
                  variables:
                    DUMMY: true
            ";

            let configuration = parse(content.as_bytes()).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.after_script,
                Some(ListOfStrings(vec!["default_command.sh".into()]))
            );
        }

        #[test]
        fn uses_template_after_script_when_job_does_not_define_one() {
            let content = "
                default:
                  after_script:
                    - default_command.sh

                .template:
                  after_script:
                    - template_command.sh

                job:
                  extends:
                    - .template
            ";

            let configuration = parse(content.as_bytes()).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.after_script,
                Some(ListOfStrings(vec!["template_command.sh".into()]))
            );
        }

        #[test]
        fn uses_job_after_script_when_job_does_define_one() {
            let content = "
                default:
                  after_script:
                    - default_command.sh

                .template:
                  after_script:
                    - template_command.sh

                job:
                  extends:
                    - .template
                  after_script:
                    - job_command.sh
            ";

            let configuration = parse(content.as_bytes()).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.after_script,
                Some(ListOfStrings(vec!["job_command.sh".into()]))
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
    mod test_merge_precedence_of_before_scripts {
        use super::*;
        use crate::gitlab::configuration::ListOfStrings;

        #[test]
        fn uses_global_before_script_when_job_does_not_define_one() {
            let content = "
                default:
                  before_script:
                    - default_command.sh

                job:
                  variables:
                    DUMMY: true
            ";

            let configuration = parse(content.as_bytes()).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.before_script,
                Some(ListOfStrings(vec!["default_command.sh".into()]))
            );
        }

        #[test]
        fn uses_job_before_script_when_job_does_define_one() {
            let content = "
                default:
                  before_script:
                    - default_command.sh
                job:
                  before_script:
                    - job_command.sh
            ";

            let configuration = parse(content.as_bytes()).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.before_script,
                Some(ListOfStrings(vec!["job_command.sh".into()]))
            );
        }
    }
}
