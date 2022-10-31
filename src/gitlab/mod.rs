mod configuration;
mod deserialise;
mod error;
mod merge;

use crate::file::FileAccess;
use crate::git::GitDetails;
use crate::gitlab::configuration::{GitLabConfiguration, Include};
use crate::gitlab::error::GitLabError;
use crate::gitlab::merge::{
    collect_template_names, merge_configuration, merge_image, merge_script, merge_variables,
};
use async_recursion::async_recursion;
use url::Url;

pub fn parse<R>(reader: R) -> Result<GitLabConfiguration, GitLabError>
where
    R: std::io::Read,
{
    let configuration: GitLabConfiguration =
        serde_yaml::from_reader(reader).map_err(GitLabError::parse)?;

    Ok(configuration)
}

pub fn merge_all(
    additional_configurations: Vec<GitLabConfiguration>,
    configuration: &mut GitLabConfiguration,
) -> Result<(), GitLabError> {
    for additional_configuration in additional_configurations {
        merge_configuration(additional_configuration, configuration);
    }

    merge_jobs(configuration)?;

    Ok(())
}

pub fn merge_jobs(configuration: &mut GitLabConfiguration) -> Result<(), GitLabError> {
    for (_name, job) in configuration.jobs.iter_mut() {
        let required_template_names = collect_template_names(job, &configuration.templates)?;

        for template_name in required_template_names {
            let template = configuration
                .templates
                .get(&template_name)
                .ok_or_else(|| GitLabError::TemplateNotFound(template_name.to_owned()))?;

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

#[derive(Clone)]
pub enum ResolvePath {
    Local,
    Remote(Url),
}

pub async fn parse_all(
    includes: &Vec<Include>,
    file_access: &impl FileAccess,
    git_details: &GitDetails,
) -> Result<Vec<GitLabConfiguration>, GitLabError> {
    // The distinction between "local" path resolving and "remote" is that on the initial read
    // through a .gitlab-ci.yml all `include:local` (https://docs.gitlab.com/ee/ci/yaml/#includelocal)
    // includes are to be read from the local file system.
    // Every additional pass from the included configurations is to be resolved as a remote path.
    parse_all_with_base(includes, file_access, git_details, &ResolvePath::Local).await
}

#[async_recursion(?Send)]
pub async fn parse_all_with_base(
    includes: &Vec<Include>,
    file_access: &impl FileAccess,
    git_details: &GitDetails,
    resolve_path: &ResolvePath,
) -> Result<Vec<GitLabConfiguration>, GitLabError> {
    let mut included_configurations = vec![];

    for include in includes {
        included_configurations
            .extend(read_and_parse(include, file_access, git_details, resolve_path).await?);
    }

    Ok(included_configurations)
}

async fn read_and_parse(
    include: &Include,
    file_access: &impl FileAccess,
    git_details: &GitDetails,
    resolve_path: &ResolvePath,
) -> Result<Vec<GitLabConfiguration>, GitLabError> {
    let mut paths_and_configurations = vec![];

    match include {
        Include::Local(local_include) => {
            let content = match resolve_path {
                ResolvePath::Local => file_access
                    .read_local_file(&local_include.local)
                    .map_err(GitLabError::file)?,
                ResolvePath::Remote(base_url) => {
                    let url1 = append(base_url, &local_include.local)?;
                    file_access
                        .read_remote_file(url1)
                        .await
                        .map_err(GitLabError::file)?
                }
            };

            let configuration = parse(*content)?;

            paths_and_configurations.push((resolve_path.clone(), configuration));
        }
        Include::File(file_include) => {
            for file in &file_include.file {
                let url = format!(
                    "{}/{}/-/raw/{}/{}",
                    &git_details.host, file_include.project, file_include.r#ref, file
                );
                let content = file_access
                    .read_remote_file(&url)
                    .await
                    .map_err(GitLabError::file)?;
                let configuration = parse(*content)?;

                paths_and_configurations
                    .push((ResolvePath::Remote(base_url(&url)?), configuration));
            }
        }
        Include::Remote(remote_include) => {
            let content = file_access
                .read_remote_file(&remote_include.remote)
                .await
                .map_err(GitLabError::file)?;
            let configuration = parse(*content)?;

            paths_and_configurations.push((
                ResolvePath::Remote(base_url(&remote_include.remote)?),
                configuration,
            ));
        }
        Include::Template(template_include) => {
            let url = format!(
                "https://gitlab.com/gitlab-org/gitlab/-/raw/master/lib/gitlab/ci/templates/{}",
                &template_include.template
            );
            let content = file_access
                .read_remote_file(&url)
                .await
                .map_err(GitLabError::file)?;
            let configuration = parse(*content)?;

            paths_and_configurations.push((ResolvePath::Remote(base_url(&url)?), configuration));
        }
    }

    let mut configurations = vec![];

    for (new_resolve_path, configuration) in paths_and_configurations {
        let more_configurations = parse_all_with_base(
            &configuration.include,
            file_access,
            git_details,
            &new_resolve_path,
        )
        .await?;

        configurations.extend(more_configurations);
        configurations.push(configuration);
    }

    Ok(configurations)
}

fn base_url(url: &str) -> Result<Url, GitLabError> {
    let mut base_url = Url::parse(url).map_err(GitLabError::create_url)?;

    base_url
        .path_segments_mut()
        .map_err(GitLabError::adjust_url)?
        .pop();

    Ok(base_url)
}

fn append(base_url: &Url, path: &str) -> Result<Url, GitLabError> {
    let mut full_url = base_url.clone();
    full_url
        .path_segments_mut()
        .map_err(GitLabError::adjust_url)?
        .pop_if_empty()
        .push(path);

    Ok(full_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_merge(content: &str) -> Result<GitLabConfiguration, GitLabError> {
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
        async fn resolves_nested_local_files() {
            let git_dummy = GitDetails::default();
            let first_content = "
                include:
                  local: second-file.yml

                variables:
                  OTHER_VARIABLE: true
            ";
            let second_content = "
                variables:
                  SOME_VARIABLE: true
            ";
            let mut files = StubFiles::default();
            files.add_file("first-file.yml", first_content);
            files.add_file("second-file.yml", second_content);
            let content = "
                include:
                  local: first-file.yml
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

        #[tokio::test]
        async fn resolves_nested_remote_files_as_local_to_the_host() {
            let git_details = GitDetails {
                host: "https://example-gitlab.com".into(),
            };
            let first_include = "
                include:
                  local: local-file.yml

                variables:
                  SECOND_VARIABLE: true
            ";
            let second_include = "
                variables:
                  FIRST_VARIABLE: true
            ";
            let mut files = StubFiles::default();
            files.add_remote_file("https://example.com/path/to/file.yml", first_include);
            files.add_remote_file("https://example.com/path/to/local-file.yml", second_include);
            let local_content = "
                include:
                  remote: https://example.com/path/to/file.yml
            ";

            let configuration = parse_and_merge(local_content).unwrap();
            let additional_configurations = parse_all(&configuration.include, &files, &git_details)
                .await
                .unwrap();

            assert_eq!(additional_configurations.len(), 2);
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

    mod test_url_helpers {
        use super::*;
        use url::Url;

        #[test]
        fn base_url_removes_the_file_part() {
            let full_raw_url = "https://example.com/path/to/file.yml";
            let expected_url = Url::parse("https://example.com/path/to").unwrap();

            assert_eq!(base_url(full_raw_url).unwrap(), expected_url);
        }

        #[test]
        fn append_path_appends_to_existing_url() {
            // I'm surprised about the %2F encodings when `push`ing new path elements
            // but I trust that it's doing the right thing even though I don't like
            // how the test reads.
            // See https://docs.rs/url/latest/url/struct.PathSegmentsMut.html#method.extend
            // for details.
            let base_url = Url::parse("https://example.com/some-path/").unwrap();

            assert_eq!(
                append(&base_url, &String::from("got/appended/file.yml")).unwrap(),
                Url::parse("https://example.com/some-path/got%2Fappended%2Ffile.yml").unwrap()
            );
        }
    }
}
