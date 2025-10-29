use bazbom_formats::OutputFormat;
use std::str::FromStr;

#[test]
fn test_from_str_spdx() {
    let format = OutputFormat::from_str("spdx").unwrap();
    assert_eq!(format, OutputFormat::Spdx);
}

#[test]
fn test_from_str_spdx_uppercase() {
    let format = OutputFormat::from_str("SPDX").unwrap();
    assert_eq!(format, OutputFormat::Spdx);
}

#[test]
fn test_from_str_cyclonedx() {
    let format = OutputFormat::from_str("cyclonedx").unwrap();
    assert_eq!(format, OutputFormat::CycloneDx);
}

#[test]
fn test_from_str_cyclonedx_short() {
    let format = OutputFormat::from_str("cdx").unwrap();
    assert_eq!(format, OutputFormat::CycloneDx);
}

#[test]
fn test_from_str_sarif() {
    let format = OutputFormat::from_str("sarif").unwrap();
    assert_eq!(format, OutputFormat::Sarif);
}

#[test]
fn test_from_str_invalid() {
    let result = OutputFormat::from_str("invalid");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unknown format: invalid");
}

#[test]
fn test_extension_spdx() {
    assert_eq!(OutputFormat::Spdx.extension(), "spdx.json");
}

#[test]
fn test_extension_cyclonedx() {
    assert_eq!(OutputFormat::CycloneDx.extension(), "cyclonedx.json");
}

#[test]
fn test_extension_sarif() {
    assert_eq!(OutputFormat::Sarif.extension(), "sarif");
}
