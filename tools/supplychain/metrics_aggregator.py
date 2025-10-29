#!/usr/bin/env python3
"""Aggregate supply chain metrics for dashboards and reporting.

This script combines data from SBOMs, SCA findings, license reports,
and conflict detection to produce comprehensive metrics.
"""

import argparse
import json
import sys
from datetime import datetime, timezone
from typing import Any, Dict, List


def load_json_file(filepath: str, required: bool = True) -> Dict[str, Any]:
    """Load JSON file with error handling.
    
    Args:
        filepath: Path to JSON file
        required: Whether file is required
        
    Returns:
        Loaded JSON data or empty dict if not required and missing
    """
    try:
        with open(filepath, "r") as f:
            return json.load(f)
    except FileNotFoundError:
        if required:
            raise
        return {}
    except json.JSONDecodeError as e:
        print(f"Error parsing {filepath}: {e}", file=sys.stderr)
        if required:
            raise
        return {}


def aggregate_vulnerability_metrics(sca_findings: Dict[str, Any]) -> Dict[str, Any]:
    """Aggregate vulnerability metrics from SCA findings.
    
    Args:
        sca_findings: SCA findings data
        
    Returns:
        Vulnerability metrics
    """
    if not sca_findings:
        return {
            "total": 0,
            "critical": 0,
            "high": 0,
            "medium": 0,
            "low": 0,
        }
    
    # Handle different SCA finding formats
    vulnerabilities = sca_findings.get("vulnerabilities", [])
    findings = sca_findings.get("findings", [])
    
    all_findings = vulnerabilities + findings
    
    metrics = {
        "total": len(all_findings),
        "critical": 0,
        "high": 0,
        "medium": 0,
        "low": 0,
    }
    
    for finding in all_findings:
        severity = finding.get("severity", "").upper()
        if severity == "CRITICAL":
            metrics["critical"] += 1
        elif severity == "HIGH":
            metrics["high"] += 1
        elif severity == "MEDIUM":
            metrics["medium"] += 1
        elif severity == "LOW":
            metrics["low"] += 1
    
    return metrics


def aggregate_dependency_metrics(sbom: Dict[str, Any], conflicts: Dict[str, Any]) -> Dict[str, Any]:
    """Aggregate dependency metrics from SBOM and conflict data.
    
    Args:
        sbom: SBOM data
        conflicts: Conflict detection data
        
    Returns:
        Dependency metrics
    """
    packages = sbom.get("packages", [])
    
    # Count direct vs transitive (simplified - assumes depth field exists)
    direct = sum(1 for pkg in packages if pkg.get("depth", 1) == 1)
    transitive = len(packages) - direct
    
    return {
        "total": len(packages),
        "direct": direct,
        "transitive": transitive,
        "conflicts": conflicts.get("conflicts_found", 0),
    }


def aggregate_license_metrics(license_report: Dict[str, Any]) -> Dict[str, Any]:
    """Aggregate license metrics from license report.
    
    Args:
        license_report: License analysis report
        
    Returns:
        License metrics
    """
    if not license_report:
        return {
            "copyleft": 0,
            "permissive": 0,
            "unknown": 0,
            "conflicts": 0,
        }
    
    summary = license_report.get("summary", {})
    conflicts_list = license_report.get("conflicts", [])
    
    return {
        "copyleft": summary.get("copyleft_count", 0),
        "permissive": summary.get("permissive_count", 0),
        "unknown": summary.get("unknown_count", 0),
        "conflicts": len(conflicts_list),
    }


def generate_metrics_report(
    sbom: Dict[str, Any],
    sca_findings: Dict[str, Any],
    license_report: Dict[str, Any],
    conflicts: Dict[str, Any],
    output_file: str,
    format_type: str = "json",
) -> None:
    """Generate comprehensive metrics report.
    
    Args:
        sbom: SBOM data
        sca_findings: SCA findings
        license_report: License analysis report
        conflicts: Conflict detection report
        output_file: Path to output file
        format_type: Output format ('json' or 'text')
    """
    metrics = {
        "version": "1.0",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "vulnerabilities": aggregate_vulnerability_metrics(sca_findings),
        "dependencies": aggregate_dependency_metrics(sbom, conflicts),
        "licenses": aggregate_license_metrics(license_report),
    }
    
    if format_type == "json":
        with open(output_file, "w") as f:
            json.dump(metrics, f, indent=2)
        print(f"Metrics report written to {output_file}", file=sys.stderr)
    else:
        # Text format
        output = []
        output.append("=" * 60)
        output.append("Supply Chain Metrics Report")
        output.append("=" * 60)
        output.append(f"Generated: {metrics['generated_at']}")
        output.append("")
        
        output.append("Vulnerabilities:")
        vuln = metrics["vulnerabilities"]
        output.append(f"  Total: {vuln['total']}")
        output.append(f"  [CRITICAL] Critical: {vuln['critical']}")
        output.append(f"  [HIGH] High: {vuln['high']}")
        output.append(f"  [MEDIUM] Medium: {vuln['medium']}")
        output.append(f"  [LOW] Low: {vuln['low']}")
        output.append("")
        
        output.append("Dependencies:")
        deps = metrics["dependencies"]
        output.append(f"  Total: {deps['total']}")
        output.append(f"  Direct: {deps['direct']}")
        output.append(f"  Transitive: {deps['transitive']}")
        output.append(f"  Conflicts: {deps['conflicts']}")
        output.append("")
        
        output.append("Licenses:")
        lics = metrics["licenses"]
        output.append(f"  Copyleft: {lics['copyleft']}")
        output.append(f"  Permissive: {lics['permissive']}")
        output.append(f"  Unknown: {lics['unknown']}")
        output.append(f"  Conflicts: {lics['conflicts']}")
        output.append("")
        
        output.append("=" * 60)
        
        text_output = "\n".join(output)
        with open(output_file, "w") as f:
            f.write(text_output)
        
        # Also print to stderr
        print(text_output, file=sys.stderr)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Aggregate supply chain metrics"
    )
    parser.add_argument(
        "--sbom",
        help="SBOM JSON file",
    )
    parser.add_argument(
        "--sca-findings",
        help="SCA findings JSON file",
    )
    parser.add_argument(
        "--license-report",
        help="License report JSON file",
    )
    parser.add_argument(
        "--conflicts",
        help="Conflict detection JSON file",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output file for metrics",
    )
    parser.add_argument(
        "--format",
        choices=["json", "text"],
        default="json",
        help="Output format (default: json)",
    )
    
    args = parser.parse_args()
    
    try:
        # Load input files
        sbom = load_json_file(args.sbom, required=False) if args.sbom else {}
        sca_findings = load_json_file(args.sca_findings, required=False) if args.sca_findings else {}
        license_report = load_json_file(args.license_report, required=False) if args.license_report else {}
        conflicts = load_json_file(args.conflicts, required=False) if args.conflicts else {}
        
        # Generate report
        generate_metrics_report(
            sbom,
            sca_findings,
            license_report,
            conflicts,
            args.output,
            args.format,
        )
        
    except Exception as e:
        print(f"Error generating metrics: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
