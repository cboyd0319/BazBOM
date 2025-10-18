#!/usr/bin/env python3
"""AI-Powered Dependency Upgrade Recommender.

This module provides intelligent recommendations for upgrading dependencies,
analyzing breaking changes, compatibility, and effort estimation.
"""

import argparse
import json
import re
import sys
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple


@dataclass
class UpgradeRecommendation:
    """Recommendation for upgrading a dependency."""
    
    package: str
    current_version: str
    recommended_version: str
    latest_version: str
    breaking_changes: List[str]
    compatibility_score: float
    effort_estimate: str
    confidence: float
    migration_guide: str
    security_fixes: List[str]
    timestamp: str = ""
    
    def __post_init__(self):
        if not self.timestamp:
            self.timestamp = datetime.now().isoformat()


@dataclass
class BreakingChange:
    """Represents a breaking change between versions."""
    
    version: str
    change_type: str  # API, BEHAVIOR, DEPENDENCY, REMOVAL
    description: str
    severity: str  # HIGH, MEDIUM, LOW
    affected_apis: List[str]
    workaround: Optional[str] = None


class BreakingChangeAnalyzer:
    """Analyze breaking changes between package versions."""
    
    # Patterns that indicate breaking changes in changelogs/release notes
    BREAKING_PATTERNS = [
        r"BREAKING\s+CHANGE",
        r"breaking\s+change",
        r"Breaking:",
        r"incompatible\s+changes?",
        r"removed\s+(?:method|class|API)",
        r"deprecated\s+and\s+removed",
        r"major\s+version",
        r"backward[s]?\s+incompatible",
    ]
    
    # Patterns for API changes
    API_CHANGE_PATTERNS = [
        r"(?:method|function|class)\s+signature\s+changed",
        r"return\s+type\s+changed",
        r"parameter\s+(?:added|removed|changed)",
        r"API\s+changed",
    ]
    
    def __init__(self):
        """Initialize breaking change analyzer."""
        self.breaking_pattern = re.compile(
            "|".join(self.BREAKING_PATTERNS),
            re.IGNORECASE
        )
        self.api_pattern = re.compile(
            "|".join(self.API_CHANGE_PATTERNS),
            re.IGNORECASE
        )
    
    def analyze_changelog(self, changelog_text: str) -> List[BreakingChange]:
        """Analyze changelog text for breaking changes.
        
        Args:
            changelog_text: Changelog or release notes text
            
        Returns:
            List of identified breaking changes
        """
        if not changelog_text:
            return []
        
        changes = []
        
        # Split by version sections (common changelog format)
        version_sections = re.split(r"\n##?\s+(?:Version\s+)?(\d+\.\d+(?:\.\d+)?)", changelog_text)
        
        for i in range(1, len(version_sections), 2):
            if i + 1 >= len(version_sections):
                break
            
            version = version_sections[i].strip()
            content = version_sections[i + 1]
            
            # Check for breaking change markers
            if self.breaking_pattern.search(content):
                # Extract breaking change descriptions
                lines = content.split('\n')
                for line in lines:
                    line = line.strip()
                    if not line or line.startswith('#'):
                        continue
                    
                    if self.breaking_pattern.search(line):
                        # Determine change type
                        change_type = "API" if self.api_pattern.search(line) else "BEHAVIOR"
                        
                        # Extract severity from keywords
                        severity = "HIGH"
                        if any(word in line.lower() for word in ["minor", "small", "slight"]):
                            severity = "LOW"
                        elif any(word in line.lower() for word in ["moderate", "some"]):
                            severity = "MEDIUM"
                        
                        changes.append(BreakingChange(
                            version=version,
                            change_type=change_type,
                            description=line[:200],  # Limit length
                            severity=severity,
                            affected_apis=self._extract_apis(line),
                        ))
        
        return changes
    
    def _extract_apis(self, text: str) -> List[str]:
        """Extract API names from text.
        
        Args:
            text: Text containing API references
            
        Returns:
            List of API names found
        """
        # Look for Java/method-like patterns
        api_pattern = r"\b([A-Z][a-zA-Z0-9]*(?:\.[a-zA-Z0-9]+)*(?:\(\))?)\b"
        matches = re.findall(api_pattern, text)
        return list(set(matches[:5]))  # Limit to 5 unique APIs
    
    def count_breaking_changes(self, from_version: str, to_version: str, 
                               changes: List[BreakingChange]) -> int:
        """Count breaking changes between two versions.
        
        Args:
            from_version: Starting version
            to_version: Target version
            changes: List of all breaking changes
            
        Returns:
            Number of breaking changes in range
        """
        count = 0
        for change in changes:
            if self._is_version_in_range(change.version, from_version, to_version):
                count += 1
        return count
    
    def _is_version_in_range(self, version: str, from_v: str, to_v: str) -> bool:
        """Check if version is between from_v and to_v.
        
        Args:
            version: Version to check
            from_v: Lower bound (exclusive)
            to_v: Upper bound (inclusive)
            
        Returns:
            True if version is in range
        """
        try:
            v = self._parse_version(version)
            f = self._parse_version(from_v)
            t = self._parse_version(to_v)
            return f < v <= t
        except (ValueError, IndexError):
            return False
    
    def _parse_version(self, version: str) -> Tuple[int, ...]:
        """Parse version string to tuple for comparison.
        
        Args:
            version: Version string (e.g., "1.2.3")
            
        Returns:
            Tuple of version numbers
            
        Raises:
            ValueError: If version format is invalid
        """
        # Remove common prefixes
        version = version.lstrip('v')
        # Extract only numeric parts
        parts = re.findall(r'\d+', version)
        if not parts:
            raise ValueError(f"Invalid version format: {version}")
        return tuple(int(p) for p in parts)


