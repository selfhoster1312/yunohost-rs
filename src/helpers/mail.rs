use regex::Regex;
use serde::Serialize;
use snafu::prelude::*;

use std::sync::OnceLock;

use crate::{
    error::*,
    helpers::{file::*, i18n, permission::*, process::*, service::*},
};

pub static REGEX_MAILUSAGE: OnceLock<Regex> = OnceLock::new();

#[derive(Clone, Debug, Serialize)]
pub struct MailStorageUse {
    limit: String,
    #[serde(rename = "use")]
    mail_use: String,
}

impl MailStorageUse {
    pub fn from_disk(user: &str, quota: &str) -> Result<Self, Error> {
        // If user does not exist it is treated as no use no quota
        let p = path(format!("/var/mail/{user}/maildirsize"));
        if !p.is_file() {
            return Ok(Self {
                limit: i18n::yunohost_no_context("unlimit")?,
                mail_use: "?".to_string(),
            });
        }

        // Need to omit the first line which contains quota information, which we already have
        // TODO: error context

        let lines: Vec<String> = p.read_lines()?.into_iter().skip(1).collect();
        let mut size_count: u64 = 0;

        // Each line has two entries separated by a space: the size count, and the mail count
        // We need to add the sizes together...
        for line in lines {
            let (size, _number) = line.split_once(' ').unwrap();
            // TODO: error
            let size: u64 = size.parse().unwrap();
            size_count += size;
        }

        let limit = if quota.starts_with("0") {
            i18n::yunohost_no_context("unlimit")?
        } else {
            quota.to_string()
        };

        // Mail use is calculated in KB so divide by 1024
        let mail_use = size_count / 1024;

        Ok(Self {
            limit,
            mail_use: format!("{mail_use}"),
        })
    }

    pub fn from_doveadm(user: &str, quota: &str) -> Result<Self, Error> {
        let limit = if quota.starts_with("0") {
            i18n::yunohost_no_context("unlimit")?
        } else {
            quota.to_string()
        };

        let mut mail_use = String::from("?");

        if !SystemCtl::is_active("dovecot") {
            warn!(
                "{}",
                i18n::yunohost_no_context("mailbox_used_space_dovecot_down").context(
                    MailStorageLookupSnafu {
                        user: user.to_string(),
                    }
                )?
            );
        } else if !YunohostPermission::get("mail.main")
            .context(MailStorageLookupSnafu {
                user: user.to_string(),
            })?
            .corresponding_users
            .contains(&user.to_string())
        {
            debug!(
                "{}",
                i18n::yunohost_context(
                    "mailbox_disabled",
                    hashmap!("user".to_string() => user.to_string())
                )
                .context(MailStorageLookupSnafu {
                    user: user.to_string(),
                })?
            );
        } else {
            let output = cmd("doveadm", vec!["-f", "flow", "quota", "get", "-u", user]).context(
                MailStorageLookupSnafu {
                    user: user.to_string(),
                },
            )?;
            let output = String::from_utf8_lossy(&output.stdout);

            // Use a global Regex for the life of the program, in case we're running in a loop, because generating
            // the regex could become the bottleneck...
            // UNWRAP NOTE: The regex cannot fail because it's well-known.
            let re = REGEX_MAILUSAGE.get_or_init(|| Regex::new(r"Value=(\d+)").unwrap());
            if let Some(captures) = re.captures(&output) {
                // TODO: human format
                mail_use = captures.get(1).unwrap().as_str().to_string();
            }
        }

        Ok(Self { limit, mail_use })
    }
}
