#!/usr/bin/env python3
"""Tests for sarif_adapter.py - SARIF generation functionality."""

import json
import sys
from pathlib import Path
import pytest

# Add parent directory to path to import sarif_adapter
sys.path.insert(0, str(Path(__file__).parent.parent))

from sarif_adapter import (
    priority_to_level,
    severity_to_level,
    format_enriched_message,
    create_sarif_document,
    main,
)


class TestPriorityToLevel:
    """Test priority to SARIF level conversion."""

    @pytest.mark.parametrize("priority,severity,expected", [
        ("P0-IMMEDIATE", "MEDIUM", "error"),
        ("P1-CRITICAL", "MEDIUM", "error"),
        ("p0-immediate", "LOW", "error"),  # Test case insensitivity
        ("p1-critical", "LOW", "error"),
        ("P2-HIGH", "HIGH", "warning"),
        ("p2-high", "CRITICAL", "warning"),
        ("P3-MODERATE", "MEDIUM", "note"),
        ("P4-LOW", "MEDIUM", "note"),
        ("", "CRITICAL", "error"),  # Fallback to severity
        ("", "HIGH", "error"),
        ("", "MEDIUM", "warning"),
        ("", "LOW", "note"),
        ("", "UNKNOWN", "note"),
    ], ids=[
        "p0_immediate", "p1_critical", "p0_lowercase", "p1_lowercase",
        "p2_high", "p2_high_critical_severity", "p3_moderate", "p4_low",
        "no_priority_critical", "no_priority_high", "no_priority_medium",
        "no_priority_low", "no_priority_unknown"
    ])
    def test_priority_to_level_combinations(self, priority, severity, expected):
        """Test various priority and severity combinations."""
        # Arrange & Act
        result = priority_to_level(priority, severity)
        
        # Assert
        assert result == expected


class TestSeverityToLevel:
    """Test severity level conversion."""

    @pytest.mark.parametrize("severity,expected", [
        ("CRITICAL", "error"),
        ("critical", "error"),
        ("HIGH", "error"),
        ("high", "error"),
        ("MEDIUM", "warning"),
        ("medium", "warning"),
        ("LOW", "note"),
        ("low", "note"),
        ("UNKNOWN", "note"),
        ("", "note"),
    ], ids=[
        "critical_upper", "critical_lower", "high_upper", "high_lower",
        "medium_upper", "medium_lower", "low_upper", "low_lower",
        "unknown", "empty"
    ])
    def test_severity_to_level(self, severity, expected):
        """Test severity to SARIF level mapping."""
        # Arrange & Act
        result = severity_to_level(severity)
        
        # Assert
        assert result == expected


