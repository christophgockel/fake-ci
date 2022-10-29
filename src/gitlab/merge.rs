pub fn merge_variables(source: &[(String, String)], target: &mut Vec<(String, String)>) {
    target.splice(0..0, source.to_owned());
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_variables {
        use super::*;

        #[test]
        fn prepends_variables_of_target() {
            let source = vec![("VARIABLE_A".into(), "1".into())];
            let mut target = vec![("VARIABLE_B".into(), "2".into())];

            merge_variables(&source, &mut target);

            assert_eq!(
                target,
                vec![
                    ("VARIABLE_A".into(), "1".into()),
                    ("VARIABLE_B".into(), "2".into())
                ]
            );
        }
    }
}
