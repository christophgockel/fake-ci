pub fn merge_variables(source: &[(String, String)], target: &mut Vec<(String, String)>) {
    target.splice(0..0, source.to_owned());
}

pub fn merge_image(source: &Option<String>, target: &mut Option<String>) {
    if let (Some(s), t @ None) = (source, target) {
        let _ = t.insert(s.to_owned());
    };
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

    mod test_image_names {
        use super::*;

        #[test]
        fn does_not_do_anything_if_no_image_given() {
            let source = None;
            let mut target = None;

            merge_image(&source, &mut target);

            assert_eq!(target, None);
        }

        #[test]
        fn does_not_overwrite_anything_if_target_has_a_value_already() {
            let source = Some("other value".into());
            let mut target = Some("value".into());

            merge_image(&source, &mut target);

            assert_eq!(target, Some("value".into()));
        }

        #[test]
        fn overwrites_target_with_source_when_not_set_yet() {
            let source = Some("value".into());
            let mut target = None;

            merge_image(&source, &mut target);

            assert_eq!(target, Some("value".into()));
        }
    }
}
