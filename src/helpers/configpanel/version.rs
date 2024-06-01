use serde::{Deserialize, Serialize};

use std::str::FromStr;

use super::error::ConfigPanelError;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct BookwormPanelVersion(pub u64);

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BullseyePanelVersion {
    #[serde(rename = "1.0")]
    V1_0,
}

impl FromStr for BullseyePanelVersion {
    type Err = ConfigPanelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.0" => Ok(Self::V1_0),
            _ => Err(ConfigPanelError::ConfigPanelVersion {
                version: s.to_string(),
            }),
        }
    }
}

impl BullseyePanelVersion {
    pub fn to_f64(&self) -> f64 {
        match self {
            Self::V1_0 => 1.0,
        }
    }

    pub fn from_f64(version: f64) -> Result<Self, ConfigPanelError> {
        match version {
            1.0 => Ok(Self::V1_0),
            _ => Err(ConfigPanelError::ConfigPanelVersion {
                version: version.to_string(),
            }),
        }
    }
}
