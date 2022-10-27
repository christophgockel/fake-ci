use serde_yaml::Value;
use std::env;
use std::env::current_dir;
use std::io::ErrorKind;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<String>>();

    let path_to_config_file = if args.len() == 1 {
        let mut path = current_dir()?;
        path.push(".gitlab-ci.yml");
        path
    } else {
        let mut path = PathBuf::new();
        if args.len() < 2 {
            return Err(Box::new(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Missing path to .gitlab-ci.yml as argument",
            )));
        }
        path.push(&args[1]);
        path
    };

    let file = std::fs::File::open(path_to_config_file)?;
    let configuration: Value = serde_yaml::from_reader(file)?;

    let content = serde_yaml::to_string(&configuration).unwrap();

    println!("{}", content);

    Ok(())
}