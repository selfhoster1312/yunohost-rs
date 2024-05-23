use clap::{Parser, Subcommand};

use crate::error::Error;

pub mod list;

#[derive(Clone, Debug, Parser)]
pub struct HookCommand {
    #[command(subcommand)]
    cmd: HookSubCommand,
}

impl HookCommand {
    pub fn run(&self) -> Result<(), Error> {
        match &self.cmd {
            HookSubCommand::List(cmd) => cmd.run(),
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum HookSubCommand {
    #[command(name = "list")]
    List(list::HookListCommand),
}
