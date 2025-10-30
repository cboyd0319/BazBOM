use crate::config::Config;
use crate::context::Context;
use anyhow::Result;
use bazbom_formats::sarif::SarifReport;

pub trait Analyzer {
    fn id(&self) -> &'static str;
    fn enabled(&self, cfg: &Config, cli_override: bool) -> bool;
    fn run(&self, ctx: &Context) -> Result<SarifReport>;
}

pub trait Publisher {
    fn id(&self) -> &'static str;
    fn enabled(&self, cfg: &Config) -> bool;
    fn publish(&self, ctx: &Context, sarif: &SarifReport) -> Result<()>;
}

pub fn merge_sarif_reports(reports: Vec<SarifReport>) -> SarifReport {
    SarifReport::merge(reports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_empty_reports() {
        let merged = merge_sarif_reports(vec![]);
        assert_eq!(merged.runs.len(), 0);
    }

    #[test]
    fn test_merge_multiple_reports() {
        let r1 = SarifReport::new("tool1", "1.0");
        let r2 = SarifReport::new("tool2", "2.0");
        let merged = merge_sarif_reports(vec![r1, r2]);
        assert_eq!(merged.runs.len(), 2);
    }
}
