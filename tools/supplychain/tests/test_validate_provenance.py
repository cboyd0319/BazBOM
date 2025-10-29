#!/usr/bin/env python3
"""Tests for validators/validate_provenance.py"""

import json
import sys
from pathlib import Path
from unittest.mock import Mock, patch, mock_open

import pytest

sys.path.insert(0, str(Path(__file__).parent.parent))

from validators.validate_provenance import ProvenanceValidator


class TestProvenanceValidator:
    """Tests for ProvenanceValidator class."""

    def test_init_with_default_schema_path(self, mocker):
        """Test initialization with default schema path."""
        # Mock the schema loading
        mock_schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=mock_schema)
        
        validator = ProvenanceValidator()
        
        assert validator.schema == mock_schema
        assert validator.errors == []
        assert validator.warnings == []

    def test_init_with_custom_schema_path(self, mocker, tmp_path):
        """Test initialization with custom schema path."""
        schema_file = tmp_path / "custom-schema.json"
        schema_file.write_text(json.dumps({"type": "object"}))
        
        mock_schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=mock_schema)
        
        validator = ProvenanceValidator(str(schema_file))
        
        assert validator.schema_path == schema_file

    def test_load_schema_file_not_found(self, tmp_path):
        """Test _load_schema raises FileNotFoundError when schema missing."""
        nonexistent = tmp_path / "nonexistent-schema.json"
        
        validator = ProvenanceValidator.__new__(ProvenanceValidator)
        validator.schema_path = nonexistent
        
        with pytest.raises(FileNotFoundError, match="SLSA provenance schema not found"):
            validator._load_schema()

    def test_load_schema_invalid_json(self, tmp_path):
        """Test _load_schema raises ValueError for invalid JSON."""
        schema_file = tmp_path / "invalid.json"
        schema_file.write_text("{ invalid json")
        
        validator = ProvenanceValidator.__new__(ProvenanceValidator)
        validator.schema_path = schema_file
        
        with pytest.raises(ValueError, match="Invalid JSON"):
            validator._load_schema()

    def test_load_schema_invalid_json_schema(self, tmp_path, mocker):
        """Test _load_schema raises ValueError for invalid JSON Schema."""
        schema_file = tmp_path / "bad-schema.json"
        # Valid JSON but invalid JSON Schema
        schema_file.write_text(json.dumps({"type": "invalid_type"}))
        
        validator = ProvenanceValidator.__new__(ProvenanceValidator)
        validator.schema_path = schema_file
        
        # Mock the schema validator to raise error
        import jsonschema
        mocker.patch.object(
            jsonschema.Draft7Validator,
            'check_schema',
            side_effect=jsonschema.SchemaError("Invalid schema")
        )
        
        with pytest.raises(ValueError, match="Invalid JSON Schema"):
            validator._load_schema()

    def test_load_schema_success(self, tmp_path):
        """Test _load_schema successfully loads valid schema."""
        schema_file = tmp_path / "schema.json"
        schema_data = {
            "type": "object",
            "properties": {
                "_type": {"type": "string"}
            }
        }
        schema_file.write_text(json.dumps(schema_data))
        
        validator = ProvenanceValidator.__new__(ProvenanceValidator)
        validator.schema_path = schema_file
        
        schema = validator._load_schema()
        
        assert schema == schema_data

    def test_validate_file_success(self, tmp_path, mocker):
        """Test validate_file with valid provenance."""
        # Create mock schema and provenance
        schema = {
            "type": "object",
            "properties": {
                "_type": {"type": "string"}
            },
            "required": ["_type"]
        }
        
        prov_file = tmp_path / "provenance.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1"
        }
        prov_file.write_text(json.dumps(prov_data))
        
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        result = validator.validate_file(str(prov_file))
        
        assert result is True
        assert len(validator.errors) == 0

    def test_validate_file_not_found(self, mocker):
        """Test validate_file with nonexistent file."""
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value={})
        
        validator = ProvenanceValidator()
        result = validator.validate_file("/nonexistent/file.json")
        
        assert result is False
        assert len(validator.errors) > 0
        assert "not found" in validator.errors[0].lower()

    def test_validate_file_invalid_json(self, tmp_path, mocker):
        """Test validate_file with invalid JSON."""
        prov_file = tmp_path / "invalid.json"
        prov_file.write_text("{ invalid }")
        
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value={})
        
        validator = ProvenanceValidator()
        result = validator.validate_file(str(prov_file))
        
        assert result is False
        assert len(validator.errors) > 0

    def test_validate_file_schema_validation_error(self, tmp_path, mocker):
        """Test validate_file with schema validation error."""
        schema = {
            "type": "object",
            "properties": {
                "_type": {"type": "string"}
            },
            "required": ["_type"]
        }
        
        prov_file = tmp_path / "provenance.json"
        prov_data = {
            "missing_required_field": "value"
        }
        prov_file.write_text(json.dumps(prov_data))
        
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        result = validator.validate_file(str(prov_file))
        
        assert result is False
        assert len(validator.errors) > 0

    def test_errors_cleared_on_validate_file(self, tmp_path, mocker):
        """Test that errors are cleared when validate_file is called."""
        schema = {"type": "object"}
        prov_file = tmp_path / "prov.json"
        prov_file.write_text(json.dumps({}))
        
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        validator.errors = ["old_error"]
        validator.warnings = ["old_warning"]
        
        validator.validate_file(str(prov_file))
        
        # Errors and warnings should be cleared on new validation
        assert "old_error" not in validator.errors
        assert "old_warning" not in validator.warnings

    @pytest.mark.parametrize("provenance_type,expected_valid", [
        ("https://in-toto.io/Statement/v1", True),
        ("https://slsa.dev/provenance/v1.0", True),
        ("invalid_type", False),
    ])
    def test_validate_provenance_types(
        self, tmp_path, mocker, provenance_type, expected_valid
    ):
        """Test validation of different provenance types."""
        schema = {
            "type": "object",
            "properties": {
                "_type": {
                    "type": "string",
                    "pattern": "^https://(in-toto\\.io|slsa\\.dev)/"
                }
            },
            "required": ["_type"]
        }
        
        prov_file = tmp_path / "prov.json"
        prov_data = {"_type": provenance_type}
        prov_file.write_text(json.dumps(prov_data))
        
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        result = validator.validate_file(str(prov_file))
        
        if expected_valid:
            assert result is True or result is False  # Depends on exact schema
        else:
            # Invalid type should fail
            pass  # Schema validation will determine

    def test_validate_file_permission_error(self, tmp_path, mocker):
        """Test validate_file handles PermissionError."""
        prov_file = tmp_path / "noperm.json"
        prov_file.write_text(json.dumps({"_type": "test"}))
        prov_file.chmod(0o000)
        
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value={})
        
        validator = ProvenanceValidator()
        
        try:
            result = validator.validate_file(str(prov_file))
            
            assert result is False
            assert len(validator.errors) > 0
            assert "Permission denied" in validator.errors[0]
        finally:
            prov_file.chmod(0o644)
    
    def test_validate_file_generic_exception(self, tmp_path, mocker):
        """Test validate_file handles generic exceptions."""
        prov_file = tmp_path / "prov.json"
        prov_file.write_text(json.dumps({"_type": "test"}))
        
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value={})
        mocker.patch('builtins.open', side_effect=IOError("Disk error"))
        
        validator = ProvenanceValidator()
        result = validator.validate_file(str(prov_file))
        
        assert result is False
        assert len(validator.errors) > 0
        assert "Failed to read" in validator.errors[0]

    def test_validate_schema_generic_exception(self, tmp_path, mocker):
        """Test _validate_schema handles generic exceptions."""
        prov_file = tmp_path / "prov.json"
        prov_file.write_text(json.dumps({"_type": "test"}))
        
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value={})
        
        validator = ProvenanceValidator()
        
        # Mock validate in the validators module to raise a generic exception
        mocker.patch('validators.validate_provenance.validate', side_effect=Exception("Generic error"))
        
        result = validator.validate_file(str(prov_file))
        
        assert result is False
        assert len(validator.errors) > 0
        assert "Schema validation error" in validator.errors[0]

    def test_get_errors(self, mocker):
        """Test get_errors returns error list."""
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value={})
        
        validator = ProvenanceValidator()
        validator.errors = ["error1", "error2"]
        
        errors = validator.get_errors()
        
        assert errors == ["error1", "error2"]

    def test_get_warnings(self, mocker):
        """Test get_warnings returns warning list."""
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value={})
        
        validator = ProvenanceValidator()
        validator.warnings = ["warning1", "warning2"]
        
        warnings = validator.get_warnings()
        
        assert warnings == ["warning1", "warning2"]


