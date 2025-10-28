#!/usr/bin/env python3
"""Tests for Bazel aspect functionality.

Tests cover aspect loading, dependency extraction, and SBOM generation
using Bazel aspects and rules.
"""

import json
import subprocess
from pathlib import Path
from unittest.mock import MagicMock, Mock, patch

import pytest


class TestBazelAspectIntegration:
    """Integration tests for Bazel aspects."""

    @pytest.fixture
    def workspace_root(self):
        """Return the actual workspace root for testing."""
        # Find the workspace root by looking for WORKSPACE file
        current = Path(__file__).parent
        while current != current.parent:
            if (current / "WORKSPACE").exists():
                return current
            current = current.parent
        pytest.skip("WORKSPACE not found")

    def test_workspace_has_required_files(self, workspace_root):
        """Test that workspace has required Bazel files."""
        assert (workspace_root / "WORKSPACE").exists()
        assert (workspace_root / "tools/supplychain/aspects.bzl").exists()
        assert (workspace_root / "tools/supplychain/defs.bzl").exists()

    def test_aspects_bzl_loads_correctly(self, workspace_root):
        """Test that aspects.bzl can be loaded by Bazel."""
        result = subprocess.run(
            ["bazel", "query", "//tools/supplychain:aspects.bzl"],
            cwd=str(workspace_root),
            capture_output=True,
            text=True,
            timeout=30,
        )
        
        assert result.returncode == 0
        assert "//tools/supplychain:aspects.bzl" in result.stdout

    def test_defs_bzl_loads_correctly(self, workspace_root):
        """Test that defs.bzl can be loaded by Bazel."""
        result = subprocess.run(
            ["bazel", "query", "//tools/supplychain:defs.bzl"],
            cwd=str(workspace_root),
            capture_output=True,
            text=True,
            timeout=30,
        )
        
        assert result.returncode == 0
        assert "//tools/supplychain:defs.bzl" in result.stdout

    def test_extract_deps_target_builds(self, workspace_root):
        """Test that the extract_deps target builds successfully."""
        result = subprocess.run(
            ["bazel", "build", "//:extract_deps"],
            cwd=str(workspace_root),
            capture_output=True,
            text=True,
            timeout=120,
        )
        
        assert result.returncode == 0, f"Build failed: {result.stderr}"
        
        # Check that the output file exists
        deps_json = workspace_root / "bazel-bin" / "workspace_deps.json"
        assert deps_json.exists(), "workspace_deps.json not generated"

    def test_workspace_deps_json_format(self, workspace_root):
        """Test that workspace_deps.json has correct format."""
        # Build first
        subprocess.run(
            ["bazel", "build", "//:extract_deps"],
            cwd=str(workspace_root),
            capture_output=True,
            text=True,
            timeout=120,
        )
        
        deps_json = workspace_root / "bazel-bin" / "workspace_deps.json"
        assert deps_json.exists()
        
        # Parse JSON
        with open(deps_json, 'r') as f:
            data = json.load(f)
        
        # Validate structure
        assert "packages" in data
        assert "source" in data
        assert isinstance(data["packages"], list)
        
        # Check package structure if any packages exist
        if data["packages"]:
            pkg = data["packages"][0]
            assert "name" in pkg
            assert "group" in pkg
            assert "version" in pkg
            assert "purl" in pkg
            assert "type" in pkg

    def test_workspace_sbom_target_builds(self, workspace_root):
        """Test that the workspace_sbom target builds successfully."""
        result = subprocess.run(
            ["bazel", "build", "//:workspace_sbom"],
            cwd=str(workspace_root),
            capture_output=True,
            text=True,
            timeout=120,
        )
        
        assert result.returncode == 0, f"Build failed: {result.stderr}"
        
        # Check that the SBOM file exists
        sbom_file = workspace_root / "bazel-bin" / "workspace_sbom.spdx.json"
        assert sbom_file.exists(), "SBOM file not generated"

    def test_sbom_file_is_valid_json(self, workspace_root):
        """Test that generated SBOM is valid JSON."""
        # Build first
        subprocess.run(
            ["bazel", "build", "//:workspace_sbom"],
            cwd=str(workspace_root),
            capture_output=True,
            text=True,
            timeout=120,
        )
        
        sbom_file = workspace_root / "bazel-bin" / "workspace_sbom.spdx.json"
        assert sbom_file.exists()
        
        # Parse JSON
        with open(sbom_file, 'r') as f:
            sbom = json.load(f)
        
        # Validate SPDX structure
        assert "spdxVersion" in sbom
        assert sbom["spdxVersion"] == "SPDX-2.3"
        assert "packages" in sbom
        assert "relationships" in sbom

    def test_aspect_on_example_target(self, workspace_root):
        """Test that aspect works on example Java target."""
        example_dir = workspace_root / "examples" / "test_aspect"
        
        # Skip if example doesn't exist
        if not example_dir.exists():
            pytest.skip("Example test_aspect not available")
        
        result = subprocess.run(
            ["bazel", "build", "//examples/test_aspect:test_sbom"],
            cwd=str(workspace_root),
            capture_output=True,
            text=True,
            timeout=120,
        )
        
        assert result.returncode == 0, f"Build failed: {result.stderr}"
        
        # Check that the SBOM file exists
        sbom_file = workspace_root / "bazel-bin" / "examples" / "test_aspect" / "test_sbom.spdx.json"
        assert sbom_file.exists(), "Test SBOM file not generated"

    def test_aspect_collects_dependencies(self, workspace_root):
        """Test that aspect correctly collects dependencies from target."""
        example_dir = workspace_root / "examples" / "test_aspect"
        
        # Skip if example doesn't exist
        if not example_dir.exists():
            pytest.skip("Example test_aspect not available")
        
        # Build the deps JSON
        subprocess.run(
            ["bazel", "build", "//examples/test_aspect:test_sbom"],
            cwd=str(workspace_root),
            capture_output=True,
            text=True,
            timeout=120,
        )
        
        deps_json = workspace_root / "bazel-bin" / "examples" / "test_aspect" / "test_sbom_deps.json"
        assert deps_json.exists()
        
        # Parse and validate
        with open(deps_json, 'r') as f:
            data = json.load(f)
        
        assert "packages" in data
        packages = data["packages"]
        
        # Should have at least guava and its transitive deps
        assert len(packages) > 0
        
        # Check that guava is in the list
        guava_found = False
        for pkg in packages:
            if pkg.get("name") == "guava" and pkg.get("group") == "com.google.guava":
                guava_found = True
                # Validate package has required fields
                assert "version" in pkg
                assert "purl" in pkg
                assert "label" in pkg  # Provenance
                assert pkg["type"] == "maven"
        
        assert guava_found, "Guava dependency not found in aspect output"


