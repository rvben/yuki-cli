//! Regional SOAP API hosts. Credentials and sessions belong to a single region.

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// The regional API serving an administration. Existing clients default to NL.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    #[default]
    Nl,
    Be,
}

impl Region {
    pub fn host(self) -> &'static str {
        match self {
            Self::Nl => "https://api.yukiworks.nl",
            Self::Be => "https://api.yukiworks.be",
        }
    }

    pub(crate) fn endpoint(self, service: &str) -> String {
        format!("{}/ws/{service}.asmx", self.host())
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Nl => "nl",
            Self::Be => "be",
        })
    }
}

impl FromStr for Region {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "nl" => Ok(Self::Nl),
            "be" => Ok(Self::Be),
            _ => Err("region must be 'nl' (Netherlands) or 'be' (Belgium)".into()),
        }
    }
}
