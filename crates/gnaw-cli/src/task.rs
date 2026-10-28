use std::{env};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use colored::Colorize;
use libloading::AsFilename;
use object::{Object, ObjectSymbol};
use gnaw_lib::os::OsArgsData;
use crate::error::{CliRunError, CliRunErrorKind};

pub const TASK_PREFIX: &str = "_gnaw_task_";

pub fn run_task<P>(library_path: P, task: String, context: PathBuf, rest: Vec<OsString>) -> Result<(), TaskRunError>
where
    P: AsRef<Path> + AsFilename
{
    let task_table = get_tasks(&library_path)?;

    let task = task_table.iter().find(|s| (*s).eq(&task)).ok_or(TaskRunError::TaskDoesntExist(task))?;
    let task_func: libloading::Symbol<unsafe extern "C" fn(OsArgsData)>;

    unsafe {
        let library = libloading::Library::new(library_path)?;
        task_func = library.get(format!("{}{}", TASK_PREFIX,  task))?;

        let current_dir = env::current_dir()?;
        env::set_current_dir(current_dir.join(context.clone())).map_err(|_| TaskRunError::ContextDirectoryDoesntExist(context))?;

        println!("\n{}", "Task Output:".blue());

        task_func(OsArgsData::from_os_strings(rest));
        
        env::set_current_dir(current_dir)?;
    }

    Ok(())
}

pub fn get_tasks<P: AsRef<Path>>(library_path: &P) -> Result<Vec<String>, TaskRunError> {
    let library_data = std::fs::read(&library_path).map_err(|_| TaskRunError::NoResolvedBuildLibrary)?;
    let object = object::File::parse(&*library_data)?;

    Ok(retrieve_tasks(object)?)
}

pub fn retrieve_tasks(obj: object::File) -> Result<Vec<String>, object::read::Error> {
    let symbols = obj.symbols();

    let mut tasks = Vec::new();

    for symbol in symbols {
        if symbol.is_global()
            && symbol.kind() == object::SymbolKind::Text
            && let Some(task_name) = normalize_symbol_name(symbol.name()?).strip_prefix(TASK_PREFIX)
        {
            tasks.push(task_name.to_string());
        }
    }

    Ok(tasks)
}

fn normalize_symbol_name(name: &str) -> &str {
    #[cfg(target_os = "macos")]
    { name.strip_prefix("_").unwrap_or(name) }

    #[cfg(not(target_os = "macos"))]
    name
}

#[derive(thiserror::Error, Debug)]
pub enum TaskRunError {
    #[error("unable to locate a build library")]
    NoResolvedBuildLibrary,
    #[error("failed to read binary data of build library: {0}")]
    BinaryReadError(#[from] object::read::Error),
    #[error("task \"{0}\" doesn't exist")]
    TaskDoesntExist(String),
    #[error("failed to load library: {0}")]
    LibraryLoadingError(#[from] libloading::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("provided context directory \"{0}\" does not exist")]
    ContextDirectoryDoesntExist(PathBuf)
}

impl From<TaskRunError> for CliRunError {
    fn from(value: TaskRunError) -> CliRunError {
        let hint = match &value {
            TaskRunError::ContextDirectoryDoesntExist(path) => Some(format!("consider making a folder at the location {:?} relative to the current folder, or edit the context property in your gnaw.toml to point at a valid directory" , path)),
            _ => None
        };

        CliRunError {
            kind: CliRunErrorKind::TaskError(value),
            hint
        }
    }
}