#!/usr/bin/env python3
"""Comprehensive tests for validators/validate_sarif.py.

Tests cover SARIF 2.1.0 validation including version checks,
required fields, run validation, and result validation.
"""

import json
from pathlib import Path
from unittest.mock import patch

import pytest

from tools.supplychain.validators.validate_sarif import (
    validate_result,
    validate_run,
    validate_sarif_file,
    validate_sarif_required_fields,
)


class TestValidateSarifRequiredFields:
    """Test cases for validate_sarif_required_fields function."""

    def test_validate_minimal_valid_sarif(self):
        """Test validation of minimal valid SARIF document."""
        sarif = {
            "version": "2.1.0",
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "TestTool"
                        }
                    },
                    "results": []
                }
            ]
        }
        
        errors = validate_sarif_required_fields(sarif)
        
        assert errors == []

    def test_validate_missing_version(self):
        """Test error when version field is missing."""
        sarif = {
            "runs": []
        }
        
        errors = validate_sarif_required_fields(sarif)
        
        assert any("Missing required field: version" in e for e in errors)

    def test_validate_invalid_version(self):
        """Test error when version is not 2.1.0."""
        sarif = {
            "version": "2.0.0",
            "runs": []
        }
        
        errors = validate_sarif_required_fields(sarif)
        
        assert any("Invalid version: 2.0.0" in e for e in errors)

    def test_validate_missing_schema(self):
        """Test warning when $schema is missing."""
        sarif = {
            "version": "2.1.0",
            "runs": []
        }
        
        errors = validate_sarif_required_fields(sarif)
        
        assert any("Missing recommended field: $schema" in e for e in errors)

    def test_validate_missing_runs(self):
        """Test error when runs field is missing."""
        sarif = {
            "version": "2.1.0"
        }
        
        errors = validate_sarif_required_fields(sarif)
        
        assert any("Missing required field: runs" in e for e in errors)

    def test_validate_runs_not_array(self):
        """Test error when runs is not an array."""
        sarif = {
            "version": "2.1.0",
            "runs": "not an array"
        }
        
        errors = validate_sarif_required_fields(sarif)
        
        assert any("Field 'runs' must be an array" in e for e in errors)

    def test_validate_runs_empty_array(self):
        """Test error when runs array is empty."""
        sarif = {
            "version": "2.1.0",
            "runs": []
        }
        
        errors = validate_sarif_required_fields(sarif)
        
        assert any("Field 'runs' must contain at least one run" in e for e in errors)

    def test_validate_multiple_runs(self):
        """Test validation of multiple runs."""
        sarif = {
            "version": "2.1.0",
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "Tool1"
                        }
                    },
                    "results": []
                },
                {
                    "tool": {
                        "driver": {
                            "name": "Tool2"
                        }
                    },
                    "results": []
                }
            ]
        }
        
        errors = validate_sarif_required_fields(sarif)
        
        assert errors == []


class TestValidateRun:
    """Test cases for validate_run function."""

    def test_validate_run_missing_tool(self):
        """Test error when tool field is missing."""
        run = {
            "results": []
        }
        
        errors = validate_run(run, 0)
        
        assert any("Missing required field: tool" in e for e in errors)

    def test_validate_run_missing_driver(self):
        """Test error when tool.driver is missing."""
        run = {
            "tool": {},
            "results": []
        }
        
        errors = validate_run(run, 0)
        
        assert any("Missing required field: tool.driver" in e for e in errors)

    def test_validate_run_missing_driver_name(self):
        """Test error when tool.driver.name is missing."""
        run = {
            "tool": {
                "driver": {}
            },
            "results": []
        }
        
        errors = validate_run(run, 0)
        
        assert any("Missing required field: tool.driver.name" in e for e in errors)

    def test_validate_run_missing_results(self):
        """Test error when results field is missing."""
        run = {
            "tool": {
                "driver": {
                    "name": "TestTool"
                }
            }
        }
        
        errors = validate_run(run, 0)
        
        assert any("Missing required field: results" in e for e in errors)

    def test_validate_run_results_not_array(self):
        """Test error when results is not an array."""
        run = {
            "tool": {
                "driver": {
                    "name": "TestTool"
                }
            },
            "results": "not an array"
        }
        
        errors = validate_run(run, 0)
        
        assert any("Field 'results' must be an array" in e for e in errors)

    def test_validate_run_valid(self):
        """Test validation of valid run."""
        run = {
            "tool": {
                "driver": {
                    "name": "TestTool"
                }
            },
            "results": []
        }
        
        errors = validate_run(run, 0)
        
        assert errors == []

    def test_validate_run_error_includes_index(self):
        """Test that errors include run index."""
        run = {
            "results": []
        }
        
        errors = validate_run(run, 5)
        
        assert any("Run 5" in e for e in errors)


