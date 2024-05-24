use clap::Parser;

use std::collections::BTreeMap;

use crate::{error::*, helpers::file::*, helpers::output, helpers::regenconf::*};

#[derive(Clone, Debug, Parser)]
pub struct RegenConfCommand {
    #[arg(short = 'p', long = "list-pending")]
    list_pending: bool,

    #[arg(short = 'd', long = "with-diff")]
    with_diff: bool,

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

        if path("/etc/yunohost/settings.json").exists()
            && !path("/etc/yunohost/settings.yml").exists()
        {
            eprintln!("This regenconf version can only run after 0025_global_settings_to_configpanel migration.");
            panic!();
        }

        if self.list_pending {
            self.run_list_pending()?;
        } else {
            unimplemented!("No command");
        }

        Ok(())
    }

    pub fn run_list_pending(&self) -> Result<(), Error> {
        let pending = _get_pending_conf(&self.names)?;

        if !self.with_diff {
            output::exit_success(pending);
        } else {
            let mut pending_diff: BTreeMap<String, BTreeMap<RelativeConfFile, PendingConfDiff>> =
                BTreeMap::new();

            for (category, conf_files) in pending {
                let mut category_files: BTreeMap<RelativeConfFile, PendingConfDiff> =
                    BTreeMap::new();
                for (system_path, pending_path) in conf_files {
                    category_files.insert(system_path, pending_path.into_pending_diff()?);
                }

                pending_diff.insert(category, category_files);
            }
            output::exit_success(pending_diff);
        }

        Ok(())
    }
}
