#!/usr/bin/env python3
"""Comprehensive unit tests for metrics_aggregator.py."""

import json
import pytest
from pathlib import Path
from unittest.mock import patch, mock_open

from tools.supplychain.metrics_aggregator import (
    load_json_file,
    aggregate_vulnerability_metrics,
    aggregate_dependency_metrics,
    aggregate_license_metrics,
    generate_metrics_report,
    main,
)


class TestLoadJsonFile:
    """Test load_json_file function."""

    def test_load_valid_json_file(self, tmp_path):
        """Test loading a valid JSON file."""
        # Arrange
        test_data = {"key": "value", "number": 42}
        test_file = tmp_path / "test.json"
        test_file.write_text(json.dumps(test_data))

        # Act
        result = load_json_file(str(test_file))

        # Assert
        assert result == test_data

    def test_load_nonexistent_file_required(self, tmp_path):
        """Test loading a nonexistent file when required raises FileNotFoundError."""
        # Arrange
        nonexistent_file = tmp_path / "nonexistent.json"

        # Act & Assert
        with pytest.raises(FileNotFoundError):
            load_json_file(str(nonexistent_file), required=True)

    def test_load_nonexistent_file_not_required(self, tmp_path):
        """Test loading a nonexistent file when not required returns empty dict."""
        # Arrange
        nonexistent_file = tmp_path / "nonexistent.json"

        # Act
        result = load_json_file(str(nonexistent_file), required=False)

        # Assert
        assert result == {}

    def test_load_invalid_json_required(self, tmp_path, capsys):
        """Test loading invalid JSON when required raises error."""
        # Arrange
        test_file = tmp_path / "invalid.json"
        test_file.write_text("{invalid json")

        # Act & Assert
        with pytest.raises(json.JSONDecodeError):
            load_json_file(str(test_file), required=True)

        # Check error message printed
        captured = capsys.readouterr()
        assert "Error parsing" in captured.err

    def test_load_invalid_json_not_required(self, tmp_path, capsys):
        """Test loading invalid JSON when not required returns empty dict."""
        # Arrange
        test_file = tmp_path / "invalid.json"
        test_file.write_text("{invalid json")

        # Act
        result = load_json_file(str(test_file), required=False)

        # Assert
        assert result == {}
        captured = capsys.readouterr()
        assert "Error parsing" in captured.err

    def test_load_empty_json_file(self, tmp_path):
        """Test loading an empty JSON object."""
        # Arrange
        test_file = tmp_path / "empty.json"
        test_file.write_text("{}")

        # Act
        result = load_json_file(str(test_file))

        # Assert
        assert result == {}


class TestAggregateVulnerabilityMetrics:
    """Test aggregate_vulnerability_metrics function."""

    def test_aggregate_empty_findings(self):
        """Test aggregating empty findings."""
        # Act
        result = aggregate_vulnerability_metrics({})

        # Assert
        assert result == {
            "total": 0,
            "critical": 0,
            "high": 0,
            "medium": 0,
            "low": 0,
        }

    def test_aggregate_vulnerabilities_key(self):
        """Test aggregating findings with 'vulnerabilities' key."""
        # Arrange
        sca_findings = {
            "vulnerabilities": [
                {"severity": "CRITICAL", "cve": "CVE-2024-0001"},
                {"severity": "HIGH", "cve": "CVE-2024-0002"},
                {"severity": "MEDIUM", "cve": "CVE-2024-0003"},
                {"severity": "LOW", "cve": "CVE-2024-0004"},
                {"severity": "CRITICAL", "cve": "CVE-2024-0005"},
            ]
        }

        # Act
        result = aggregate_vulnerability_metrics(sca_findings)

        # Assert
        assert result == {
            "total": 5,
            "critical": 2,
            "high": 1,
            "medium": 1,
            "low": 1,
        }

    def test_aggregate_findings_key(self):
        """Test aggregating findings with 'findings' key."""
        # Arrange
        sca_findings = {
            "findings": [
                {"severity": "HIGH", "id": "GHSA-0001"},
                {"severity": "MEDIUM", "id": "GHSA-0002"},
            ]
        }

        # Act
        result = aggregate_vulnerability_metrics(sca_findings)

        # Assert
        assert result == {
            "total": 2,
            "critical": 0,
            "high": 1,
            "medium": 1,
            "low": 0,
        }

    def test_aggregate_mixed_keys(self):
        """Test aggregating findings with both keys."""
        # Arrange
        sca_findings = {
            "vulnerabilities": [
                {"severity": "CRITICAL", "cve": "CVE-2024-0001"},
            ],
            "findings": [
                {"severity": "HIGH", "id": "GHSA-0001"},
            ],
        }

        # Act
        result = aggregate_vulnerability_metrics(sca_findings)

        # Assert
        assert result == {
            "total": 2,
            "critical": 1,
            "high": 1,
            "medium": 0,
            "low": 0,
        }

    def test_aggregate_lowercase_severity(self):
        """Test aggregating findings with lowercase severity."""
        # Arrange
        sca_findings = {
            "vulnerabilities": [
                {"severity": "critical", "cve": "CVE-2024-0001"},
                {"severity": "high", "cve": "CVE-2024-0002"},
            ]
        }

        # Act
        result = aggregate_vulnerability_metrics(sca_findings)

        # Assert
        assert result == {
            "total": 2,
            "critical": 1,
            "high": 1,
            "medium": 0,
            "low": 0,
        }

    def test_aggregate_missing_severity(self):
        """Test aggregating findings with missing severity."""
        # Arrange
        sca_findings = {
            "vulnerabilities": [
                {"cve": "CVE-2024-0001"},
                {"severity": "HIGH", "cve": "CVE-2024-0002"},
            ]
        }

        # Act
        result = aggregate_vulnerability_metrics(sca_findings)

        # Assert
        assert result["total"] == 2
        assert result["high"] == 1

    def test_aggregate_unknown_severity(self):
        """Test aggregating findings with unknown severity."""
        # Arrange
        sca_findings = {
            "vulnerabilities": [
                {"severity": "UNKNOWN", "cve": "CVE-2024-0001"},
                {"severity": "INFO", "cve": "CVE-2024-0002"},
            ]
        }

        # Act
        result = aggregate_vulnerability_metrics(sca_findings)

        # Assert
        assert result["total"] == 2
        assert result["critical"] == 0
        assert result["high"] == 0
        assert result["medium"] == 0
        assert result["low"] == 0