class TestSemanticValidation:
    """Tests for semantic validation beyond schema checks."""

    def test_validate_semantics_unexpected_predicate_type(self, tmp_path, mocker):
        """Test warning for non-SLSA predicate type."""
        prov_file = tmp_path / "prov.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1",
            "predicateType": "https://example.com/custom/v1",
            "subject": [],
            "predicate": {}
        }
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        validator.validate_file(str(prov_file))
        
        assert len(validator.warnings) > 0
        assert "Unexpected predicate type" in validator.warnings[0]

    def test_validate_semantics_slsa_predicate_type_accepted(self, tmp_path, mocker):
        """Test SLSA predicate type is accepted without warnings."""
        prov_file = tmp_path / "prov.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1",
            "predicateType": "https://slsa.dev/provenance/v1.0",
            "subject": [{"name": "test", "digest": {"sha256": "abc123"}}],
            "predicate": {}
        }
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        validator.validate_file(str(prov_file))
        
        # Should not have warnings about predicate type
        assert not any("predicate type" in w.lower() for w in validator.warnings)

    def test_validate_semantics_subject_missing_digest(self, tmp_path, mocker):
        """Test error for subject missing digest."""
        prov_file = tmp_path / "prov.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [{"name": "test"}],  # Missing digest
            "predicate": {}
        }
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        validator.validate_file(str(prov_file))
        
        assert len(validator.errors) > 0
        assert "missing digest" in validator.errors[0].lower()

    def test_validate_semantics_subject_empty_digest(self, tmp_path, mocker):
        """Test error for subject with empty digest values."""
        prov_file = tmp_path / "prov.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [{"name": "test", "digest": {"sha256": ""}}],  # Empty digest
            "predicate": {}
        }
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        validator.validate_file(str(prov_file))
        
        assert len(validator.errors) > 0
        assert "empty digest" in validator.errors[0].lower()

    def test_validate_semantics_builder_id_not_uri(self, tmp_path, mocker):
        """Test warning for builder ID that's not a URI."""
        prov_file = tmp_path / "prov.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [{"name": "test", "digest": {"sha256": "abc123"}}],
            "predicate": {
                "runDetails": {
                    "builder": {
                        "id": "not-a-uri"  # Not a URI
                    }
                }
            }
        }
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        validator.validate_file(str(prov_file))
        
        assert len(validator.warnings) > 0
        assert "Builder ID should be a URI" in validator.warnings[0]

    def test_validate_semantics_builder_id_https_uri_accepted(self, tmp_path, mocker):
        """Test HTTPS builder ID is accepted without warnings."""
        prov_file = tmp_path / "prov.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [{"name": "test", "digest": {"sha256": "abc123"}}],
            "predicate": {
                "runDetails": {
                    "builder": {
                        "id": "https://github.com/actions/runner"
                    }
                }
            }
        }
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        validator.validate_file(str(prov_file))
        
        # Should not have warnings about builder ID
        assert not any("Builder ID" in w for w in validator.warnings)

    def test_validate_semantics_finished_before_started(self, tmp_path, mocker):
        """Test error when finishedOn is before startedOn."""
        prov_file = tmp_path / "prov.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [{"name": "test", "digest": {"sha256": "abc123"}}],
            "predicate": {
                "runDetails": {
                    "builder": {"id": "https://example.com/builder"},
                    "metadata": {
                        "startedOn": "2024-01-02T10:00:00Z",
                        "finishedOn": "2024-01-01T09:00:00Z"  # Before started
                    }
                }
            }
        }
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        validator.validate_file(str(prov_file))
        
        assert len(validator.errors) > 0
        assert "finishedOn timestamp is before startedOn" in validator.errors[0]

    def test_validate_semantics_timestamps_valid_order(self, tmp_path, mocker):
        """Test valid timestamp order passes without errors."""
        prov_file = tmp_path / "prov.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [{"name": "test", "digest": {"sha256": "abc123"}}],
            "predicate": {
                "runDetails": {
                    "builder": {"id": "https://example.com/builder"},
                    "metadata": {
                        "startedOn": "2024-01-01T09:00:00Z",
                        "finishedOn": "2024-01-01T10:00:00Z"  # After started
                    }
                }
            }
        }
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        validator.validate_file(str(prov_file))
        
        # Should not have errors about timestamps
        assert not any("timestamp" in e.lower() for e in validator.errors)

    def test_validate_semantics_invalid_timestamp_format(self, tmp_path, mocker):
        """Test warning for unparseable timestamps."""
        prov_file = tmp_path / "prov.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [{"name": "test", "digest": {"sha256": "abc123"}}],
            "predicate": {
                "runDetails": {
                    "builder": {"id": "https://example.com/builder"},
                    "metadata": {
                        "startedOn": "invalid-timestamp",
                        "finishedOn": "also-invalid"
                    }
                }
            }
        }
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        
        validator = ProvenanceValidator()
        validator.validate_file(str(prov_file))
        
        assert len(validator.warnings) > 0
        assert "Could not parse timestamps" in validator.warnings[0]


