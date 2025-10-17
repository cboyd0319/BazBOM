#!/usr/bin/env python3
"""Changelog Generator - Generate release notes from SBOM diffs.

This module generates human-readable changelogs focused on security and
dependency changes for release managers.
"""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    from sbom_diff import SBOMDiff, load_sbom
except ImportError:
    print("ERROR: sbom_diff module not found", file=sys.stderr)
    print("Ensure sbom_diff.py is in the same directory", file=sys.stderr)
    sys.exit(1)


class ChangelogGenerator:
    """Generate release notes from SBOM diffs."""
    
    def __init__(self, vulnerability_data: Optional[Dict[str, Any]] = None):
        """Initialize changelog generator.
        
        Args:
            vulnerability_data: Optional vulnerability data (CVE mappings)
        """
        self.vulnerability_data = vulnerability_data or {}
    
    def generate(
        self,
        diff: SBOMDiff,
        old_version: str = "previous",
        new_version: str = "current",
        format: str = "markdown"
    ) -> str:
        """Generate changelog from SBOM diff.
        
        Args:
            diff: SBOM diff results
            old_version: Old version label (e.g., "v1.0.0")
            new_version: New version label (e.g., "v1.1.0")
            format: Output format (markdown, html, text)
            
        Returns:
            Generated changelog as string
        """
        if format == "markdown":
            return self._generate_markdown(diff, old_version, new_version)
        elif format == "html":
            return self._generate_html(diff, old_version, new_version)
        elif format == "text":
            return self._generate_text(diff, old_version, new_version)
        else:
            raise ValueError(f"Unsupported format: {format}")
    
    def _generate_markdown(
        self,
        diff: SBOMDiff,
        old_version: str,
        new_version: str
    ) -> str:
        """Generate markdown changelog.
        
        Args:
            diff: SBOM diff results
            old_version: Old version label
            new_version: New version label
            
        Returns:
            Markdown-formatted changelog
        """
        lines = []
        
        # Header
        lines.append(f"# Release Notes: {old_version} → {new_version}")
        lines.append("")
        lines.append(f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        lines.append("")
        
        # Executive summary
        lines.append("## 📊 Summary")
        lines.append("")
        
        summary = diff.to_dict()['summary']
        
        lines.append("| Metric | Count |")
        lines.append("|--------|-------|")
        lines.append(f"| Total Dependencies (old) | {summary['total_old']} |")
        lines.append(f"| Total Dependencies (new) | {summary['total_new']} |")
        lines.append(f"| ➕ Added | {summary['added']} |")
        lines.append(f"| ➖ Removed | {summary['removed']} |")
        lines.append(f"| ⬆️ Upgraded | {summary['upgraded']} |")
        lines.append(f"| ⬇️ Downgraded | {summary['downgraded']} |")
        lines.append(f"| 📄 License Changed | {summary['license_changed']} |")
        lines.append("")
        
        # Security impact (if vulnerability data available)
        if self.vulnerability_data:
            lines.append("## 🔐 Security Impact")
            lines.append("")
            lines.append(self._generate_security_section(diff))
            lines.append("")
        
        # New dependencies
        added = diff.get_added_packages()
        if added:
            lines.append(f"## ➕ New Dependencies ({len(added)})")
            lines.append("")
            for pkg in added:
                lines.append(f"- **{pkg.name}** `{pkg.version}`")
                if pkg.license != "NOASSERTION":
                    lines.append(f"  - License: `{pkg.license}`")
                if pkg.purl:
                    lines.append(f"  - PURL: `{pkg.purl}`")
            lines.append("")
        
        # Removed dependencies
        removed = diff.get_removed_packages()
        if removed:
            lines.append(f"## ➖ Removed Dependencies ({len(removed)})")
            lines.append("")
            for pkg in removed:
                lines.append(f"- **{pkg.name}** `{pkg.version}`")
            lines.append("")
        
        # Upgraded dependencies
        upgraded = diff.get_upgraded_packages()
        if upgraded:
            lines.append(f"## ⬆️ Upgraded Dependencies ({len(upgraded)})")
            lines.append("")
            for old, new in upgraded:
                lines.append(f"- **{old.name}**: `{old.version}` → `{new.version}`")
                if old.license != new.license:
                    lines.append(f"  - License: `{old.license}` → `{new.license}`")
            lines.append("")
        
        # Downgraded dependencies
        downgraded = diff.get_downgraded_packages()
        if downgraded:
            lines.append(f"## ⚠️ Downgraded Dependencies ({len(downgraded)})")
            lines.append("")
            lines.append("> **Warning:** Downgrades may reintroduce known vulnerabilities.")
            lines.append("")
            for old, new in downgraded:
                lines.append(f"- **{old.name}**: `{old.version}` → `{new.version}`")
            lines.append("")
        
        # License changes
        license_changes = diff.get_license_changes()
        if license_changes:
            lines.append(f"## 📄 License Changes ({len(license_changes)})")
            lines.append("")
            for old, new in license_changes:
                lines.append(f"- **{old.name}** `{old.version}`")
                lines.append(f"  - `{old.license}` → `{new.license}`")
            lines.append("")
        
        # Footer
        lines.append("---")
        lines.append("")
        lines.append("*This changelog was automatically generated from SBOM comparison.*")
        lines.append("")
        
        return "\n".join(lines)
    
    def _generate_security_section(self, diff: SBOMDiff) -> str:
        """Generate security impact section.
        
        Args:
            diff: SBOM diff results
            
        Returns:
            Formatted security section
        """
        lines = []
        
        # Look up vulnerabilities for changed packages
        cves_fixed = []
        cves_introduced = []
        
        # Check upgraded packages (potential fixes)
        for old, new in diff.get_upgraded_packages():
            if old.name in self.vulnerability_data:
                old_cves = set(self.vulnerability_data[old.name].get(old.version, []))
                new_cves = set(self.vulnerability_data[old.name].get(new.version, []))
                
                fixed = old_cves - new_cves
                introduced = new_cves - old_cves
                
                cves_fixed.extend(list(fixed))
                cves_introduced.extend(list(introduced))
        
        # Check new packages
        for pkg in diff.get_added_packages():
            if pkg.name in self.vulnerability_data:
                cves = self.vulnerability_data[pkg.name].get(pkg.version, [])
                cves_introduced.extend(cves)
        
        if cves_fixed:
            lines.append(f"### ✅ Vulnerabilities Fixed ({len(cves_fixed)})")
            lines.append("")
            for cve in sorted(set(cves_fixed)):
                lines.append(f"- {cve}")
            lines.append("")
        
        if cves_introduced:
            lines.append(f"### ⚠️ New Vulnerabilities Introduced ({len(cves_introduced)})")
            lines.append("")
            for cve in sorted(set(cves_introduced)):
                lines.append(f"- {cve}")
            lines.append("")
        
        if not cves_fixed and not cves_introduced:
            lines.append("No significant security changes detected.")
            lines.append("")
        
        return "\n".join(lines)
    
    def _generate_html(
        self,
        diff: SBOMDiff,
        old_version: str,
        new_version: str
    ) -> str:
        """Generate HTML changelog.
        
        Args:
            diff: SBOM diff results
            old_version: Old version label
            new_version: New version label
            
        Returns:
            HTML-formatted changelog
        """
        # Convert markdown to HTML (simple conversion)
        markdown = self._generate_markdown(diff, old_version, new_version)
        
        lines = [
            '<!DOCTYPE html>',
            '<html>',
            '<head>',
            f'  <title>Release Notes: {old_version} → {new_version}</title>',
            '  <style>',
            '    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; max-width: 900px; margin: 40px auto; padding: 0 20px; }',
            '    h1 { color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 10px; }',
            '    h2 { color: #34495e; margin-top: 30px; }',
            '    table { border-collapse: collapse; width: 100%; margin: 20px 0; }',
            '    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }',
            '    th { background-color: #3498db; color: white; }',
            '    code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }',
            '    .warning { color: #e67e22; }',
            '    .success { color: #27ae60; }',
            '  </style>',
            '</head>',
            '<body>',
        ]
        
        # Convert markdown headings and lists to HTML (basic conversion)
        for line in markdown.split('\n'):
            if line.startswith('# '):
                lines.append(f'  <h1>{line[2:]}</h1>')
            elif line.startswith('## '):
                lines.append(f'  <h2>{line[3:]}</h2>')
            elif line.startswith('### '):
                lines.append(f'  <h3>{line[4:]}</h3>')
            elif line.startswith('| '):
                # Table row (simplified)
                lines.append(f'  <p>{line}</p>')
            elif line.startswith('- '):
                lines.append(f'  <li>{line[2:]}</li>')
            elif line.startswith('> '):
                lines.append(f'  <blockquote>{line[2:]}</blockquote>')
            elif line.strip():
                lines.append(f'  <p>{line}</p>')
        
        lines.extend([
            '</body>',
            '</html>',
        ])
        
        return '\n'.join(lines)
    
    def _generate_text(
        self,
        diff: SBOMDiff,
        old_version: str,
        new_version: str
    ) -> str:
        """Generate plain text changelog.
        
        Args:
            diff: SBOM diff results
            old_version: Old version label
            new_version: New version label
            
        Returns:
            Plain text changelog
        """
        lines = []
        
        # Header
        lines.append("=" * 80)
        lines.append(f"RELEASE NOTES: {old_version} → {new_version}")
        lines.append("=" * 80)
        lines.append("")
        lines.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        lines.append("")
        
        # Summary
        summary = diff.to_dict()['summary']
        lines.append("SUMMARY")
        lines.append("-" * 80)
        lines.append(f"  Total Dependencies (old):  {summary['total_old']}")
        lines.append(f"  Total Dependencies (new):  {summary['total_new']}")
        lines.append(f"  Added:                     {summary['added']}")
        lines.append(f"  Removed:                   {summary['removed']}")
        lines.append(f"  Upgraded:                  {summary['upgraded']}")
        lines.append(f"  Downgraded:                {summary['downgraded']}")
        lines.append(f"  License Changed:           {summary['license_changed']}")
        lines.append("")
        
        # New dependencies
        added = diff.get_added_packages()
        if added:
            lines.append(f"NEW DEPENDENCIES ({len(added)})")
            lines.append("-" * 80)
            for pkg in added:
                lines.append(f"  + {pkg.name}@{pkg.version} ({pkg.license})")
            lines.append("")
        
        # Removed dependencies
        removed = diff.get_removed_packages()
        if removed:
            lines.append(f"REMOVED DEPENDENCIES ({len(removed)})")
            lines.append("-" * 80)
            for pkg in removed:
                lines.append(f"  - {pkg.name}@{pkg.version}")
            lines.append("")
        
        # Upgraded dependencies
        upgraded = diff.get_upgraded_packages()
        if upgraded:
            lines.append(f"UPGRADED DEPENDENCIES ({len(upgraded)})")
            lines.append("-" * 80)
            for old, new in upgraded:
                lines.append(f"  ↑ {old.name}: {old.version} → {new.version}")
            lines.append("")
        
        # Downgraded dependencies
        downgraded = diff.get_downgraded_packages()
        if downgraded:
            lines.append(f"DOWNGRADED DEPENDENCIES ({len(downgraded)})")
            lines.append("-" * 80)
            lines.append("  WARNING: Downgrades may reintroduce known vulnerabilities.")
            lines.append("")
            for old, new in downgraded:
                lines.append(f"  ↓ {old.name}: {old.version} → {new.version}")
            lines.append("")
        
        # License changes
        license_changes = diff.get_license_changes()
        if license_changes:
            lines.append(f"LICENSE CHANGES ({len(license_changes)})")
            lines.append("-" * 80)
            for old, new in license_changes:
                lines.append(f"  {old.name}@{old.version}: {old.license} → {new.license}")
            lines.append("")
        
        lines.append("=" * 80)
        lines.append("")
        lines.append("This changelog was automatically generated from SBOM comparison.")
        lines.append("")
        
        return "\n".join(lines)


def main() -> int:
    """Main entry point for CLI.
    
    Returns:
        Exit code (0 for success, non-zero for errors)
    """
    parser = argparse.ArgumentParser(
        description='Generate release notes from SBOM diff',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate markdown release notes
  %(prog)s v1.0.0.json v1.1.0.json --format markdown
  
  # Generate HTML changelog
  %(prog)s old.json new.json --format html -o CHANGELOG.html
  
  # Generate plain text with version labels
  %(prog)s baseline.json current.json --old-version v1.0.0 --new-version v1.1.0
"""
    )
    
    parser.add_argument(
        'old_sbom',
        help='Path to old (baseline) SBOM file'
    )
    
    parser.add_argument(
        'new_sbom',
        help='Path to new (current) SBOM file'
    )
    
    parser.add_argument(
        '--format',
        choices=['markdown', 'html', 'text'],
        default='markdown',
        help='Output format (default: markdown)'
    )
    
    parser.add_argument(
        '--old-version',
        default='previous',
        help='Old version label (default: "previous")'
    )
    
    parser.add_argument(
        '--new-version',
        default='current',
        help='New version label (default: "current")'
    )
    
    parser.add_argument(
        '-o', '--output',
        help='Output file (default: stdout)'
    )
    
    parser.add_argument(
        '--vulnerability-data',
        help='Path to JSON file with vulnerability data (CVE mappings)'
    )
    
    args = parser.parse_args()
    
    try:
        # Load SBOMs
        print(f"Loading old SBOM: {args.old_sbom}", file=sys.stderr)
        sbom_old = load_sbom(args.old_sbom)
        
        print(f"Loading new SBOM: {args.new_sbom}", file=sys.stderr)
        sbom_new = load_sbom(args.new_sbom)
        
        # Load vulnerability data if provided
        vuln_data = None
        if args.vulnerability_data:
            print(f"Loading vulnerability data: {args.vulnerability_data}", file=sys.stderr)
            with open(args.vulnerability_data, 'r', encoding='utf-8') as f:
                vuln_data = json.load(f)
        
        # Calculate diff
        print("Calculating diff...", file=sys.stderr)
        diff = SBOMDiff(sbom_old, sbom_new)
        
        # Generate changelog
        print(f"Generating {args.format} changelog...", file=sys.stderr)
        generator = ChangelogGenerator(vulnerability_data=vuln_data)
        changelog = generator.generate(
            diff,
            old_version=args.old_version,
            new_version=args.new_version,
            format=args.format
        )
        
        # Write output
        if args.output:
            print(f"Writing output to: {args.output}", file=sys.stderr)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(changelog)
            print("Done!", file=sys.stderr)
        else:
            print(changelog)
        
        return 0
        
    except FileNotFoundError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    except json.JSONDecodeError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 2
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 2
    except Exception as e:
        print(f"ERROR: Unexpected error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        return 3


if __name__ == '__main__':
    sys.exit(main())