class TestAggregateDependencyMetrics:
    """Test aggregate_dependency_metrics function."""

    def test_aggregate_empty_sbom(self):
        """Test aggregating empty SBOM."""
        # Act
        result = aggregate_dependency_metrics({}, {})

        # Assert
        assert result == {
            "total": 0,
            "direct": 0,
            "transitive": 0,
            "conflicts": 0,
        }

    def test_aggregate_with_packages(self):
        """Test aggregating SBOM with packages."""
        # Arrange
        sbom = {
            "packages": [
                {"name": "pkg1", "depth": 1},
                {"name": "pkg2", "depth": 1},
                {"name": "pkg3", "depth": 2},
                {"name": "pkg4", "depth": 3},
            ]
        }
        conflicts = {"conflicts_found": 2}

        # Act
        result = aggregate_dependency_metrics(sbom, conflicts)

        # Assert
        assert result == {
            "total": 4,
            "direct": 2,
            "transitive": 2,
            "conflicts": 2,
        }

    def test_aggregate_without_depth_field(self):
        """Test aggregating packages without depth field (defaults to direct)."""
        # Arrange
        sbom = {
            "packages": [
                {"name": "pkg1"},
                {"name": "pkg2"},
                {"name": "pkg3"},
            ]
        }
        conflicts = {}

        # Act
        result = aggregate_dependency_metrics(sbom, conflicts)

        # Assert
        assert result == {
            "total": 3,
            "direct": 3,  # All default to depth 1
            "transitive": 0,
            "conflicts": 0,
        }

    def test_aggregate_without_conflicts_field(self):
        """Test aggregating without conflicts_found field."""
        # Arrange
        sbom = {
            "packages": [
                {"name": "pkg1", "depth": 1},
            ]
        }
        conflicts = {"other_field": "value"}

        # Act
        result = aggregate_dependency_metrics(sbom, conflicts)

        # Assert
        assert result["conflicts"] == 0