class TestBazelAspectProvenance:
    """Tests for provenance tracking in Bazel aspects."""

    @pytest.fixture
    def workspace_root(self):
        """Return the actual workspace root for testing."""
        current = Path(__file__).parent
        while current != current.parent:
            if (current / "WORKSPACE").exists():
                return current
            current = current.parent
        pytest.skip("WORKSPACE not found")

    def test_aspect_tracks_labels(self, workspace_root):
        """Test that aspect tracks Bazel target labels for provenance."""
        example_dir = workspace_root / "examples" / "test_aspect"
        
        if not example_dir.exists():
            pytest.skip("Example test_aspect not available")
        
        # Build
        subprocess.run(
            ["bazel", "build", "//examples/test_aspect:test_sbom"],
            cwd=str(workspace_root),
            capture_output=True,
            text=True,
            timeout=120,
        )
        
        deps_json = workspace_root / "bazel-bin" / "examples" / "test_aspect" / "test_sbom_deps.json"
        
        with open(deps_json, 'r') as f:
            data = json.load(f)
        
        # All packages should have labels
        for pkg in data["packages"]:
            assert "label" in pkg, f"Package {pkg.get('name')} missing label"
            assert pkg["label"].startswith("@@maven//"), f"Invalid label format: {pkg['label']}"


class TestBazelCLIIntegration:
    """Tests for bazbom_cli.py Bazel integration."""

    @pytest.fixture
    def workspace_root(self):
        """Return the actual workspace root for testing."""
        current = Path(__file__).parent
        while current != current.parent:
            if (current / "WORKSPACE").exists():
                return current
            current = current.parent
        pytest.skip("WORKSPACE not found")

    def test_cli_detects_bazel(self, workspace_root, tmp_path):
        """Test that CLI detects Bazel build system."""
        from tools.supplychain.build_system import detect_build_system
        
        system = detect_build_system(workspace_root)
        assert system is not None
        assert system.get_name() == "Bazel"

    def test_cli_run_bazel_aspect_scan(self, workspace_root):
        """Test run_bazel_aspect_scan function."""
        from tools.supplychain.bazbom_cli import run_bazel_aspect_scan
        
        # Run the scan
        dependencies = run_bazel_aspect_scan(workspace_root)
        
        # Should have dependencies
        assert len(dependencies) > 0
        
        # Check structure
        dep = dependencies[0]
        assert "name" in dep
        assert "group" in dep
        assert "version" in dep
        assert "purl" in dep

    def test_cli_scan_command_with_bazel(self, workspace_root, tmp_path):
        """Test CLI scan command on Bazel project."""
        import sys
        from pathlib import Path
        
        # Add tools/supplychain to path
        sys.path.insert(0, str(workspace_root / "tools" / "supplychain"))
        
        from bazbom_cli import scan_command
        
        # Create mock args
        class MockArgs:
            path = str(workspace_root)
            include_test = False
            output = str(tmp_path / "scan-output.json")
        
        args = MockArgs()
        
        # Run scan
        result = scan_command(args)
        
        # Should succeed
        assert result == 0
        
        # Output file should exist
        output_file = tmp_path / "scan-output.json"
        assert output_file.exists()
        
        # Parse output
        with open(output_file, 'r') as f:
            data = json.load(f)
        
        assert data["build_system"] == "Bazel"
        assert "dependencies" in data
        assert len(data["dependencies"]) > 0
