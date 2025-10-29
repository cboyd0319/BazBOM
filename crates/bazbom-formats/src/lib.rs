pub mod spdx;
pub mod cyclonedx;
pub mod sarif;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Spdx,
    CycloneDx,
    Sarif,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "spdx" => Some(Self::Spdx),
            "cyclonedx" | "cdx" => Some(Self::CycloneDx),
            "sarif" => Some(Self::Sarif),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Spdx => "spdx.json",
            Self::CycloneDx => "cyclonedx.json",
            Self::Sarif => "sarif",
        }
    }
}
