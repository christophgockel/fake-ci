mod configuration;
mod deserialise;
mod merge;

use crate::file::FileAccess;
use crate::gitlab::configuration::{GitLabConfiguration, Include, OneOrMoreIncludes};
use crate::gitlab::merge::{
    collect_template_names, merge_configuration, merge_image, merge_script, merge_variables,
};
use std::io::ErrorKind;

pub fn parse<R>(
    reader: R,
    file_access: &impl FileAccess,
) -> Result<GitLabConfiguration, Box<dyn std::error::Error>>
where
    R: std::io::Read,
{
    let mut configuration: GitLabConfiguration = serde_yaml::from_reader(reader)?;

    if let Some(additional_includes) = &configuration.include {
        let additional_configurations = read_all(additional_includes, file_access)?;

        for additional_configuration in additional_configurations {
            merge_configuration(&additional_configuration, &mut configuration);
        }
    }

    for (_name, job) in configuration.jobs.iter_mut() {
        let required_template_names = collect_template_names(job, &configuration.templates)?;

        for template_name in required_template_names {
            let template = configuration
                .templates
                .get(&template_name)
                .ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, "template not found"))?;

            merge_variables(&template.variables, &mut job.variables);
            merge_script(&template.after_script, &mut job.after_script);
            merge_script(&template.before_script, &mut job.before_script);
            merge_image(&template.image, &mut job.image);
        }

        merge_variables(&configuration.variables, &mut job.variables);

        if let Some(defaults) = &configuration.default {
            merge_script(&defaults.after_script, &mut job.after_script);
            merge_script(&defaults.before_script, &mut job.before_script);
            merge_image(&defaults.image, &mut job.image);
        }
    }

    Ok(configuration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::StubFiles;

    #[test]
    fn parses_yaml_configuration() {
        let file_dummy = StubFiles::default();
        let content = "
            stages:
              - test
        ";

        let configuration = parse(content.as_bytes(), &file_dummy).unwrap();

        assert_eq!(configuration.stages, vec!["test".to_string(),]);
    }

    mod test_merge_precedence_of_variables {
        use super::*;

        #[test]
        fn prepends_global_variables_into_jobs() {
            let file_dummy = StubFiles::default();
            let content = "
                variables:
                  VARIABLE_A: 1

                job:
                  variables:
                    VARIABLE_B: 2
            ";

            let configuration = parse(content.as_bytes(), &file_dummy).unwrap();
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
            let file_dummy = StubFiles::default();
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

            let configuration = parse(content.as_bytes(), &file_dummy).unwrap();
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
            let file_dummy = StubFiles::default();
            let content = "
                default:
                  after_script:
                    - default_command.sh

                job:
                  variables:
                    DUMMY: true
            ";

            let configuration = parse(content.as_bytes(), &file_dummy).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.after_script,
                Some(ListOfStrings(vec!["default_command.sh".into()]))
            );
        }

        #[test]
        fn uses_template_after_script_when_job_does_not_define_one() {
            let file_dummy = StubFiles::default();
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

            let configuration = parse(content.as_bytes(), &file_dummy).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.after_script,
                Some(ListOfStrings(vec!["template_command.sh".into()]))
            );
        }

        #[test]
        fn uses_job_after_script_when_job_does_define_one() {
            let file_dummy = StubFiles::default();
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

            let configuration = parse(content.as_bytes(), &file_dummy).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.after_script,
                Some(ListOfStrings(vec!["job_command.sh".into()]))
            );
        }
    }

    mod test_merge_precedence_of_before_scripts {
        use super::*;
        use crate::gitlab::configuration::ListOfStrings;

        #[test]
        fn uses_global_before_script_when_job_does_not_define_one() {
            let file_dummy = StubFiles::default();
            let content = "
                default:
                  before_script:
                    - default_command.sh

                job:
                  variables:
                    DUMMY: true
            ";

            let configuration = parse(content.as_bytes(), &file_dummy).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.before_script,
                Some(ListOfStrings(vec!["default_command.sh".into()]))
            );
        }

        #[test]
        fn uses_template_before_script_when_job_does_not_define_one() {
            let file_dummy = StubFiles::default();
            let content = "
                default:
                  before_script:
                    - default_command.sh

                .template:
                  before_script:
                    - template_command.sh

                job:
                  extends:
                    - .template
            ";

            let configuration = parse(content.as_bytes(), &file_dummy).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.before_script,
                Some(ListOfStrings(vec!["template_command.sh".into()]))
            );
        }

        #[test]
        fn uses_job_before_script_when_job_does_define_one() {
            let file_dummy = StubFiles::default();
            let content = "
                default:
                  before_script:
                    - default_command.sh

                .template:
                  before_script:
                    - template_command.sh

                job:
                  extends:
                    - .template
                  before_script:
                    - job_command.sh
            ";

            let configuration = parse(content.as_bytes(), &file_dummy).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.before_script,
                Some(ListOfStrings(vec!["job_command.sh".into()]))
            );
        }
    }

    mod test_merge_precedence_of_image_names {
        use super::*;

        #[test]
        fn uses_global_image_when_job_does_not_define_one() {
            let file_dummy = StubFiles::default();
            let content = "
                default:
                  image: default:image

                job:
                  variables:
                    DUMMY: true
            ";

            let configuration = parse(content.as_bytes(), &file_dummy).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(job.image, Some("default:image".into()));
        }

        #[test]
        fn uses_template_image_when_job_does_not_define_one() {
            let file_dummy = StubFiles::default();
            let content = "
                default:
                  image: default:image

                .template:
                  image: template:image

                job:
                  extends:
                    - .template
            ";

            let configuration = parse(content.as_bytes(), &file_dummy).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(job.image, Some("template:image".into()));
        }

        #[test]
        fn uses_job_image_when_job_does_define_one() {
            let file_dummy = StubFiles::default();
            let content = "
                default:
                  image: default:image

                .template:
                  image: template:image

                job:
                  extends:
                    - .template
                  image: job:image
            ";

            let configuration = parse(content.as_bytes(), &file_dummy).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(job.image, Some("job:image".into()));
        }
    }

    mod test_includes {
        use super::*;
        use crate::file::StubFiles;

        #[test]
        fn resolves_and_adds_variables_from_local_files() {
            let other_content = "
                variables:
                  OTHER_VARIABLE: true
            ";
            let files = StubFiles::with_file("other.yml", other_content);
            let content = "
                include:
                  local: other.yml
            ";

            let config = parse(content.as_bytes(), &files).unwrap();

            assert_eq!(config.variables.len(), 1);
        }

        #[test]
        fn resolves_and_adds_variables_from_multiple_local_files() {
            let file_a_content = "
                variables:
                  FILE_A: value a
            ";
            let file_b_content = "
                variables:
                  FILE_B: value b
            ";
            let mut files = StubFiles::default();
            files.add_file("file-a.yml", file_a_content);
            files.add_file("file-b.yml", file_b_content);

            let content = "
                include:
                  - local: file-a.yml
                  - local: file-b.yml
            ";

            let config = parse(content.as_bytes(), &files).unwrap();

            assert_eq!(config.variables.len(), 2);
        }
    }
}

fn read_all(
    includes: &OneOrMoreIncludes,
    file_access: &impl FileAccess,
) -> Result<Vec<GitLabConfiguration>, Box<dyn std::error::Error>> {
    let mut included_configurations = vec![];

    match includes {
        OneOrMoreIncludes::Single(include) => {
            included_configurations.push(read_and_parse(include, file_access)?);
        }
        OneOrMoreIncludes::Multiple(includes) => {
            for include in includes {
                included_configurations.push(read_and_parse(include, file_access)?);
            }
        }
    }

    Ok(included_configurations)
}

fn read_and_parse(
    include: &Include,
    file_access: &impl FileAccess,
) -> Result<GitLabConfiguration, Box<dyn std::error::Error>> {
    match include {
        Include::Local(local_include) => {
            let content = file_access.read_local_file(&local_include.local)?;
            let configuration = parse(*content, file_access)?;

            Ok(configuration)
        }
        Include::File(_) => todo!(),
        Include::Remote(_) => todo!(),
        Include::Template(_) => todo!(),
    }
}
