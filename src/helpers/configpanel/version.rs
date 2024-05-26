use serde::{Deserialize, Serialize};

use std::str::FromStr;

use crate::error::*;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConfigPanelVersion {
    #[serde(rename = "1.0")]
    V1_0,
}

impl FromStr for ConfigPanelVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.0" => Ok(Self::V1_0),
            _ => Err(Error::ConfigPanelConfigVersionWrongStr {
                version: s.to_string(),
            }),
        }
    }
}

impl ConfigPanelVersion {
    pub fn to_f64(&self) -> f64 {
        match self {
            Self::V1_0 => 1.0,
        }
    }

    pub fn from_f64(version: f64) -> Result<Self, Error> {
        match version {
            1.0 => Ok(Self::V1_0),
            _ => Err(Error::ConfigPanelConfigVersionWrongFloat { version: version }),
        }
    }
}
