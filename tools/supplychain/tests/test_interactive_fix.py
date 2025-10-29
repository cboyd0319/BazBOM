#!/usr/bin/env python3
"""Comprehensive tests for interactive_fix.py - Interactive dependency fix tool."""

import json
import sys
from pathlib import Path
from unittest.mock import MagicMock, Mock, patch, mock_open

import pytest

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from interactive_fix import InteractiveFixer


class TestInteractiveFixerInit:
    """Test InteractiveFixer initialization."""

    def test_init_with_valid_findings_and_project(self, tmp_path):
        """Test initialization with valid findings and project paths."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Assert
        assert fixer.findings_path == findings_file
        assert fixer.project_path == project_dir
        assert fixer.findings == {"vulnerabilities": []}
        assert fixer.build_system == "maven"
        assert fixer.fixes_applied == []

    def test_init_with_nonexistent_findings_raises_error(self, tmp_path):
        """Test initialization fails with nonexistent findings file."""
        # Arrange
        findings_file = tmp_path / "nonexistent.json"
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        
        # Act & Assert
        with pytest.raises(FileNotFoundError, match="Findings file not found"):
            InteractiveFixer(findings_file, project_dir)

    def test_init_with_invalid_json_raises_error(self, tmp_path):
        """Test initialization fails with invalid JSON in findings file."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text("{invalid json")
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        
        # Act & Assert
        with pytest.raises(ValueError, match="Invalid JSON"):
            InteractiveFixer(findings_file, project_dir)


class TestDetectBuildSystem:
    """Test build system detection."""

    def test_detect_maven_build_system(self, tmp_path):
        """Test detection of Maven build system."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Assert
        assert fixer.build_system == "maven"

    def test_detect_gradle_build_system(self, tmp_path):
        """Test detection of Gradle build system with build.gradle."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "build.gradle").write_text("plugins { }")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Assert
        assert fixer.build_system == "gradle"

    def test_detect_gradle_kotlin_build_system(self, tmp_path):
        """Test detection of Gradle build system with build.gradle.kts."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "build.gradle.kts").write_text("plugins { }")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Assert
        assert fixer.build_system == "gradle"

    def test_detect_bazel_workspace_build_system(self, tmp_path):
        """Test detection of Bazel build system with WORKSPACE."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "WORKSPACE").write_text("workspace(name = 'test')")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Assert
        assert fixer.build_system == "bazel"

    def test_detect_bazel_module_build_system(self, tmp_path):
        """Test detection of Bazel build system with MODULE.bazel."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "MODULE.bazel").write_text("module(name = 'test')")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Assert
        assert fixer.build_system == "bazel"

    def test_detect_build_system_fails_without_markers(self, tmp_path):
        """Test build system detection fails without marker files."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        # No build files
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="Could not detect build system"):
            InteractiveFixer(findings_file, project_dir)


