use std::error::Error;
use std::fmt::Display;
use colored::Colorize;
use crate::compilation::{CompileError, SetupError};
use crate::config::TomlError;
use crate::task::TaskRunError;

#[derive(thiserror::Error, Debug)]
pub enum CliRunErrorKind {
    #[error(transparent)]
    TaskError(#[from] TaskRunError),
    #[error(transparent)]
    TomlError(#[from] TomlError),
    #[error("I/O failed: {0}")]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SetupError(#[from] SetupError),
    #[error(transparent)]
    CompileError(#[from] CompileError),
}

impl From<std::io::Error> for CliRunError {
    fn from(value: std::io::Error) -> CliRunError {
        CliRunError {
            kind: CliRunErrorKind::IoError(value),
            hint: None,
        }
    }
}

#[derive(Debug)]
pub struct CliRunError {
    pub(crate) kind: CliRunErrorKind,
    pub(crate) hint: Option<String>,
}

impl Display for CliRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}  {}", "error:".red().bold(), self.kind)?;
        if let Some(hint) = &self.hint {
            write!(f, "\n {} {}  {}", "╰".dimmed(), "hint:".blue().bold(), hint)?;
        }

        Ok(())
    }
}

impl Error for CliRunError {}
