mod configuration;
mod deserialise;
mod merge;

use crate::gitlab::configuration::GitLabConfiguration;
use crate::gitlab::merge::merge_variables;

pub fn parse<R>(reader: R) -> Result<GitLabConfiguration, Box<dyn std::error::Error>>
where
    R: std::io::Read,
{
    let mut configuration: GitLabConfiguration = serde_yaml::from_reader(reader)?;

    for (_name, job) in configuration.jobs.iter_mut() {
        merge_variables(&configuration.variables, &mut job.variables);
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

    mod test_merge_precedences {
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
}