class TestGetFixableVulnerabilities:
    """Test getting fixable vulnerabilities."""

    def test_get_fixable_vulnerabilities_empty_list(self, tmp_path):
        """Test getting fixable vulnerabilities with empty list."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        fixable = fixer.get_fixable_vulnerabilities()
        
        # Assert
        assert fixable == []

    def test_get_fixable_vulnerabilities_with_fixes(self, tmp_path):
        """Test getting fixable vulnerabilities with fixes available."""
        # Arrange
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-2023-0001",
                    "package": "log4j-core",
                    "severity": "CRITICAL",
                    "fixed_in": "2.17.0"
                },
                {
                    "id": "CVE-2023-0002",
                    "package": "guava",
                    "severity": "MEDIUM",
                    "fixed_in": "31.1"
                }
            ]
        }
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps(findings))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        fixable = fixer.get_fixable_vulnerabilities()
        
        # Assert
        assert len(fixable) == 2
        assert fixable[0]["id"] == "CVE-2023-0001"  # CRITICAL first
        assert fixable[1]["id"] == "CVE-2023-0002"

    def test_get_fixable_vulnerabilities_without_fixes(self, tmp_path):
        """Test getting fixable vulnerabilities filters out unfixable ones."""
        # Arrange
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-2023-0001",
                    "package": "log4j-core",
                    "severity": "CRITICAL"
                    # No fixed_in field
                }
            ]
        }
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps(findings))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        fixable = fixer.get_fixable_vulnerabilities()
        
        # Assert
        assert fixable == []

    def test_get_fixable_vulnerabilities_sorted_by_severity(self, tmp_path):
        """Test fixable vulnerabilities are sorted by severity."""
        # Arrange
        findings = {
            "vulnerabilities": [
                {"id": "CVE-1", "severity": "LOW", "fixed_in": "1.0"},
                {"id": "CVE-2", "severity": "CRITICAL", "fixed_in": "1.0"},
                {"id": "CVE-3", "severity": "MEDIUM", "fixed_in": "1.0"},
                {"id": "CVE-4", "severity": "HIGH", "fixed_in": "1.0"},
            ]
        }
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps(findings))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        fixable = fixer.get_fixable_vulnerabilities()
        
        # Assert
        assert len(fixable) == 4
        assert fixable[0]["severity"] == "CRITICAL"
        assert fixable[1]["severity"] == "HIGH"
        assert fixable[2]["severity"] == "MEDIUM"
        assert fixable[3]["severity"] == "LOW"


class TestAnalyzeFix:
    """Test fix analysis."""

    def test_analyze_fix_with_basic_vulnerability(self, tmp_path):
        """Test analyzing a fix for basic vulnerability."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        vulnerability = {
            "id": "CVE-2023-0001",
            "package": "log4j-core",
            "current_version": "2.14.1",
            "fixed_in": "2.17.0"
        }
        
        # Act
        analysis = fixer.analyze_fix(vulnerability)
        
        # Assert
        assert isinstance(analysis, dict)

    def test_analyze_fix_returns_dict(self, tmp_path):
        """Test analyze_fix returns a dictionary."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        vulnerability = {
            "id": "CVE-2023-0001",
            "package": "log4j-core"
        }
        
        # Act
        result = fixer.analyze_fix(vulnerability)
        
        # Assert
        assert isinstance(result, dict)


class TestEdgeCases:
    """Test edge cases and error conditions."""

    def test_findings_with_no_vulnerabilities_key(self, tmp_path):
        """Test handling findings without vulnerabilities key."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        fixable = fixer.get_fixable_vulnerabilities()
        
        # Assert
        assert fixable == []

    def test_unicode_in_vulnerability_data(self, tmp_path):
        """Test handling Unicode characters in vulnerability data."""
        # Arrange
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-2023-0001",
                    "package": "tëst-päckage",
                    "severity": "HIGH",
                    "fixed_in": "1.0.0",
                    "description": "Vulnérabilité avec caractères spéciaux "
                }
            ]
        }
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps(findings, ensure_ascii=False))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        fixable = fixer.get_fixable_vulnerabilities()
        
        # Assert
        assert len(fixable) == 1
        assert fixable[0]["package"] == "tëst-päckage"

    def test_very_large_findings_file(self, tmp_path):
        """Test handling a very large findings file."""
        # Arrange
        findings = {
            "vulnerabilities": [
                {
                    "id": f"CVE-2023-{i:04d}",
                    "package": f"package-{i}",
                    "severity": "HIGH",
                    "fixed_in": "1.0.0"
                }
                for i in range(1000)
            ]
        }
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps(findings))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        fixable = fixer.get_fixable_vulnerabilities()
        
        # Assert
        assert len(fixable) == 1000

    def test_empty_findings_file(self, tmp_path):
        """Test handling empty findings file."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text("{}")
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Assert
        assert fixer.findings == {}
        assert fixer.get_fixable_vulnerabilities() == []


class TestBuildSystemPriority:
    """Test build system detection priority when multiple files exist."""

    def test_maven_takes_priority_over_gradle(self, tmp_path):
        """Test Maven is detected when both Maven and Gradle files exist."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        (project_dir / "build.gradle").write_text("plugins { }")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Assert
        assert fixer.build_system == "maven"

    def test_gradle_takes_priority_over_bazel(self, tmp_path):
        """Test Gradle is detected when both Gradle and Bazel files exist."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "build.gradle").write_text("plugins { }")
        (project_dir / "WORKSPACE").write_text("workspace(name = 'test')")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Assert
        assert fixer.build_system == "gradle"


class TestFixesAppliedTracking:
    """Test tracking of applied fixes."""

    def test_fixes_applied_starts_empty(self, tmp_path):
        """Test fixes_applied list starts empty."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        
        # Act
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Assert
        assert fixer.fixes_applied == []
        assert isinstance(fixer.fixes_applied, list)


