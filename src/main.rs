use serde_yaml::Value;
use std::env::current_dir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path_to_config_file = current_dir()?;
    path_to_config_file.push(".gitlab-ci.yml");

    let file = std::fs::File::open(path_to_config_file)?;
    let configuration: Value = serde_yaml::from_reader(file)?;

    let content = serde_yaml::to_string(&configuration).unwrap();

    println!("{}", content);

    Ok(())
}
