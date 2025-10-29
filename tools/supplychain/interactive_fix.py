#!/usr/bin/env python3
"""Interactive Dependency Fix Tool for BazBOM.

This tool provides an interactive interface for fixing vulnerable dependencies,
generating override configurations for Maven, Gradle, and Bazel projects.
Based on roadmap Section #9: Transitive Dependency Override Recommendations.
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple


class InteractiveFixer:
    """Interactive tool for fixing vulnerable dependencies."""
    
    def __init__(self, findings_path: Path, project_path: Path):
        """Initialize interactive fixer.
        
        Args:
            findings_path: Path to SCA findings JSON file
            project_path: Path to project directory
        """
        self.findings_path = findings_path
        self.project_path = project_path
        self.findings = self._load_findings()
        self.build_system = self._detect_build_system()
        self.fixes_applied = []
    
    def _load_findings(self) -> Dict:
        """Load vulnerability findings.
        
        Returns:
            Findings dictionary
            
        Raises:
            FileNotFoundError: If findings file doesn't exist
            ValueError: If findings JSON is invalid
        """
        if not self.findings_path.exists():
            raise FileNotFoundError(f"Findings file not found: {self.findings_path}")
        
        try:
            with open(self.findings_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON in findings file: {str(e)}")
    
    def _detect_build_system(self) -> str:
        """Detect project build system.
        
        Returns:
            Build system name (maven, gradle, bazel)
            
        Raises:
            RuntimeError: If build system cannot be detected
        """
        if (self.project_path / "pom.xml").exists():
            return "maven"
        elif ((self.project_path / "build.gradle").exists() or 
              (self.project_path / "build.gradle.kts").exists()):
            return "gradle"
        elif ((self.project_path / "WORKSPACE").exists() or 
              (self.project_path / "MODULE.bazel").exists()):
            return "bazel"
        
        raise RuntimeError(
            "Could not detect build system. "
            "Expected pom.xml, build.gradle, or WORKSPACE file."
        )
    
    def get_fixable_vulnerabilities(self) -> List[Dict]:
        """Get vulnerabilities that can be fixed by upgrading.
        
        Returns:
            List of fixable vulnerabilities
        """
        fixable = []
        
        for vuln in self.findings.get('vulnerabilities', []):
            # Check if fix is available
            if vuln.get('fixed_in'):
                fixable.append(vuln)
        
        # Sort by severity (CRITICAL first)
        severity_order = {'CRITICAL': 0, 'HIGH': 1, 'MEDIUM': 2, 'LOW': 3}
        fixable.sort(key=lambda v: severity_order.get(v.get('severity', 'LOW'), 99))
        
        return fixable
    
    def analyze_fix(self, vulnerability: Dict) -> Dict:
        """Analyze what's needed to fix a vulnerability.
        
        Args:
            vulnerability: Vulnerability dictionary
            
        Returns:
            Fix analysis
        """
        package = vulnerability.get('package', '')
        current_version = vulnerability.get('version', '')
        fixed_in = vulnerability.get('fixed_in', [])
        
        # Get recommended fix version (lowest version that fixes the issue)
        recommended_version = None
        if fixed_in:
            # Sort versions and pick the earliest fix
            recommended_version = sorted(fixed_in)[0] if fixed_in else None
        
        # Determine if this is a direct or transitive dependency
        is_transitive = vulnerability.get('dependency_type') == 'transitive'
        
        return {
            'package': package,
            'current_version': current_version,
            'recommended_version': recommended_version,
            'is_transitive': is_transitive,
            'direct_parent': vulnerability.get('direct_parent'),
            'breaking_changes_likely': self._check_breaking_changes(
                current_version, recommended_version
            ),
        }
    
    def _check_breaking_changes(
        self, 
        current: str, 
        recommended: str
    ) -> bool:
        """Check if upgrade likely has breaking changes.
        
        Args:
            current: Current version
            recommended: Recommended version
            
        Returns:
            True if major version change detected
        """
        if not current or not recommended:
            return False
        
        # Parse major version (simple heuristic)
        try:
            current_major = int(current.split('.')[0])
            recommended_major = int(recommended.split('.')[0])
            return recommended_major > current_major
        except (ValueError, IndexError):
            return False
    
    def generate_maven_fix(self, fix_analysis: Dict) -> str:
        """Generate Maven dependency management override.
        
        Args:
            fix_analysis: Fix analysis from analyze_fix()
            
        Returns:
            Maven XML snippet
        """
        package = fix_analysis['package']
        version = fix_analysis['recommended_version']
        
        # Parse Maven coordinates (group:artifact format)
        parts = package.split(':')
        if len(parts) != 2:
            parts = package.split('/')
        
        group_id = parts[0] if len(parts) > 0 else 'unknown'
        artifact_id = parts[1] if len(parts) > 1 else package
        
        return f"""<!-- BazBOM auto-generated fix -->
