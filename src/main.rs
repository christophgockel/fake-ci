pub mod file;
mod git;
mod gitlab;

use crate::git::read_details;
use crate::gitlab::{merge_all, parse, parse_all};
use anyhow::anyhow;
use file::RealFileSystem;
use std::env;
use std::env::current_dir;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = env::args().collect::<Vec<String>>();

    let path_to_config_file = if args.len() == 1 {
        let mut path = current_dir()?;
        path.push(".gitlab-ci.yml");
        path
    } else {
        let mut path = PathBuf::new();
        if args.len() < 2 {
            return Err(anyhow!("Missing path to .gitlab-ci.yml as argument"));
        }
        path.push(&args[1]);
        path
    };

    let file_access = RealFileSystem::default();
    let file = std::fs::File::open(path_to_config_file)?;
    let git_details = read_details().map_err(|_| anyhow!("Couldn't read git details"))?;

    let mut configuration =
        parse(file).map_err(|_| anyhow!("Couldn't parse configuration file"))?;
    let additional_configurations = parse_all(&configuration.include, &file_access, &git_details)
        .await
        .map_err(|_| anyhow!("Couldn't load additional includes"))?;
    merge_all(additional_configurations, &mut configuration)
        .map_err(|_| anyhow!("Couldn't merge configuration files"))?;

    let content = serde_yaml::to_string(&configuration).unwrap();

    println!("{}", content);

    Ok(())
}
