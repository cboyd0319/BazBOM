#!/usr/bin/env python3
"""Comprehensive tests for scan_container.py - Container image SBOM generation."""

import json
import subprocess
import sys
from pathlib import Path
from unittest.mock import MagicMock, Mock, patch, call

import pytest

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from scan_container import ContainerInspector


class TestContainerInspectorInit:
    """Test ContainerInspector initialization."""

    def test_init_with_image_name(self):
        """Test initialization with image name."""
        # Act
        inspector = ContainerInspector("nginx:latest")
        
        # Assert
        assert inspector.image == "nginx:latest"
        assert inspector.temp_dir is None

    def test_init_with_image_tag(self):
        """Test initialization with image and tag."""
        # Act
        inspector = ContainerInspector("myrepo/myimage:1.0.0")
        
        # Assert
        assert inspector.image == "myrepo/myimage:1.0.0"

    def test_init_with_registry_url(self):
        """Test initialization with full registry URL."""
        # Act
        inspector = ContainerInspector("ghcr.io/org/image:latest")
        
        # Assert
        assert inspector.image == "ghcr.io/org/image:latest"


class TestIsRuntimeAvailable:
    """Test container runtime availability check."""

    @patch('scan_container.subprocess.run')
    def test_is_runtime_available_docker_exists(self, mock_run):
        """Test detecting available Docker runtime."""
        # Arrange
        mock_run.return_value = Mock(returncode=0)
        inspector = ContainerInspector("test:latest")
        
        # Act
        result = inspector._is_runtime_available("docker")
        
        # Assert
        assert result is True
        mock_run.assert_called_once()
        assert mock_run.call_args[0][0] == ["docker", "--version"]

    @patch('scan_container.subprocess.run')
    def test_is_runtime_available_podman_exists(self, mock_run):
        """Test detecting available Podman runtime."""
        # Arrange
        mock_run.return_value = Mock(returncode=0)
        inspector = ContainerInspector("test:latest")
        
        # Act
        result = inspector._is_runtime_available("podman")
        
        # Assert
        assert result is True
        mock_run.assert_called_once()
        assert mock_run.call_args[0][0] == ["podman", "--version"]

    @patch('scan_container.subprocess.run')
    def test_is_runtime_available_not_found(self, mock_run):
        """Test detecting unavailable runtime."""
        # Arrange
        mock_run.side_effect = FileNotFoundError()
        inspector = ContainerInspector("test:latest")
        
        # Act
        result = inspector._is_runtime_available("docker")
        
        # Assert
        assert result is False

    @patch('scan_container.subprocess.run')
    def test_is_runtime_available_command_fails(self, mock_run):
        """Test handling runtime command failure."""
        # Arrange
        mock_run.side_effect = subprocess.CalledProcessError(1, ["docker", "--version"])
        inspector = ContainerInspector("test:latest")
        
        # Act
        result = inspector._is_runtime_available("docker")
        
        # Assert
        assert result is False

    @patch('scan_container.subprocess.run')
    def test_is_runtime_available_timeout(self, mock_run):
        """Test handling runtime command timeout."""
        # Arrange
        mock_run.side_effect = subprocess.TimeoutExpired(["docker", "--version"], 5)
        inspector = ContainerInspector("test:latest")
        
        # Act
        result = inspector._is_runtime_available("docker")
        
        # Assert
        assert result is False


