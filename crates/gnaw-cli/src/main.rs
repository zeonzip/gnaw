mod task;
mod config;
mod compilation;
mod error;

use std::ffi::OsString;
use clap::{Parser, Subcommand};
use colored::Colorize;
use crate::compilation::{create_directory_template, build_tasks};
use crate::config::get_toml_file;
use crate::error::CliRunError;
use crate::task::{get_tasks, run_task};

#[derive(Parser)]
pub struct GnawArgs {
    #[clap(subcommand)]
    subcommand: GnawSubcommand,
}

impl GnawArgs {
    pub fn run(self) -> Result<(), CliRunError> {
        match self.subcommand {
            GnawSubcommand::Run { task, rest } => {
                let config = get_toml_file()?;
                let path = build_tasks(config.task_project.path)?;

                run_task(path, task, config.task_project.context, rest)?;
            },
            GnawSubcommand::List => {
                let config = get_toml_file()?;
                let path = build_tasks(config.task_project.path)?;

                let mut tasks = get_tasks(&path)?;

                println!("{}", "Available tasks:".bold());

                if tasks.is_empty() {
                    println!(" {} (no tasks found)", "•".dimmed());
                } else {
                    tasks.sort();

                    for task in tasks {
                        println!(" {} {}", "•".dimmed(), task);
                    }
                }
            }
            GnawSubcommand::Setup => {
                println!("ℹ Setting up gnaw in the current directory...");

                create_directory_template(std::env::current_dir()?)?;

                println!("{} {} {} {}", "✔ Successfully setup gnaw!".green().bold(), "Get started in the", "gnaw/".italic().dimmed(), "folder!");
            }
        }

        Ok(())
    }
}

#[derive(Subcommand)]
pub enum GnawSubcommand {
    Run {
        task: String,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        rest: Vec<OsString>,
    },
    Setup,
    List
}

fn main() {
    let app = GnawArgs::parse();

    if let Err(err) = app.run() {
        println!("{}", err)
    }
}
