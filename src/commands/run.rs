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

pub fn command<PROMPT: Prompts, PROCESSES: ProcessesToExecute>(
    prompt: &mut PROMPT,
    processes: &mut PROCESSES,
    context: &Context,
    definition: &CiDefinition,
    job_name: String,
) -> Result<(), CommandError> {
    if let Some(job) = definition.jobs.get(&job_name) {
        processes
            .run_job(prompt, context, &job_name, job)
            .map_err(CommandError::execution)?;
        processes
            .extract_artifacts(prompt, job)
            .map_err(CommandError::execution)?;

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
    use crate::io::prompt::tests::FakePrompt;
    use std::collections::HashMap;

    #[test]
    fn returns_error_if_job_name_is_unknown() {
        let mut prompt = FakePrompt::always_confirming();
        let mut processes = ProcessesSpy::default();
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
    fn valid_jobs_are_passed_to_be_executed() {
        let mut prompt = FakePrompt::always_confirming();
        let mut processes = ProcessesSpy::default();
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

        assert_eq!(processes.run_job_call_count, 1);
    }

    #[test]
    fn extracts_artifacts() {
        let mut prompt = FakePrompt::always_confirming();
        let mut processes = ProcessesSpy::default();
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

        assert_eq!(processes.extract_artifacts_call_count, 1);
    }
}