class UpgradeRecommender:
    """Recommend safe upgrade paths for dependencies."""
    
    def __init__(self, sbom_path: Optional[str] = None):
        """Initialize upgrade recommender.
        
        Args:
            sbom_path: Optional path to SBOM file
            
        Raises:
            FileNotFoundError: If sbom_path doesn't exist
            ValueError: If SBOM is invalid
        """
        self.sbom_data: Optional[Dict] = None
        self.packages: Dict[str, Dict] = {}
        self.analyzer = BreakingChangeAnalyzer()
        
        if sbom_path:
            self.load_sbom(sbom_path)
    
    def load_sbom(self, sbom_path: str) -> None:
        """Load SBOM file.
        
        Args:
            sbom_path: Path to SPDX JSON SBOM file
            
        Raises:
            FileNotFoundError: If file doesn't exist
            ValueError: If file is not valid SPDX JSON
        """
        path = Path(sbom_path)
        if not path.exists():
            raise FileNotFoundError(f"SBOM file not found: {sbom_path}")
        
        try:
            with open(path, 'r', encoding='utf-8') as f:
                self.sbom_data = json.load(f)
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON in SBOM file: {e}")
        
        # Index packages by name
        for pkg in self.sbom_data.get("packages", []):
            name = pkg.get("name", "")
            self.packages[name] = pkg
    
    def recommend_upgrade(
        self,
        package_name: str,
        current_version: str,
        available_versions: Optional[List[str]] = None,
        changelog: Optional[str] = None
    ) -> UpgradeRecommendation:
        """Generate upgrade recommendation for a package.
        
        Args:
            package_name: Package name
            current_version: Current version
            available_versions: List of available versions (optional)
            changelog: Changelog text (optional)
            
        Returns:
            UpgradeRecommendation with analysis
            
        Raises:
            ValueError: If inputs are invalid
        """
        if not package_name:
            raise ValueError("Package name is required")
        if not current_version:
            raise ValueError("Current version is required")
        
        # If no versions provided, use heuristics
        if not available_versions:
            available_versions = self._generate_candidate_versions(current_version)
        
        # Sort versions
        sorted_versions = sorted(
            available_versions,
            key=lambda v: self.analyzer._parse_version(v) if v else (0,)
        )
        latest_version = sorted_versions[-1] if sorted_versions else current_version
        
        # Analyze breaking changes
        breaking_changes = []
        if changelog:
            changes = self.analyzer.analyze_changelog(changelog)
            breaking_changes = [c.description for c in changes]
        
        # Find safest upgrade path
        recommended = self._find_safest_upgrade(
            current_version,
            sorted_versions,
            breaking_changes
        )
        
        # Calculate compatibility score
        compatibility = self._calculate_compatibility(
            current_version,
            recommended,
            len(breaking_changes)
        )
        
        # Estimate effort
        effort = self._estimate_effort(len(breaking_changes))
        
        # Generate migration guide
        migration_guide = self._generate_migration_guide(
            package_name,
            current_version,
            recommended,
            breaking_changes
        )
        
        # Mock security fixes (would come from vulnerability data in real impl)
        security_fixes = []
        
        # Calculate confidence
        confidence = 0.8 if changelog else 0.5
        
        return UpgradeRecommendation(
            package=package_name,
            current_version=current_version,
            recommended_version=recommended,
            latest_version=latest_version,
            breaking_changes=breaking_changes[:5],  # Limit to top 5
            compatibility_score=compatibility,
            effort_estimate=effort,
            confidence=confidence,
            migration_guide=migration_guide,
            security_fixes=security_fixes
        )
    
    def _generate_candidate_versions(self, current: str) -> List[str]:
        """Generate candidate versions for upgrade.
        
        Args:
            current: Current version
            
        Returns:
            List of candidate versions
        """
        try:
            parts = self.analyzer._parse_version(current)
            candidates = [current]
            
            # Generate patch upgrades (1.0.0 -> 1.0.1, 1.0.2, ...)
            for i in range(1, 4):
                new_parts = list(parts)
                if len(new_parts) >= 3:
                    new_parts[2] += i
                    candidates.append('.'.join(map(str, new_parts)))
            
            # Generate minor upgrades (1.0.0 -> 1.1.0, 1.2.0)
            for i in range(1, 3):
                new_parts = list(parts)
                if len(new_parts) >= 2:
                    new_parts[1] += i
                    if len(new_parts) >= 3:
                        new_parts[2] = 0
                    candidates.append('.'.join(map(str, new_parts)))
            
            # Generate major upgrade (1.0.0 -> 2.0.0)
            new_parts = list(parts)
            new_parts[0] += 1
            for i in range(1, len(new_parts)):
                new_parts[i] = 0
            candidates.append('.'.join(map(str, new_parts)))
            
            return candidates
        except (ValueError, IndexError):
            return [current]
    
    def _find_safest_upgrade(
        self,
        current: str,
        versions: List[str],
        breaking_changes: List[str]
    ) -> str:
        """Find safest upgrade version.
        
        Args:
            current: Current version
            versions: Available versions
            breaking_changes: List of breaking changes
            
        Returns:
            Recommended version
        """
        if not versions or len(versions) == 1:
            return current
        
        try:
            current_parts = self.analyzer._parse_version(current)
            
            # Prefer patch upgrades (same major.minor)
            for v in versions:
                v_parts = self.analyzer._parse_version(v)
                if (len(v_parts) >= 2 and len(current_parts) >= 2 and
                    v_parts[0] == current_parts[0] and 
                    v_parts[1] == current_parts[1] and
                    v_parts > current_parts):
                    # Found newer patch version in same minor
                    return v
            
            # Next, try minor upgrade in same major
            for v in versions:
                v_parts = self.analyzer._parse_version(v)
                if (v_parts[0] == current_parts[0] and v_parts > current_parts):
                    return v
            
            # If no safe upgrade, return latest
            return versions[-1]
            
        except (ValueError, IndexError):
            return versions[-1] if versions else current
    
    def _calculate_compatibility(
        self,
        current: str,
        recommended: str,
        breaking_count: int
    ) -> float:
        """Calculate compatibility score (0-1).
        
        Args:
            current: Current version
            recommended: Recommended version
            breaking_count: Number of breaking changes
            
        Returns:
            Compatibility score
        """
        try:
            current_parts = self.analyzer._parse_version(current)
            rec_parts = self.analyzer._parse_version(recommended)
            
            # Start at 1.0 for patch updates
            score = 1.0
            
            # Penalty for major version change
            if rec_parts[0] > current_parts[0]:
                score -= 0.4
            
            # Penalty for minor version change
            if len(rec_parts) > 1 and len(current_parts) > 1:
                if rec_parts[1] > current_parts[1]:
                    score -= 0.2
            
            # Penalty for breaking changes
            score -= min(breaking_count * 0.05, 0.3)
            
            return max(0.0, min(1.0, score))
            
        except (ValueError, IndexError):
            return 0.5
    
    def _estimate_effort(self, breaking_count: int) -> str:
        """Estimate effort for upgrade.
        
        Args:
            breaking_count: Number of breaking changes
            
        Returns:
            Effort estimate (LOW, MEDIUM, HIGH)
        """
        if breaking_count == 0:
            return "LOW (1-2 hours)"
        elif breaking_count <= 3:
            return "MEDIUM (2-8 hours)"
        else:
            return "HIGH (1-3 days)"
    
    def _generate_migration_guide(
        self,
        package: str,
        from_v: str,
        to_v: str,
        breaking_changes: List[str]
    ) -> str:
        """Generate migration guide text.
        
        Args:
            package: Package name
            from_v: Source version
            to_v: Target version
            breaking_changes: List of breaking changes
            
        Returns:
            Migration guide markdown
        """
        guide = f"# Migration Guide: {package} {from_v} → {to_v}\n\n"
        guide += f"## Overview\n\n"
        guide += f"Upgrading from version {from_v} to {to_v}.\n\n"
        
        if breaking_changes:
            guide += f"## Breaking Changes ({len(breaking_changes)})\n\n"
            for i, change in enumerate(breaking_changes[:5], 1):
                guide += f"{i}. {change}\n"
            
            if len(breaking_changes) > 5:
                guide += f"\n... and {len(breaking_changes) - 5} more\n"
        else:
            guide += "## Breaking Changes\n\nNo breaking changes detected.\n"
        
        guide += "\n## Migration Steps\n\n"
        guide += "1. Update dependency version in build file\n"
        guide += "2. Run tests to identify compilation errors\n"
        guide += "3. Fix API changes as needed\n"
        guide += "4. Re-run full test suite\n"
        guide += "5. Deploy to staging for validation\n"
        
        guide += "\n## Rollback Plan\n\n"
        guide += f"If issues occur, revert to {from_v} and investigate failures.\n"
        
        return guide


