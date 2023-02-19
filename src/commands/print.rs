use crate::commands::CommandError;
use crate::file::FileAccess;
use crate::git::GitDetails;
use crate::gitlab::read_gitlab_configuration;
use clap::Args;

#[derive(Args)]
pub struct Print;

pub async fn command(
    path_to_config_file: String,
    file_access: &impl FileAccess,
    git: &GitDetails,
    gitlab_host: &String,
) -> Result<(), CommandError> {
    let configuration =
        read_gitlab_configuration(path_to_config_file, file_access, git, gitlab_host).await?;
    let content = serde_yaml::to_string(&configuration).unwrap();

    println!("{}", content);

    Ok(())
}
