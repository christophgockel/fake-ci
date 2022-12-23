use crate::git::GitDetails;
use crate::io::docker::DIRECTORIES;
use regex::Regex;

const MAXIMUM_LENGTH_OF_SLUG: usize = 63;

pub fn predefined_variables(git: &GitDetails) -> Vec<(String, String)> {
    vec![
        ("CI_COMMIT_REF_NAME".into(), git.branch_name.clone()),
        ("CI_COMMIT_REF_SLUG".into(), ref_slug(&git.branch_name)),
        ("CI_COMMIT_SHA".into(), git.sha.clone()),
        ("CI_COMMIT_SHORT_SHA".into(), git.short_sha.clone()),
        ("CI_PIPELINE_ID".into(), "1000".into()),
        ("CI_PROJECT_DIR".into(), DIRECTORIES.job.into()),
    ]
}

fn ref_slug(branch_name: &str) -> String {
    // From: https://docs.gitlab.com/ee/ci/variables/predefined_variables.html
    // About `CI_COMMIT_REF_SLUG`:
    //       CI_COMMIT_REF_NAME in lowercase, shortened to 63 bytes, and with everything
    //       except 0-9 and a-z replaced with -. No leading / trailing -.
    //       Use in URLs, host names and domain names.
    let lowercase_branch_name = branch_name.to_lowercase();
    let non_alphanumeric_characters = Regex::new(r"[^a-z\d]").unwrap();

    let result = non_alphanumeric_characters.replace_all(&lowercase_branch_name, "-");
    let result = result.trim_start_matches('-');
    let result = result.trim_end_matches('-');

    let result = match result.char_indices().nth(MAXIMUM_LENGTH_OF_SLUG) {
        None => result,
        Some((idx, _)) => &result[..idx],
    };

    result.into()
}

#[cfg(test)]
pub fn without_predefined_variables(
    variables: &[(String, String)],
    git: &GitDetails,
) -> Vec<(String, String)> {
    let all_predefined_variables = predefined_variables(git)
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

    fn value_of(variable_name: &str, variables: &[(String, String)]) -> Option<String> {
        let maybe_tuple = variables
            .iter()
            .find(|(name, _value)| name == variable_name);

        maybe_tuple.map(|(_key, value)| value.clone())
    }

    #[test]
    fn adds_git_details_to_some_variables() {
        let git = GitDetails {
            branch_name: "branch-name".to_string(),
            sha: "1234567890abcde".to_string(),
            short_sha: "12345678".to_string(),
            ..Default::default()
        };
        let variables = predefined_variables(&git);

        assert_eq!(
            value_of("CI_COMMIT_SHA", &variables),
            Some("1234567890abcde".into())
        );
        assert_eq!(
            value_of("CI_COMMIT_SHORT_SHA", &variables),
            Some("12345678".into())
        );
        assert_eq!(
            value_of("CI_COMMIT_REF_NAME", &variables),
            Some("branch-name".into())
        );
    }

    #[test]
    fn sanitizes_ref_name_for_slug() {
        // Rules as to what gets sanitized and how can be found at:
        // https://docs.gitlab.com/ee/ci/variables/predefined_variables.html
        let git = GitDetails {
            branch_name: "/SOME/Long-Branch-NaMe-with.special.characters/".to_string(),
            ..Default::default()
        };
        let variables = predefined_variables(&git);

        assert_eq!(
            value_of("CI_COMMIT_REF_SLUG", &variables),
            Some("some-long-branch-name-with-special-characters".into())
        );
    }

    #[test]
    fn trims_ref_name_for_slug() {
        let git = GitDetails {
            branch_name:
                "loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong-branch-name"
                    .to_string(),
            ..Default::default()
        };
        let variables = predefined_variables(&git);

        assert_eq!(
            value_of("CI_COMMIT_REF_SLUG", &variables),
            Some("loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong".into())
        );
    }

    #[test]
    fn filters_predefined_variables_from_list() {
        let git = GitDetails::default();
        let variables = vec![
            ("VARIABLE".to_string(), "VALUE".to_string()),
            ("CI_PROJECT_DIR".to_string(), "/dummy-directory".to_string()),
        ];

        let filtered_variables = without_predefined_variables(&variables, &git);

        assert_eq!(
            filtered_variables,
            vec![("VARIABLE".to_string(), "VALUE".to_string()),]
        );
    }
}
