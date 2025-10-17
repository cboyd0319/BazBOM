#!/usr/bin/env python3
"""Contribution Tracker - Track OSV vulnerability contributions.

This module tracks contributions to the OSV database, provides
statistics, and gamifies the contribution process.
"""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional


class ContributionTracker:
    """Track OSV vulnerability contributions."""
    
    def __init__(self, tracker_file: str = ".osv_contributions.json"):
        """Initialize contribution tracker.
        
        Args:
            tracker_file: Path to tracker state file
        """
        self.tracker_file = tracker_file
        self.contributions = self._load_contributions()
    
    def _load_contributions(self) -> Dict[str, Any]:
        """Load contribution history from file.
        
        Returns:
            Contribution data dictionary
        """
        if Path(self.tracker_file).exists():
            try:
                with open(self.tracker_file, 'r', encoding='utf-8') as f:
                    return json.load(f)
            except (json.JSONDecodeError, IOError):
                # If file is corrupted, start fresh
                pass
        
        # Initialize new tracker
        return {
            "version": "1.0",
            "created": datetime.now().isoformat(),
            "total_contributions": 0,
            "contributions": [],
            "statistics": {
                "by_ecosystem": {},
                "by_severity": {},
                "by_year": {},
                "by_contributor": {}
            }
        }
    
    def _save_contributions(self) -> None:
        """Save contribution history to file.
        
        Raises:
            IOError: If file cannot be written
        """
        try:
            with open(self.tracker_file, 'w', encoding='utf-8') as f:
                json.dump(self.contributions, f, indent=2)
        except IOError as e:
            raise IOError(f"Failed to save contributions to {self.tracker_file}: {e}")
    
    def add_contribution(
        self,
        vulnerability_id: str,
        package_name: str,
        ecosystem: str,
        severity: Optional[str] = None,
        contributor: str = "unknown",
        notes: str = ""
    ) -> None:
        """Add a new contribution record.
        
        Args:
            vulnerability_id: Vulnerability ID (CVE, GHSA, etc.)
            package_name: Package name
            ecosystem: Package ecosystem
            severity: Severity level
            contributor: Contributor name or email
            notes: Additional notes
            
        Raises:
            ValueError: If required fields are missing
        """
        if not vulnerability_id:
            raise ValueError("vulnerability_id is required")
        if not package_name:
            raise ValueError("package_name is required")
        if not ecosystem:
            raise ValueError("ecosystem is required")
        
        # Check for duplicates
        if any(c['id'] == vulnerability_id for c in self.contributions['contributions']):
            raise ValueError(f"Contribution {vulnerability_id} already recorded")
        
        # Create contribution record
        contribution = {
            "id": vulnerability_id,
            "package": package_name,
            "ecosystem": ecosystem,
            "severity": severity or "UNKNOWN",
            "contributor": contributor,
            "date": datetime.now().isoformat(),
            "notes": notes
        }
        
        # Add to list
        self.contributions['contributions'].append(contribution)
        self.contributions['total_contributions'] += 1
        
        # Update statistics
        self._update_statistics(contribution)
        
        # Save
        self._save_contributions()
    
    def _update_statistics(self, contribution: Dict[str, Any]) -> None:
        """Update statistics with new contribution.
        
        Args:
            contribution: Contribution record
        """
        stats = self.contributions['statistics']
        
        # By ecosystem
        ecosystem = contribution['ecosystem']
        stats['by_ecosystem'][ecosystem] = stats['by_ecosystem'].get(ecosystem, 0) + 1
        
        # By severity
        severity = contribution['severity']
        stats['by_severity'][severity] = stats['by_severity'].get(severity, 0) + 1
        
        # By year
        year = contribution['date'][:4]
        stats['by_year'][year] = stats['by_year'].get(year, 0) + 1
        
        # By contributor
        contributor = contribution['contributor']
        stats['by_contributor'][contributor] = stats['by_contributor'].get(contributor, 0) + 1
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get contribution statistics.
        
        Returns:
            Statistics dictionary
        """
        return self.contributions['statistics']
    
    def get_contributions(
        self,
        ecosystem: Optional[str] = None,
        contributor: Optional[str] = None,
        severity: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """Get list of contributions with optional filters.
        
        Args:
            ecosystem: Filter by ecosystem
            contributor: Filter by contributor
            severity: Filter by severity
            
        Returns:
            List of contribution records
        """
        contributions = self.contributions['contributions']
        
        # Apply filters
        if ecosystem:
            contributions = [c for c in contributions if c['ecosystem'] == ecosystem]
        if contributor:
            contributions = [c for c in contributions if c['contributor'] == contributor]
        if severity:
            contributions = [c for c in contributions if c['severity'] == severity]
        
        return contributions
    
    def generate_report(self) -> str:
        """Generate human-readable contribution report.
        
        Returns:
            Formatted report string
        """
        lines = []
        
        # Header
        lines.append("=" * 80)
        lines.append("OSV CONTRIBUTION REPORT")
        lines.append("=" * 80)
        lines.append("")
        
        # Summary
        total = self.contributions['total_contributions']
        lines.append(f"Total Contributions: {total}")
        lines.append(f"Tracking Since: {self.contributions['created'][:10]}")
        lines.append("")
        
        if total == 0:
            lines.append("No contributions recorded yet.")
            lines.append("")
            lines.append("Start contributing vulnerabilities to OSV!")
            lines.append("Use osv_contributor.py to generate OSV entries.")
            lines.append("")
            return "\n".join(lines)
        
        # Statistics
        stats = self.contributions['statistics']
        
        lines.append("-" * 80)
        lines.append("BY ECOSYSTEM")
        lines.append("-" * 80)
        for ecosystem, count in sorted(stats['by_ecosystem'].items(), key=lambda x: x[1], reverse=True):
            percentage = (count / total) * 100
            lines.append(f"  {ecosystem:20} {count:5} ({percentage:5.1f}%)")
        lines.append("")
        
        lines.append("-" * 80)
        lines.append("BY SEVERITY")
        lines.append("-" * 80)
        severity_order = ['CRITICAL', 'HIGH', 'MEDIUM', 'LOW', 'UNKNOWN']
        for severity in severity_order:
            count = stats['by_severity'].get(severity, 0)
            if count > 0:
                percentage = (count / total) * 100
                lines.append(f"  {severity:20} {count:5} ({percentage:5.1f}%)")
        lines.append("")
        
        lines.append("-" * 80)
        lines.append("BY YEAR")
        lines.append("-" * 80)
        for year, count in sorted(stats['by_year'].items(), reverse=True):
            percentage = (count / total) * 100
            lines.append(f"  {year:20} {count:5} ({percentage:5.1f}%)")
        lines.append("")
        
        lines.append("-" * 80)
        lines.append("TOP CONTRIBUTORS")
        lines.append("-" * 80)
        contributors = sorted(stats['by_contributor'].items(), key=lambda x: x[1], reverse=True)[:10]
        for i, (contributor, count) in enumerate(contributors, 1):
            percentage = (count / total) * 100
            lines.append(f"  {i:2}. {contributor:30} {count:5} ({percentage:5.1f}%)")
        lines.append("")
        
        # Recent contributions
        lines.append("-" * 80)
        lines.append("RECENT CONTRIBUTIONS (Last 10)")
        lines.append("-" * 80)
        recent = sorted(
            self.contributions['contributions'],
            key=lambda c: c['date'],
            reverse=True
        )[:10]
        
        for contrib in recent:
            lines.append(f"  {contrib['date'][:10]}  {contrib['id']:25}  {contrib['package']:30}")
            lines.append(f"                   {contrib['ecosystem']:10}  {contrib['severity']:10}  by {contrib['contributor']}")
            lines.append("")
        
        # Gamification
        lines.append("=" * 80)
        lines.append("ACHIEVEMENT BADGES")
        lines.append("=" * 80)
        badges = self._calculate_badges(total, stats)
        for badge in badges:
            icon = "🏆" if badge['achieved'] else "🔒"
            status = "UNLOCKED" if badge['achieved'] else "LOCKED"
            lines.append(f"{icon} {badge['name']:30} {status:10}  {badge['description']}")
        lines.append("")
        
        lines.append("=" * 80)
        
        return "\n".join(lines)
    
    def _calculate_badges(self, total: int, stats: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Calculate achievement badges.
        
        Args:
            total: Total contributions
            stats: Statistics dictionary
            
        Returns:
            List of badge dictionaries
        """
        badges = [
            {
                "name": "First Contribution",
                "description": "Make your first OSV contribution",
                "achieved": total >= 1
            },
            {
                "name": "Active Contributor",
                "description": "Contribute 10 vulnerabilities",
                "achieved": total >= 10
            },
            {
                "name": "Security Champion",
                "description": "Contribute 50 vulnerabilities",
                "achieved": total >= 50
            },
            {
                "name": "OSV Hero",
                "description": "Contribute 100 vulnerabilities",
                "achieved": total >= 100
            },
            {
                "name": "Multi-Ecosystem",
                "description": "Contribute to 3+ ecosystems",
                "achieved": len(stats['by_ecosystem']) >= 3
            },
            {
                "name": "Critical Finder",
                "description": "Report 5+ CRITICAL vulnerabilities",
                "achieved": stats['by_severity'].get('CRITICAL', 0) >= 5
            },
        ]
        
        return badges


