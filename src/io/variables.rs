use crate::error::FakeCiError;
use duct::cmd;

pub fn interpolate(value: &str, variables: &Vec<(String, String)>) -> Result<String, FakeCiError> {
    let variables = concatenate_variables(variables);
    let command = format!("{} echo \"{}\"", variables, value);

    cmd!("sh", "-c", command).read().map_err(FakeCiError::other)
}

fn concatenate_variables(variables: &Vec<(String, String)>) -> String {
    let mut lines = vec![];

    for (name, value) in variables {
        lines.push(format!("export {}=\"{}\";", name, value));
    }

    lines.join("")
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn returns_same_value_when_nothing_is_interpolated() {
        assert_eq!("the-value", interpolate("the-value", &vec![]).unwrap());
    }

    #[test]
    fn interpolates_values() {
        assert_eq!(
            "some-interpolated-value",
            interpolate(
                "some-${VARIABLE}-value",
                &vec![("VARIABLE".to_string(), "interpolated".to_string())]
            )
            .unwrap()
        );
    }
}