class TestGenerateMavenFix:
    """Test Maven fix generation."""

    def test_generate_maven_fix_basic(self, tmp_path):
        """Test generating Maven fix with basic package format."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        fix_analysis = {
            'package': 'org.apache.logging.log4j:log4j-core',
            'recommended_version': '2.17.0'
        }
        
        # Act
        result = fixer.generate_maven_fix(fix_analysis)
        
        # Assert
        assert '<groupId>org.apache.logging.log4j</groupId>' in result
        assert '<artifactId>log4j-core</artifactId>' in result
        assert '<version>2.17.0</version>' in result
        assert '<!-- BazBOM auto-generated fix -->' in result

    def test_generate_maven_fix_with_slash_separator(self, tmp_path):
        """Test generating Maven fix with slash-separated package format."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        fix_analysis = {
            'package': 'com.google.guava/guava',
            'recommended_version': '31.1-jre'
        }
        
        # Act
        result = fixer.generate_maven_fix(fix_analysis)
        
        # Assert
        assert '<groupId>com.google.guava</groupId>' in result
        assert '<artifactId>guava</artifactId>' in result
        assert '<version>31.1-jre</version>' in result

    def test_generate_maven_fix_with_invalid_package_format(self, tmp_path):
        """Test generating Maven fix with invalid package format."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        fix_analysis = {
            'package': 'single-part',
            'recommended_version': '1.0'
        }
        
        # Act
        result = fixer.generate_maven_fix(fix_analysis)
        
        # Assert
        assert '<version>1.0</version>' in result


class TestGenerateGradleFix:
    """Test Gradle fix generation."""

    def test_generate_gradle_fix_basic(self, tmp_path):
        """Test generating Gradle fix."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "build.gradle").write_text("plugins { }")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        fix_analysis = {
            'package': 'com.google.guava:guava',
            'recommended_version': '31.1-jre'
        }
        
        # Act
        result = fixer.generate_gradle_fix(fix_analysis)
        
        # Assert
        assert 'resolutionStrategy' in result
        assert 'force(' in result
        assert 'com.google.guava:guava:31.1-jre' in result
        assert '// BazBOM auto-generated fix' in result


