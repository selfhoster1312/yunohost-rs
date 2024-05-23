use clap::{Parser, Subcommand};

use crate::error::Error;

pub mod regen_conf;

#[derive(Clone, Debug, Parser)]
pub struct ToolsCommand {
    #[command(subcommand)]
    cmd: ToolsSubCommand,
}

impl ToolsCommand {
    pub fn run(&self) -> Result<(), Error> {
        match &self.cmd {
            ToolsSubCommand::RegenConf(cmd) => cmd.run(),
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum ToolsSubCommand {
    #[command(name = "regen-conf")]
    RegenConf(regen_conf::RegenConfCommand),
}
