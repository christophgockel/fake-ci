use crate::io::docker::DIRECTORIES;

pub fn predefined_variables() -> Vec<(String, String)> {
    vec![("CI_PROJECT_DIR".into(), DIRECTORIES.job.into())]
}

#[cfg(test)]
pub fn without_predefined_variables(variables: &[(String, String)]) -> Vec<(String, String)> {
    let all_predefined_variables = predefined_variables()
        .iter()
        .map(|(key, _)| key.clone())
        .collect::<Vec<String>>();

    variables
        .iter()
        .filter(|(key, _)| !all_predefined_variables.contains(key))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_predefined_variables_from_list() {
        let variables = vec![
            ("VARIABLE".to_string(), "VALUE".to_string()),
            ("CI_PROJECT_DIR".to_string(), "/dummy-directory".to_string()),
        ];

        let filtered_variables = without_predefined_variables(&variables);

        assert_eq!(
            filtered_variables,
            vec![("VARIABLE".to_string(), "VALUE".to_string()),]
        );
    }
}