<dependency>
  <groupId>{group_id}</groupId>
  <artifactId>{artifact_id}</artifactId>
  <version>{version}</version>
</dependency>"""
    
    def generate_gradle_fix(self, fix_analysis: Dict) -> str:
        """Generate Gradle dependency override.
        
        Args:
            fix_analysis: Fix analysis from analyze_fix()
            
        Returns:
            Gradle configuration snippet
        """
        package = fix_analysis['package']
        version = fix_analysis['recommended_version']
        
        return f"""// BazBOM auto-generated fix
configurations.all {{
    resolutionStrategy {{
        force('{package}:{version}')
    }}
}}"""
    
    def generate_bazel_fix(self, fix_analysis: Dict) -> str:
        """Generate Bazel override configuration.
        
        Args:
            fix_analysis: Fix analysis from analyze_fix()
            
        Returns:
            Bazel override snippet
        """
        package = fix_analysis['package']
        version = fix_analysis['recommended_version']
        
        # Parse Maven coordinates
        parts = package.split(':')
        group_id = parts[0] if len(parts) > 0 else 'unknown'
        artifact_id = parts[1] if len(parts) > 1 else package
        
        # Create Bazel target name
        target_name = f"{group_id.replace('.', '_')}_{artifact_id.replace('-', '_')}"
        
        return f"""# BazBOM auto-generated fix
