use clap::Parser;

use crate::{error::Error, helpers::output};

#[derive(Clone, Debug, Parser)]
pub struct SettingsListCommand {
    #[arg(short, long, name = "full")]
    _full: bool,

    #[arg(long)]
    json: bool,

    #[arg()]
    setting: String,
}

impl SettingsListCommand {
    pub fn run(&self) -> Result<(), Error> {
        if self.json {
            output::enable_json();
        }

        Ok(())
    }
}
