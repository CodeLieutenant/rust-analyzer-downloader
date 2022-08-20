use std::{
    borrow::Cow,
    io::{Error as IoError, ErrorKind},
};

use thiserror::Error as ThisError;
use tokio::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub struct Version {
    pub date_version: String,
    pub semantic_version: String,
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("Failed to execute command: {0}")]
    Command(String),

    #[error("Failed to parse rust-analyzer version, Reason: {0}")]
    Parse(String),
}

fn parse_version(output: Cow<'_, str>) -> Result<Version, Error> {
    // eg. rust-analyzer 0.4.1173-standalone (82ff74050 2022-08-17)
    let semantic_version = output
        .chars()
        .skip_while(|c| *c != ' ')
        .skip(1)
        .take_while(|c| *c != ' ')
        .collect::<String>();

    let date_version = output
        .chars()
        .skip_while(|c| *c != '(')
        .skip_while(|c| *c != ' ')
        .skip(1) // skip space
        .take_while(|c| *c != ')')
        .collect::<String>();

    if semantic_version.is_empty() {
        Err(Error::Parse(format!("no semantic version '{}'", output)))
    } else if date_version.is_empty(){
        Err(Error::Parse(format!("no date version '{}'", output)))
    } else {
        Ok(Version{
            date_version,
            semantic_version,
        })
    }
}

pub async fn get<'a>() -> Result<Version, Error> {
    let version = Command::new("rust-analyzer")
        .arg("--version")
        .output()
        .await?;

    if version.status.success() {
        let output = version.stdout.split(|b| b == &b'\n').next();

        match output {
            Some(value) => parse_version(String::from_utf8_lossy(value)),
            None => Err(Error::Io(IoError::new(
                ErrorKind::Other,
                "No new line found in output",
            ))),
        }
    } else {
        Err(Error::Command("rust-analyzer --version".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let output = "rust-analyzer 0.4.1173-standalone (82ff74050 2022-08-17)";
        let version = parse_version(output.into()).unwrap();
        assert_eq!(version, Version {date_version: "2022-08-17".to_string(), semantic_version: "0.4.1173-standalone".to_string()});
    }

    #[test]
    fn test_parse_version_no_semantic_version() {
        let output = "rust-analyzer";
        let version = parse_version(output.into());

        assert!(version.is_err());
        assert_eq!(
            version.unwrap_err().to_string(),
            "Failed to parse rust-analyzer version, Reason: no semantic version 'rust-analyzer'"
        );
    }

    #[test]
    fn test_parse_version_no_date_version() {
        let output = "rust-analyzer 0.4.1173-standalone";
        let version = parse_version(output.into());
        assert!(version.is_err());
        assert_eq!(
            version.unwrap_err().to_string(),
            "Failed to parse rust-analyzer version, Reason: no date version 'rust-analyzer 0.4.1173-standalone'"
        );
    }
}
