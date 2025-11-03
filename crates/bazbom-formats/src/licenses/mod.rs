pub mod compatibility;
pub mod detection;
pub mod obligations;

pub use compatibility::{ContaminationWarning, Dependency, LicenseCompatibility, LicenseRisk};
pub use detection::{LicenseDetector, LicenseInfo};
pub use obligations::{LicenseObligations, Obligation, ObligationType};
