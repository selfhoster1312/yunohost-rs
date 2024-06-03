use snafu::prelude::*;

use std::str::FromStr;
use std::sync::OnceLock;

use crate::error::*;
use crate::helpers::file::path;

pub(crate) static DEBIAN_VERSION: OnceLock<DebianRelease> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebianRelease {
    Bullseye,
    Bookworm,
}

impl FromStr for DebianRelease {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "bullseye" | "11" => Ok(Self::Bullseye),
            "bookworm" | "12" => Ok(Self::Bookworm),
            _ => Err(Error::UnsupportedDebianRelease { version: s }),
        }
    }
}

impl DebianRelease {
    pub fn from_disk() -> Result<Self, Error> {
        let p = path("/etc/os-release");
        let s = p.read().context(DistroSnafu)?;
        for line in s.lines() {
            if line.starts_with("VERSION_ID=") {
                return Self::from_str(
                    line.trim_start_matches("VERSION_ID=\"")
                        .trim_end_matches("\""),
                );
            }
        }
        unreachable!("malformed /etc/os-release file.. missing VERSION_ID");
    }
}

pub fn debian_version() -> Result<&'static DebianRelease, Error> {
    if let Some(version) = DEBIAN_VERSION.get() {
        Ok(version)
    } else {
        let version = DebianRelease::from_disk()?;
        Ok(DEBIAN_VERSION.get_or_init(|| version))
    }
}
