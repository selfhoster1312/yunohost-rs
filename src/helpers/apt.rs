use snafu::prelude::*;

use std::path::{Path, PathBuf};

use crate::{
    error::*,
    helpers::file::{readlink_canonicalize, symlink},
};

/// Change dpkg vendor, as per the [Debian documentation](https://wiki.debian.org/Derivatives/Guidelines#Vendor).
pub fn change_dpkg_vendor<T: AsRef<Path>>(vendor: T) -> Result<(), Error> {
    let vendor = vendor.as_ref();

    let default_origins = PathBuf::from("/etc/dpkg/origins/default");
    let current_default =
        readlink_canonicalize(&default_origins).context(ChangeDPKGVendorReadSnafu)?;

    if current_default != vendor {
        symlink(vendor, &default_origins, true).context(ChangeDPKGVendorWriteSnafu)?;
    }

    Ok(())
}
