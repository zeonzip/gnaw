use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::error::{CliRunError, CliRunErrorKind};

#[derive(Deserialize, Serialize, Debug)]
pub struct GnawTomlFormat {
    #[serde(rename = "task-project")]
    pub task_project: TaskProject,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TaskProject {
    pub path: PathBuf,
    pub context: PathBuf,
}

pub fn get_toml_file() -> Result<GnawTomlFormat, TomlError> {
    let file = fs::read("./gnaw.toml").map_err(|err| {
        if let std::io::ErrorKind::NotFound = err.kind() {
            TomlError::NoConfig
        } else {
            TomlError::Io(err)
        }
    })?;

    let deserialized: GnawTomlFormat = toml::from_slice(&file)?;

    Ok(deserialized)
}

#[derive(thiserror::Error, Debug)]
pub enum TomlError {
    #[error("gnaw.toml doesn't exist in the current working directory. Please setup gnaw by running 'gnaw setup' to get started.")]
    NoConfig,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Failed to read config file: {0}")]
    Deserialize(#[from] toml::de::Error),
}

impl From<TomlError> for CliRunError {
    fn from(value: TomlError) -> Self {
        CliRunError {
            kind: CliRunErrorKind::TomlError(value),
            hint: None,
        }
    }
}