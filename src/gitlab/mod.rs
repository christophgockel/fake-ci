mod configuration;
mod deserialise;

use crate::gitlab::configuration::GitLabConfiguration;

pub fn parse<R>(reader: R) -> Result<GitLabConfiguration, Box<dyn std::error::Error>>
where
    R: std::io::Read,
{
    let configuration: GitLabConfiguration = serde_yaml::from_reader(reader)?;

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
}
