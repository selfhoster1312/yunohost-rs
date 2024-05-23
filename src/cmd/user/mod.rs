use clap::{Parser, Subcommand};

use crate::error::Error;

pub mod info;
pub mod list;

#[derive(Clone, Debug, Parser)]
pub struct UserCommand {
    #[command(subcommand)]
    cmd: UserSubCommand,
}

impl UserCommand {
    pub fn run(&self) -> Result<(), Error> {
        match &self.cmd {
            UserSubCommand::UserInfo(cmd) => cmd.run(),
            UserSubCommand::UserList(cmd) => cmd.run(),
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum UserSubCommand {
    #[command(name = "info")]
    UserInfo(info::UserInfoCommand),
    #[command(name = "list")]
    UserList(list::UserListCommand),
}
