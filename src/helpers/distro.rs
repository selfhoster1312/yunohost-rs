use std::str::FromStr;

use crate::error::*;
use crate::helpers::process::cmd;

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
    // TODO: this is apparently super slow... just did +0.03s on settings get --full --json ????
    pub fn from_cmd() -> Result<Self, Error> {
        let output = cmd("lsb_release", vec!["-rs"]).unwrap();
        Self::from_str(&String::from_utf8_lossy(&output.stdout))
    }
}