class TestAggregateLicenseMetrics:
    """Test aggregate_license_metrics function."""

    def test_aggregate_empty_license_report(self):
        """Test aggregating empty license report."""
        # Act
        result = aggregate_license_metrics({})

        # Assert
        assert result == {
            "copyleft": 0,
            "permissive": 0,
            "unknown": 0,
            "conflicts": 0,
        }

    def test_aggregate_with_summary(self):
        """Test aggregating license report with summary."""
        # Arrange
        license_report = {
            "summary": {
                "copyleft_count": 5,
                "permissive_count": 20,
                "unknown_count": 3,
            },
            "conflicts": [
                {"package1": "GPL-2.0", "package2": "Apache-2.0"},
                {"package3": "GPL-3.0", "package4": "MIT"},
            ],
        }

        # Act
        result = aggregate_license_metrics(license_report)

        # Assert
        assert result == {
            "copyleft": 5,
            "permissive": 20,
            "unknown": 3,
            "conflicts": 2,
        }

    def test_aggregate_with_partial_summary(self):
        """Test aggregating license report with partial summary."""
        # Arrange
        license_report = {
            "summary": {
                "copyleft_count": 5,
            },
            "conflicts": [],
        }

        # Act
        result = aggregate_license_metrics(license_report)

        # Assert
        assert result == {
            "copyleft": 5,
            "permissive": 0,
            "unknown": 0,
            "conflicts": 0,
        }

    def test_aggregate_without_conflicts(self):
        """Test aggregating license report without conflicts."""
        # Arrange
        license_report = {
            "summary": {
                "copyleft_count": 5,
                "permissive_count": 20,
                "unknown_count": 3,
            },
        }

        # Act
        result = aggregate_license_metrics(license_report)

        # Assert
        assert result["conflicts"] == 0


class TestGenerateMetricsReport:
    """Test generate_metrics_report function."""

    def test_generate_json_report(self, tmp_path, capsys):
        """Test generating JSON format report."""
        # Arrange
        output_file = tmp_path / "metrics.json"
        sbom = {"packages": [{"name": "pkg1", "depth": 1}]}
        sca_findings = {"vulnerabilities": [{"severity": "HIGH"}]}
        license_report = {"summary": {"copyleft_count": 1}, "conflicts": []}
        conflicts = {"conflicts_found": 0}

        # Act
        generate_metrics_report(
            sbom, sca_findings, license_report, conflicts, str(output_file), "json"
        )

        # Assert
        assert output_file.exists()
        with open(output_file) as f:
            result = json.load(f)

        assert result["version"] == "1.0"
        assert "generated_at" in result
        assert result["vulnerabilities"]["total"] == 1
        assert result["vulnerabilities"]["high"] == 1
        assert result["dependencies"]["total"] == 1
        assert result["licenses"]["copyleft"] == 1

        captured = capsys.readouterr()
        assert "Metrics report written" in captured.err

    def test_generate_text_report(self, tmp_path, capsys):
        """Test generating text format report."""
        # Arrange
        output_file = tmp_path / "metrics.txt"
        sbom = {"packages": [{"name": "pkg1", "depth": 1}]}
        sca_findings = {
            "vulnerabilities": [
                {"severity": "CRITICAL"},
                {"severity": "HIGH"},
                {"severity": "MEDIUM"},
                {"severity": "LOW"},
            ]
        }
        license_report = {
            "summary": {
                "copyleft_count": 2,
                "permissive_count": 5,
                "unknown_count": 1,
            },
            "conflicts": [{"pkg1": "GPL", "pkg2": "Apache"}],
        }
        conflicts = {"conflicts_found": 3}

        # Act
        generate_metrics_report(
            sbom, sca_findings, license_report, conflicts, str(output_file), "text"
        )

        # Assert
        assert output_file.exists()
        content = output_file.read_text()

        # Check key sections
        assert "Supply Chain Metrics Report" in content
        assert "Vulnerabilities:" in content
        assert "[CRITICAL] Critical: 1" in content
        assert "[HIGH] High: 1" in content
        assert "[MEDIUM] Medium: 1" in content
        assert "[LOW] Low: 1" in content
        assert "Dependencies:" in content
        assert "Total: 1" in content
        assert "Conflicts: 3" in content
        assert "Licenses:" in content
        assert "Copyleft: 2" in content
        assert "Permissive: 5" in content
        assert "Unknown: 1" in content

        # Also printed to stderr
        captured = capsys.readouterr()
        assert "Supply Chain Metrics Report" in captured.err

    def test_generate_report_with_empty_data(self, tmp_path):
        """Test generating report with all empty data."""
        # Arrange
        output_file = tmp_path / "metrics.json"

        # Act
        generate_metrics_report({}, {}, {}, {}, str(output_file), "json")

        # Assert
        assert output_file.exists()
        with open(output_file) as f:
            result = json.load(f)

        assert result["vulnerabilities"]["total"] == 0
        assert result["dependencies"]["total"] == 0
        assert result["licenses"]["copyleft"] == 0


