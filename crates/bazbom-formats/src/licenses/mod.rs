pub mod detection;
pub mod compatibility;
pub mod obligations;

pub use detection::{LicenseDetector, LicenseInfo};
pub use compatibility::{LicenseCompatibility, LicenseRisk, ContaminationWarning, Dependency};
pub use obligations::{LicenseObligations, Obligation, ObligationType};
