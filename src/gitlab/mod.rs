mod configuration;
mod deserialise;
mod merge;

use crate::file::FileAccess;
use crate::git::GitDetails;
use crate::gitlab::configuration::{GitLabConfiguration, Include, OneOrMoreIncludes};
use crate::gitlab::merge::{
    collect_template_names, merge_configuration, merge_image, merge_script, merge_variables,
};
use std::io::ErrorKind;

pub fn parse<R>(reader: R) -> Result<GitLabConfiguration, Box<dyn std::error::Error>>
where
    R: std::io::Read,
{
    let configuration: GitLabConfiguration = serde_yaml::from_reader(reader)?;

    Ok(configuration)
}

pub fn merge_all(
    additional_configurations: Vec<GitLabConfiguration>,
    configuration: &mut GitLabConfiguration,
) -> Result<(), Box<dyn std::error::Error>> {
    for additional_configuration in additional_configurations {
        merge_configuration(&additional_configuration, configuration);
    }

    merge_jobs(configuration)?;

    Ok(())
}

pub fn merge_jobs(
    configuration: &mut GitLabConfiguration,
) -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}

pub async fn parse_all(
    maybe_includes: &Option<OneOrMoreIncludes>,
    file_access: &impl FileAccess,
    git_details: &GitDetails,
) -> Result<Vec<GitLabConfiguration>, Box<dyn std::error::Error>> {
    let mut included_configurations = vec![];

    if let Some(includes) = maybe_includes {
        match includes {
            OneOrMoreIncludes::Single(include) => {
                included_configurations
                    .extend(read_and_parse(include, file_access, git_details).await?);
            }
            OneOrMoreIncludes::Multiple(includes) => {
                for include in includes {
                    included_configurations
                        .extend(read_and_parse(include, file_access, git_details).await?);
                }
            }
        }
    }

    Ok(included_configurations)
}

