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
pub use tests::ProcessesSpy as Processes;

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

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Default)]
    pub struct ProcessesSpy {
        pub docker_prune_call_count: usize,
    }

    impl ProcessesToExecute for Processes {
        fn docker_prune(&mut self) -> Result<(), std::io::Error> {
            self.docker_prune_call_count += 1;

            Ok(())
        }
    }
}
