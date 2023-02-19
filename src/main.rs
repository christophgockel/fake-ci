mod commands;
mod core;
mod error;
pub mod file;
mod git;
mod gitlab;
mod io;
mod settings;

use crate::commands::{image, print, prune, run};
use crate::core::read_ci_definition;
use crate::error::FakeCiError;
use crate::file::FileAccess;
use crate::git::read_details;
use crate::io::processes::Processes;
use crate::io::prompt::{Prompt, Prompts};
use crate::settings::structure::Settings;
use crate::settings::{load_settings, LoadedSettings};
use anyhow::anyhow;
use clap::{Parser, Subcommand};
use file::RealFileSystem;
use std::env::current_dir;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let arguments = Arguments::parse();

    match run(arguments).await {
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
            FakeCiError::Command(e) => Err(anyhow!("Error running command: {}", e)),
            FakeCiError::Settings(e) => Err(anyhow!("Error reading settings: {}", e)),
        },
    }
}

async fn run(arguments: Arguments) -> Result<(), FakeCiError> {
    let git_details = read_details()?;
    let file_access = RealFileSystem::default();
    let mut prompt = Prompt::default();
    let mut path_to_settings_file = current_dir().map_err(FakeCiError::other)?;
    let path_to_configuration_file = match arguments.configuration_file {
        None => {
            let mut path = current_dir()?;
            path.push(".gitlab-ci.yml");
            path.as_path().display().to_string()
        }
        Some(path) => path,
    };

    path_to_settings_file.push(".fake-ci.yml");

    let settings = match load_settings(path_to_settings_file, &file_access).await? {
        LoadedSettings::FromFile(s) => {
            prompt.info("Using settings from .fake-ci.yml");
            s
        }
        LoadedSettings::Default(s) => s,
    };
    let gitlab_host = settings.gitlab.host.clone();
    let context = Context {
        current_directory: file_access.read_current_directory()?,
        git_sha: git_details.sha.clone(),
        image_tag: format!("fake-ci:{}", env!("CARGO_PKG_VERSION")),
    };
    let mut processes = Processes::default();

    match arguments.command {
        Command::Image(image) => Ok(image::command(
            &mut prompt,
            &mut processes,
            &context,
            &image,
        )?),
        Command::Prune(_) => Ok(prune::command(&mut prompt, &mut processes)?),
        Command::Run(run) => {
            let definition = read_ci_definition(
                path_to_configuration_file,
                &file_access,
                &git_details,
                &gitlab_host,
            )
            .await?;

            Ok(run::command(
                &mut prompt,
                &mut processes,
                &context,
                &definition,
                run.job,
            )?)
        }
        Command::Print(_) => Ok(print::command(
            path_to_configuration_file,
            &file_access,
            &git_details,
            &gitlab_host,
        )
        .await?),
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[clap(short, long)]
    configuration_file: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create Fake CI's base image.
    Image(image::Image),
    /// Remove all Docker artifacts.
    Prune(prune::Prune),
    /// Run a job.
    Run(run::Run),
    /// Print the fully parsed CI definition.
    Print(print::Print),
}

#[derive(Default)]
pub struct Context {
    pub current_directory: String,
    pub git_sha: String,
    pub image_tag: String,
}
