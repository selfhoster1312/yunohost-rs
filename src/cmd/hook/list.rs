use clap::Parser;

use crate::{
    error::*,
    helpers::{hook::*, output},
};

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
