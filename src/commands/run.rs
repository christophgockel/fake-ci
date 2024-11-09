use crate::commands::CommandError;
use crate::core::CiDefinition;
use crate::io::processes::ProcessesToExecute;
use crate::io::prompt::Prompts;
use crate::Context;
use clap::Args;

#[derive(Args)]
pub struct Run {
    /// The job name.
    pub job: String,
}

pub fn command<PROMPTS: Prompts, PROCESSES: ProcessesToExecute>(
    prompt: &mut PROMPTS,
    processes: &mut PROCESSES,
    context: &Context,
    definition: &CiDefinition,
    job_name: String,
) -> Result<(), CommandError> {
    if let Some(job) = definition.jobs.get(&job_name) {
        if processes.image_needs_to_be_built(&context.image_tag)? {
            prompt.info("Building Fake CI image first");
            processes.build_image(&context.image_tag)?;
        }
        prompt.info("Checking out code");

        processes.prune_checkout_container()?;
        let checkout_container_id = processes.start_checkout_container(context)?;

        processes.checkout_code(&checkout_container_id, context)?;

        if !job.required_artifacts.is_empty() {
            prompt.info("Preparing artifacts");

            processes.prepare_artifacts(&checkout_container_id, &job.required_artifacts)?;
        } else {
            prompt.info("No artifacts to prepare");
        }

        prompt.info("Running job");

        processes.prune_job_container()?;
        let job_container_id = processes.start_job_container(job, &checkout_container_id)?;
        processes.run_job(&job_container_id, job)?;

        if !job.artifacts.is_empty() {
            prompt.info("Extracting artifacts");
            processes.extract_artifacts(&job_container_id, &job_name, job)?;
        } else {
            prompt.info("No artifacts to be extracted");
        }

        Ok(())
    } else {
        Err(CommandError::UnknownJob(job_name.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CiDefinition, Job};
    use crate::io::processes::tests::ProcessesSpy;
    use crate::io::prompt::tests::{FakePrompt, SpyPrompt};
    use std::collections::HashMap;

    #[test]
    fn returns_error_if_job_name_is_unknown() {
        let mut prompt = FakePrompt::always_confirming();
        let mut processes = ProcessesSpy::new();
        let context = Context::default();
        let definition = CiDefinition::default();

        let result = command(
            &mut prompt,
            &mut processes,
            &context,
            &definition,
            "unknown job".into(),
        );

        assert!(matches!(result.err().unwrap(), CommandError::UnknownJob(_)));
    }

    #[test]
    fn checks_if_image_is_available_and_builds_when_it_is_not() {
        let mut prompt = SpyPrompt::new();
        let mut processes = ProcessesSpy::with_image_to_be_built();
        let context = Context::default();
        let job = Job::default();
        let definition = CiDefinition {
            jobs: HashMap::from([("job".into(), job)]),
        };

        command(
            &mut prompt,
            &mut processes,
            &context,
            &definition,
            "job".into(),
        )
        .unwrap();

        assert_eq!(processes.build_image_call_count, 1);
        assert!(prompt.info_call_count > 0);
    }

    #[test]
    fn valid_jobs_are_passed_to_be_executed() {
        let mut prompt = FakePrompt::always_confirming();
        let mut processes = ProcessesSpy::new();
        let context = Context::default();
        let job = Job::default();
        let definition = CiDefinition {
            jobs: HashMap::from([("job".into(), job)]),
        };

        command(
            &mut prompt,
            &mut processes,
            &context,
            &definition,
            "job".into(),
        )
        .unwrap();

        assert_eq!(processes.prune_checkout_container_call_count, 1);
        assert_eq!(processes.start_checkout_container_call_count, 1);
        assert_eq!(processes.checkout_code_call_count, 1);
        assert_eq!(processes.prepare_artifacts_call_count, 0);
        assert_eq!(processes.prune_job_container_call_count, 1);
        assert_eq!(processes.run_job_call_count, 1);
        assert_eq!(processes.extract_artifacts_call_count, 0);
    }

    #[test]
    fn prepares_artifacts_when_job_requires_them() {
        let mut prompt = FakePrompt::always_confirming();
        let mut processes = ProcessesSpy::new();
        let context = Context::default();
        let job = Job {
            required_artifacts: HashMap::from([("other-job".into(), vec!["file-1".into()])]),
            ..Default::default()
        };
        let definition = CiDefinition {
            jobs: HashMap::from([("job".into(), job)]),
        };

        command(
            &mut prompt,
            &mut processes,
            &context,
            &definition,
            "job".into(),
        )
        .unwrap();

        assert_eq!(processes.prepare_artifacts_call_count, 1);
    }

    #[test]
    fn extracts_artifacts_when_job_defines_some() {
        let mut prompt = FakePrompt::always_confirming();
        let mut processes = ProcessesSpy::new();
        let context = Context::default();
        let job = Job {
            artifacts: vec!["file-1".into()],
            ..Default::default()
        };
        let definition = CiDefinition {
            jobs: HashMap::from([("job".into(), job)]),
        };

        command(
            &mut prompt,
            &mut processes,
            &context,
            &definition,
            "job".into(),
        )
        .unwrap();

        assert_eq!(processes.extract_artifacts_call_count, 1);
    }
}
