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

        if path_exists("/etc/yunohost/settings.json") && !path_exists("/etc/yunohost/settings.yml")
        {
            eprintln!("This regenconf version can only run after 0025_global_settings_to_configpanel migration.");
            panic!();
        }

        if self.list_pending {
            let pending = _get_pending_conf(&self.names)?;

            if !self.with_diff {
                println!("{}", output::format(&pending)?);
            } else {
                let mut pending_diff: BTreeMap<
                    String,
                    BTreeMap<RelativeConfFile, PendingConfDiff>,
                > = BTreeMap::new();

                for (category, conf_files) in pending {
                    let mut category_files: BTreeMap<RelativeConfFile, PendingConfDiff> =
                        BTreeMap::new();
                    for (system_path, pending_path) in conf_files {
                        category_files.insert(system_path, pending_path.into_pending_diff()?);
                    }

                    pending_diff.insert(category, category_files);
                }
                println!("{}", output::format(&pending_diff)?);
            }
        } else {
            unimplemented!("No command");
        }

        Ok(())
    }
}