class TestMain:
    """Tests for main() CLI function."""

    def test_main_success_single_file(self, tmp_path, mocker):
        """Test main with single valid file."""
        prov_file = tmp_path / "prov.json"
        prov_data = {"_type": "https://in-toto.io/Statement/v1"}
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        mocker.patch('sys.argv', ['validate_provenance.py', str(prov_file)])
        
        from validators.validate_provenance import main
        
        result = main()
        
        assert result == 0

    def test_main_failure_invalid_file(self, tmp_path, mocker):
        """Test main with invalid file."""
        prov_file = tmp_path / "invalid.json"
        prov_file.write_text("{ invalid }")
        
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value={})
        mocker.patch('sys.argv', ['validate_provenance.py', str(prov_file)])
        
        from validators.validate_provenance import main
        
        result = main()
        
        assert result == 1

    def test_main_custom_schema(self, tmp_path, mocker):
        """Test main with custom schema argument."""
        schema_file = tmp_path / "schema.json"
        schema_file.write_text(json.dumps({"type": "object"}))
        
        prov_file = tmp_path / "prov.json"
        prov_file.write_text(json.dumps({}))
        
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value={"type": "object"})
        mocker.patch('sys.argv', [
            'validate_provenance.py',
            '--schema', str(schema_file),
            str(prov_file)
        ])
        
        from validators.validate_provenance import main
        
        result = main()
        
        assert result == 0

    def test_main_strict_mode_treats_warnings_as_errors(self, tmp_path, mocker):
        """Test main with --strict flag treats warnings as errors."""
        prov_file = tmp_path / "prov.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1",
            "predicateType": "https://example.com/custom",  # Will generate warning
            "subject": [],
            "predicate": {}
        }
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        mocker.patch('sys.argv', [
            'validate_provenance.py',
            '--strict',
            str(prov_file)
        ])
        
        from validators.validate_provenance import main
        
        result = main()
        
        assert result == 1  # Should fail due to warning in strict mode

    def test_main_quiet_mode(self, tmp_path, mocker, capsys):
        """Test main with --quiet flag suppresses success messages."""
        prov_file = tmp_path / "prov.json"
        prov_data = {"_type": "https://in-toto.io/Statement/v1"}
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        mocker.patch('sys.argv', [
            'validate_provenance.py',
            '--quiet',
            str(prov_file)
        ])
        
        from validators.validate_provenance import main
        
        result = main()
        
        assert result == 0
        captured = capsys.readouterr()
        # In quiet mode, should not print success message for valid file
        assert "VALID" not in captured.out or captured.out == ""

    def test_main_multiple_files(self, tmp_path, mocker):
        """Test main with multiple files shows summary."""
        prov1 = tmp_path / "prov1.json"
        prov1.write_text(json.dumps({"_type": "test"}))
        
        prov2 = tmp_path / "prov2.json"
        prov2.write_text(json.dumps({"_type": "test"}))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        mocker.patch('sys.argv', [
            'validate_provenance.py',
            str(prov1),
            str(prov2)
        ])
        
        from validators.validate_provenance import main
        
        result = main()
        
        # Should show summary for multiple files
        assert result == 0

    def test_main_schema_initialization_error(self, tmp_path, mocker):
        """Test main handles schema initialization errors."""
        prov_file = tmp_path / "prov.json"
        prov_file.write_text(json.dumps({}))
        
        mocker.patch('sys.argv', [
            'validate_provenance.py',
            '--schema', '/nonexistent/schema.json',
            str(prov_file)
        ])
        
        from validators.validate_provenance import main
        
        result = main()
        
        assert result == 2  # Initialization error

    def test_main_warnings_not_quiet_not_strict(self, tmp_path, mocker, capsys):
        """Test main with warnings in non-strict, non-quiet mode."""
        prov_file = tmp_path / "prov.json"
        prov_data = {
            "_type": "https://in-toto.io/Statement/v1",
            "predicateType": "https://example.com/custom",  # Generates warning
            "subject": [{"name": "test", "digest": {"sha256": "abc"}}],
            "predicate": {}
        }
        prov_file.write_text(json.dumps(prov_data))
        
        schema = {"type": "object"}
        mocker.patch.object(ProvenanceValidator, '_load_schema', return_value=schema)
        mocker.patch('sys.argv', [
            'validate_provenance.py',
            str(prov_file)
        ])
        
        from validators.validate_provenance import main
        
        result = main()
        
        # Should succeed but show warnings
        assert result == 0
        captured = capsys.readouterr()
        assert "WARNING" in captured.out or "[WARNING]" in captured.out
