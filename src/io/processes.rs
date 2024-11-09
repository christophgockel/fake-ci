use crate::core::Job;
#[cfg(not(test))]
use crate::io::docker;
#[cfg(not(test))]
use crate::io::docker::DIRECTORIES;
#[cfg(not(test))]
use crate::io::shell::combine_lines;
#[cfg(not(test))]
use crate::io::variables::{concatenate_variables, interpolate};
use crate::Context;
use std::collections::HashMap;
#[cfg(not(test))]
use std::io::Error;

pub trait ProcessesToExecute {
    fn image_needs_to_be_built(&mut self, tag: &str) -> Result<bool, std::io::Error>;
    fn build_image(&mut self, tag: &str) -> Result<(), std::io::Error>;

    fn prune_containers(&mut self) -> Result<usize, std::io::Error>;
    fn prune_volumes(&mut self) -> Result<usize, std::io::Error>;
    fn prune_images(&mut self) -> Result<usize, std::io::Error>;

    fn prune_checkout_container(&mut self) -> Result<(), std::io::Error>;
    fn start_checkout_container(&mut self, context: &Context) -> Result<String, std::io::Error>;
    fn checkout_code(
        &mut self,
        container_id: &str,
        context: &Context,
    ) -> Result<(), std::io::Error>;

    fn prepare_artifacts(
        &mut self,
        container_id: &str,
        artifacts: &HashMap<String, Vec<String>>,
    ) -> Result<(), std::io::Error>;

    fn prune_job_container(&mut self) -> Result<(), std::io::Error>;
    fn start_job_container(
        &mut self,
        job: &Job,
        source_container_id: &str,
    ) -> Result<String, std::io::Error>;
    fn run_job(&mut self, container_id: &str, job: &Job) -> Result<(), std::io::Error>;

    fn extract_artifacts(
        &mut self,
        container_id: &str,
        job_name: &str,
        job: &Job,
    ) -> Result<(), std::io::Error>;
}

#[cfg(not(test))]
pub struct Processes;
#[cfg(test)]
pub use tests::ProcessesSpy as Processes;

