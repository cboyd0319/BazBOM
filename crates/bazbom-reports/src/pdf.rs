/// PDF report generation module
///
/// Provides PDF generation infrastructure for BazBOM reports.
/// Full PDF generation with genpdf requires proper font loading.
/// Currently exports helper functions for future PDF integration.

use anyhow::Result;
use std::path::Path;

use crate::ReportGenerator;

/// Generate a PDF report from a ReportGenerator
///
/// Currently returns a not-implemented error. Full PDF generation
/// will be completed in a future update with proper font handling.
///
/// For now, HTML reports can be converted to PDF using external tools like:
/// - wkhtmltopdf
/// - Chrome/Chromium headless
/// - LibreOffice command-line
pub fn generate_pdf(_generator: &ReportGenerator, _output_path: &Path) -> Result<()> {
    anyhow::bail!("PDF generation not yet fully implemented. Use HTML reports and convert with wkhtmltopdf or similar tools.")
}

/// Check if PDF generation is available
pub fn is_pdf_available() -> bool {
    false // Will be true once genpdf font loading is properly configured
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_availability() {
        assert!(!is_pdf_available());
    }
}
