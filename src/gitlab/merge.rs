use crate::gitlab::configuration::{GitLabConfiguration, Job, ListOfStrings};
use crate::gitlab::error::GitLabError;
use std::collections::HashMap;

pub fn merge_variables(source: &[(String, String)], target: &mut Vec<(String, String)>) {
    target.splice(0..0, source.to_owned());
}

pub fn merge_image(source: &Option<String>, target: &mut Option<String>) {
    if let (Some(s), t @ None) = (source, target) {
        let _ = t.insert(s.to_owned());
    };
}

pub fn merge_script(source: &Option<ListOfStrings>, target: &mut Option<ListOfStrings>) {
    if let (Some(s), t @ None) = (source, target) {
        let _ = t.insert(ListOfStrings(s.0.clone()));
    };
}

pub fn collect_template_names(
    job: &Job,
    all_templates: &HashMap<String, Job>,
) -> Result<Vec<String>, GitLabError> {
    let mut collected_names = vec![];

    if let Some(ListOfStrings(template_names)) = &job.extends {
        for template_name in template_names {
            let template = all_templates
                .get(template_name)
                .ok_or_else(|| GitLabError::TemplateNotFound(template_name.to_owned()))?;

            if template.extends.is_some() {
                collected_names.extend(collect_template_names(template, all_templates)?);
            }

            collected_names.push(template_name.to_owned());
        }
    }

    Ok(collected_names)
}

pub fn merge_configuration(source: GitLabConfiguration, target: &mut GitLabConfiguration) {
    target.variables.splice(0..0, source.variables.to_owned());
    target.templates.extend(source.templates);
    target.jobs.extend(source.jobs);
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

    mod test_scripts {
        use super::*;

        #[test]
        fn does_not_do_anything_if_no_values_are_given() {
            let source = None;
            let mut target = None;

            merge_script(&source, &mut target);

            assert_eq!(target, None);
        }

        #[test]
        fn does_not_overwrite_anything_if_target_has_a_value_already() {
            let source = Some(ListOfStrings(vec!["other value".into()]));
            let mut target = Some(ListOfStrings(vec!["value".into()]));

            merge_script(&source, &mut target);

            assert_eq!(target, Some(ListOfStrings(vec!["value".into()])));
        }

        #[test]
        fn overwrites_target_with_source_when_not_set_yet() {
            let source = Some(ListOfStrings(vec!["value".into()]));
            let mut target = None;

            merge_script(&source, &mut target);

            assert_eq!(target, Some(ListOfStrings(vec!["value".into()])));
        }
    }

    mod test_collecting_templates {
        use super::*;
        use crate::gitlab::configuration::Job;
        use std::collections::HashMap;

        #[test]
        fn fails_when_template_does_not_exist() {
            let empty_templates = HashMap::new();
            let job_with_templates = Job {
                extends: Some(ListOfStrings(vec![".template-name".into()])),
                ..Default::default()
            };

            let result = collect_template_names(&job_with_templates, &empty_templates);

            assert!(result.is_err());
        }

        #[test]
        fn collects_template_names_that_are_used_by_job() {
            let templates = HashMap::from([
                (".template-a".into(), Job::default()),
                (".template-b".into(), Job::default()),
            ]);
            let job_with_templates = Job {
                extends: Some(ListOfStrings(vec![".template-b".into()])),
                ..Default::default()
            };

            let names = collect_template_names(&job_with_templates, &templates).unwrap();

            assert_eq!(names, vec![".template-b".to_string()])
        }

        #[test]
        fn looks_further_into_templates_to_collect_all_their_templates_ordered_by_hierarchy() {
            let template_with_additional_extend = Job {
                extends: Some(ListOfStrings(vec![".parent".into()])),
                ..Default::default()
            };

            let templates = HashMap::from([
                (".template-a".into(), Job::default()),
                (".template-b".into(), template_with_additional_extend),
                (".parent".into(), Job::default()),
            ]);
            let job_with_templates = Job {
                extends: Some(ListOfStrings(vec![".template-b".into()])),
                ..Default::default()
            };

            let names = collect_template_names(&job_with_templates, &templates).unwrap();

            assert_eq!(names, vec![".parent".into(), ".template-b".to_string()])
        }
    }

    mod test_merging_of_configurations {
        use super::*;

        #[test]
        fn merges_variables() {
            let mut source = GitLabConfiguration::default();
            source.variables.push(("VARIABLE_A".into(), "1".into()));

            let mut target = GitLabConfiguration::default();
            source.variables.push(("VARIABLE_B".into(), "2".into()));

            merge_configuration(source, &mut target);

            assert_eq!(
                target.variables,
                vec![
                    ("VARIABLE_A".into(), "1".into()),
                    ("VARIABLE_B".into(), "2".into())
                ]
            );
        }

        #[test]
        fn merges_templates() {
            let mut source = GitLabConfiguration::default();
            source
                .templates
                .insert(".template-a".into(), Job::default());

            let mut target = GitLabConfiguration::default();
            source
                .templates
                .insert(".template-b".into(), Job::default());

            merge_configuration(source, &mut target);

            assert_eq!(target.templates.len(), 2);
        }

        #[test]
        fn merges_jobs() {
            let mut source = GitLabConfiguration::default();
            source.jobs.insert("job-a".into(), Job::default());

            let mut target = GitLabConfiguration::default();
            source.jobs.insert("job-b".into(), Job::default());

            merge_configuration(source, &mut target);

            assert_eq!(target.jobs.len(), 2);
        }
    }
}