class TestFormatEnrichedMessage:
    """Test enriched message formatting."""

    def test_basic_message_format(self):
        """Test basic message without enrichment."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {}
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "Test vulnerability" in message
        assert "test-pkg@1.0.0" in message

    def test_message_with_kev_enrichment(self):
        """Test message includes KEV warning."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {},
            "kev": {
                "in_kev": True,
                "vulnerability_name": "Critical RCE",
                "due_date": "2024-12-31",
                "required_action": "Apply patch immediately"
            }
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "KNOWN EXPLOITED IN THE WILD" in message
        assert "Critical RCE" in message
        assert "2024-12-31" in message
        assert "Apply patch immediately" in message

    def test_message_with_epss_enrichment(self):
        """Test message includes EPSS probability."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {},
            "epss": {
                "epss_score": 0.85,
                "epss_percentile": 0.95
            },
            "exploitation_probability": "85.0%"
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "85.0%" in message
        assert "EPSS" in message
        assert "Top 5.0%" in message

    def test_message_with_weaponized_exploit(self):
        """Test message includes weaponized exploit warning."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {},
            "exploit": {
                "weaponized": True,
                "exploit_maturity": "FUNCTIONAL"
            }
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "WEAPONIZED EXPLOIT AVAILABLE" in message
        assert "FUNCTIONAL" in message

    def test_message_with_exploit_available(self):
        """Test message includes public exploit info."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {},
            "exploit": {
                "exploit_available": True,
                "exploit_maturity": "POC"
            }
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "Public exploit available" in message
        assert "POC" in message

    def test_message_with_fixed_version_in_remediation(self):
        """Test message includes fixed version from remediation."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {},
            "remediation": {
                "fixed_version": "1.2.4"
            }
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "Fixed in version: 1.2.4" in message

    def test_message_with_fixed_version_in_affected(self):
        """Test message extracts fixed version from affected ranges."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {
                "affected": [
                    {
                        "ranges": [
                            {
                                "events": [
                                    {"introduced": "0"},
                                    {"fixed": "1.3.0"}
                                ]
                            }
                        ]
                    }
                ]
            }
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "Fixed in version: 1.3.0" in message

    def test_message_with_nested_fixed_version_breaks_early(self):
        """Test message extraction breaks on first fixed version found."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {
                "affected": [
                    {
                        "ranges": [
                            {
                                "events": [
                                    {"introduced": "0"},
                                    {"fixed": "1.3.0"}
                                ]
                            },
                            {
                                "events": [
                                    {"fixed": "2.0.0"}  # Should not reach this
                                ]
                            }
                        ]
                    }
                ]
            }
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "Fixed in version: 1.3.0" in message
        assert "2.0.0" not in message

    def test_message_with_ghsa_enrichment(self):
        """Test message includes GHSA advisory info."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {},
            "ghsa": {
                "ghsa_id": "GHSA-1234-5678-90ab",
                "permalink": "https://github.com/advisories/GHSA-1234-5678-90ab"
            }
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "GHSA-1234-5678-90ab" in message
        assert "https://github.com/advisories/GHSA-1234-5678-90ab" in message

    def test_message_with_priority_and_risk_score(self):
        """Test message includes priority and risk score."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {},
            "priority": "P1-CRITICAL",
            "risk_score": 95
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "Priority: P1-CRITICAL" in message
        assert "Risk Score: 95/100" in message


