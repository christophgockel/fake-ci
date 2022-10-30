use regex::Regex;
use std::error::Error;
use std::io::ErrorKind;
use std::process::Command;
use url::Url;

#[derive(Default)]
pub struct GitDetails {
    pub host: String,
}

pub fn read_details() -> Result<GitDetails, Box<dyn Error>> {
    let output = Command::new("git")
        .arg("ls-remote")
        .arg("--get-url")
        .output()?;
    let content = std::str::from_utf8(&output.stdout).unwrap();
    let mut content = content.trim().to_string();

    if content.is_empty() {
        return Err(Box::new(std::io::Error::new(
            ErrorKind::Other,
            "no git host configured",
        )));
    }

    let mut host = None;

    // looks for structure like git@github.com:username/repo.git
    let ssh_url = Regex::new(r"^\S+(@)\S+(:).*$")?;

    if ssh_url.is_match(&content) {
        content = content.replace(':', "/");
        content = content.replace("git@", "https://");

        let url = Url::parse(&content)?;

        match url.host() {
            None => {}
            Some(h) => {
                host = Some(format!("{}://{}", url.scheme(), h));
            }
        }
    } else {
        return Err(Box::new(std::io::Error::new(
            ErrorKind::Other,
            "non-ssh git remote is not supported yet.",
        )));
    }

    Ok(GitDetails {
        host: host.unwrap_or_else(|| "".into()),
    })
}
