#!/usr/bin/env python3
"""Comprehensive tests for bazbom_cli.py"""

import json
import sys
from pathlib import Path
from unittest.mock import MagicMock, Mock, patch, call

import pytest

# Import modules under test
from tools.supplychain import bazbom_cli
from tools.supplychain.build_system import Dependency


class TestPerformScan:
    """Tests for perform_scan function."""

    def test_perform_scan_no_build_system_detected_returns_error(
        self, tmp_path, mocker, capsys
    ):
        """Test perform_scan returns error when no build system detected."""
        # Arrange
        mock_detect = mocker.patch(
            'tools.supplychain.bazbom_cli.detect_build_system',
            return_value=None
        )
        mock_config = Mock()
        args = Mock()
        
        # Act
        result = bazbom_cli.perform_scan(tmp_path, mock_config, args)
        
        # Assert
        assert result == 1
        captured = capsys.readouterr()
        assert "ERROR: Could not detect build system" in captured.err
        assert "Supported: Maven" in captured.err
        mock_detect.assert_called_once_with(tmp_path)

    def test_perform_scan_bazel_project_returns_instructions(
        self, tmp_path, mocker, capsys
    ):
        """Test perform_scan provides Bazel instructions for Bazel projects."""
        # Arrange
        mock_build_system = Mock()
        mock_build_system.get_name.return_value = "Bazel"
        mock_detect = mocker.patch(
            'tools.supplychain.bazbom_cli.detect_build_system',
            return_value=mock_build_system
        )
        mock_config = Mock()
        args = Mock()
        
        # Act
        result = bazbom_cli.perform_scan(tmp_path, mock_config, args)
        
        # Assert
        assert result == 0
        captured = capsys.readouterr()
        assert "Detected build system: Bazel" in captured.out
        assert "bazel build //:sbom_all" in captured.out
        assert "bazel build //:sca_scan_osv" in captured.out

    def test_perform_scan_maven_success_writes_json(
        self, tmp_path, mocker, capsys, monkeypatch
    ):
        """Test perform_scan successfully processes Maven project."""
        # Arrange
        # Change to tmp_path so default output goes there
        monkeypatch.chdir(tmp_path)
        
        mock_build_system = Mock()
        mock_build_system.get_name.return_value = "Maven"
        
        # Create mock dependencies
        mock_deps = [
            Dependency(
                name="guava",
                version="31.1-jre",
                group_id="com.google.guava",
                artifact_id="guava",
                scope="compile"
            ),
            Dependency(
                name="junit",
                version="4.13.2",
                group_id="junit",
                artifact_id="junit",
                scope="test"
            )
        ]
        mock_build_system.resolve_dependencies.return_value = mock_deps
        
        mock_detect = mocker.patch(
            'tools.supplychain.bazbom_cli.detect_build_system',
            return_value=mock_build_system
        )
        
        mock_config = Mock()
        mock_config.get_include_test_deps.return_value = False
        
        args = Mock()
        args.include_test = False
        args.output = None
        
        # Act
        result = bazbom_cli.perform_scan(tmp_path, mock_config, args)
        
        # Assert
        assert result == 0
        
        # Check JSON was written
        output_file = tmp_path / "dependencies.json"
        assert output_file.exists()
        
        with open(output_file, 'r') as f:
            data = json.load(f)
        
        assert data["build_system"] == "Maven"
        assert data["total_dependencies"] == 2
        assert len(data["dependencies"]) == 2

    def test_perform_scan_custom_output_path(
        self, tmp_path, mocker
    ):
        """Test perform_scan respects custom output path."""
        # Arrange
        mock_build_system = Mock()
        mock_build_system.get_name.return_value = "Gradle"
        mock_build_system.resolve_dependencies.return_value = []
        
        mocker.patch(
            'tools.supplychain.bazbom_cli.detect_build_system',
            return_value=mock_build_system
        )
        
        mock_config = Mock()
        mock_config.get_include_test_deps.return_value = False
        
        output_path = tmp_path / "custom_output.json"
        args = Mock()
        args.include_test = False
        args.output = str(output_path)
        
        # Act
        result = bazbom_cli.perform_scan(tmp_path, mock_config, args)
        
        # Assert
        assert result == 0
        assert output_path.exists()

    def test_perform_scan_include_test_deps(self, tmp_path, mocker):
        """Test perform_scan includes test dependencies when requested."""
        # Arrange
        mock_build_system = Mock()
        mock_build_system.get_name.return_value = "Maven"
        mock_build_system.resolve_dependencies.return_value = []
        
        mocker.patch(
            'tools.supplychain.bazbom_cli.detect_build_system',
            return_value=mock_build_system
        )
        
        mock_config = Mock()
        mock_config.get_include_test_deps.return_value = False
        
        args = Mock()
        args.include_test = True
        args.output = str(tmp_path / "deps.json")
        
        # Act
        result = bazbom_cli.perform_scan(tmp_path, mock_config, args)
        
        # Assert
        assert result == 0
        mock_build_system.resolve_dependencies.assert_called_once_with(
            tmp_path,
            include_test_deps=True
        )

    def test_perform_scan_config_include_test_deps(self, tmp_path, mocker):
        """Test perform_scan uses config test deps when args not set."""
        # Arrange
        mock_build_system = Mock()
        mock_build_system.get_name.return_value = "Maven"
        mock_build_system.resolve_dependencies.return_value = []
        
        mocker.patch(
            'tools.supplychain.bazbom_cli.detect_build_system',
            return_value=mock_build_system
        )
        
        mock_config = Mock()
        mock_config.get_include_test_deps.return_value = True
        
        args = Mock()
        args.include_test = False
        args.output = str(tmp_path / "deps.json")
        
        # Act
        result = bazbom_cli.perform_scan(tmp_path, mock_config, args)
        
        # Assert
        assert result == 0
        mock_build_system.resolve_dependencies.assert_called_once_with(
            tmp_path,
            include_test_deps=True
        )

    def test_perform_scan_runtime_error_returns_error_code(
        self, tmp_path, mocker, capsys
    ):
        """Test perform_scan handles RuntimeError gracefully."""
        # Arrange
        mock_build_system = Mock()
        mock_build_system.get_name.return_value = "Maven"
        mock_build_system.resolve_dependencies.side_effect = RuntimeError(
            "Failed to resolve dependencies"
        )
        
        mocker.patch(
            'tools.supplychain.bazbom_cli.detect_build_system',
            return_value=mock_build_system
        )
        
        mock_config = Mock()
        mock_config.get_include_test_deps.return_value = False
        
        args = Mock()
        args.include_test = False
        args.output = None
        
        # Act
        result = bazbom_cli.perform_scan(tmp_path, mock_config, args)
        
        # Assert
        assert result == 1
        captured = capsys.readouterr()
        assert "ERROR: Failed to resolve dependencies" in captured.err


