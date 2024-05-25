use snafu::prelude::*;

use crate::{error::*, helpers::file::*};

/// Change dpkg vendor, as per the [Debian documentation](https://wiki.debian.org/Derivatives/Guidelines#Vendor).
pub fn change_dpkg_vendor(vendor: &StrPath) -> Result<(), Error> {
    let default_origins = path("/etc/dpkg/origins/default");
    let current_default = default_origins
        .canonicalize()
        .context(ChangeDPKGVendorReadSnafu)?;

    if &current_default != vendor {
        default_origins
            .symlink_to_target(vendor, true)
            .context(ChangeDPKGVendorWriteSnafu)?;
    }

    Ok(())
}
