mod commands;
mod core;
mod error;
pub mod file;
mod git;
mod gitlab;
mod io;

use crate::commands::{image, prune, run};
use crate::core::{convert_configuration, CiDefinition};
use crate::error::FakeCiError;
use crate::file::{FileAccess, FileAccessError};
use crate::git::{read_details, GitDetails};
use crate::gitlab::configuration::GitLabConfiguration;
use crate::gitlab::{merge_all, parse_all, read_configuration};
use crate::io::processes::Processes;
use crate::io::prompt::Prompt;
use anyhow::anyhow;
use clap::{Parser, Subcommand};
use file::RealFileSystem;
use std::env::current_dir;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let arguments = Arguments::parse();
    let git_details = read_details()?;

    if let Some(command) = arguments.command {
        let file_access = RealFileSystem::default();
        let context = Context {
            current_directory: file_access.read_current_directory()?,
            git_sha: git_details.sha.clone(),
            image_tag: format!("fake-ci:{}", env!("CARGO_PKG_VERSION")),
        };
        let mut prompt = Prompt::default();
        let mut processes = Processes::default();

        match command {
            Command::Image(image) => Ok(image::command(
                &mut prompt,
                &mut processes,
                &context,
                &image,
            )?),
            Command::Prune(_) => Ok(prune::command(&mut prompt, &mut processes)?),
            Command::Run(run) => {
                let definition = read_ci_definition(arguments.file_path, &git_details).await?;

                Ok(run::command(
                    &mut prompt,
                    &mut processes,
                    &context,
                    &definition,
                    run.job,
                )?)
            }
        }
    } else {
        match print_merged_configuration(arguments.file_path, &git_details).await {
            Ok(_) => Ok(()),
            Err(e) => match e {
                FakeCiError::File(e) => Err(anyhow!("{}", e)),
                FakeCiError::Git(e) => Err(anyhow!("Couldn't gather git details: {}", e)),
                FakeCiError::GitLab(e) => Err(anyhow!(
                    "Ran into an issue while parsing configuration file: {}",
                    e
                )),
                FakeCiError::Other(e) => Err(anyhow!("Unexpected error: {}", e)),
                FakeCiError::IO(e) => Err(anyhow!("Unexpected IO error: {}", e)),
            },
        }
    }
}

async fn print_merged_configuration(
    maybe_file_path: Option<String>,
    git: &GitDetails,
) -> Result<(), FakeCiError> {
    let configuration = read_gitlab_configuration(maybe_file_path, git).await?;
    let content = serde_yaml::to_string(&configuration).unwrap();

    println!("{}", content);

    Ok(())
}

async fn read_gitlab_configuration(
    maybe_file_path: Option<String>,
    git: &GitDetails,
) -> Result<GitLabConfiguration, FakeCiError> {
    let path_to_config_file = if maybe_file_path.is_none() {
        let mut path = current_dir().map_err(FakeCiError::other)?;
        path.push(".gitlab-ci.yml");
        path
    } else {
        let mut path = PathBuf::new();
        path.push(maybe_file_path.unwrap());
        path
    };

    let file_access = RealFileSystem::default();
    let file = std::fs::File::open(&path_to_config_file)
        .map_err(|e| FileAccessError::cannot_read(&path_to_config_file, e))?;
    let git_details = read_details()?;

    let mut configuration = read_configuration(file, git)?;
    let additional_configurations =
        parse_all(&configuration.include, &file_access, &git_details).await?;
    merge_all(additional_configurations, &mut configuration)?;

    Ok(configuration)
}

async fn read_ci_definition(
    maybe_file_path: Option<String>,
    git: &GitDetails,
) -> Result<CiDefinition, FakeCiError> {
    let configuration = read_gitlab_configuration(maybe_file_path, git).await?;
    let definition = convert_configuration(&configuration)?;

    Ok(definition)
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    file_path: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Create Fake CI's base image.
    Image(image::Image),
    /// Remove all Docker artifacts.
    Prune(prune::Prune),
    /// Run a job.
    Run(run::Run),
}

#[derive(Default)]
pub struct Context {
    pub current_directory: String,
    pub git_sha: String,
    pub image_tag: String,
}