class TestMainFunction:
    """Test main CLI function."""

    @patch("tools.supplychain.metrics_aggregator.load_json_file")
    @patch("tools.supplychain.metrics_aggregator.generate_metrics_report")
    def test_main_with_all_inputs(
        self, mock_generate, mock_load, tmp_path, monkeypatch
    ):
        """Test main with all input files."""
        # Arrange
        output_file = tmp_path / "metrics.json"
        mock_load.return_value = {"test": "data"}

        monkeypatch.setattr(
            "sys.argv",
            [
                "metrics_aggregator.py",
                "--sbom",
                "sbom.json",
                "--sca-findings",
                "sca.json",
                "--license-report",
                "licenses.json",
                "--conflicts",
                "conflicts.json",
                "--output",
                str(output_file),
                "--format",
                "json",
            ],
        )

        # Act
        main()

        # Assert
        assert mock_load.call_count == 4
        mock_generate.assert_called_once()

    @patch("tools.supplychain.metrics_aggregator.load_json_file")
    @patch("tools.supplychain.metrics_aggregator.generate_metrics_report")
    def test_main_with_minimal_inputs(
        self, mock_generate, mock_load, tmp_path, monkeypatch
    ):
        """Test main with only output file (no inputs)."""
        # Arrange
        output_file = tmp_path / "metrics.json"

        monkeypatch.setattr(
            "sys.argv",
            [
                "metrics_aggregator.py",
                "--output",
                str(output_file),
            ],
        )

        # Act
        main()

        # Assert
        mock_load.assert_not_called()
        mock_generate.assert_called_once()
        # Check that empty dicts were passed
        call_args = mock_generate.call_args[0]
        assert call_args[0] == {}  # sbom
        assert call_args[1] == {}  # sca_findings
        assert call_args[2] == {}  # license_report
        assert call_args[3] == {}  # conflicts

    @patch("tools.supplychain.metrics_aggregator.load_json_file")
    def test_main_with_error(self, mock_load, tmp_path, monkeypatch, capsys):
        """Test main with error during processing."""
        # Arrange
        output_file = tmp_path / "metrics.json"
        mock_load.side_effect = Exception("Test error")

        monkeypatch.setattr(
            "sys.argv",
            [
                "metrics_aggregator.py",
                "--sbom",
                "sbom.json",
                "--output",
                str(output_file),
            ],
        )

        # Act & Assert
        with pytest.raises(SystemExit) as exc_info:
            main()

        assert exc_info.value.code == 1
        captured = capsys.readouterr()
        assert "Error generating metrics" in captured.err

    def test_main_without_output_arg(self, monkeypatch):
        """Test main without required output argument."""
        # Arrange
        monkeypatch.setattr(
            "sys.argv",
            [
                "metrics_aggregator.py",
                "--sbom",
                "sbom.json",
            ],
        )

        # Act & Assert
        with pytest.raises(SystemExit) as exc_info:
            main()

        # argparse exits with code 2 for invalid arguments
        assert exc_info.value.code == 2


class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_aggregate_vulnerability_metrics_with_none_severity(self):
        """Test handling None severity value - should raise AttributeError."""
        # Arrange
        sca_findings = {
            "vulnerabilities": [
                {"severity": None, "cve": "CVE-2024-0001"},
            ]
        }

        # Act & Assert
        # Current implementation doesn't handle None, will raise AttributeError
        with pytest.raises(AttributeError):
            aggregate_vulnerability_metrics(sca_findings)

    def test_aggregate_vulnerability_metrics_with_empty_string_severity(self):
        """Test handling empty string severity."""
        # Arrange
        sca_findings = {
            "vulnerabilities": [
                {"severity": "", "cve": "CVE-2024-0001"},
            ]
        }

        # Act
        result = aggregate_vulnerability_metrics(sca_findings)

        # Assert
        assert result["total"] == 1

    def test_aggregate_dependency_metrics_with_zero_depth(self):
        """Test handling packages with depth 0."""
        # Arrange
        sbom = {
            "packages": [
                {"name": "pkg1", "depth": 0},
                {"name": "pkg2", "depth": 1},
            ]
        }

        # Act
        result = aggregate_dependency_metrics(sbom, {})

        # Assert
        assert result["total"] == 2
        assert result["direct"] == 1
        assert result["transitive"] == 1

    def test_generate_metrics_report_handles_unicode(self, tmp_path):
        """Test report generation with Unicode characters."""
        # Arrange
        output_file = tmp_path / "metrics.json"
        sbom = {
            "packages": [
                {"name": "café-", "depth": 1},
                {"name": "", "depth": 1},
            ]
        }

        # Act
        generate_metrics_report(sbom, {}, {}, {}, str(output_file), "json")

        # Assert
        assert output_file.exists()
        with open(output_file, encoding="utf-8") as f:
            result = json.load(f)

        assert result["dependencies"]["total"] == 2
