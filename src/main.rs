mod error;
pub mod file;
mod git;
mod gitlab;

use crate::error::FakeCiError;
use crate::file::FileAccessError;
use crate::git::read_details;
use crate::gitlab::{merge_all, parse, parse_all};
use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use file::RealFileSystem;
use std::env::current_dir;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let arguments = Arguments::parse();

    if let Some(command) = arguments.command {
        match command {
            Command::Prune(_) => todo!(),
        }
    } else {
        match run(arguments.file_path).await {
            Ok(_) => Ok(()),
            Err(e) => match e {
                FakeCiError::File(e) => Err(anyhow!("{}", e)),
                FakeCiError::Git(e) => Err(anyhow!("Couldn't gather git details: {}", e)),
                FakeCiError::GitLab(e) => Err(anyhow!(
                    "Ran into an issue while parsing configuration file: {}",
                    e
                )),
                FakeCiError::Other(e) => Err(anyhow!("Unexpected error: {}", e)),
            },
        }
    }
}

async fn run(maybe_file_path: Option<String>) -> Result<(), FakeCiError> {
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

    let mut configuration = parse(file)?;
    let additional_configurations =
        parse_all(&configuration.include, &file_access, &git_details).await?;
    merge_all(additional_configurations, &mut configuration)?;

    let content = serde_yaml::to_string(&configuration).unwrap();

    println!("{}", content);

    Ok(())
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
    /// Remove all Docker artifacts.
    Prune(Prune),
}

#[derive(Args)]
struct Prune;