class TestGenerateBazelFix:
    """Test Bazel fix generation."""

    def test_generate_bazel_fix_basic(self, tmp_path):
        """Test generating Bazel fix."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "WORKSPACE").write_text("workspace(name = 'test')")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        fix_analysis = {
            'package': 'com.google.guava:guava',
            'recommended_version': '31.1-jre'
        }
        
        # Act
        result = fixer.generate_bazel_fix(fix_analysis)
        
        # Assert
        assert 'override_targets' in result
        assert 'com.google.guava:guava' in result
        assert '31.1-jre' in result
        assert '# BazBOM auto-generated fix' in result


class TestCheckBreakingChanges:
    """Test breaking changes detection."""

    def test_check_breaking_changes_major_version_upgrade(self, tmp_path):
        """Test detecting major version upgrade."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        result = fixer._check_breaking_changes('1.0.0', '2.0.0')
        
        # Assert
        assert result is True

    def test_check_breaking_changes_minor_version_upgrade(self, tmp_path):
        """Test minor version upgrade not detected as breaking."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        result = fixer._check_breaking_changes('1.0.0', '1.1.0')
        
        # Assert
        assert result is False

    def test_check_breaking_changes_invalid_versions(self, tmp_path):
        """Test handling of invalid version strings."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        result = fixer._check_breaking_changes('invalid', 'also-invalid')
        
        # Assert
        assert result is False

    def test_check_breaking_changes_none_values(self, tmp_path):
        """Test handling of None values."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        result = fixer._check_breaking_changes(None, '2.0.0')
        
        # Assert
        assert result is False


class TestAnalyzeFixDetailed:
    """Test detailed fix analysis."""

    def test_analyze_fix_with_transitive_dependency(self, tmp_path):
        """Test analyzing fix for transitive dependency."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        vulnerability = {
            "package": "log4j-core",
            "version": "2.14.1",
            "fixed_in": ["2.17.0", "2.18.0"],
            "dependency_type": "transitive",
            "direct_parent": "spring-boot-starter"
        }
        
        # Act
        analysis = fixer.analyze_fix(vulnerability)
        
        # Assert
        assert analysis['package'] == 'log4j-core'
        assert analysis['current_version'] == '2.14.1'
        assert analysis['recommended_version'] == '2.17.0'
        assert analysis['is_transitive'] is True
        assert analysis['direct_parent'] == 'spring-boot-starter'

    def test_analyze_fix_with_multiple_fix_versions(self, tmp_path):
        """Test analyzing fix picks earliest version."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        vulnerability = {
            "package": "guava",
            "version": "20.0",
            "fixed_in": ["31.1-jre", "30.0-jre", "32.0-jre"]
        }
        
        # Act
        analysis = fixer.analyze_fix(vulnerability)
        
        # Assert
        assert analysis['recommended_version'] == '30.0-jre'

    def test_analyze_fix_with_breaking_changes(self, tmp_path):
        """Test analyzing fix detects breaking changes."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        vulnerability = {
            "package": "junit",
            "version": "4.13.2",
            "fixed_in": ["5.9.0"]
        }
        
        # Act
        analysis = fixer.analyze_fix(vulnerability)
        
        # Assert
        assert analysis['breaking_changes_likely'] is True


class TestApplyMavenFixes:
    """Test applying Maven fixes."""

    @patch('builtins.input', return_value='n')
    def test_apply_maven_fixes_without_dependency_management(self, mock_input, tmp_path):
        """Test applying Maven fixes when no dependencyManagement exists."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        pom_file = project_dir / "pom.xml"
        pom_file.write_text("<project>\n</project>")
        
        fixer = InteractiveFixer(findings_file, project_dir)
        fixer.fixes_applied = [{
            'fix': '<dependency>\n  <groupId>test</groupId>\n  <artifactId>test</artifactId>\n  <version>1.0</version>\n</dependency>'
        }]
        
        # Act
        fixer._apply_maven_fixes()
        
        # Assert
        content = pom_file.read_text()
        assert '<dependencyManagement>' in content
        assert '<groupId>test</groupId>' in content

    @patch('builtins.input', return_value='n')
    def test_apply_maven_fixes_with_existing_dependency_management(self, mock_input, tmp_path):
        """Test applying Maven fixes with existing dependencyManagement."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        pom_file = project_dir / "pom.xml"
        pom_file.write_text("""<project>
  <dependencyManagement>
    <dependencies>
    </dependencies>
  </dependencyManagement>
</project>""")
        
        fixer = InteractiveFixer(findings_file, project_dir)
        fixer.fixes_applied = [{
            'fix': '<dependency>\n  <groupId>test</groupId>\n</dependency>'
        }]
        
        # Act
        fixer._apply_maven_fixes()
        
        # Assert
        content = pom_file.read_text()
        assert '<groupId>test</groupId>' in content


