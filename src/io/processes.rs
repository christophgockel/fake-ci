use crate::core::Job;
use crate::io::prompt::Prompts;
#[cfg(not(test))]
use duct::cmd;
#[cfg(not(test))]
use std::io::Error;

pub trait ProcessesToExecute {
    fn docker_prune(&mut self) -> Result<(), std::io::Error>;
    fn run_job<P: Prompts>(
        &mut self,
        prompt: &P,
        context: &Context,
        job: &Job,
    ) -> Result<(), std::io::Error>;
    fn extract_artifacts<P: Prompts>(
        &mut self,
        prompt: &P,
        job: &Job,
    ) -> Result<(), std::io::Error>;
}

#[cfg(not(test))]
#[derive(Default)]
pub struct Processes;
use crate::Context;
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

    fn run_job<P: Prompts>(
        &mut self,
        _prompt: &P,
        context: &Context,
        job: &Job,
    ) -> Result<(), std::io::Error> {
        let checkout_commands_to_run = format!(
            "
              cd /checkout;
              git init;
              git remote add origin /project;
              git fetch origin --quiet;
              git checkout --quiet {};
              (cd /project; git add --intent-to-add .; git diff) | git apply --allow-empty --quiet;
              (cd /project; git reset --mixed)
            ",
            context.git_sha,
        );

        println!("cleaning up first...");

        cmd!(
            "docker",
            "ps",
            "--all",
            "--quiet",
            "--filter",
            "name=fake-ci-checkout"
        )
        .pipe(cmd!("xargs", "docker", "rm", "--force"))
        .read()?;

        println!("run checkout container");

        let run = cmd!(
            "docker",
            "run",
            "--tty",
            "--detach",
            "--volume",
            format!("{}:/project", context.current_directory),
            "--volume",
            "/checkout",
            "--volume",
            "fake-ci-artifacts:/artifacts",
            "--volume",
            "/job",
            "--name",
            "fake-ci-checkout",
            "fake-ci:latest"
        )
        .read()?;

        println!("  {}", run);

        println!("exec in container...");

        let exec = cmd!(
            "docker",
            "exec",
            "fake-ci-checkout",
            "sh",
            "-c",
            checkout_commands_to_run
        )
        .read()?;
        println!(">  {}", exec);

        let prepare_commands_to_run = "
          cp -Rp /checkout/. /job;
          chmod 0777 /job;
          chmod 0777 /artifacts;
        ";

        println!("preparing code");

        let run_prepare = cmd!(
            "docker",
            "exec",
            "fake-ci-checkout",
            "sh",
            "-c",
            prepare_commands_to_run,
        )
        .read()?;

        println!(">  {}", run_prepare);

        println!("preparing artifacts");
        let mut artifact_commands = vec![];

        for (dependant_job_name, files) in &job.required_artifacts {
            //
            for file in files {
                artifact_commands.push(format!(
                    "cp -Rp \"/artifacts/{}/{}\" /job;",
                    dependant_job_name, file
                ));
            }
        }

        if !artifact_commands.is_empty() {
            let run_artifacts = cmd!(
                "docker",
                "exec",
                "fake-ci-checkout",
                "sh",
                "-c",
                prepare_commands_to_run,
            )
            .read()?;
            println!(">  {}", run_artifacts);
        } else {
            println!("no artifacts");
        }

        Ok(())
    }

    fn extract_artifacts<P: Prompts>(
        &mut self,
        _prompt: &P,
        _job: &Job,
    ) -> Result<(), std::io::Error> {
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Default)]
    pub struct ProcessesSpy {
        pub docker_prune_call_count: usize,
        pub run_job_call_count: usize,
        pub extract_artifacts_call_count: usize,
    }

    impl ProcessesToExecute for ProcessesSpy {
        fn docker_prune(&mut self) -> Result<(), std::io::Error> {
            self.docker_prune_call_count += 1;

            Ok(())
        }

        fn run_job<P: Prompts>(
            &mut self,
            _prompt: &P,
            _context: &Context,
            _job: &Job,
        ) -> Result<(), std::io::Error> {
            self.run_job_call_count += 1;

            Ok(())
        }

        fn extract_artifacts<P: Prompts>(
            &mut self,
            _prompt: &P,
            _job: &Job,
        ) -> Result<(), std::io::Error> {
            self.extract_artifacts_call_count += 1;

            Ok(())
        }
    }
}