class TestInspectWithRuntime:
    """Test image inspection with specific runtime."""

    @patch('scan_container.subprocess.run')
    def test_inspect_with_runtime_docker_success(self, mock_run):
        """Test successful image inspection with Docker."""
        # Arrange
        inspection_data = {
            "Id": "sha256:abc123",
            "RepoTags": ["nginx:latest"],
            "Config": {"Cmd": ["/bin/bash"]}
        }
        mock_run.return_value = Mock(
            stdout=json.dumps([inspection_data]),
            returncode=0
        )
        inspector = ContainerInspector("nginx:latest")
        
        # Act
        result = inspector._inspect_with_runtime("docker")
        
        # Assert
        assert result == inspection_data
        mock_run.assert_called_once()
        assert mock_run.call_args[0][0] == ["docker", "inspect", "nginx:latest"]

    @patch('scan_container.subprocess.run')
    def test_inspect_with_runtime_podman_success(self, mock_run):
        """Test successful image inspection with Podman."""
        # Arrange
        inspection_data = {"Id": "sha256:def456"}
        mock_run.return_value = Mock(
            stdout=json.dumps([inspection_data]),
            returncode=0
        )
        inspector = ContainerInspector("test:latest")
        
        # Act
        result = inspector._inspect_with_runtime("podman")
        
        # Assert
        assert result == inspection_data

    @patch('scan_container.subprocess.run')
    def test_inspect_with_runtime_command_fails(self, mock_run):
        """Test handling inspection command failure."""
        # Arrange
        mock_run.side_effect = subprocess.CalledProcessError(
            1, ["docker", "inspect", "test"], stderr="Error: No such image"
        )
        inspector = ContainerInspector("test:latest")
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="Failed to inspect image"):
            inspector._inspect_with_runtime("docker")

    @patch('scan_container.subprocess.run')
    def test_inspect_with_runtime_invalid_json(self, mock_run):
        """Test handling invalid JSON response."""
        # Arrange
        mock_run.return_value = Mock(
            stdout="{invalid json",
            returncode=0
        )
        inspector = ContainerInspector("test:latest")
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="Failed to parse inspection output"):
            inspector._inspect_with_runtime("docker")

    @patch('scan_container.subprocess.run')
    def test_inspect_with_runtime_empty_response(self, mock_run):
        """Test handling empty inspection response."""
        # Arrange
        mock_run.return_value = Mock(
            stdout=json.dumps([]),
            returncode=0
        )
        inspector = ContainerInspector("test:latest")
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="No inspection data returned"):
            inspector._inspect_with_runtime("docker")


class TestInspectImage:
    """Test high-level image inspection."""

    @patch.object(ContainerInspector, '_is_runtime_available')
    @patch.object(ContainerInspector, '_inspect_with_runtime')
    def test_inspect_image_uses_docker_first(self, mock_inspect, mock_available):
        """Test Docker is tried first if available."""
        # Arrange
        mock_available.side_effect = [True, False]  # docker available, podman not
        mock_inspect.return_value = {"Id": "sha256:abc123"}
        inspector = ContainerInspector("test:latest")
        
        # Act
        result = inspector.inspect_image()
        
        # Assert
        assert result == {"Id": "sha256:abc123"}
        mock_available.assert_called_once_with("docker")
        mock_inspect.assert_called_once_with("docker")

    @patch.object(ContainerInspector, '_is_runtime_available')
    @patch.object(ContainerInspector, '_inspect_with_runtime')
    def test_inspect_image_fallback_to_podman(self, mock_inspect, mock_available):
        """Test fallback to Podman if Docker unavailable."""
        # Arrange
        mock_available.side_effect = [False, True]  # docker not available, podman is
        mock_inspect.return_value = {"Id": "sha256:def456"}
        inspector = ContainerInspector("test:latest")
        
        # Act
        result = inspector.inspect_image()
        
        # Assert
        assert result == {"Id": "sha256:def456"}
        assert mock_available.call_count == 2
        mock_inspect.assert_called_once_with("podman")

    @patch.object(ContainerInspector, '_is_runtime_available')
    def test_inspect_image_no_runtime_available(self, mock_available):
        """Test error when neither Docker nor Podman is available."""
        # Arrange
        mock_available.return_value = False
        inspector = ContainerInspector("test:latest")
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="Neither docker nor podman is available"):
            inspector.inspect_image()