class TestApplyGradleFixes:
    """Test applying Gradle fixes."""

    @patch('builtins.input', return_value='n')
    def test_apply_gradle_fixes_to_build_gradle(self, mock_input, tmp_path):
        """Test applying Gradle fixes to build.gradle."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        build_file = project_dir / "build.gradle"
        build_file.write_text("plugins { }")
        
        fixer = InteractiveFixer(findings_file, project_dir)
        fixer.fixes_applied = [{
            'fix': 'configurations.all {\n  resolutionStrategy {\n    force("test:test:1.0")\n  }\n}'
        }]
        
        # Act
        fixer._apply_gradle_fixes()
        
        # Assert
        content = build_file.read_text()
        assert 'force("test:test:1.0")' in content

    @patch('builtins.input', return_value='n')
    def test_apply_gradle_fixes_to_build_gradle_kts(self, mock_input, tmp_path):
        """Test applying Gradle fixes to build.gradle.kts."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        build_file = project_dir / "build.gradle.kts"
        build_file.write_text("plugins { }")
        
        fixer = InteractiveFixer(findings_file, project_dir)
        fixer.fixes_applied = [{
            'fix': 'force("test:test:1.0")'
        }]
        
        # Act
        fixer._apply_gradle_fixes()
        
        # Assert
        content = build_file.read_text()
        assert 'force("test:test:1.0")' in content


class TestApplyBazelFixes:
    """Test applying Bazel fixes."""

    @patch('builtins.input', return_value='n')
    def test_apply_bazel_fixes(self, mock_input, tmp_path):
        """Test applying Bazel fixes to WORKSPACE."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        workspace_file = project_dir / "WORKSPACE"
        workspace_file.write_text("workspace(name = 'test')")
        
        fixer = InteractiveFixer(findings_file, project_dir)
        fixer.fixes_applied = [{
            'fix': 'override_targets = {\n  "test:test": "@maven//:test"\n}'
        }]
        
        # Act
        fixer._apply_bazel_fixes()
        
        # Assert
        content = workspace_file.read_text()
        assert '# BazBOM fixes' in content
        assert 'override_targets' in content


class TestMainFunction:
    """Test main CLI function."""

    @patch('sys.argv', ['bazbom-fix', '--findings', '/tmp/findings.json', '--project', '/tmp/project'])
    @patch('interactive_fix.InteractiveFixer')
    def test_main_success(self, mock_fixer_class):
        """Test main function successful execution."""
        # Arrange
        mock_fixer = MagicMock()
        mock_fixer.run_interactive.return_value = 0
        mock_fixer_class.return_value = mock_fixer
        from interactive_fix import main
        
        # Act
        result = main()
        
        # Assert
        assert result == 0
        mock_fixer.run_interactive.assert_called_once()

    @patch('sys.argv', ['bazbom-fix', '--findings', '/nonexistent.json'])
    @patch('interactive_fix.InteractiveFixer')
    def test_main_file_not_found(self, mock_fixer_class):
        """Test main function with nonexistent findings file."""
        # Arrange
        mock_fixer_class.side_effect = FileNotFoundError("File not found")
        from interactive_fix import main
        
        # Act
        result = main()
        
        # Assert
        assert result == 1

    @patch('sys.argv', ['bazbom-fix', '--findings', '/tmp/invalid.json'])
    @patch('interactive_fix.InteractiveFixer')
    def test_main_value_error(self, mock_fixer_class):
        """Test main function with invalid JSON."""
        # Arrange
        mock_fixer_class.side_effect = ValueError("Invalid JSON")
        from interactive_fix import main
        
        # Act
        result = main()
        
        # Assert
        assert result == 1

    @patch('sys.argv', ['bazbom-fix', '--findings', '/tmp/findings.json'])
    @patch('interactive_fix.InteractiveFixer')
    def test_main_runtime_error(self, mock_fixer_class):
        """Test main function with runtime error."""
        # Arrange
        mock_fixer_class.side_effect = RuntimeError("Build system error")
        from interactive_fix import main
        
        # Act
        result = main()
        
        # Assert
        assert result == 1


class TestRunInteractive:
    """Test interactive run session."""

    @patch('builtins.input', return_value='n')
    @patch('builtins.print')
    def test_run_interactive_no_fixable_vulnerabilities(self, mock_print, mock_input, tmp_path):
        """Test interactive run with no fixable vulnerabilities."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        result = fixer.run_interactive()
        
        # Assert
        assert result == 0

    @patch('builtins.input', return_value='skip all')
    @patch('builtins.print')
    def test_run_interactive_skip_all(self, mock_print, mock_input, tmp_path):
        """Test interactive run with skip all."""
        # Arrange
        findings = {
            "vulnerabilities": [
                {"id": "CVE-1", "package": "test", "severity": "HIGH", "fixed_in": ["1.0"]},
                {"id": "CVE-2", "package": "test2", "severity": "HIGH", "fixed_in": ["2.0"]}
            ]
        }
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps(findings))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project></project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        result = fixer.run_interactive()
        
        # Assert
        assert result == 0
        assert len(fixer.fixes_applied) == 0

    @patch('builtins.input', side_effect=['y', 'n'])
    @patch('builtins.print')
    def test_run_interactive_apply_one_fix(self, mock_print, mock_input, tmp_path):
        """Test interactive run applying one fix."""
        # Arrange
        findings = {
            "vulnerabilities": [
                {"id": "CVE-1", "package": "test:test", "version": "1.0", "severity": "HIGH", "fixed_in": ["2.0"]},
                {"id": "CVE-2", "package": "test2:test2", "version": "1.0", "severity": "MEDIUM", "fixed_in": ["2.0"]}
            ]
        }
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps(findings))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "pom.xml").write_text("<project>\n</project>")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        # Act
        result = fixer.run_interactive()
        
        # Assert
        assert result == 0
        assert len(fixer.fixes_applied) == 1