async fn read_and_parse(
    include: &Include,
    file_access: &impl FileAccess,
    git_details: &GitDetails,
) -> Result<Vec<GitLabConfiguration>, Box<dyn std::error::Error>> {
    match include {
        Include::Local(local_include) => {
            let content = file_access.read_local_file(&local_include.local)?;
            let configuration = parse(*content)?;

            Ok(vec![configuration])
        }
        Include::File(file_include) => {
            let mut configurations = vec![];

            for file in &file_include.file {
                let url = format!(
                    "{}/{}/-/raw/{}/{}",
                    &git_details.host, file_include.project, file_include.r#ref, file
                );
                let content = file_access.read_remote_file(&url).await?;
                let configuration = parse(*content)?;

                configurations.push(configuration);
            }

            Ok(configurations)
        }
        Include::Remote(remote_include) => {
            let content = file_access.read_remote_file(&remote_include.remote).await?;
            let configuration = parse(*content)?;

            Ok(vec![configuration])
        }
        Include::Template(template_include) => {
            let url = format!(
                "https://gitlab.com/gitlab-org/gitlab/-/raw/master/lib/gitlab/ci/templates/{}",
                &template_include.template
            );
            let content = file_access.read_remote_file(&url).await?;
            let configuration = parse(*content)?;

            Ok(vec![configuration])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_merge(content: &str) -> Result<GitLabConfiguration, Box<dyn std::error::Error>> {
        let mut configuration = parse(content.as_bytes())?;

        merge_jobs(&mut configuration)?;

        Ok(configuration)
    }

    #[test]
    fn parses_yaml_configuration() {
        let content = "
            stages:
              - test
        ";

        let configuration = parse_and_merge(content).unwrap();

        assert_eq!(configuration.stages, vec!["test".to_string()]);
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

            let configuration = parse_and_merge(content).unwrap();
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

            let configuration = parse_and_merge(content).unwrap();
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

            let configuration = parse_and_merge(content).unwrap();
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

            let configuration = parse_and_merge(content).unwrap();
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

            let configuration = parse_and_merge(content).unwrap();
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
            let content = "
                default:
                  before_script:
                    - default_command.sh

                job:
                  variables:
                    DUMMY: true
            ";

            let configuration = parse_and_merge(content).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.before_script,
                Some(ListOfStrings(vec!["default_command.sh".into()]))
            );
        }

        #[test]
        fn uses_template_before_script_when_job_does_not_define_one() {
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

            let configuration = parse_and_merge(content).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(
                job.before_script,
                Some(ListOfStrings(vec!["template_command.sh".into()]))
            );
        }

        #[test]
        fn uses_job_before_script_when_job_does_define_one() {
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

            let configuration = parse_and_merge(content).unwrap();
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
            let content = "
                default:
                  image: default:image

                job:
                  variables:
                    DUMMY: true
            ";

            let configuration = parse_and_merge(content).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(job.image, Some("default:image".into()));
        }

        #[test]
        fn uses_template_image_when_job_does_not_define_one() {
            let content = "
                default:
                  image: default:image

                .template:
                  image: template:image

                job:
                  extends:
                    - .template
            ";

            let configuration = parse_and_merge(content).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(job.image, Some("template:image".into()));
        }

        #[test]
        fn uses_job_image_when_job_does_define_one() {
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

            let configuration = parse_and_merge(content).unwrap();
            let job = configuration.jobs.get("job").unwrap();

            assert_eq!(job.image, Some("job:image".into()));
        }
    }

    mod test_include_parsing {
        use super::*;
        use crate::file::StubFiles;

        #[tokio::test]
        async fn resolves_local_files() {
            let git_dummy = GitDetails::default();
            let other_content = "
                variables:
                  OTHER_VARIABLE: true
            ";
            let files = StubFiles::with_file("other.yml", other_content);
            let content = "
                include:
                  local: other.yml
            ";

            let configuration = parse_and_merge(content).unwrap();
            let additional_configurations = parse_all(&configuration.include, &files, &git_dummy)
                .await
                .unwrap();

            assert_eq!(additional_configurations.len(), 1);
        }

        #[tokio::test]
        async fn resolves_multiple_local_files() {
            let git_dummy = GitDetails::default();
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

            let configuration = parse_and_merge(content).unwrap();
            let additional_configurations = parse_all(&configuration.include, &files, &git_dummy)
                .await
                .unwrap();

            assert_eq!(additional_configurations.len(), 2);
        }

        #[tokio::test]
        async fn resolves_gitlab_project_files() {
            let git_details = GitDetails {
                host: "https://example-gitlab.com".into(),
            };
            let file_a_content = "
                variables:
                  FILE_A: value a
            ";
            let file_b_content = "
                variables:
                  FILE_B: value b
            ";
            let mut files = StubFiles::default();
            files.add_remote_file(
                "https://example-gitlab.com/the-group/the-project/-/raw/main/file-a.yml",
                file_a_content,
            );
            files.add_remote_file(
                "https://example-gitlab.com/the-group/the-project/-/raw/main/file-b.yml",
                file_b_content,
            );

            let content = "
                include:
                  project: the-group/the-project
                  ref: main
                  file:
                    - file-a.yml
                    - file-b.yml
            ";

            let configuration = parse_and_merge(content).unwrap();
            let additional_configurations = parse_all(&configuration.include, &files, &git_details)
                .await
                .unwrap();

            assert_eq!(additional_configurations.len(), 2);
        }

        #[tokio::test]
        async fn resolves_remote_files() {
            let git_dummy = GitDetails::default();
            let file_content = "
                variables:
                  FILE: value
            ";
            let mut files = StubFiles::default();
            files.add_remote_file("https://example.com/path/to/file.yml", file_content);

            let content = "
                include:
                  remote: https://example.com/path/to/file.yml
            ";

            let configuration = parse_and_merge(content).unwrap();
            let additional_configurations = parse_all(&configuration.include, &files, &git_dummy)
                .await
                .unwrap();

            assert_eq!(additional_configurations.len(), 1);
        }

        #[tokio::test]
        async fn resolves_template_files() {
            let git_dummy = GitDetails::default();
            let file_content = "
                variables:
                  FILE: value
            ";
            let mut files = StubFiles::default();
            files.add_remote_file("https://gitlab.com/gitlab-org/gitlab/-/raw/master/lib/gitlab/ci/templates/Rust.gitlab-ci.yml", file_content);

            let content = "
                include:
                  template: Rust.gitlab-ci.yml
            ";

            let configuration = parse_and_merge(content).unwrap();
            let additional_configurations = parse_all(&configuration.include, &files, &git_dummy)
                .await
                .unwrap();

            assert_eq!(additional_configurations.len(), 1);
        }
    }

    mod test_merging_of_configurations {
        use super::*;

        #[test]
        fn adds_variables_from_other_configurations() {
            let other_content = "
                variables:
                  OTHER_VARIABLE: true
            ";
            let content = "";

            let other_configuration = parse_and_merge(other_content).unwrap();
            let mut configuration = parse_and_merge(content).unwrap();

            merge_all(vec![other_configuration], &mut configuration).unwrap();

            assert_eq!(configuration.variables.len(), 1);
        }
    }
}