class TestGetBuildFilesMtimes:
    """Tests for get_build_files_mtimes function."""

    def test_get_build_files_mtimes_maven_finds_pom(self, tmp_path):
        """Test get_build_files_mtimes finds Maven pom.xml files."""
        # Arrange
        pom_file = tmp_path / "pom.xml"
        pom_file.write_text("<project></project>")
        
        # Act
        mtimes = bazbom_cli.get_build_files_mtimes(tmp_path, "Maven")
        
        # Assert
        assert str(pom_file) in mtimes
        assert mtimes[str(pom_file)] > 0

    def test_get_build_files_mtimes_gradle_finds_build_gradle(self, tmp_path):
        """Test get_build_files_mtimes finds Gradle build files."""
        # Arrange
        build_file = tmp_path / "build.gradle"
        build_file.write_text("plugins { }")
        
        settings_file = tmp_path / "settings.gradle"
        settings_file.write_text("rootProject.name = 'test'")
        
        # Act
        mtimes = bazbom_cli.get_build_files_mtimes(tmp_path, "Gradle")
        
        # Assert
        assert str(build_file) in mtimes
        assert str(settings_file) in mtimes

    def test_get_build_files_mtimes_bazel_finds_workspace(self, tmp_path):
        """Test get_build_files_mtimes finds Bazel WORKSPACE."""
        # Arrange
        workspace_file = tmp_path / "WORKSPACE"
        workspace_file.write_text("workspace(name = 'test')")
        
        # Act
        mtimes = bazbom_cli.get_build_files_mtimes(tmp_path, "Bazel")
        
        # Assert
        assert str(workspace_file) in mtimes

    def test_get_build_files_mtimes_recursive_finds_nested(self, tmp_path):
        """Test get_build_files_mtimes finds nested build files."""
        # Arrange
        subdir = tmp_path / "module1"
        subdir.mkdir()
        nested_pom = subdir / "pom.xml"
        nested_pom.write_text("<project></project>")
        
        # Act
        mtimes = bazbom_cli.get_build_files_mtimes(tmp_path, "Maven")
        
        # Assert
        assert str(nested_pom) in mtimes

    def test_get_build_files_mtimes_empty_when_no_files(self, tmp_path):
        """Test get_build_files_mtimes returns empty dict when no files."""
        # Act
        mtimes = bazbom_cli.get_build_files_mtimes(tmp_path, "Maven")
        
        # Assert
        assert mtimes == {}

    def test_get_build_files_mtimes_ignores_directories(self, tmp_path):
        """Test get_build_files_mtimes ignores directories."""
        # Arrange
        dir_with_build_name = tmp_path / "BUILD"
        dir_with_build_name.mkdir()
        
        # Act
        mtimes = bazbom_cli.get_build_files_mtimes(tmp_path, "Bazel")
        
        # Assert
        assert str(dir_with_build_name) not in mtimes