class TestCreateSarifDocument:
    """Test SARIF document generation."""

    def test_basic_structure(self):
        """Test that SARIF document has required fields."""
        # Arrange
        vulnerabilities = [
            {
                "package": "test-pkg",
                "version": "1.0.0",
                "vulnerability": {
                    "id": "GHSA-1234",
                    "summary": "Test vulnerability"
                }
            }
        ]
        
        # Act
        doc = create_sarif_document(vulnerabilities)
        
        # Assert
        assert doc["version"] == "2.1.0"
        assert doc["$schema"] == "https://json.schemastore.org/sarif-2.1.0.json"
        assert "runs" in doc
        assert len(doc["runs"]) == 1

    def test_run_structure(self):
        """Test that run has required fields."""
        # Arrange
        vulnerabilities = [
            {
                "package": "test-pkg",
                "version": "1.0.0",
                "vulnerability": {
                    "id": "VULN-1",
                    "summary": "Test"
                }
            }
        ]
        
        # Act
        doc = create_sarif_document(vulnerabilities)
        run = doc["runs"][0]
        
        # Assert
        assert "tool" in run
        assert "results" in run
        assert run["tool"]["driver"]["name"] == "BazBOM SCA"

    def test_empty_vulnerabilities(self):
        """Test handling of empty vulnerability list."""
        # Arrange & Act
        doc = create_sarif_document([])
        
        # Assert
        assert doc["version"] == "2.1.0"
        assert len(doc["runs"][0]["results"]) == 0

    def test_result_with_cvss_score(self):
        """Test that CVSS score is added to properties."""
        # Arrange
        vulnerabilities = [
            {
                "package": "test-pkg",
                "version": "1.0.0",
                "vulnerability": {"id": "VULN-1", "summary": "Test"},
                "cvss_score": 7.5
            }
        ]
        
        # Act
        doc = create_sarif_document(vulnerabilities)
        result = doc["runs"][0]["results"][0]
        
        # Assert
        assert result["properties"]["cvssScore"] == 7.5

    def test_result_with_risk_score(self):
        """Test that risk score is added to properties."""
        # Arrange
        vulnerabilities = [
            {
                "package": "test-pkg",
                "version": "1.0.0",
                "vulnerability": {"id": "VULN-1", "summary": "Test"},
                "risk_score": 85
            }
        ]
        
        # Act
        doc = create_sarif_document(vulnerabilities)
        result = doc["runs"][0]["results"][0]
        
        # Assert
        assert result["properties"]["riskScore"] == 85

    def test_result_with_priority(self):
        """Test that priority is added to properties."""
        # Arrange
        vulnerabilities = [
            {
                "package": "test-pkg",
                "version": "1.0.0",
                "vulnerability": {"id": "VULN-1", "summary": "Test"},
                "priority": "P1-CRITICAL"
            }
        ]
        
        # Act
        doc = create_sarif_document(vulnerabilities)
        result = doc["runs"][0]["results"][0]
        
        # Assert
        assert result["properties"]["priority"] == "P1-CRITICAL"

    def test_result_with_kev(self):
        """Test that KEV properties are added."""
        # Arrange
        vulnerabilities = [
            {
                "package": "test-pkg",
                "version": "1.0.0",
                "vulnerability": {"id": "VULN-1", "summary": "Test"},
                "kev": {
                    "in_kev": True,
                    "due_date": "2024-12-31"
                }
            }
        ]
        
        # Act
        doc = create_sarif_document(vulnerabilities)
        result = doc["runs"][0]["results"][0]
        
        # Assert
        assert result["properties"]["inKEV"] is True
        assert result["properties"]["kevDueDate"] == "2024-12-31"

    def test_result_with_epss(self):
        """Test that EPSS properties are added."""
        # Arrange
        vulnerabilities = [
            {
                "package": "test-pkg",
                "version": "1.0.0",
                "vulnerability": {"id": "VULN-1", "summary": "Test"},
                "epss": {
                    "epss_score": 0.85
                },
                "exploitation_probability": "85.0%"
            }
        ]
        
        # Act
        doc = create_sarif_document(vulnerabilities)
        result = doc["runs"][0]["results"][0]
        
        # Assert
        assert result["properties"]["epssScore"] == 0.85
        assert result["properties"]["exploitationProbability"] == "85.0%"

    def test_result_with_weaponized_exploit(self):
        """Test that weaponized exploit flag is added."""
        # Arrange
        vulnerabilities = [
            {
                "package": "test-pkg",
                "version": "1.0.0",
                "vulnerability": {"id": "VULN-1", "summary": "Test"},
                "exploit": {
                    "weaponized": True
                }
            }
        ]
        
        # Act
        doc = create_sarif_document(vulnerabilities)
        result = doc["runs"][0]["results"][0]
        
        # Assert
        assert result["properties"]["weaponizedExploit"] is True

    def test_rules_generated_for_unique_vulnerabilities(self):
        """Test that rules are created for unique vulnerability IDs."""
        # Arrange
        vulnerabilities = [
            {
                "package": "pkg1",
                "version": "1.0.0",
                "vulnerability": {
                    "id": "VULN-1",
                    "summary": "Vulnerability 1"
                }
            },
            {
                "package": "pkg2",
                "version": "2.0.0",
                "vulnerability": {
                    "id": "VULN-2",
                    "summary": "Vulnerability 2"
                }
            }
        ]
        
        # Act
        doc = create_sarif_document(vulnerabilities)
        rules = doc["runs"][0]["tool"]["driver"]["rules"]
        
        # Assert
        assert len(rules) == 2
        rule_ids = {r["id"] for r in rules}
        assert "VULN-1" in rule_ids
        assert "VULN-2" in rule_ids

    def test_rules_deduplicated_for_duplicate_vulnerability_ids(self):
        """Test that duplicate vulnerability IDs create only one rule."""
        # Arrange - Same vulnerability affecting multiple packages
        vulnerabilities = [
            {
                "package": "pkg1",
                "version": "1.0.0",
                "vulnerability": {
                    "id": "VULN-SAME",
                    "summary": "Same vulnerability"
                }
            },
            {
                "package": "pkg2",
                "version": "2.0.0",
                "vulnerability": {
                    "id": "VULN-SAME",  # Duplicate ID
                    "summary": "Same vulnerability"
                }
            }
        ]
        
        # Act
        doc = create_sarif_document(vulnerabilities)
        rules = doc["runs"][0]["tool"]["driver"]["rules"]
        
        # Assert
        assert len(rules) == 1  # Only one rule despite 2 findings
        assert rules[0]["id"] == "VULN-SAME"


