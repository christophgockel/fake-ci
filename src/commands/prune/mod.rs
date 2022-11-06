use crate::commands::CommandError;
use crate::io::prompt::{PromptResponse, Prompts};
use clap::Args;
#[cfg(not(test))]
use duct::cmd;
#[cfg(not(test))]
use std::io::Error;

pub trait ProcessesToExecute {
    fn docker_prune(&mut self) -> Result<(), std::io::Error>;
}

#[cfg(not(test))]
#[derive(Default)]
pub struct Processes;
#[cfg(test)]
pub use crate::commands::prune::tests::ProcessesSpy as Processes;

#[cfg(not(test))]
impl ProcessesToExecute for Processes {
    fn docker_prune(&mut self) -> Result<(), Error> {
        let container_output = cmd!(
            "docker",
            "container",
            "ls",
            "--filter",
            "name=fake-ci",
            "--quiet"
        )
        .pipe(cmd!("xargs", "docker", "container", "rm", "-f"))
        .read()?;

        let container_lines = container_output
            .split('\n')
            .filter(|s| !s.is_empty())
            .count();

        println!("Pruned {} containers", container_lines);

        let volume_output = cmd!("docker", "volume", "ls", "--filter", "name=fake", "--quiet")
            .pipe(cmd!("xargs", "docker", "volume", "rm", "-f"))
            .read()?;

        let volume_lines = volume_output.split('\n').filter(|s| !s.is_empty()).count();

        println!("Pruned {} volumes", volume_lines);

        let image_output = cmd!(
            "docker",
            "image",
            "ls",
            "--filter",
            "reference=fake-ci:latest",
            "--quiet"
        )
        .pipe(cmd!("xargs", "docker", "image", "rm", "-f"))
        .read()?;

        let image_lines = image_output.split('\n').filter(|s| !s.is_empty()).count();

        println!("Pruned {} images", image_lines);

        Ok(())
    }
}

#[derive(Args)]
pub struct Prune;

pub fn command<PROMPT: Prompts, PROCESSES: ProcessesToExecute>(
    prompt: &mut PROMPT,
    processes: &mut PROCESSES,
) -> Result<(), CommandError> {
    if let PromptResponse::Yes = prompt.question() {
        processes.docker_prune().map_err(CommandError::unknown)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::prompt::tests::FakePrompt;
    use std::io::Error;

    #[derive(Default)]
    pub struct ProcessesSpy {
        docker_prune_call_count: usize,
    }

    impl ProcessesToExecute for Processes {
        fn docker_prune(&mut self) -> Result<(), Error> {
            self.docker_prune_call_count += 1;

            Ok(())
        }
    }

    #[test]
    fn asks_for_confirmation() {
        let mut prompt = FakePrompt::always_confirming();
        let mut processes = Processes::default();

        command(&mut prompt, &mut processes).unwrap();

        assert!(prompt.has_been_asked_to_confirm);
    }

    #[test]
    fn prunes_when_confirmed() {
        let mut prompt = FakePrompt::always_confirming();
        let mut processes = Processes::default();

        command(&mut prompt, &mut processes).unwrap();

        assert_eq!(processes.docker_prune_call_count, 1);
    }

    #[test]
    fn does_not_prune_when_not_confirmed() {
        let mut prompt = FakePrompt::always_denying();
        let mut processes = Processes::default();

        command(&mut prompt, &mut processes).unwrap();

        assert_eq!(processes.docker_prune_call_count, 0);
    }
}
