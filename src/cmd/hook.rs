use clap::{Parser, Subcommand};

use crate::{
    error::*,
    helpers::{hook::*, output},
};

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
    List(HookListCommand),
}

#[derive(Clone, Debug, Parser)]
pub struct HookListCommand {
    #[arg(long)]
    json: bool,

    #[arg()]
    action: String,
}

impl HookListCommand {
    pub fn run(&self) -> Result<(), Error> {
        if self.json {
            output::enable_json();
        }

        let list = HookList::for_action(&self.action);
        println!("{}", output::format(&list.names())?);

        Ok(())
    }
}
