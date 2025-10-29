#!/usr/bin/env python3
"""SBOM Diff Engine - Compare SBOMs to identify changes between releases.

This module compares two SPDX SBOMs to identify:
- New dependencies added
- Dependencies removed
- Dependencies upgraded/downgraded
- License changes
- Vulnerability deltas

Supports both JSON output for automation and human-readable output for reports.
"""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple

try:
    from purl_generator import parse_purl
except ImportError:
    def parse_purl(purl: str) -> Dict[str, str]:
        """Minimal PURL parser fallback."""
        parts = purl.split('@')
        if len(parts) == 2:
            coords = parts[0].replace('pkg:maven/', '')
            group_artifact = coords.rsplit('/', 1)
            if len(group_artifact) == 2:
                return {
                    'group': group_artifact[0],
                    'artifact': group_artifact[1],
                    'version': parts[1]
                }
        return {}


class Package:
    """Represents a package from an SBOM."""
    
    def __init__(
        self,
        name: str,
        version: str,
        purl: Optional[str] = None,
        license: Optional[str] = None,
        vulnerabilities: Optional[List[str]] = None,
        spdxid: Optional[str] = None
    ):
        """Initialize a package.
        
        Args:
            name: Package name
            version: Package version
            purl: Package URL (PURL)
            license: License expression (SPDX format)
            vulnerabilities: List of CVE IDs affecting this package
            spdxid: SPDX element ID
        """
        self.name = name
        self.version = version
        self.purl = purl or ""
        self.license = license or "NOASSERTION"
        self.vulnerabilities = vulnerabilities or []
        self.spdxid = spdxid or ""
    
    def __eq__(self, other: Any) -> bool:
        """Compare packages by name and version."""
        if not isinstance(other, Package):
            return False
        return self.name == other.name and self.version == other.version
    
    def __hash__(self) -> int:
        """Hash by name and version for set operations."""
        return hash((self.name, self.version))
    
    def __repr__(self) -> str:
        """String representation."""
        return f"Package({self.name}@{self.version})"
    
    @property
    def key(self) -> str:
        """Unique key for this package (name only, version-agnostic)."""
        return self.name
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'name': self.name,
            'version': self.version,
            'purl': self.purl,
            'license': self.license,
            'vulnerabilities': self.vulnerabilities,
            'spdxid': self.spdxid
        }


