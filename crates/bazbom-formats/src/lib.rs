pub mod cyclonedx;
pub mod licenses;
pub mod sarif;
pub mod spdx;

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Spdx,
    CycloneDx,
    Sarif,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "spdx" => Ok(Self::Spdx),
            "cyclonedx" | "cdx" => Ok(Self::CycloneDx),
            "sarif" => Ok(Self::Sarif),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Spdx => "spdx.json",
            Self::CycloneDx => "cyclonedx.json",
            Self::Sarif => "sarif",
        }
    }
}