def main() -> int:
    """Main entry point for CLI.
    
    Returns:
        Exit code (0 for success, non-zero for errors)
    """
    parser = argparse.ArgumentParser(
        description='Track OSV vulnerability contributions',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Add a contribution
  %(prog)s add --id CVE-2023-1234 \\
    --package com.example:mylib \\
    --ecosystem Maven \\
    --severity HIGH \\
    --contributor "security-team@example.com"
  
  # View contribution report
  %(prog)s report
  
  # List all contributions
  %(prog)s list
  
  # Filter by ecosystem
  %(prog)s list --ecosystem Maven
  
  # Export to JSON
  %(prog)s export -o contributions.json
"""
    )
    
    parser.add_argument(
        'action',
        choices=['add', 'list', 'report', 'export'],
        help='Action to perform'
    )
    
    parser.add_argument(
        '--tracker-file',
        default='.osv_contributions.json',
        help='Path to tracker file (default: .osv_contributions.json)'
    )
    
    parser.add_argument(
        '--id',
        help='Vulnerability ID (for add action)'
    )
    
    parser.add_argument(
        '--package',
        help='Package name (for add action)'
    )
    
    parser.add_argument(
        '--ecosystem',
        help='Ecosystem (for add/list actions)'
    )
    
    parser.add_argument(
        '--severity',
        choices=['CRITICAL', 'HIGH', 'MEDIUM', 'LOW'],
        help='Severity level (for add/list actions)'
    )
    
    parser.add_argument(
        '--contributor',
        help='Contributor name or email (for add/list actions)'
    )
    
    parser.add_argument(
        '--notes',
        default='',
        help='Additional notes (for add action)'
    )
    
    parser.add_argument(
        '-o', '--output',
        help='Output file (for export action)'
    )
    
    args = parser.parse_args()
    
    try:
        tracker = ContributionTracker(tracker_file=args.tracker_file)
        
        if args.action == 'add':
            if not all([args.id, args.package, args.ecosystem]):
                print("ERROR: --id, --package, and --ecosystem required for add", file=sys.stderr)
                return 1
            
            tracker.add_contribution(
                vulnerability_id=args.id,
                package_name=args.package,
                ecosystem=args.ecosystem,
                severity=args.severity,
                contributor=args.contributor or "unknown",
                notes=args.notes
            )
            
            print(f"✅ Added contribution: {args.id}", file=sys.stderr)
            print(f"   Package: {args.package}", file=sys.stderr)
            print(f"   Ecosystem: {args.ecosystem}", file=sys.stderr)
            if args.severity:
                print(f"   Severity: {args.severity}", file=sys.stderr)
        
        elif args.action == 'list':
            contributions = tracker.get_contributions(
                ecosystem=args.ecosystem,
                contributor=args.contributor,
                severity=args.severity
            )
            
            print(f"Found {len(contributions)} contributions:")
            print("")
            
            for contrib in sorted(contributions, key=lambda c: c['date'], reverse=True):
                print(f"{contrib['date'][:10]}  {contrib['id']:25}  {contrib['package']:30}")
                print(f"             {contrib['ecosystem']:10}  {contrib['severity']:10}  by {contrib['contributor']}")
                if contrib['notes']:
                    print(f"             Note: {contrib['notes']}")
                print("")
        
        elif args.action == 'report':
            report = tracker.generate_report()
            print(report)
        
        elif args.action == 'export':
            data = {
                "metadata": {
                    "version": tracker.contributions['version'],
                    "created": tracker.contributions['created'],
                    "total": tracker.contributions['total_contributions']
                },
                "contributions": tracker.contributions['contributions'],
                "statistics": tracker.contributions['statistics']
            }
            
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(data, f, indent=2)
                print(f"Exported to: {args.output}", file=sys.stderr)
            else:
                print(json.dumps(data, indent=2))
        
        return 0
        
    except FileNotFoundError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
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