class TestMain:
    """Test main entry point and CLI handling."""

    def test_successful_conversion(self, tmp_path, mocker):
        """Test successful conversion to SARIF."""
        # Arrange
        input_file = tmp_path / "input.json"
        output_file = tmp_path / "output.sarif"
        
        sample_data = {
            "vulnerabilities": [
                {
                    "package": "test-pkg",
                    "version": "1.0.0",
                    "vulnerability": {
                        "id": "TEST-VULN",
                        "summary": "Test vulnerability"
                    }
                }
            ]
        }
        input_file.write_text(json.dumps(sample_data))
        
        mocker.patch('sys.argv', [
            "sarif_adapter.py",
            "--input", str(input_file),
            "--output", str(output_file)
        ])
        
        # Act
        result = main()
        
        # Assert
        assert result == 0
        assert output_file.exists()
        
        with open(output_file) as f:
            data = json.load(f)
        assert data["version"] == "2.1.0"

    def test_missing_input_file(self, tmp_path, mocker):
        """Test handling of missing input file."""
        # Arrange
        output_file = tmp_path / "output.sarif"
        
        mocker.patch('sys.argv', [
            "sarif_adapter.py",
            "--input", "/nonexistent/file.json",
            "--output", str(output_file)
        ])
        
        # Act
        result = main()
        
        # Assert
        assert result == 1

    def test_invalid_json_input(self, tmp_path, mocker):
        """Test handling of invalid JSON input."""
        # Arrange
        bad_input = tmp_path / "bad.json"
        bad_input.write_text("not valid json{")
        
        output_file = tmp_path / "output.sarif"
        
        mocker.patch('sys.argv', [
            "sarif_adapter.py",
            "--input", str(bad_input),
            "--output", str(output_file)
        ])
        
        # Act
        result = main()
        
        # Assert
        assert result == 1

    def test_output_write_error(self, tmp_path, mocker):
        """Test handling of output write errors."""
        # Arrange
        input_file = tmp_path / "input.json"
        sample_data = {"vulnerabilities": []}
        input_file.write_text(json.dumps(sample_data))
        
        # Use a directory as output file to cause write error
        output_file = tmp_path / "cant_write"
        output_file.mkdir()
        
        mocker.patch('sys.argv', [
            "sarif_adapter.py",
            "--input", str(input_file),
            "--output", str(output_file)
        ])
        
        # Act
        result = main()
        
        # Assert
        assert result == 1

    def test_message_with_kev_partial_fields(self):
        """Test KEV message with some fields missing."""
        # Arrange - Test partial KEV data (no vulnerability_name)
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {},
            "kev": {
                "in_kev": True
                # Missing vulnerability_name, due_date, required_action
            }
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "KNOWN EXPLOITED IN THE WILD" in message

    def test_message_with_multiple_affected_ranges(self):
        """Test fixed version extraction with multiple affected packages."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {
                "affected": [
                    {
                        "ranges": [
                            {
                                "events": [
                                    {"introduced": "0"}
                                    # No fix in first range
                                ]
                            }
                        ]
                    },
                    {
                        "ranges": [
                            {
                                "events": [
                                    {"introduced": "1.0.0"},
                                    {"fixed": "2.0.0"}  # Fix in second affected package
                                ]
                            }
                        ]
                    }
                ]
            }
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "Fixed in version: 2.0.0" in message

    def test_message_with_ghsa_missing_permalink(self):
        """Test GHSA message with ID but no permalink."""
        # Arrange
        vuln_data = {
            "package": "test-pkg",
            "version": "1.0.0",
            "cve": "CVE-2024-0001",
            "summary": "Test vulnerability",
            "vulnerability": {},
            "ghsa": {
                "ghsa_id": "GHSA-1234-5678-90ab"
                # Missing permalink
            }
        }
        
        # Act
        message = format_enriched_message(vuln_data)
        
        # Assert
        assert "GHSA-1234-5678-90ab" in message
