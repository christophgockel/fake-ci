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
        let container_count = processes.prune_containers()?;
        prompts.info(&format!("Pruned {} containers", container_count));

        let image_count = processes.prune_images()?;
        prompts.info(&format!("Pruned {} images", image_count));

        let volume_count = processes.prune_volumes()?;
        prompts.info(&format!("Pruned {} volumes", volume_count));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::processes::tests::ProcessesSpy;
    use crate::io::prompt::tests::{FakePrompt, SpyPrompt};

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

        assert_eq!(processes.prune_containers_call_count, 1);
        assert_eq!(processes.prune_volumes_call_count, 1);
        assert_eq!(processes.prune_images_call_count, 1);
    }

    #[test]
    fn provides_info_message_for_each_prune_step_when_confirmed() {
        let mut prompt = SpyPrompt::default();
        let mut processes = ProcessesSpy::default();

        command(&mut prompt, &mut processes).unwrap();

        assert_eq!(prompt.info_call_count, 3);
    }

    #[test]
    fn does_not_prune_when_not_confirmed() {
        let mut prompt = FakePrompt::always_denying();
        let mut processes = ProcessesSpy::default();

        command(&mut prompt, &mut processes).unwrap();

        assert_eq!(processes.prune_containers_call_count, 0);
        assert_eq!(processes.prune_volumes_call_count, 0);
        assert_eq!(processes.prune_images_call_count, 0);
    }
}