class TestScanCommand:
    """Tests for scan_command function."""

    def test_scan_command_single_scan_mode(self, tmp_path, mocker):
        """Test scan_command performs single scan when watch not enabled."""
        # Arrange
        mock_config = Mock()
        mocker.patch(
            'tools.supplychain.bazbom_cli.BazBOMConfig.find_and_load',
            return_value=mock_config
        )
        
        mock_perform = mocker.patch(
            'tools.supplychain.bazbom_cli.perform_scan',
            return_value=0
        )
        
        args = Mock()
        args.path = str(tmp_path)
        args.watch = False
        
        # Act
        result = bazbom_cli.scan_command(args)
        
        # Assert
        assert result == 0
        mock_perform.assert_called_once_with(tmp_path, mock_config, args)

    def test_scan_command_watch_mode_initial_scan(
        self, tmp_path, mocker, capsys
    ):
        """Test scan_command performs initial scan in watch mode."""
        # Arrange
        mock_config = Mock()
        mocker.patch(
            'tools.supplychain.bazbom_cli.BazBOMConfig.find_and_load',
            return_value=mock_config
        )
        
        mock_build_system = Mock()
        mock_build_system.get_name.return_value = "Maven"
        mocker.patch(
            'tools.supplychain.bazbom_cli.detect_build_system',
            return_value=mock_build_system
        )
        
        mock_perform = mocker.patch(
            'tools.supplychain.bazbom_cli.perform_scan',
            return_value=0
        )
        
        # Mock time.sleep to raise KeyboardInterrupt after first sleep
        sleep_count = [0]
        def mock_sleep(seconds):
            sleep_count[0] += 1
            if sleep_count[0] >= 1:
                raise KeyboardInterrupt()
        
        mocker.patch('time.sleep', side_effect=mock_sleep)
        
        args = Mock()
        args.path = str(tmp_path)
        args.watch = True
        
        # Act
        result = bazbom_cli.scan_command(args)
        
        # Assert
        assert result == 0
        assert mock_perform.call_count >= 1
        captured = capsys.readouterr()
        assert "Watch mode enabled" in captured.out

    def test_scan_command_watch_mode_no_build_system_error(
        self, tmp_path, mocker, capsys
    ):
        """Test scan_command watch mode fails when no build system detected."""
        # Arrange
        mock_config = Mock()
        mocker.patch(
            'tools.supplychain.bazbom_cli.BazBOMConfig.find_and_load',
            return_value=mock_config
        )
        
        mocker.patch(
            'tools.supplychain.bazbom_cli.detect_build_system',
            return_value=None
        )
        
        args = Mock()
        args.path = str(tmp_path)
        args.watch = True
        
        # Act
        result = bazbom_cli.scan_command(args)
        
        # Assert
        assert result == 1
        captured = capsys.readouterr()
        assert "ERROR: Could not detect build system" in captured.err