#[cfg(not(test))]
impl Processes {
    pub fn new() -> Self {
        Self {}
    }
}

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

    fn prune_checkout_container(&mut self) -> Result<(), Error> {
        docker::prune_container("fake-ci-checkout")
    }

    fn start_checkout_container(&mut self, context: &Context) -> Result<String, Error> {
        docker::start_checkout_container(
            "fake-ci-checkout",
            &context.image_tag,
            &context.current_directory,
        )
    }

    fn checkout_code(
        &mut self,
        container_id: &str,
        context: &Context,
    ) -> Result<(), std::io::Error> {
        let checkout_directory = DIRECTORIES.checkout;
        let project_directory = DIRECTORIES.project;
        let job_directory = DIRECTORIES.job;
        let artifacts_directory = DIRECTORIES.artifacts;
        let git_sha = &context.git_sha;

        let checkout_commands = format!(
            "
              cd {checkout_directory};
              git init;
              git remote add origin {project_directory};
              git fetch origin --quiet;
              git checkout --quiet {git_sha};
              (cd {project_directory}; git add --intent-to-add .; git diff) | git apply --allow-empty --quiet;
              (cd {project_directory}; git reset --mixed)
            "
        );
        docker::execute_commands(container_id, &checkout_commands)?;

        let other_preparation_commands = format!(
            "
              cp -Rp {checkout_directory}/. {job_directory};
              chmod 0777 {job_directory};
              chmod 0777 {artifacts_directory};
            "
        );
        docker::execute_commands(container_id, &other_preparation_commands)?;

        Ok(())
    }

    fn prepare_artifacts(
        &mut self,
        container_id: &str,
        artifacts: &HashMap<String, Vec<String>>,
    ) -> Result<(), std::io::Error> {
        let mut artifact_commands = vec![];
        let job_directory = DIRECTORIES.job;
        let artifacts_directory = DIRECTORIES.artifacts;

        for (job_name, files) in artifacts {
            for file in files {
                artifact_commands.push(format!(
                    "cp -Rp \"{artifacts_directory}/{job_name}/{file}\" {job_directory};"
                ));
            }
        }

        docker::execute_commands(container_id, &artifact_commands.join(";"))?;

        Ok(())
    }

    fn prune_job_container(&mut self) -> Result<(), std::io::Error> {
        docker::prune_container("fake-ci-job")
    }

    fn start_job_container(
        &mut self,
        job: &Job,
        source_container_id: &str,
    ) -> Result<String, std::io::Error> {
        let interpolated_image_name = interpolate(&job.image, &job.variables)?;

        docker::start_job_container("fake-ci-job", &interpolated_image_name, source_container_id)
    }

    fn run_job(&mut self, container_id: &str, job: &Job) -> Result<(), std::io::Error> {
        let variables = concatenate_variables(&job.variables);
        let script_commands = combine_lines(&job.script);
        let job_directory = DIRECTORIES.job;
        let full_script = format!("cd {job_directory}; {variables} {script_commands}");

        docker::execute_commands(container_id, &full_script)
    }

    fn extract_artifacts(
        &mut self,
        job_container_id: &str,
        job_name: &str,
        job: &Job,
    ) -> Result<(), std::io::Error> {
        let job_directory = DIRECTORIES.job;
        let artifacts_directory = DIRECTORIES.artifacts;
        let mut artifact_commands = vec![format!("mkdir -p \"{artifacts_directory}/{job_name}\"")];

        for artifact in &job.artifacts {
            artifact_commands.push(format!(
                "cp -R {job_directory}/{artifact} \"{artifacts_directory}/{job_name}/\""
            ));
        }

        docker::execute_commands(job_container_id, &artifact_commands.join(";"))
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
        pub prune_checkout_container_call_count: usize,
        pub start_checkout_container_call_count: usize,
        pub checkout_code_call_count: usize,
        pub prepare_artifacts_call_count: usize,
        pub prune_job_container_call_count: usize,
        pub start_job_container_call_count: usize,
        pub run_job_call_count: usize,
        pub extract_artifacts_call_count: usize,
    }

    impl ProcessesSpy {
        pub fn new() -> Self {
            ProcessesSpy::default()
        }

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

        fn prune_checkout_container(&mut self) -> Result<(), std::io::Error> {
            self.prune_checkout_container_call_count += 1;

            Ok(())
        }

        fn start_checkout_container(
            &mut self,
            _context: &Context,
        ) -> Result<String, std::io::Error> {
            self.start_checkout_container_call_count += 1;

            Ok("container-id".into())
        }

        fn checkout_code(
            &mut self,
            _container_id: &str,
            _context: &Context,
        ) -> Result<(), std::io::Error> {
            self.checkout_code_call_count += 1;

            Ok(())
        }

        fn prepare_artifacts(
            &mut self,
            _container_id: &str,
            _artifacts: &HashMap<String, Vec<String>>,
        ) -> Result<(), std::io::Error> {
            self.prepare_artifacts_call_count += 1;

            Ok(())
        }

        fn prune_job_container(&mut self) -> Result<(), std::io::Error> {
            self.prune_job_container_call_count += 1;

            Ok(())
        }

        fn start_job_container(
            &mut self,
            _job: &Job,
            _source_container_id: &str,
        ) -> Result<String, std::io::Error> {
            self.start_job_container_call_count += 1;

            Ok("container-id".into())
        }

        fn run_job(&mut self, _container_id: &str, _job: &Job) -> Result<(), std::io::Error> {
            self.run_job_call_count += 1;

            Ok(())
        }

        fn extract_artifacts(
            &mut self,
            _container_id: &str,
            _job_name: &str,
            _job: &Job,
        ) -> Result<(), std::io::Error> {
            self.extract_artifacts_call_count += 1;

            Ok(())
        }
    }
}
