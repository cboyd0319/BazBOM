#!/usr/bin/env python3
"""Tests for provenance_builder.py - SLSA provenance generation."""

import json
import sys
from pathlib import Path
from unittest.mock import patch

import pytest
from freezegun import freeze_time

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from provenance_builder import generate_slsa_provenance


class TestGenerateSlsaProvenance:
    """Test SLSA provenance generation."""
    
    @freeze_time("2025-01-01 00:00:00", tz_offset=0)
    def test_generate_with_all_parameters(self):
        """Test generating provenance with all parameters specified."""
        provenance = generate_slsa_provenance(
            artifact_name="myapp.jar",
            artifact_digest="abc123def456",
            commit_sha="commit-sha-123",
            build_id="build-456",
            builder="myorg/myrepo",
        )
        
        assert provenance["_type"] == "https://in-toto.io/Statement/v1"
        assert provenance["predicateType"] == "https://slsa.dev/provenance/v1"
        
        # Check subject
        assert len(provenance["subject"]) == 1
        assert provenance["subject"][0]["name"] == "myapp.jar"
        assert provenance["subject"][0]["digest"]["sha256"] == "abc123def456"
        
        # Check predicate
        predicate = provenance["predicate"]
        assert predicate["buildDefinition"]["buildType"] == "https://github.com/bazel-contrib/bazbom@v1"
        assert predicate["buildDefinition"]["externalParameters"]["repository"] == "myorg/myrepo"
        assert predicate["buildDefinition"]["externalParameters"]["ref"] == "commit-sha-123"
        assert predicate["buildDefinition"]["internalParameters"]["buildId"] == "build-456"
        
        # Check run details
        assert predicate["runDetails"]["builder"]["id"] == "https://github.com/myorg/myrepo/actions"
        assert predicate["runDetails"]["metadata"]["invocationId"] == "build-456"
        assert predicate["runDetails"]["metadata"]["startedOn"] == "2025-01-01T00:00:00Z"
        assert predicate["runDetails"]["metadata"]["finishedOn"] == "2025-01-01T00:00:00Z"
    
    def test_generate_with_minimal_parameters(self, env_vars):
        """Test generating provenance with only required parameter."""
        # Set environment variables for defaults
        env_vars(
            GITHUB_SHA="env-commit-sha",
            GITHUB_RUN_ID="env-build-id",
            GITHUB_REPOSITORY="env-org/env-repo",
        )
        
        provenance = generate_slsa_provenance(artifact_name="app.jar")
        
        # Should use environment variables for defaults
        assert provenance["subject"][0]["name"] == "app.jar"
        assert provenance["subject"][0]["digest"]["sha256"] == "unknown"
        assert provenance["predicate"]["buildDefinition"]["externalParameters"]["ref"] == "env-commit-sha"
        assert provenance["predicate"]["buildDefinition"]["internalParameters"]["buildId"] == "env-build-id"
        assert provenance["predicate"]["buildDefinition"]["externalParameters"]["repository"] == "env-org/env-repo"
    
    def test_generate_without_environment_vars(self, monkeypatch):
        """Test generating provenance without environment variables."""
        # Clear all GitHub environment variables
        for key in ["GITHUB_SHA", "GITHUB_RUN_ID", "GITHUB_REPOSITORY"]:
            monkeypatch.delenv(key, raising=False)
        
        provenance = generate_slsa_provenance(artifact_name="test.jar")
        
        # Should use "unknown" defaults
        assert provenance["predicate"]["buildDefinition"]["externalParameters"]["ref"] == "unknown"
        assert provenance["predicate"]["buildDefinition"]["internalParameters"]["buildId"] == "unknown"
        assert provenance["predicate"]["buildDefinition"]["externalParameters"]["repository"] == "unknown-builder"
    
    def test_generate_parameters_override_env_vars(self, env_vars):
        """Test that explicit parameters override environment variables."""
        env_vars(
            GITHUB_SHA="env-sha",
            GITHUB_RUN_ID="env-build",
            GITHUB_REPOSITORY="env-repo",
        )
        
        provenance = generate_slsa_provenance(
            artifact_name="app.jar",
            commit_sha="explicit-sha",
            build_id="explicit-build",
            builder="explicit-repo",
        )
        
        # Should use explicit parameters
        assert provenance["predicate"]["buildDefinition"]["externalParameters"]["ref"] == "explicit-sha"
        assert provenance["predicate"]["buildDefinition"]["internalParameters"]["buildId"] == "explicit-build"
        assert provenance["predicate"]["buildDefinition"]["externalParameters"]["repository"] == "explicit-repo"
    
    def test_generate_with_none_digest(self):
        """Test generating provenance with None digest."""
        provenance = generate_slsa_provenance(
            artifact_name="app.jar",
            artifact_digest=None,
        )
        
        assert provenance["subject"][0]["digest"]["sha256"] == "unknown"
    
    def test_generate_structure_completeness(self):
        """Test that generated provenance has all required SLSA fields."""
        provenance = generate_slsa_provenance(artifact_name="test.jar")
        
        # Check top-level structure
        assert "_type" in provenance
        assert "subject" in provenance
        assert "predicateType" in provenance
        assert "predicate" in provenance
        
        # Check predicate structure
        predicate = provenance["predicate"]
        assert "buildDefinition" in predicate
        assert "runDetails" in predicate
        
        # Check buildDefinition
        build_def = predicate["buildDefinition"]
        assert "buildType" in build_def
        assert "externalParameters" in build_def
        assert "internalParameters" in build_def
        assert "resolvedDependencies" in build_def
        
        # Check runDetails
        run_details = predicate["runDetails"]
        assert "builder" in run_details
        assert "metadata" in run_details
        assert "byproducts" in run_details
    
    def test_generate_timestamps_are_utc(self):
        """Test that timestamps are in UTC timezone."""
        provenance = generate_slsa_provenance(artifact_name="test.jar")
        
        timestamp = provenance["predicate"]["runDetails"]["metadata"]["startedOn"]
        # Should end with 'Z' indicating UTC
        assert timestamp.endswith("Z")
        # Should be ISO 8601 format
        assert "T" in timestamp
    
    def test_generate_resolved_dependencies_empty(self):
        """Test that resolvedDependencies is an empty list."""
        provenance = generate_slsa_provenance(artifact_name="test.jar")
        
        assert provenance["predicate"]["buildDefinition"]["resolvedDependencies"] == []
    
    def test_generate_byproducts_empty(self):
        """Test that byproducts is an empty list."""
        provenance = generate_slsa_provenance(artifact_name="test.jar")
        
        assert provenance["predicate"]["runDetails"]["byproducts"] == []
    
    def test_generate_builder_id_format(self):
        """Test that builder ID follows expected format."""
        provenance = generate_slsa_provenance(
            artifact_name="test.jar",
            builder="myorg/myrepo",
        )
        
        builder_id = provenance["predicate"]["runDetails"]["builder"]["id"]
        assert builder_id == "https://github.com/myorg/myrepo/actions"
    
    @pytest.mark.parametrize("artifact_name,digest", [
        ("app.jar", "abc123"),
        ("lib.war", "def456"),
        ("service.tar", "789ghi"),
    ], ids=["jar", "war", "tar"])
    def test_generate_various_artifacts(self, artifact_name, digest):
        """Test generating provenance for various artifact types."""
        provenance = generate_slsa_provenance(
            artifact_name=artifact_name,
            artifact_digest=digest,
        )
        
        assert provenance["subject"][0]["name"] == artifact_name
        assert provenance["subject"][0]["digest"]["sha256"] == digest
    
    def test_generate_is_json_serializable(self):
        """Test that generated provenance can be serialized to JSON."""
        provenance = generate_slsa_provenance(artifact_name="test.jar")
        
        # Should not raise an exception
        json_str = json.dumps(provenance)
        assert isinstance(json_str, str)
        
        # Should be deserializable back to same structure
        deserialized = json.loads(json_str)
        assert deserialized == provenance


