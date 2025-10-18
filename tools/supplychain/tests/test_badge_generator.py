#!/usr/bin/env python3
"""Tests for badge_generator.py - Security badge generation functionality."""

import json
import sys
from pathlib import Path

import pytest

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from badge_generator import (
    calculate_badge_data,
    generate_shields_json,
    generate_markdown_badge,
    generate_html_badge,
)


class TestCalculateBadgeData:
    """Test badge data calculation from vulnerability findings."""
    
    def test_no_vulnerabilities(self):
        """Test badge data when no vulnerabilities are present."""
        findings = {"vulnerabilities": []}
        
        result = calculate_badge_data(findings)
        
        assert result["schemaVersion"] == 1
        assert result["label"] == "security"
        assert result["message"] == "no known vulnerabilities"
        assert result["color"] == "success"
        assert result["namedLogo"] == "github"
    
    def test_critical_vulnerability(self):
        """Test badge data with critical severity vulnerability."""
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-2021-44228",
                    "database_specific": {
                        "severity": [
                            {"type": "CVSS_V3", "severity": "CRITICAL"}
                        ]
                    }
                }
            ]
        }
        
        result = calculate_badge_data(findings)
        
        assert result["message"] == "1 critical"
        assert result["color"] == "critical"
    
    def test_high_vulnerability(self):
        """Test badge data with high severity vulnerability."""
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-2023-12345",
                    "database_specific": {
                        "severity": [
                            {"type": "CVSS_V3", "severity": "HIGH"}
                        ]
                    }
                }
            ]
        }
        
        result = calculate_badge_data(findings)
        
        assert result["message"] == "1 high"
        assert result["color"] == "important"
    
    def test_medium_vulnerability(self):
        """Test badge data with medium severity vulnerability."""
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-2023-12345",
                    "database_specific": {
                        "severity": [
                            {"type": "CVSS_V3", "severity": "MEDIUM"}
                        ]
                    }
                }
            ]
        }
        
        result = calculate_badge_data(findings)
        
        assert result["message"] == "1 medium"
        assert result["color"] == "yellow"
    
    def test_low_vulnerability(self):
        """Test badge data with low severity vulnerability."""
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-2023-12345",
                    "database_specific": {
                        "severity": [
                            {"type": "CVSS_V3", "severity": "LOW"}
                        ]
                    }
                }
            ]
        }
        
        result = calculate_badge_data(findings)
        
        assert result["message"] == "1 low"
        assert result["color"] == "informational"
    
    def test_multiple_vulnerabilities_prioritizes_highest(self):
        """Test that highest severity is shown when multiple vulnerabilities exist."""
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-1",
                    "database_specific": {
                        "severity": [{"type": "CVSS_V3", "severity": "LOW"}]
                    }
                },
                {
                    "id": "CVE-2",
                    "database_specific": {
                        "severity": [{"type": "CVSS_V3", "severity": "HIGH"}]
                    }
                },
                {
                    "id": "CVE-3",
                    "database_specific": {
                        "severity": [{"type": "CVSS_V3", "severity": "MEDIUM"}]
                    }
                },
            ]
        }
        
        result = calculate_badge_data(findings)
        
        assert result["message"] == "1 high"
        assert result["color"] == "important"
    
    def test_multiple_critical_vulnerabilities(self):
        """Test count when multiple critical vulnerabilities exist."""
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-1",
                    "database_specific": {
                        "severity": [{"type": "CVSS_V3", "severity": "CRITICAL"}]
                    }
                },
                {
                    "id": "CVE-2",
                    "database_specific": {
                        "severity": [{"type": "CVSS_V3", "severity": "CRITICAL"}]
                    }
                },
            ]
        }
        
        result = calculate_badge_data(findings)
        
        assert result["message"] == "2 critical"
        assert result["color"] == "critical"
    
    def test_vulnerability_without_severity(self):
        """Test handling vulnerability without severity information."""
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-2023-12345",
                    "database_specific": {}
                }
            ]
        }
        
        result = calculate_badge_data(findings)
        
        # Should be treated as no known vulnerabilities since severity is unknown
        assert result["color"] == "success"
    
    def test_vulnerability_with_non_cvss_severity(self):
        """Test handling vulnerability with non-CVSS severity."""
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-2023-12345",
                    "database_specific": {
                        "severity": [
                            {"type": "CUSTOM", "severity": "HIGH"}
                        ]
                    }
                }
            ]
        }
        
        result = calculate_badge_data(findings)
        
        # Should be treated as no known vulnerabilities since CVSS_V3 not found
        assert result["color"] == "success"
    
    def test_license_data_with_copyleft(self):
        """Test badge data includes label color for copyleft licenses."""
        findings = {"vulnerabilities": []}
        license_data = {
            "packages": [
                {"name": "pkg1", "license": "MIT", "is_copyleft": False},
                {"name": "pkg2", "license": "GPL-3.0", "is_copyleft": True},
            ]
        }
        
        result = calculate_badge_data(findings, license_data)
        
        assert result["labelColor"] == "orange"
    
    def test_license_data_without_copyleft(self):
        """Test badge data without copyleft licenses."""
        findings = {"vulnerabilities": []}
        license_data = {
            "packages": [
                {"name": "pkg1", "license": "MIT", "is_copyleft": False},
                {"name": "pkg2", "license": "Apache-2.0", "is_copyleft": False},
            ]
        }
        
        result = calculate_badge_data(findings, license_data)
        
        assert "labelColor" not in result
    
    def test_missing_vulnerabilities_key_raises_error(self):
        """Test that missing vulnerabilities key raises ValueError."""
        findings = {"something_else": []}
        
        with pytest.raises(ValueError, match="missing 'vulnerabilities' array"):
            calculate_badge_data(findings)
    
    def test_vulnerabilities_not_list_raises_error(self):
        """Test that non-list vulnerabilities raises ValueError."""
        findings = {"vulnerabilities": "not a list"}
        
        with pytest.raises(ValueError, match="missing 'vulnerabilities' array"):
            calculate_badge_data(findings)
    
    @pytest.mark.parametrize("severity,expected_color,expected_message", [
        ("CRITICAL", "critical", "critical"),
        ("HIGH", "important", "high"),
        ("MEDIUM", "yellow", "medium"),
        ("LOW", "informational", "low"),
    ], ids=["critical", "high", "medium", "low"])
    def test_severity_levels_parametrized(self, severity, expected_color, expected_message):
        """Test all severity levels with parametrized test."""
        findings = {
            "vulnerabilities": [
                {
                    "id": "CVE-TEST",
                    "database_specific": {
                        "severity": [{"type": "CVSS_V3", "severity": severity}]
                    }
                }
            ]
        }
        
        result = calculate_badge_data(findings)
        
        assert result["color"] == expected_color
        assert expected_message in result["message"]


