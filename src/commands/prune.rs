use crate::commands::CommandError;
use crate::io::processes::ProcessesToExecute;
use crate::io::prompt::{PromptResponse, Prompts};
use clap::Args;

#[derive(Args)]
pub struct Prune;

pub fn command<PROMPTS: Prompts, PROCESSES: ProcessesToExecute>(
    prompts: &mut PROMPTS,
    processes: &mut PROCESSES,
) -> Result<(), CommandError> {
    if let PromptResponse::Yes = prompts.question("Do you really want to prune all artifacts?") {
        processes
            .docker_prune(prompts)
            .map_err(CommandError::unknown)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::processes::tests::ProcessesSpy;
    use crate::io::prompt::tests::FakePrompt;

    #[test]
    fn asks_for_confirmation() {
        let mut prompt = FakePrompt::always_confirming();
        let mut processes = ProcessesSpy::default();

        command(&mut prompt, &mut processes).unwrap();

        assert!(prompt.has_been_asked_to_confirm);
    }

    #[test]
    fn prunes_when_confirmed() {
        let mut prompt = FakePrompt::always_confirming();
        let mut processes = ProcessesSpy::default();

        command(&mut prompt, &mut processes).unwrap();

        assert_eq!(processes.docker_prune_call_count, 1);
    }

    #[test]
    fn does_not_prune_when_not_confirmed() {
        let mut prompt = FakePrompt::always_denying();
        let mut processes = ProcessesSpy::default();

        command(&mut prompt, &mut processes).unwrap();

        assert_eq!(processes.docker_prune_call_count, 0);
    }
}