def main():
    """Main entry point for upgrade recommender."""
    parser = argparse.ArgumentParser(
        description="BazBOM AI-Powered Upgrade Recommender",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Get upgrade recommendation
  bazel run //tools/supplychain:upgrade_recommender -- \\
    --package com.google.guava:guava \\
    --current 30.1-jre \\
    --versions 31.0-jre 31.1-jre 32.0-jre

  # With changelog analysis
  bazel run //tools/supplychain:upgrade_recommender -- \\
    --package com.google.guava:guava \\
    --current 30.1-jre \\
    --changelog changelog.txt

  # JSON output
  bazel run //tools/supplychain:upgrade_recommender -- \\
    --package com.google.guava:guava \\
    --current 30.1-jre \\
    --json
        """
    )
    
    parser.add_argument(
        "--package",
        required=True,
        help="Package name (e.g., com.google.guava:guava)"
    )
    
    parser.add_argument(
        "--current",
        required=True,
        help="Current version"
    )
    
    parser.add_argument(
        "--versions",
        nargs="+",
        help="Available versions to consider"
    )
    
    parser.add_argument(
        "--changelog",
        help="Path to changelog file"
    )
    
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output as JSON"
    )
    
    args = parser.parse_args()
    
    try:
        recommender = UpgradeRecommender()
        
        # Load changelog if provided
        changelog_text = None
        if args.changelog:
            with open(args.changelog, 'r', encoding='utf-8') as f:
                changelog_text = f.read()
        
        # Generate recommendation
        recommendation = recommender.recommend_upgrade(
            package_name=args.package,
            current_version=args.current,
            available_versions=args.versions,
            changelog=changelog_text
        )
        
        if args.json:
            # JSON output
            print(json.dumps(asdict(recommendation), indent=2))
        else:
            # Human-readable output
            print(f"🔍 Upgrade Analysis: {recommendation.package}")
            print("=" * 60)
            print(f"\nCurrent Version:     {recommendation.current_version}")
            print(f"✅ Recommended:      {recommendation.recommended_version}")
            print(f"⚠️  Latest Available:  {recommendation.latest_version}")
            print(f"\n📊 Compatibility Score: {recommendation.compatibility_score:.0%}")
            print(f"⏱️  Effort Estimate:    {recommendation.effort_estimate}")
            print(f"🎯 Confidence:         {recommendation.confidence:.0%}")
            
            if recommendation.breaking_changes:
                print(f"\n⚠️  Breaking Changes:")
                for change in recommendation.breaking_changes:
                    print(f"   - {change[:100]}")
            else:
                print(f"\n✅ No breaking changes detected")
            
            if recommendation.security_fixes:
                print(f"\n🔒 Security Fixes:")
                for fix in recommendation.security_fixes:
                    print(f"   - {fix}")
            
            print(f"\n📝 Migration Guide:")
            print(recommendation.migration_guide)
        
        return 0
        
    except FileNotFoundError as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        return 1
    except ValueError as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"❌ Unexpected error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