# Add to maven_install() in WORKSPACE:
override_targets = {{
    "{group_id}:{artifact_id}": "@maven//:{target_name}",
}},
# Update maven_install.json with version {version}"""
    
    def run_interactive(self) -> int:
        """Run interactive fix session.
        
        Returns:
            Exit code (0 for success)
        """
        fixable = self.get_fixable_vulnerabilities()
        
        if not fixable:
            print("[OK] No fixable vulnerabilities found!")
            return 0
        
        print(f"\n Found {len(fixable)} fixable vulnerabilities\n")
        print(f"Build system: {self.build_system}")
        print("=" * 60)
        
        for idx, vuln in enumerate(fixable, 1):
            print(f"\nFix {idx}/{len(fixable)}")
            print("-" * 60)
            
            # Analyze fix
            fix_analysis = self.analyze_fix(vuln)
            
            # Display vulnerability info
            print(f"Package: {fix_analysis['package']}")
            print(f"Current version: {fix_analysis['current_version']}")
            print(f"Recommended version: {fix_analysis['recommended_version']}")
            print(f"Severity: {vuln.get('severity', 'UNKNOWN')}")
            print(f"CVE: {vuln.get('cve_id', 'N/A')}")
            
            if fix_analysis['is_transitive']:
                print(f"[WARNING]  Transitive dependency (via {fix_analysis.get('direct_parent', 'unknown')})")
            
            if fix_analysis['breaking_changes_likely']:
                print("[WARNING]  WARNING: Major version change detected - may have breaking changes")
            
            # Generate fix
            print("\nGenerated fix:")
            print()
            
            if self.build_system == "maven":
                fix_code = self.generate_maven_fix(fix_analysis)
            elif self.build_system == "gradle":
                fix_code = self.generate_gradle_fix(fix_analysis)
            else:  # bazel
                fix_code = self.generate_bazel_fix(fix_analysis)
            
            print(fix_code)
            print()
            
            # Interactive prompt
            response = input("Apply fix? [y/N/skip all] ").strip().lower()
            
            if response == 'skip all':
                print("\n⏩ Skipping remaining fixes")
                break
            elif response == 'y':
                self.fixes_applied.append({
                    'vulnerability': vuln,
                    'fix': fix_code,
                    'analysis': fix_analysis,
                })
                print("[OK] Fix queued for application")
            else:
                print("⏭  Skipped")
        
        # Apply fixes
        if self.fixes_applied:
            print("\n" + "=" * 60)
            print(f"Applying {len(self.fixes_applied)} fixes...")
            print("=" * 60)
            self._apply_fixes()
            
            print(f"\n[OK] Successfully applied {len(self.fixes_applied)} fixes")
            print(f"\nNext steps:")
            if self.build_system == "maven":
                print("  1. Review changes in pom.xml")
                print("  2. Run: mvn clean install")
                print("  3. Run tests to verify compatibility")
            elif self.build_system == "gradle":
                print("  1. Review changes in build.gradle")
                print("  2. Run: ./gradlew build")
                print("  3. Run tests to verify compatibility")
            else:  # bazel
                print("  1. Review changes in WORKSPACE")
                print("  2. Update maven_install.json")
                print("  3. Run: bazel build //...")
        else:
            print("\n⏭  No fixes applied")
        
        return 0
    
    def _apply_fixes(self):
        """Apply queued fixes to build files."""
        if self.build_system == "maven":
            self._apply_maven_fixes()
        elif self.build_system == "gradle":
            self._apply_gradle_fixes()
        else:  # bazel
            self._apply_bazel_fixes()
    
    def _apply_maven_fixes(self):
        """Apply fixes to Maven pom.xml."""
        pom_path = self.project_path / "pom.xml"
        
        # Read existing pom.xml
        with open(pom_path, 'r', encoding='utf-8') as f:
            pom_content = f.read()
        
        # Find or create dependencyManagement section
        if '<dependencyManagement>' not in pom_content:
            # Add before </project>
            insert_point = pom_content.rfind('</project>')
            if insert_point > 0:
                fixes_xml = "\n  <dependencyManagement>\n    <dependencies>\n"
                for fix in self.fixes_applied:
                    fixes_xml += "      " + fix['fix'].replace('\n', '\n      ') + "\n"
                fixes_xml += "    </dependencies>\n  </dependencyManagement>\n"
                
                pom_content = (
                    pom_content[:insert_point] +
                    fixes_xml +
                    pom_content[insert_point:]
                )
        else:
            # Add to existing dependencyManagement
            insert_point = pom_content.find('</dependencies>', 
                                           pom_content.find('<dependencyManagement>'))
            if insert_point > 0:
                fixes_xml = ""
                for fix in self.fixes_applied:
                    fixes_xml += "      " + fix['fix'].replace('\n', '\n      ') + "\n"
                
                pom_content = (
                    pom_content[:insert_point] +
                    fixes_xml +
                    pom_content[insert_point:]
                )
        
        # Write updated pom.xml
        with open(pom_path, 'w', encoding='utf-8') as f:
            f.write(pom_content)
        
        print(f" Updated {pom_path}")
    
    def _apply_gradle_fixes(self):
        """Apply fixes to Gradle build file."""
        # Try build.gradle.kts first, then build.gradle
        gradle_path = self.project_path / "build.gradle.kts"
        if not gradle_path.exists():
            gradle_path = self.project_path / "build.gradle"
        
        # Read existing build file
        with open(gradle_path, 'r', encoding='utf-8') as f:
            gradle_content = f.read()
        
        # Append fixes at the end
        fixes_gradle = "\n// BazBOM fixes\n"
        for fix in self.fixes_applied:
            fixes_gradle += fix['fix'] + "\n\n"
        
        gradle_content += fixes_gradle
        
        # Write updated build file
        with open(gradle_path, 'w', encoding='utf-8') as f:
            f.write(gradle_content)
        
        print(f" Updated {gradle_path}")
    
    def _apply_bazel_fixes(self):
        """Apply fixes to Bazel WORKSPACE."""
        workspace_path = self.project_path / "WORKSPACE"
        
        # Read existing WORKSPACE
        with open(workspace_path, 'r', encoding='utf-8') as f:
            workspace_content = f.read()
        
        # Append fixes as comments (requires manual maven_install.json update)
        fixes_bazel = "\n# BazBOM fixes (manual action required)\n"
        for fix in self.fixes_applied:
            fixes_bazel += "# " + fix['fix'].replace('\n', '\n# ') + "\n\n"
        
        workspace_content += fixes_bazel
        
        # Write updated WORKSPACE
        with open(workspace_path, 'w', encoding='utf-8') as f:
            f.write(workspace_content)
        
        print(f" Updated {workspace_path} (with instructions)")


def main():
    """Main entry point for interactive fix tool."""
    parser = argparse.ArgumentParser(
        prog='bazbom-fix',
        description='Interactive tool for fixing vulnerable dependencies',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Interactive fix mode
  bazbom-fix --findings sca_findings.json
  
  # Specify project directory
  bazbom-fix --findings sca_findings.json --project /path/to/project
        """
    )
    
    parser.add_argument(
        '--findings',
        type=Path,
        required=True,
        help='Path to SCA findings JSON file'
    )
    
    parser.add_argument(
        '--project',
        type=Path,
        default=Path('.'),
        help='Project directory (default: current directory)'
    )
    
    args = parser.parse_args()
    
    try:
        fixer = InteractiveFixer(args.findings, args.project)
        return fixer.run_interactive()
    except (FileNotFoundError, ValueError, RuntimeError) as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
