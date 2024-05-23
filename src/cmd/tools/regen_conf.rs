use clap::Parser;

use std::process::exit;

use crate::{error::*, helpers::file::*, helpers::output, helpers::regenconf::*};

#[derive(Clone, Debug, Parser)]
pub struct RegenConfCommand {
    #[arg(long = "list-pending")]
    list_pending: bool,

    #[arg(long)]
    json: bool,

    #[arg()]
    names: Vec<String>,
}

impl RegenConfCommand {
    pub fn run(&self) -> Result<(), Error> {
        if self.json {
            output::enable_json();
        }

        if path_exists("/etc/yunohost/settings.json") && !path_exists("/etc/yunohost/settings.yml")
        {
            eprintln!("This regenconf version can only run after 0025_global_settings_to_configpanel migration.");
            exit(1);
        }

        if self.list_pending {
            let pending = _get_pending_conf(&self.names)?;
            println!("{}", output::format(&pending)?);
        } else {
            println!("No command");
            exit(1);
        }

        Ok(())
    }
}