class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_init_with_empty_image_name(self):
        """Test initialization with empty image name."""
        # Act
        inspector = ContainerInspector("")
        
        # Assert
        assert inspector.image == ""

    def test_init_with_unicode_image_name(self):
        """Test initialization with Unicode characters in image name."""
        # Act
        inspector = ContainerInspector("tëst-imgæ:latest")
        
        # Assert
        assert inspector.image == "tëst-imgæ:latest"

    def test_init_with_very_long_image_name(self):
        """Test initialization with very long image name."""
        # Arrange
        long_name = "a" * 1000 + ":latest"
        
        # Act
        inspector = ContainerInspector(long_name)
        
        # Assert
        assert inspector.image == long_name

    @patch('scan_container.subprocess.run')
    def test_is_runtime_available_with_special_characters(self, mock_run):
        """Test runtime check with special characters in runtime name."""
        # Arrange
        mock_run.return_value = Mock(returncode=0)
        inspector = ContainerInspector("test:latest")
        
        # Act
        result = inspector._is_runtime_available("my-custom-runtime")
        
        # Assert
        assert result is True
        mock_run.assert_called_once()


class TestMultipleImages:
    """Test handling multiple image formats."""

    @pytest.mark.parametrize("image_name", [
        "nginx",
        "nginx:latest",
        "nginx:1.21.0",
        "myregistry.io/nginx:latest",
        "ghcr.io/org/repo/image:v1.0.0",
        "docker.io/library/alpine:3.14",
    ], ids=["no-tag", "latest", "version", "registry", "ghcr", "docker-hub"])
    def test_init_with_various_image_formats(self, image_name):
        """Test initialization with various image name formats."""
        # Act
        inspector = ContainerInspector(image_name)
        
        # Assert
        assert inspector.image == image_name


class TestRuntimeConfiguration:
    """Test runtime-specific configuration and behavior."""

    @patch('scan_container.subprocess.run')
    def test_runtime_check_uses_correct_timeout(self, mock_run):
        """Test runtime availability check uses 5 second timeout."""
        # Arrange
        mock_run.return_value = Mock(returncode=0)
        inspector = ContainerInspector("test:latest")
        
        # Act
        inspector._is_runtime_available("docker")
        
        # Assert
        assert mock_run.call_args[1]['timeout'] == 5

    @patch('scan_container.subprocess.run')
    def test_inspect_uses_correct_timeout(self, mock_run):
        """Test image inspection uses 30 second timeout."""
        # Arrange
        mock_run.return_value = Mock(
            stdout=json.dumps([{"Id": "sha256:abc123"}]),
            returncode=0
        )
        inspector = ContainerInspector("test:latest")
        
        # Act
        inspector._inspect_with_runtime("docker")
        
        # Assert
        assert mock_run.call_args[1]['timeout'] == 30

    @patch('scan_container.subprocess.run')
    def test_inspect_captures_output(self, mock_run):
        """Test inspection captures stdout and stderr."""
        # Arrange
        mock_run.return_value = Mock(
            stdout=json.dumps([{"Id": "sha256:abc123"}]),
            returncode=0
        )
        inspector = ContainerInspector("test:latest")
        
        # Act
        inspector._inspect_with_runtime("docker")
        
        # Assert
        assert mock_run.call_args[1]['capture_output'] is True
        assert mock_run.call_args[1]['text'] is True


class TestTempDirectoryManagement:
    """Test temporary directory management."""

    def test_temp_dir_initially_none(self):
        """Test temp_dir is None on initialization."""
        # Act
        inspector = ContainerInspector("test:latest")
        
        # Assert
        assert inspector.temp_dir is None


class TestErrorMessages:
    """Test error message clarity and content."""

    @patch('scan_container.subprocess.run')
    def test_inspect_error_includes_image_name(self, mock_run):
        """Test error message includes the image name."""
        # Arrange
        mock_run.side_effect = subprocess.CalledProcessError(
            1, ["docker", "inspect", "missing:image"], stderr="Error: No such image"
        )
        inspector = ContainerInspector("missing:image")
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="missing:image"):
            inspector._inspect_with_runtime("docker")

    @patch('scan_container.subprocess.run')
    def test_inspect_error_includes_stderr(self, mock_run):
        """Test error message includes stderr output."""
        # Arrange
        error_msg = "Error: No such image"
        mock_run.side_effect = subprocess.CalledProcessError(
            1, ["docker", "inspect", "test"], stderr=error_msg
        )
        inspector = ContainerInspector("test:latest")
        
        # Act & Assert
        with pytest.raises(RuntimeError, match=error_msg):
            inspector._inspect_with_runtime("docker")
