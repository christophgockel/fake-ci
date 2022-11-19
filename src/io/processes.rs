use crate::core::Job;
#[cfg(not(test))]
use crate::io::docker::{build_image, image_needs_to_be_built};
use crate::io::prompt::Prompts;
#[cfg(not(test))]
use crate::io::variables::{concatenate_variables, interpolate};
use crate::Context;
#[cfg(not(test))]
use duct::cmd;
#[cfg(not(test))]
use std::io::Error;

pub trait ProcessesToExecute {
    fn docker_prune<P: Prompts>(&mut self, prompts: &mut P) -> Result<(), std::io::Error>;
    fn run_job<P: Prompts>(
        &mut self,
        prompts: &mut P,
        context: &Context,
        job_name: &str,
        job: &Job,
    ) -> Result<(), std::io::Error>;
    fn extract_artifacts<P: Prompts>(
        &mut self,
        prompts: &P,
        job: &Job,
    ) -> Result<(), std::io::Error>;
}

#[cfg(not(test))]
#[derive(Default)]
pub struct Processes;
#[cfg(test)]
pub use tests::ProcessesSpy as Processes;

#[cfg(not(test))]
impl ProcessesToExecute for Processes {
    fn docker_prune<P: Prompts>(&mut self, prompts: &mut P) -> Result<(), Error> {
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

        prompts.info(&format!("Pruned {} containers", container_lines));

        let volume_output = cmd!("docker", "volume", "ls", "--filter", "name=fake", "--quiet")
            .pipe(cmd!("xargs", "docker", "volume", "rm", "-f"))
            .read()?;

        let volume_lines = volume_output.split('\n').filter(|s| !s.is_empty()).count();

        prompts.info(&format!("Pruned {} volumes", volume_lines));

        let image_output = cmd!(
            "docker",
            "image",
            "ls",
            "--filter",
            "reference=fake-ci",
            "--quiet"
        )
        .pipe(cmd!("xargs", "docker", "image", "rm", "-f"))
        .read()?;

        let image_lines = image_output.split('\n').filter(|s| !s.is_empty()).count();

        prompts.info(&format!("Pruned {} images", image_lines));

        Ok(())
    }

    fn run_job<P: Prompts>(
        &mut self,
        prompts: &mut P,
        context: &Context,
        job_name: &str,
        job: &Job,
    ) -> Result<(), std::io::Error> {
        if image_needs_to_be_built(&context.image_tag)? {
            prompts.info("Building Fake CI image first");
            build_image(&context.image_tag)?;
        }

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

        cmd!(
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
            &context.image_tag
        )
        .read()?;

        prompts.info("Checking out code");

        cmd!(
            "docker",
            "exec",
            "fake-ci-checkout",
            "sh",
            "-c",
            checkout_commands_to_run
        )
        .run()?;

        let prepare_commands_to_run = "
          cp -Rp /checkout/. /job;
          chmod 0777 /job;
          chmod 0777 /artifacts;
        ";

        cmd!(
            "docker",
            "exec",
            "fake-ci-checkout",
            "sh",
            "-c",
            prepare_commands_to_run,
        )
        .read()?;

        let mut artifact_commands = vec![];

        for (dependant_job_name, files) in &job.required_artifacts {
            for file in files {
                artifact_commands.push(format!(
                    "cp -Rp \"/artifacts/{}/{}\" /job;",
                    dependant_job_name, file
                ));
            }
        }

        if !artifact_commands.is_empty() {
            prompts.info("Preparing artifacts");

            cmd!(
                "docker",
                "exec",
                "fake-ci-checkout",
                "sh",
                "-c",
                artifact_commands.join(";"),
            )
            .read()?;
        } else {
            prompts.info("No artifacts to prepare");
        }

        let interpolated_image_name = interpolate(&job.image, &job.variables)?;

        cmd!("docker", "ps", "-aq", "--filter", "name=fake-ci-job")
            .pipe(cmd!("xargs", "docker", "rm", "-f"))
            .read()?;

        prompts.info("Running job");

        cmd!(
            "docker",
            "run",
            "--tty",
            "--detach",
            "--volumes-from",
            "fake-ci-checkout",
            "--name",
            "fake-ci-job",
            interpolated_image_name
        )
        .read()?;

        let variables = concatenate_variables(&job.variables);

        let full_script = format!("set -x\ncd /job; {} {}", variables, job.script.join(";"));

        cmd!("docker", "exec", "fake-ci-job", "sh", "-c", full_script).read()?;

        if !job.artifacts.is_empty() {
            prompts.info("Extracting artifacts");

            let mut artifact_commands = vec![format!("mkdir -p \"/artifacts/{}\"", job_name)];

            for artifact in &job.artifacts {
                artifact_commands.push(format!(
                    "cp -R /job/{} \"/artifacts/{}/\"",
                    artifact, job_name
                ));
            }

            cmd!(
                "docker",
                "exec",
                "fake-ci-job",
                "sh",
                "-c",
                artifact_commands.join(";")
            )
            .read()?;
        } else {
            prompts.info("No artifacts to be extracted");
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
        fn docker_prune<P: Prompts>(&mut self, _prompts: &mut P) -> Result<(), std::io::Error> {
            self.docker_prune_call_count += 1;

            Ok(())
        }

        fn run_job<P: Prompts>(
            &mut self,
            _prompts: &mut P,
            _context: &Context,
            _job_name: &str,
            _job: &Job,
        ) -> Result<(), std::io::Error> {
            self.run_job_call_count += 1;

            Ok(())
        }

        fn extract_artifacts<P: Prompts>(
            &mut self,
            _prompts: &P,
            _job: &Job,
        ) -> Result<(), std::io::Error> {
            self.extract_artifacts_call_count += 1;

            Ok(())
        }
    }
}