class TestMainFunction:
    """Test the main CLI function."""
    
    @patch('provenance_builder.argparse.ArgumentParser.parse_args')
    def test_main_success(self, mock_parse_args, tmp_path, capsys):
        """Test successful provenance generation via main()."""
        from provenance_builder import main
        
        output_file = tmp_path / "provenance.json"
        
        mock_parse_args.return_value = type('Args', (), {
            'artifact': 'myapp.jar',
            'output': str(output_file),
            'digest': 'abc123',
            'commit': 'commit-sha',
            'build_id': 'build-123',
            'builder': 'myorg/myrepo'
        })()
        
        result = main()
        
        assert result == 0
        assert output_file.exists()
        
        # Verify content
        with open(output_file) as f:
            provenance = json.load(f)
        
        assert provenance["subject"][0]["name"] == "myapp.jar"
        assert provenance["subject"][0]["digest"]["sha256"] == "abc123"
        
        captured = capsys.readouterr()
        assert "Provenance written to" in captured.out
    
    @patch('provenance_builder.argparse.ArgumentParser.parse_args')
    def test_main_minimal_args(self, mock_parse_args, tmp_path):
        """Test main() with only required arguments."""
        from provenance_builder import main
        
        output_file = tmp_path / "provenance.json"
        
        mock_parse_args.return_value = type('Args', (), {
            'artifact': 'app.jar',
            'output': str(output_file),
            'digest': None,
            'commit': None,
            'build_id': None,
            'builder': None
        })()
        
        result = main()
        
        assert result == 0
        assert output_file.exists()
    
    @patch('provenance_builder.argparse.ArgumentParser.parse_args')
    def test_main_io_error(self, mock_parse_args, capsys):
        """Test main() handles IO errors gracefully."""
        from provenance_builder import main
        
        # Try to write to invalid path
        mock_parse_args.return_value = type('Args', (), {
            'artifact': 'app.jar',
            'output': '/invalid/path/provenance.json',
            'digest': None,
            'commit': None,
            'build_id': None,
            'builder': None
        })()
        
        result = main()
        
        assert result == 1
        captured = capsys.readouterr()
        assert "Error writing output file" in captured.err