class TestInitCommand:
    """Tests for init_command function."""

    def test_init_command_creates_config_file(self, tmp_path, capsys):
        """Test init_command creates bazbom.yml successfully."""
        # Arrange
        args = Mock()
        args.path = str(tmp_path)
        args.force = False
        
        # Act
        result = bazbom_cli.init_command(args)
        
        # Assert
        assert result == 0
        config_file = tmp_path / "bazbom.yml"
        assert config_file.exists()
        captured = capsys.readouterr()
        assert "Created configuration file" in captured.out

    def test_init_command_fails_when_config_exists(
        self, tmp_path, capsys
    ):
        """Test init_command fails when config exists without force."""
        # Arrange
        config_file = tmp_path / "bazbom.yml"
        config_file.write_text("existing: config")
        
        args = Mock()
        args.path = str(tmp_path)
        args.force = False
        
        # Act
        result = bazbom_cli.init_command(args)
        
        # Assert
        assert result == 1
        captured = capsys.readouterr()
        assert "ERROR: Configuration file already exists" in captured.err
        assert "Use --force to overwrite" in captured.err

    def test_init_command_overwrites_with_force(self, tmp_path, capsys):
        """Test init_command overwrites existing config with force flag."""
        # Arrange
        config_file = tmp_path / "bazbom.yml"
        config_file.write_text("existing: config")
        
        args = Mock()
        args.path = str(tmp_path)
        args.force = True
        
        # Act
        result = bazbom_cli.init_command(args)
        
        # Assert
        assert result == 0
        assert config_file.exists()
        # Verify file was overwritten (content should be different)
        content = config_file.read_text()
        assert "existing: config" not in content

    def test_init_command_handles_io_error(self, tmp_path, mocker, capsys):
        """Test init_command handles IOError gracefully."""
        # Arrange
        mocker.patch(
            'tools.supplychain.bazbom_cli.BazBOMConfig.save',
            side_effect=IOError("Permission denied")
        )
        
        args = Mock()
        args.path = str(tmp_path)
        args.force = False
        
        # Act
        result = bazbom_cli.init_command(args)
        
        # Assert
        assert result == 1
        captured = capsys.readouterr()
        assert "ERROR: Permission denied" in captured.err


class TestVersionCommand:
    """Tests for version_command function."""

    def test_version_command_prints_version(self, capsys):
        """Test version_command prints version information."""
        # Arrange
        args = Mock()
        
        # Act
        result = bazbom_cli.version_command(args)
        
        # Assert
        assert result == 0
        captured = capsys.readouterr()
        assert "BazBOM version" in captured.out
        assert bazbom_cli.__version__ in captured.out
        assert "Bazel-native SBOM" in captured.out


class TestMain:
    """Tests for main function."""

    def test_main_scan_command_executes(self, mocker):
        """Test main function routes to scan command."""
        # Arrange
        mock_scan = mocker.patch(
            'tools.supplychain.bazbom_cli.scan_command',
            return_value=0
        )
        
        mocker.patch.object(
            sys, 'argv',
            ['bazbom', 'scan', '.']
        )
        
        # Act & Assert
        with pytest.raises(SystemExit) as exc_info:
            bazbom_cli.main()
        
        assert exc_info.value.code == 0
        mock_scan.assert_called_once()

    def test_main_init_command_executes(self, mocker):
        """Test main function routes to init command."""
        # Arrange
        mock_init = mocker.patch(
            'tools.supplychain.bazbom_cli.init_command',
            return_value=0
        )
        
        mocker.patch.object(
            sys, 'argv',
            ['bazbom', 'init']
        )
        
        # Act & Assert
        with pytest.raises(SystemExit) as exc_info:
            bazbom_cli.main()
        
        assert exc_info.value.code == 0
        mock_init.assert_called_once()

    def test_main_version_command_executes(self, mocker):
        """Test main function routes to version command."""
        # Arrange
        mock_version = mocker.patch(
            'tools.supplychain.bazbom_cli.version_command',
            return_value=0
        )
        
        mocker.patch.object(
            sys, 'argv',
            ['bazbom', 'version']
        )
        
        # Act & Assert
        with pytest.raises(SystemExit) as exc_info:
            bazbom_cli.main()
        
        assert exc_info.value.code == 0
        mock_version.assert_called_once()

    def test_main_no_command_shows_help(self, mocker, capsys):
        """Test main function shows help when no command provided."""
        # Arrange
        mocker.patch.object(
            sys, 'argv',
            ['bazbom']
        )
        
        # Act & Assert
        with pytest.raises(SystemExit) as exc_info:
            bazbom_cli.main()
        
        assert exc_info.value.code == 1

    def test_main_version_flag(self, mocker):
        """Test main function handles --version flag."""
        # Arrange
        mocker.patch.object(
            sys, 'argv',
            ['bazbom', '--version']
        )
        
        # Act & Assert
        with pytest.raises(SystemExit) as exc_info:
            bazbom_cli.main()
        
        # argparse exits with 0 for --version
        assert exc_info.value.code == 0


class TestModuleAttributes:
    """Tests for module-level attributes."""

    def test_version_defined(self):
        """Test that __version__ is defined."""
        assert hasattr(bazbom_cli, '__version__')
        assert isinstance(bazbom_cli.__version__, str)
        assert len(bazbom_cli.__version__) > 0
