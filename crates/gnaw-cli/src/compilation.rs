use crate::error::{CliRunError, CliRunErrorKind};
use serde::Deserialize;
use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
use std::env::current_dir;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};
use colored::Colorize;

const DEFAULT_TOML: &[u8] = include_bytes!("../templates/gnaw.toml");
const DEFAULT_CARGO_TOML: &[u8] = include_bytes!("../templates/Lib_Cargo.toml");
const DEFAULT_TASKS_RS: &[u8] = include_bytes!("../templates/Lib_tasks.rs");

pub fn create_directory_template<P: AsRef<Path>>(root_path: P) -> Result<(), SetupError> {
    let toml_path = root_path.as_ref().join("gnaw.toml");

    if fs::exists(&toml_path)? {
        return Err(SetupError::AlreadySetup)
    }

    let project_path = root_path.as_ref().join("gnaw");
    fs::create_dir(&project_path)?;

    create_file_with_data(toml_path, DEFAULT_TOML)?;
    create_cargo_project(&project_path)?;

    Ok(())
}

pub fn create_cargo_project<P: AsRef<Path>>(project_path: P) -> io::Result<()> {
    create_file_with_data(project_path.as_ref().join("Cargo.toml"), DEFAULT_CARGO_TOML)?;
    create_file_with_data(project_path.as_ref().join("tasks.rs"), DEFAULT_TASKS_RS)?;

    Ok(())
}

fn create_file_with_data<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, data: C) -> io::Result<()> {
    let mut file = File::create(&path)?;
    file.write_all(data.as_ref())?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum SetupError {
    #[error("gnaw is already set up in this directory")]
    AlreadySetup,
    #[error(transparent)]
    IoError(#[from] io::Error),
}

impl From<SetupError> for CliRunError {
    fn from(value: SetupError) -> CliRunError {
        let hint = match value {
            SetupError::AlreadySetup => Some("remove templates belonging to the last gnaw setup, or run this command in a different directory".to_owned()),
            _ => None
        };

        CliRunError {
            kind: CliRunErrorKind::SetupError(value),
            hint,
        }
    }
}

// Cargo toml data we need for gnaw
#[derive(Deserialize)]
struct CargoTomlFormat {
    package: CargoTomlPackage
}

#[derive(Deserialize)]
struct CargoTomlPackage {
    name: String,
}

pub fn build_tasks(path: PathBuf) -> Result<PathBuf, CompileError> {
    let project_path = current_dir()?.join(path);

    let file = fs::read(&project_path.join("Cargo.toml"))?;
    let serialized: CargoTomlFormat = toml::from_slice(&file)?;

    println!("{} {}", "info:".green().dimmed(), "Compiling tasks...".dimmed());

    let output = Command::new("cargo")
        .current_dir(&project_path)
        .arg("build")
        .arg("--release")
        .arg("-q")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !output.success() {
        return Err(CompileError::CompilationError)
    }

    let library_path = project_path.join(format!("target/release/{DLL_PREFIX}{}{DLL_SUFFIX}", serialized.package.name));

    Ok(library_path)
}

#[derive(thiserror::Error, Debug)]
pub enum CompileError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("Error deserializing Cargo.toml")]
    TomlError(#[from] toml::de::Error),
    #[error("error compiling tasks")]
    CompilationError,
}

impl From<CompileError> for CliRunError {
    fn from(value: CompileError) -> Self {
        CliRunError {
            kind: CliRunErrorKind::CompileError(value),
            hint: None,
        }
    }
}