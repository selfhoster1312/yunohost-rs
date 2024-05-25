use snafu::prelude::*;

use crate::{
    error::*,
    helpers::{file::*, process::cmd},
};

pub struct YunohostGroup;

impl YunohostGroup {
    /// Checks whether a POSIX group exists.
    ///
    /// Errors when:
    ///   - reading /etc/group failed
    pub fn exists(name: &str) -> Result<bool, Error> {
        let expected = format!("{name}:");
        let found_group = path("/etc/group")
            .read_lines()
            .context(YunohostGroupExistsReadSnafu {
                name: name.to_string(),
            })?
            .iter()
            .any(|line| line.starts_with(&expected));

        Ok(found_group)
    }

    /// Creates a POSIX group on the system.
    ///
    /// Errors when:
    ///   - reading /etc/group failed
    ///   - group `name` already exists
    ///   - groupadd command fails
    pub fn add(name: &str) -> Result<(), Error> {
        if Self::exists(name)? {
            return Err(Error::YunohostGroupExists {
                name: name.to_string(),
            });
        }

        if !cmd("groupadd", vec![name]).unwrap().status.success() {
            return Err(Error::YunohostGroupCreate {
                name: name.to_string(),
            });
        }

        Ok(())
    }

    /// Make sure a POSIX group exists on the system.
    ///
    /// Errors when:
    ///   - reading /etc/group failed
    ///   - group `name` did not exist and groupadd command failed
    ///
    /// Does not error when the group does not exist.
    pub fn ensure_exists(name: &str) -> Result<(), Error> {
        if Self::exists(name)? {
            return Ok(());
        }

        if !cmd("groupadd", vec![name]).unwrap().status.success() {
            return Err(Error::YunohostGroupCreate {
                name: name.to_string(),
            });
        }

        Ok(())
    }
}