class SBOMDiff:
    """Compare two SBOMs and identify changes."""
    
    def __init__(self, sbom_old: Dict[str, Any], sbom_new: Dict[str, Any]):
        """Initialize SBOM diff.
        
        Args:
            sbom_old: Old SBOM (baseline) as parsed JSON dict
            sbom_new: New SBOM (current) as parsed JSON dict
            
        Raises:
            ValueError: If SBOMs are invalid or incompatible
        """
        self._validate_sbom(sbom_old, "old")
        self._validate_sbom(sbom_new, "new")
        
        self.sbom_old = sbom_old
        self.sbom_new = sbom_new
        
        # Extract packages
        self.packages_old = self._extract_packages(sbom_old)
        self.packages_new = self._extract_packages(sbom_new)
        
        # Create lookup maps (name -> package)
        self.map_old = {pkg.name: pkg for pkg in self.packages_old}
        self.map_new = {pkg.name: pkg for pkg in self.packages_new}
        
        # Calculate diff
        self._calculate_diff()
    
    def _validate_sbom(self, sbom: Dict[str, Any], label: str) -> None:
        """Validate SBOM structure.
        
        Args:
            sbom: SBOM to validate
            label: Label for error messages (old/new)
            
        Raises:
            ValueError: If SBOM is invalid
        """
        if not isinstance(sbom, dict):
            raise ValueError(f"{label} SBOM must be a dictionary")
        
        if 'spdxVersion' not in sbom:
            raise ValueError(f"{label} SBOM missing 'spdxVersion' field")
        
        if 'packages' not in sbom:
            raise ValueError(f"{label} SBOM missing 'packages' field")
        
        if not isinstance(sbom['packages'], list):
            raise ValueError(f"{label} SBOM 'packages' must be a list")
    
    def _extract_packages(self, sbom: Dict[str, Any]) -> List[Package]:
        """Extract packages from SBOM.
        
        Args:
            sbom: SBOM as parsed JSON dict
            
        Returns:
            List of Package objects
        """
        packages = []
        
        for pkg_data in sbom.get('packages', []):
            # Extract basic info
            name = pkg_data.get('name', '')
            version = pkg_data.get('versionInfo', '')
            spdxid = pkg_data.get('SPDXID', '')
            
            # Skip if missing critical fields
            if not name or not version:
                continue
            
            # Extract PURL
            purl = None
            for ext_ref in pkg_data.get('externalRefs', []):
                if ext_ref.get('referenceType') == 'purl':
                    purl = ext_ref.get('referenceLocator', '')
                    break
            
            # Extract license
            license_concluded = pkg_data.get('licenseConcluded', 'NOASSERTION')
            
            # Create package
            pkg = Package(
                name=name,
                version=version,
                purl=purl,
                license=license_concluded,
                spdxid=spdxid
            )
            
            packages.append(pkg)
        
        return packages
    
    def _calculate_diff(self) -> None:
        """Calculate differences between SBOMs."""
        # Get package name sets
        names_old = set(self.map_old.keys())
        names_new = set(self.map_new.keys())
        
        # Calculate set differences
        self.added_names = names_new - names_old
        self.removed_names = names_old - names_new
        self.common_names = names_old & names_new
        
        # Find upgraded/downgraded packages
        self.upgraded: List[Tuple[Package, Package]] = []
        self.downgraded: List[Tuple[Package, Package]] = []
        self.unchanged: List[Package] = []
        self.license_changed: List[Tuple[Package, Package]] = []
        
        for name in self.common_names:
            old_pkg = self.map_old[name]
            new_pkg = self.map_new[name]
            
            if old_pkg.version != new_pkg.version:
                # Version changed - determine if upgrade or downgrade
                # Simple heuristic: compare version strings
                if self._is_upgrade(old_pkg.version, new_pkg.version):
                    self.upgraded.append((old_pkg, new_pkg))
                else:
                    self.downgraded.append((old_pkg, new_pkg))
            else:
                # Version same, but check license
                if old_pkg.license != new_pkg.license:
                    self.license_changed.append((old_pkg, new_pkg))
                else:
                    self.unchanged.append(old_pkg)
    
    def _is_upgrade(self, old_version: str, new_version: str) -> bool:
        """Determine if version change is an upgrade.
        
        Args:
            old_version: Old version string
            new_version: New version string
            
        Returns:
            True if new_version > old_version, False otherwise
        """
        # Try to parse as semantic version
        try:
            old_parts = self._parse_version(old_version)
            new_parts = self._parse_version(new_version)
            return new_parts > old_parts
        except (ValueError, AttributeError):
            # Fallback to string comparison
            return new_version > old_version
    
    def _parse_version(self, version: str) -> Tuple[int, ...]:
        """Parse version string into tuple of integers.
        
        Args:
            version: Version string (e.g., "1.2.3", "2.0.0-beta")
            
        Returns:
            Tuple of version parts as integers
            
        Raises:
            ValueError: If version can't be parsed
        """
        # Remove common suffixes
        version = version.split('-')[0].split('+')[0]
        
        # Split and convert to integers
        parts = []
        for part in version.split('.'):
            try:
                parts.append(int(part))
            except ValueError:
                # Handle non-numeric parts (e.g., "1.2.3a")
                numeric = ''.join(c for c in part if c.isdigit())
                if numeric:
                    parts.append(int(numeric))
                else:
                    raise ValueError(f"Cannot parse version part: {part}")
        
        return tuple(parts)
    
    def get_added_packages(self) -> List[Package]:
        """Get list of packages added in new SBOM."""
        return [self.map_new[name] for name in sorted(self.added_names)]
    
    def get_removed_packages(self) -> List[Package]:
        """Get list of packages removed from old SBOM."""
        return [self.map_old[name] for name in sorted(self.removed_names)]
    
    def get_upgraded_packages(self) -> List[Tuple[Package, Package]]:
        """Get list of (old, new) tuples for upgraded packages."""
        return sorted(self.upgraded, key=lambda t: t[0].name)
    
    def get_downgraded_packages(self) -> List[Tuple[Package, Package]]:
        """Get list of (old, new) tuples for downgraded packages."""
        return sorted(self.downgraded, key=lambda t: t[0].name)
    
    def get_unchanged_packages(self) -> List[Package]:
        """Get list of packages that didn't change."""
        return sorted(self.unchanged, key=lambda p: p.name)
    
    def get_license_changes(self) -> List[Tuple[Package, Package]]:
        """Get list of (old, new) tuples for packages with license changes."""
        return sorted(self.license_changed, key=lambda t: t[0].name)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert diff results to dictionary.
        
        Returns:
            Dictionary with diff results in structured format
        """
        return {
            'metadata': {
                'old_sbom_name': self.sbom_old.get('name', 'unknown'),
                'new_sbom_name': self.sbom_new.get('name', 'unknown'),
                'old_sbom_created': self.sbom_old.get('creationInfo', {}).get('created', ''),
                'new_sbom_created': self.sbom_new.get('creationInfo', {}).get('created', ''),
                'comparison_date': datetime.now().isoformat(),
            },
            'summary': {
                'total_old': len(self.packages_old),
                'total_new': len(self.packages_new),
                'added': len(self.added_names),
                'removed': len(self.removed_names),
                'upgraded': len(self.upgraded),
                'downgraded': len(self.downgraded),
                'unchanged': len(self.unchanged),
                'license_changed': len(self.license_changed),
            },
            'changes': {
                'added': [pkg.to_dict() for pkg in self.get_added_packages()],
                'removed': [pkg.to_dict() for pkg in self.get_removed_packages()],
                'upgraded': [
                    {
                        'name': old.name,
                        'old_version': old.version,
                        'new_version': new.version,
                        'old_license': old.license,
                        'new_license': new.license,
                    }
                    for old, new in self.get_upgraded_packages()
                ],
                'downgraded': [
                    {
                        'name': old.name,
                        'old_version': old.version,
                        'new_version': new.version,
                        'old_license': old.license,
                        'new_license': new.license,
                    }
                    for old, new in self.get_downgraded_packages()
                ],
                'license_changed': [
                    {
                        'name': old.name,
                        'version': old.version,
                        'old_license': old.license,
                        'new_license': new.license,
                    }
                    for old, new in self.get_license_changes()
                ],
            }
        }
    
    def to_human_readable(self) -> str:
        """Generate human-readable diff report.
        
        Returns:
            Multi-line string with formatted diff results
        """
        lines = []
        
        # Header
        lines.append("=" * 80)
        lines.append("SBOM DIFF REPORT")
        lines.append("=" * 80)
        lines.append("")
        
        # Metadata
        old_name = self.sbom_old.get('name', 'unknown')
        new_name = self.sbom_new.get('name', 'unknown')
        lines.append(f"Comparing:")
        lines.append(f"  Old: {old_name}")
        lines.append(f"  New: {new_name}")
        lines.append("")
        
        # Summary
        lines.append("Summary:")
        lines.append(f"  Total packages (old): {len(self.packages_old)}")
        lines.append(f"  Total packages (new): {len(self.packages_new)}")
        lines.append(f"  Added:                {len(self.added_names)}")
        lines.append(f"  Removed:              {len(self.removed_names)}")
        lines.append(f"  Upgraded:             {len(self.upgraded)}")
        lines.append(f"  Downgraded:           {len(self.downgraded)}")
        lines.append(f"  License changed:      {len(self.license_changed)}")
        lines.append(f"  Unchanged:            {len(self.unchanged)}")
        lines.append("")
        
        # Added packages
        if self.added_names:
            lines.append("-" * 80)
            lines.append(f"NEW DEPENDENCIES ({len(self.added_names)}):")
            lines.append("-" * 80)
            for pkg in self.get_added_packages():
                lines.append(f"  + {pkg.name}@{pkg.version} ({pkg.license})")
            lines.append("")
        
        # Removed packages
        if self.removed_names:
            lines.append("-" * 80)
            lines.append(f"REMOVED DEPENDENCIES ({len(self.removed_names)}):")
            lines.append("-" * 80)
            for pkg in self.get_removed_packages():
                lines.append(f"  - {pkg.name}@{pkg.version} ({pkg.license})")
            lines.append("")
        
        # Upgraded packages
        if self.upgraded:
            lines.append("-" * 80)
            lines.append(f"UPGRADED DEPENDENCIES ({len(self.upgraded)}):")
            lines.append("-" * 80)
            for old, new in self.get_upgraded_packages():
                lines.append(f"  ↑ {old.name}: {old.version} → {new.version}")
                if old.license != new.license:
                    lines.append(f"    License: {old.license} → {new.license}")
            lines.append("")
        
        # Downgraded packages
        if self.downgraded:
            lines.append("-" * 80)
            lines.append(f"DOWNGRADED DEPENDENCIES ({len(self.downgraded)}):")
            lines.append("-" * 80)
            for old, new in self.get_downgraded_packages():
                lines.append(f"  ↓ {old.name}: {old.version} → {new.version}")
                if old.license != new.license:
                    lines.append(f"    License: {old.license} → {new.license}")
            lines.append("")
        
        # License changes (same version)
        if self.license_changed:
            lines.append("-" * 80)
            lines.append(f"LICENSE CHANGES ({len(self.license_changed)}):")
            lines.append("-" * 80)
            for old, new in self.get_license_changes():
                lines.append(f"  [WARNING]  {old.name}@{old.version}")
                lines.append(f"      {old.license} → {new.license}")
            lines.append("")
        
        lines.append("=" * 80)
        
        return "\n".join(lines)


def load_sbom(path: str) -> Dict[str, Any]:
    """Load SBOM from file.
    
    Args:
        path: Path to SBOM file (JSON)
        
    Returns:
        Parsed SBOM as dictionary
        
    Raises:
        FileNotFoundError: If file doesn't exist
        json.JSONDecodeError: If file is not valid JSON
        ValueError: If file is not a valid SBOM
    """
    if not Path(path).exists():
        raise FileNotFoundError(f"SBOM file not found: {path}")
    
    with open(path, 'r', encoding='utf-8') as f:
        try:
            sbom = json.load(f)
        except json.JSONDecodeError as e:
            raise json.JSONDecodeError(
                f"Invalid JSON in {path}: {e.msg}",
                e.doc,
                e.pos
            )
    
    if not isinstance(sbom, dict):
        raise ValueError(f"SBOM must be a JSON object, got {type(sbom).__name__}")
    
    return sbom


def main() -> int:
    """Main entry point for CLI.
    
    Returns:
        Exit code (0 for success, non-zero for errors)
    """
    parser = argparse.ArgumentParser(
        description='Compare two SBOMs to identify changes',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Compare two SBOMs
  %(prog)s baseline.spdx.json current.spdx.json
  
  # Output JSON for automation
  %(prog)s old.json new.json --format json > diff.json
  
  # Save human-readable report
  %(prog)s v1.0.0.json v1.1.0.json --format text -o report.txt
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
        choices=['text', 'json'],
        default='text',
        help='Output format (default: text)'
    )
    
    parser.add_argument(
        '-o', '--output',
        help='Output file (default: stdout)'
    )
    
    args = parser.parse_args()
    
    try:
        # Load SBOMs
        print(f"Loading old SBOM: {args.old_sbom}", file=sys.stderr)
        sbom_old = load_sbom(args.old_sbom)
        
        print(f"Loading new SBOM: {args.new_sbom}", file=sys.stderr)
        sbom_new = load_sbom(args.new_sbom)
        
        # Calculate diff
        print("Calculating diff...", file=sys.stderr)
        diff = SBOMDiff(sbom_old, sbom_new)
        
        # Generate output
        if args.format == 'json':
            output = json.dumps(diff.to_dict(), indent=2)
        else:
            output = diff.to_human_readable()
        
        # Write output
        if args.output:
            print(f"Writing output to: {args.output}", file=sys.stderr)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(output)
            print("Done!", file=sys.stderr)
        else:
            print(output)
        
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