class TestValidateResult:
    """Test cases for validate_result function."""

    def test_validate_result_missing_rule_id(self):
        """Test error when ruleId is missing."""
        result = {
            "message": {
                "text": "Test message"
            }
        }
        
        errors = validate_result(result, 0, 0)
        
        assert any("Missing required field: ruleId" in e for e in errors)

    def test_validate_result_missing_message(self):
        """Test error when message is missing."""
        result = {
            "ruleId": "test-rule"
        }
        
        errors = validate_result(result, 0, 0)
        
        assert any("Missing required field: message" in e for e in errors)

    def test_validate_result_missing_message_text(self):
        """Test error when message.text is missing."""
        result = {
            "ruleId": "test-rule",
            "message": {}
        }
        
        errors = validate_result(result, 0, 0)
        
        assert any("Missing required field: message.text" in e for e in errors)

    def test_validate_result_invalid_level(self):
        """Test error when level is invalid."""
        result = {
            "ruleId": "test-rule",
            "message": {
                "text": "Test message"
            },
            "level": "invalid"
        }
        
        errors = validate_result(result, 0, 0)
        
        assert any("Invalid level: invalid" in e for e in errors)

    @pytest.mark.parametrize("level", ["none", "note", "warning", "error"])
    def test_validate_result_valid_levels(self, level):
        """Test validation with valid level values."""
        result = {
            "ruleId": "test-rule",
            "message": {
                "text": "Test message"
            },
            "level": level
        }
        
        errors = validate_result(result, 0, 0)
        
        assert errors == []

    def test_validate_result_valid_minimal(self):
        """Test validation of minimal valid result."""
        result = {
            "ruleId": "test-rule",
            "message": {
                "text": "Test message"
            }
        }
        
        errors = validate_result(result, 0, 0)
        
        assert errors == []

    def test_validate_result_error_includes_indices(self):
        """Test that errors include run and result indices."""
        result = {}
        
        errors = validate_result(result, 3, 5)
        
        assert any("Run 3, Result 5" in e for e in errors)


class TestValidateSarifFile:
    """Test cases for validate_sarif_file function."""

    def test_validate_file_not_found(self):
        """Test handling of non-existent file."""
        is_valid, errors = validate_sarif_file("/nonexistent/file.sarif")
        
        assert is_valid is False
        assert any("File not found" in e for e in errors)

    def test_validate_file_invalid_json(self, tmp_path):
        """Test handling of invalid JSON."""
        sarif_file = tmp_path / "invalid.sarif"
        sarif_file.write_text("not valid json{")
        
        is_valid, errors = validate_sarif_file(str(sarif_file))
        
        assert is_valid is False
        assert any("Invalid JSON" in e for e in errors)

    def test_validate_file_valid_sarif(self, tmp_path):
        """Test validation of valid SARIF file."""
        sarif_file = tmp_path / "valid.sarif"
        sarif = {
            "version": "2.1.0",
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "TestTool"
                        }
                    },
                    "results": [
                        {
                            "ruleId": "test-rule",
                            "message": {
                                "text": "Test finding"
                            },
                            "level": "warning"
                        }
                    ]
                }
            ]
        }
        sarif_file.write_text(json.dumps(sarif))
        
        is_valid, errors = validate_sarif_file(str(sarif_file))
        
        assert is_valid is True
        assert errors == []

    def test_validate_file_invalid_sarif(self, tmp_path):
        """Test validation of invalid SARIF file."""
        sarif_file = tmp_path / "invalid.sarif"
        sarif = {
            "version": "1.0.0",  # Wrong version
            "runs": []  # Empty runs
        }
        sarif_file.write_text(json.dumps(sarif))
        
        is_valid, errors = validate_sarif_file(str(sarif_file))
        
        assert is_valid is False
        assert len(errors) > 0


