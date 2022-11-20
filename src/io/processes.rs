use crate::core::Job;
#[cfg(not(test))]
use crate::io::docker;
use crate::io::prompt::Prompts;
#[cfg(not(test))]
use crate::io::variables::{concatenate_variables, interpolate};
use crate::Context;
#[cfg(not(test))]
use duct::cmd;
#[cfg(not(test))]
use std::io::Error;

pub trait ProcessesToExecute {
    fn image_needs_to_be_built(&mut self, tag: &str) -> Result<bool, std::io::Error>;
    fn build_image(&mut self, tag: &str) -> Result<(), std::io::Error>;

    fn prune_containers(&mut self) -> Result<usize, std::io::Error>;
    fn prune_volumes(&mut self) -> Result<usize, std::io::Error>;
    fn prune_images(&mut self) -> Result<usize, std::io::Error>;

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
    fn image_needs_to_be_built(&mut self, tag: &str) -> Result<bool, Error> {
        docker::image_needs_to_be_built(tag)
    }

    fn build_image(&mut self, tag: &str) -> Result<(), Error> {
        docker::build_image(tag)
    }

    fn prune_containers(&mut self) -> Result<usize, Error> {
        docker::prune_containers()
    }

    fn prune_volumes(&mut self) -> Result<usize, Error> {
        docker::prune_volumes()
    }

    fn prune_images(&mut self) -> Result<usize, Error> {
        docker::prune_images()
    }

    fn run_job<P: Prompts>(
        &mut self,
        prompts: &mut P,
        context: &Context,
        job_name: &str,
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
        pub image_needs_to_be_built: bool,
        pub build_image_call_count: usize,
        pub prune_containers_call_count: usize,
        pub prune_volumes_call_count: usize,
        pub prune_images_call_count: usize,
        pub run_job_call_count: usize,
        pub extract_artifacts_call_count: usize,
    }

    impl ProcessesSpy {
        pub fn with_image_to_be_built() -> Self {
            Self {
                image_needs_to_be_built: true,
                ..Default::default()
            }
        }
    }

    impl ProcessesToExecute for ProcessesSpy {
        fn image_needs_to_be_built(&mut self, _tag: &str) -> Result<bool, std::io::Error> {
            Ok(self.image_needs_to_be_built)
        }

        fn build_image(&mut self, _tag: &str) -> Result<(), std::io::Error> {
            self.build_image_call_count += 1;

            Ok(())
        }

        fn prune_containers(&mut self) -> Result<usize, std::io::Error> {
            self.prune_containers_call_count += 1;

            Ok(1)
        }

        fn prune_volumes(&mut self) -> Result<usize, std::io::Error> {
            self.prune_volumes_call_count += 1;

            Ok(1)
        }

        fn prune_images(&mut self) -> Result<usize, std::io::Error> {
            self.prune_images_call_count += 1;

            Ok(1)
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