class TestGenerateGradleFix:
    """Test Gradle fix generation."""

    def test_generate_gradle_fix_basic(self, tmp_path):
        """Test generating Gradle fix."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "build.gradle").write_text("plugins { }")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        fix_analysis = {
            'package': 'org.apache.logging.log4j:log4j-core',
            'recommended_version': '2.17.0'
        }
        
        # Act
        result = fixer.generate_gradle_fix(fix_analysis)
        
        # Assert
        assert '// BazBOM auto-generated fix' in result
        assert 'configurations.all' in result
        assert 'resolutionStrategy' in result
        assert "force('org.apache.logging.log4j:log4j-core:2.17.0')" in result


class TestGenerateBazelFix:
    """Test Bazel fix generation."""

    def test_generate_bazel_fix_basic(self, tmp_path):
        """Test generating Bazel fix."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "WORKSPACE").write_text("workspace(name = 'test')")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        fix_analysis = {
            'package': 'org.apache.logging.log4j:log4j-core',
            'recommended_version': '2.17.0'
        }
        
        # Act
        result = fixer.generate_bazel_fix(fix_analysis)
        
        # Assert
        assert '# BazBOM auto-generated fix' in result
        assert 'override_targets' in result
        assert 'org.apache.logging.log4j:log4j-core' in result
        assert '2.17.0' in result

    def test_generate_bazel_fix_creates_valid_target_name(self, tmp_path):
        """Test Bazel fix creates valid target name."""
        # Arrange
        findings_file = tmp_path / "findings.json"
        findings_file.write_text(json.dumps({"vulnerabilities": []}))
        project_dir = tmp_path / "project"
        project_dir.mkdir()
        (project_dir / "WORKSPACE").write_text("workspace(name = 'test')")
        fixer = InteractiveFixer(findings_file, project_dir)
        
        fix_analysis = {
            'package': 'com.google.guava:guava',
            'recommended_version': '31.1-jre'
        }
        
        # Act
        result = fixer.generate_bazel_fix(fix_analysis)
        
        # Assert
        # Target name should replace dots and hyphens with underscores
        assert 'com_google_guava_guava' in result