class TestMainFunction:
    """Test cases for main() function."""

    @patch('tools.supplychain.validators.validate_sarif.argparse.ArgumentParser.parse_args')
    def test_main_valid_files(self, mock_parse_args, tmp_path, capsys):
        """Test main() with valid SARIF files."""
        from tools.supplychain.validators.validate_sarif import main
        
        sarif1 = tmp_path / "file1.sarif"
        sarif2 = tmp_path / "file2.sarif"
        
        valid_sarif = {
            "version": "2.1.0",
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "Tool"
                        }
                    },
                    "results": []
                }
            ]
        }
        
        sarif1.write_text(json.dumps(valid_sarif))
        sarif2.write_text(json.dumps(valid_sarif))
        
        mock_parse_args.return_value = type('Args', (), {
            'files': [str(sarif1), str(sarif2)],
            'verbose': False
        })()
        
        result = main()
        
        assert result == 0
        captured = capsys.readouterr()
        assert "2/2 files valid" in captured.out

    @patch('tools.supplychain.validators.validate_sarif.argparse.ArgumentParser.parse_args')
    def test_main_invalid_files(self, mock_parse_args, tmp_path, capsys):
        """Test main() with invalid SARIF files."""
        from tools.supplychain.validators.validate_sarif import main
        
        sarif1 = tmp_path / "invalid.sarif"
        sarif1.write_text(json.dumps({"version": "1.0.0"}))
        
        mock_parse_args.return_value = type('Args', (), {
            'files': [str(sarif1)],
            'verbose': False
        })()
        
        result = main()
        
        assert result == 1
        captured = capsys.readouterr()
        assert "0/1 files valid" in captured.out
        assert "Invalid" in captured.out

    @patch('tools.supplychain.validators.validate_sarif.argparse.ArgumentParser.parse_args')
    def test_main_verbose_mode(self, mock_parse_args, tmp_path, capsys):
        """Test main() with verbose mode enabled."""
        from tools.supplychain.validators.validate_sarif import main
        
        sarif1 = tmp_path / "valid.sarif"
        valid_sarif = {
            "version": "2.1.0",
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "Tool"
                        }
                    },
                    "results": []
                }
            ]
        }
        sarif1.write_text(json.dumps(valid_sarif))
        
        mock_parse_args.return_value = type('Args', (), {
            'files': [str(sarif1)],
            'verbose': True
        })()
        
        result = main()
        
        assert result == 0
        captured = capsys.readouterr()
        assert "✓" in captured.out
        assert "Valid" in captured.out

    @patch('tools.supplychain.validators.validate_sarif.argparse.ArgumentParser.parse_args')
    def test_main_mixed_valid_invalid(self, mock_parse_args, tmp_path, capsys):
        """Test main() with mix of valid and invalid files."""
        from tools.supplychain.validators.validate_sarif import main
        
        valid_file = tmp_path / "valid.sarif"
        invalid_file = tmp_path / "invalid.sarif"
        
        valid_sarif = {
            "version": "2.1.0",
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "Tool"
                        }
                    },
                    "results": []
                }
            ]
        }
        
        valid_file.write_text(json.dumps(valid_sarif))
        invalid_file.write_text(json.dumps({"version": "1.0.0"}))
        
        mock_parse_args.return_value = type('Args', (), {
            'files': [str(valid_file), str(invalid_file)],
            'verbose': False
        })()
        
        result = main()
        
        assert result == 1
        captured = capsys.readouterr()
        assert "1/2 files valid" in captured.out