class TestGenerateShieldsJson:
    """Test shields.io JSON generation."""
    
    def test_generate_shields_json(self):
        """Test generating shields.io compatible JSON."""
        findings = {"vulnerabilities": []}
        
        result = generate_shields_json(findings)
        
        # Should be valid JSON
        parsed = json.loads(result)
        assert parsed["schemaVersion"] == 1
        assert parsed["message"] == "no known vulnerabilities"
    
    def test_generate_shields_json_with_license(self):
        """Test generating shields.io JSON with license data."""
        findings = {"vulnerabilities": []}
        license_data = {
            "packages": [
                {"name": "pkg", "license": "GPL-3.0", "is_copyleft": True}
            ]
        }
        
        result = generate_shields_json(findings, license_data)
        
        parsed = json.loads(result)
        assert parsed["labelColor"] == "orange"
    
    def test_json_is_formatted(self):
        """Test that generated JSON is formatted with indentation."""
        findings = {"vulnerabilities": []}
        
        result = generate_shields_json(findings)
        
        # Should have newlines (formatted)
        assert "\n" in result


class TestGenerateMarkdownBadge:
    """Test Markdown badge snippet generation."""
    
    def test_generate_markdown_badge(self):
        """Test generating Markdown badge snippet."""
        badge_url = "https://example.com/badge.json"
        
        result = generate_markdown_badge(badge_url)
        
        assert result.startswith("![Security Status]")
        assert "https://img.shields.io/endpoint?url=" in result
        assert badge_url in result
    
    def test_generate_markdown_badge_custom_alt_text(self):
        """Test generating Markdown badge with custom alt text."""
        badge_url = "https://example.com/badge.json"
        alt_text = "Custom Badge"
        
        result = generate_markdown_badge(badge_url, alt_text)
        
        assert result.startswith(f"![{alt_text}]")
    
    def test_markdown_badge_format(self):
        """Test Markdown badge has correct format."""
        badge_url = "https://example.com/badge.json"
        
        result = generate_markdown_badge(badge_url)
        
        # Should match Markdown image format
        assert result.startswith("![")
        assert "](" in result
        assert result.endswith(")")


class TestGenerateHtmlBadge:
    """Test HTML badge snippet generation."""
    
    def test_generate_html_badge(self):
        """Test generating HTML badge snippet."""
        badge_url = "https://example.com/badge.json"
        
        result = generate_html_badge(badge_url)
        
        assert result.startswith('<img src="')
        assert "https://img.shields.io/endpoint?url=" in result
        assert badge_url in result
        assert 'alt="Security Status"' in result
    
    def test_generate_html_badge_custom_alt_text(self):
        """Test generating HTML badge with custom alt text."""
        badge_url = "https://example.com/badge.json"
        alt_text = "Custom Badge"
        
        result = generate_html_badge(badge_url, alt_text)
        
        assert f'alt="{alt_text}"' in result
    
    def test_html_badge_format(self):
        """Test HTML badge has correct format."""
        badge_url = "https://example.com/badge.json"
        
        result = generate_html_badge(badge_url)
        
        # Should be valid HTML img tag
        assert result.startswith("<img")
        assert result.endswith("/>")
        assert "src=" in result
        assert "alt=" in result
