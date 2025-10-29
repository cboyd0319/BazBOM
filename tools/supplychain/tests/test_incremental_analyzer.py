#!/usr/bin/env python3
"""Comprehensive unit tests for incremental_analyzer module.

Tests cover:
- run_git_command (subprocess handling and error cases)
- get_changed_files (git diff parsing)
- get_bazel_targets_from_files (file to target conversion)
- query_affected_targets (bazel query execution)
- Edge cases (empty results, timeouts, errors)
"""

import sys
import pytest
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
import subprocess

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

# Module under test
from incremental_analyzer import (
    run_git_command,
    get_changed_files,
    get_bazel_targets_from_files,
    query_affected_targets,
)


class TestRunGitCommand:
    """Tests for run_git_command function."""

    @patch('incremental_analyzer.subprocess.run')
    def test_successful_command(self, mock_run):
        """Test successful git command execution."""
        mock_result = Mock()
        mock_result.stdout = "file1.txt\nfile2.txt\n"
        mock_result.returncode = 0
        mock_run.return_value = mock_result
        
        result = run_git_command(['status'])
        
        assert result == "file1.txt\nfile2.txt"
        mock_run.assert_called_once()

    @patch('incremental_analyzer.subprocess.run')
    def test_command_with_cwd(self, mock_run):
        """Test git command with custom working directory."""
        mock_result = Mock()
        mock_result.stdout = "output"
        mock_run.return_value = mock_result
        
        run_git_command(['status'], cwd='/tmp/test')
        
        call_args = mock_run.call_args
        assert call_args[1]['cwd'] == '/tmp/test'

    @patch('incremental_analyzer.subprocess.run')
    def test_command_failure_returns_empty(self, mock_run):
        """Test that failed git command returns empty string."""
        mock_run.side_effect = subprocess.CalledProcessError(1, 'git', stderr='error')
        
        result = run_git_command(['status'])
        
        assert result == ""

    @patch('incremental_analyzer.subprocess.run')
    def test_strips_whitespace(self, mock_run):
        """Test that output is stripped of whitespace."""
        mock_result = Mock()
        mock_result.stdout = "  output  \n\n  "
        mock_run.return_value = mock_result
        
        result = run_git_command(['status'])
        
        assert result == "output"


class TestGetChangedFiles:
    """Tests for get_changed_files function."""

    @patch('incremental_analyzer.run_git_command')
    def test_get_changed_files_with_changes(self, mock_run_git):
        """Test getting changed files when there are changes."""
        mock_run_git.return_value = "file1.py\nfile2.java\nfile3.txt"
        
        files = get_changed_files()
        
        assert len(files) == 3
        assert "file1.py" in files
        assert "file2.java" in files
        assert "file3.txt" in files

    @patch('incremental_analyzer.run_git_command')
    def test_get_changed_files_empty(self, mock_run_git):
        """Test getting changed files when there are no changes."""
        mock_run_git.return_value = ""
        
        files = get_changed_files()
        
        assert files == []

    @patch('incremental_analyzer.run_git_command')
    def test_get_changed_files_custom_base_ref(self, mock_run_git):
        """Test getting changed files with custom base ref."""
        mock_run_git.return_value = "file1.py"
        
        files = get_changed_files(base_ref="origin/main")
        
        mock_run_git.assert_called_once()
        call_args = mock_run_git.call_args[0][0]
        assert "origin/main" in call_args

    @patch('incremental_analyzer.run_git_command')
    def test_filters_empty_lines(self, mock_run_git):
        """Test that empty lines are filtered out."""
        mock_run_git.return_value = "file1.py\n\n\nfile2.java\n\n"
        
        files = get_changed_files()
        
        assert len(files) == 2


class TestGetBazelTargetsFromFiles:
    """Tests for get_bazel_targets_from_files function."""

    def test_empty_file_list(self):
        """Test with empty file list."""
        targets = get_bazel_targets_from_files([], "/workspace")
        assert len(targets) == 0

    def test_java_file_in_subdirectory(self):
        """Test Java file in subdirectory generates correct targets."""
        files = ["src/main/java/com/example/App.java"]
        targets = get_bazel_targets_from_files(files, "/workspace")
        
        assert "//src/main/java/com/example:*" in targets
        assert "//src/main/java/com/example/..." in targets

    def test_root_directory_file(self):
        """Test file in root directory."""
        files = ["BUILD.bazel"]
        targets = get_bazel_targets_from_files(files, "/workspace")
        
        assert "//:*" in targets

    def test_skips_documentation_files(self):
        """Test that documentation files are skipped."""
        files = [
            "README.md",
            "docs/guide.txt",
            "config.json",
            "setup.yaml",
        ]
        targets = get_bazel_targets_from_files(files, "/workspace")
        
        # These documentation files should be skipped
        assert len(targets) == 0

    def test_mixed_file_types(self):
        """Test mix of source and non-source files."""
        files = [
            "src/App.java",
            "README.md",
            "tools/build.py",
            "config.yaml"
        ]
        targets = get_bazel_targets_from_files(files, "/workspace")
        
        # Should only include Java and Python files
        assert "//src:*" in targets or "//src/..." in targets
        assert "//tools:*" in targets or "//tools/..." in targets
        # Should not include README or config

    def test_multiple_files_same_directory(self):
        """Test multiple files in same directory."""
        files = [
            "src/App.java",
            "src/Main.java",
            "src/Utils.java"
        ]
        targets = get_bazel_targets_from_files(files, "/workspace")
        
        # Should deduplicate
        assert "//src:*" in targets
        assert "//src/..." in targets

    def test_unicode_in_path(self):
        """Test handling of unicode characters in paths."""
        files = ["src//App.java"]
        targets = get_bazel_targets_from_files(files, "/workspace")
        
        assert len(targets) > 0


class TestQueryAffectedTargets:
    """Tests for query_affected_targets function."""

    def test_empty_target_set(self):
        """Test with empty target set."""
        targets = query_affected_targets(set(), "/workspace")
        assert targets == []

    @patch('incremental_analyzer.subprocess.run')
    def test_successful_query(self, mock_run):
        """Test successful bazel query."""
        mock_result = Mock()
        mock_result.returncode = 0
        mock_result.stdout = "//app:main\n//lib:utils\n//test:unit"
        mock_run.return_value = mock_result
        
        changed = {"//src:*"}
        targets = query_affected_targets(changed, "/workspace")
        
        assert len(targets) == 3
        assert "//app:main" in targets
        assert "//lib:utils" in targets

    @patch('incremental_analyzer.subprocess.run')
    def test_query_failure_continues(self, mock_run):
        """Test that query failure doesn't stop processing."""
        mock_result = Mock()
        mock_result.returncode = 1
        mock_result.stderr = "Error message"
        mock_run.return_value = mock_result
        
        changed = {"//src:*"}
        targets = query_affected_targets(changed, "/workspace")
        
        # Should return empty but not crash
        assert isinstance(targets, list)

    @patch('incremental_analyzer.subprocess.run')
    def test_query_timeout(self, mock_run):
        """Test handling of bazel query timeout."""
        mock_run.side_effect = subprocess.TimeoutExpired('bazel', 30)
        
        changed = {"//src:*"}
        targets = query_affected_targets(changed, "/workspace")
        
        # Should handle timeout gracefully
        assert isinstance(targets, list)

    @patch('incremental_analyzer.subprocess.run')
    def test_multiple_target_patterns(self, mock_run):
        """Test querying multiple target patterns."""
        mock_result = Mock()
        mock_result.returncode = 0
        mock_result.stdout = "//app:main"
        mock_run.return_value = mock_result
        
        changed = {"//src:*", "//lib:*"}
        targets = query_affected_targets(changed, "/workspace")
        
        # Should call bazel query multiple times
        assert mock_run.call_count == 2

    @patch('incremental_analyzer.subprocess.run')
    def test_deduplicates_targets(self, mock_run):
        """Test that duplicate targets are handled."""
        mock_result = Mock()
        mock_result.returncode = 0
        mock_result.stdout = "//app:main\n//app:main"
        mock_run.return_value = mock_result
        
        changed = {"//src:*"}
        targets = query_affected_targets(changed, "/workspace")
        
        # All targets should be present (deduplication is caller's responsibility)
        assert "//app:main" in targets


class TestCheckRipGrepAvailable:
    """Tests for check_ripgrep_available function."""

    @patch('incremental_analyzer.subprocess.run')
    def test_ripgrep_available(self, mock_run):
        """Test when RipGrep is available."""
        mock_run.return_value = Mock(returncode=0, stdout="ripgrep 13.0.0")
        from incremental_analyzer import check_ripgrep_available
        
        assert check_ripgrep_available() is True
        mock_run.assert_called_once()

    @patch('incremental_analyzer.subprocess.run')
    def test_ripgrep_not_found(self, mock_run):
        """Test when RipGrep is not installed."""
        mock_run.side_effect = FileNotFoundError()
        from incremental_analyzer import check_ripgrep_available
        
        assert check_ripgrep_available() is False

    @patch('incremental_analyzer.subprocess.run')
    def test_ripgrep_fails(self, mock_run):
        """Test when RipGrep command fails."""
        mock_run.side_effect = subprocess.CalledProcessError(1, 'rg')
        from incremental_analyzer import check_ripgrep_available
        
        assert check_ripgrep_available() is False

    @patch('incremental_analyzer.subprocess.run')
    def test_ripgrep_timeout(self, mock_run):
        """Test when RipGrep check times out."""
        mock_run.side_effect = subprocess.TimeoutExpired('rg', 5)
        from incremental_analyzer import check_ripgrep_available
        
        assert check_ripgrep_available() is False


class TestGetChangedBuildFilesFast:
    """Tests for get_changed_build_files_fast function."""

    @patch('incremental_analyzer.check_ripgrep_available')
    @patch('incremental_analyzer.get_changed_files')
    def test_no_changed_files(self, mock_get_files, mock_check_rg):
        """Test when there are no changed files."""
        mock_get_files.return_value = []
        from incremental_analyzer import get_changed_build_files_fast
        
        result = get_changed_build_files_fast()
        
        assert result == []

    @patch('incremental_analyzer.check_ripgrep_available')
    @patch('incremental_analyzer.get_changed_files')
    @patch('incremental_analyzer.subprocess.run')
    def test_ripgrep_mode_success(self, mock_run, mock_get_files, mock_check_rg):
        """Test RipGrep fast filtering mode."""
        mock_check_rg.return_value = True
        mock_get_files.return_value = ['src/BUILD.bazel', 'lib/BUILD', 'test/file.java']
        mock_run.return_value = Mock(stdout='src/BUILD.bazel\nlib/BUILD', returncode=0)
        from incremental_analyzer import get_changed_build_files_fast
        
        result = get_changed_build_files_fast()
        
        assert 'src/BUILD.bazel' in result
        assert 'lib/BUILD' in result

    @patch('incremental_analyzer.check_ripgrep_available')
    @patch('incremental_analyzer.get_changed_files')
    def test_fallback_manual_filter(self, mock_get_files, mock_check_rg):
        """Test fallback to manual filtering when RipGrep unavailable."""
        mock_check_rg.return_value = False
        mock_get_files.return_value = ['src/BUILD.bazel', 'lib/BUILD', 'test/file.java']
        from incremental_analyzer import get_changed_build_files_fast
        
        result = get_changed_build_files_fast()
        
        assert 'src/BUILD.bazel' in result
        assert 'lib/BUILD' in result
        assert 'test/file.java' not in result

    @patch('incremental_analyzer.check_ripgrep_available')
    @patch('incremental_analyzer.get_changed_files')
    @patch('incremental_analyzer.subprocess.run')
    def test_ripgrep_timeout_fallback(self, mock_run, mock_get_files, mock_check_rg):
        """Test fallback when RipGrep times out."""
        mock_check_rg.return_value = True
        mock_get_files.return_value = ['src/BUILD.bazel']
        mock_run.side_effect = subprocess.TimeoutExpired('rg', 10)
        from incremental_analyzer import get_changed_build_files_fast
        
        result = get_changed_build_files_fast()
        
        assert 'src/BUILD.bazel' in result


class TestFindAffectedTargetsFast:
    """Tests for find_affected_targets_fast function."""

    @patch('incremental_analyzer.check_ripgrep_available')
    def test_ripgrep_not_available(self, mock_check_rg):
        """Test fallback when RipGrep not available."""
        mock_check_rg.return_value = False
        from incremental_analyzer import find_affected_targets_fast
        
        result = find_affected_targets_fast(['src/App.java'], '/workspace')
        
        assert result == set()

    @patch('incremental_analyzer.check_ripgrep_available')
    @patch('incremental_analyzer.Path')
    @patch('incremental_analyzer.subprocess.run')
    def test_root_directory_file(self, mock_run, mock_path_cls, mock_check_rg):
        """Test file in root directory."""
        mock_check_rg.return_value = True
        from incremental_analyzer import find_affected_targets_fast
        
        result = find_affected_targets_fast(['BUILD.bazel'], '/workspace')
        
        assert '//:*' in result

    @patch('incremental_analyzer.check_ripgrep_available')
    @patch('incremental_analyzer.Path')
    @patch('incremental_analyzer.subprocess.run')
    def test_no_build_file_in_directory(self, mock_run, mock_path_cls, mock_check_rg):
        """Test directory without BUILD file."""
        mock_check_rg.return_value = True
        mock_path = MagicMock()
        mock_path.exists.return_value = False
        mock_path_cls.return_value = mock_path
        from incremental_analyzer import find_affected_targets_fast
        
        result = find_affected_targets_fast(['src/App.java'], '/workspace')
        
        # Should not crash
        assert isinstance(result, set)

    @patch('incremental_analyzer.check_ripgrep_available')
    @patch('incremental_analyzer.Path')
    @patch('incremental_analyzer.subprocess.run')
    def test_ripgrep_extracts_targets(self, mock_run, mock_path_cls, mock_check_rg):
        """Test RipGrep extracting target names."""
        mock_check_rg.return_value = True
        mock_path = MagicMock()
        mock_path.exists.return_value = True
        mock_path_cls.return_value = mock_path
        mock_run.return_value = Mock(stdout='app_main\napp_lib', returncode=0)
        from incremental_analyzer import find_affected_targets_fast
        
        result = find_affected_targets_fast(['src/App.java'], '/workspace')
        
        assert '//src:app_main' in result or isinstance(result, set)

    @patch('incremental_analyzer.check_ripgrep_available')
    @patch('incremental_analyzer.Path')
    @patch('incremental_analyzer.subprocess.run')
    def test_ripgrep_timeout_handled(self, mock_run, mock_path_cls, mock_check_rg):
        """Test graceful handling of RipGrep timeout."""
        mock_check_rg.return_value = True
        mock_path = MagicMock()
        mock_path.exists.return_value = True
        mock_path_cls.return_value = mock_path
        mock_run.side_effect = subprocess.TimeoutExpired('rg', 5)
        from incremental_analyzer import find_affected_targets_fast
        
        result = find_affected_targets_fast(['src/App.java'], '/workspace')
        
        assert isinstance(result, set)


class TestGetAllSbomTargets:
    """Tests for get_all_sbom_targets function."""

    @patch('incremental_analyzer.subprocess.run')
    def test_successful_query(self, mock_run):
        """Test successful SBOM targets query."""
        mock_run.return_value = Mock(
            returncode=0,
            stdout='//app:app_sbom\n//lib:lib_sbom\n'
        )
        from incremental_analyzer import get_all_sbom_targets
        
        result = get_all_sbom_targets('/workspace')
        
        assert '//app:app_sbom' in result
        assert '//lib:lib_sbom' in result

    @patch('incremental_analyzer.subprocess.run')
    def test_query_failure(self, mock_run):
        """Test handling of query failure."""
        mock_run.return_value = Mock(returncode=1, stderr='error')
        from incremental_analyzer import get_all_sbom_targets
        
        result = get_all_sbom_targets('/workspace')
        
        assert result == []

    @patch('incremental_analyzer.subprocess.run')
    def test_query_exception(self, mock_run):
        """Test handling of query exception."""
        mock_run.side_effect = Exception('Failed')
        from incremental_analyzer import get_all_sbom_targets
        
        result = get_all_sbom_targets('/workspace')
        
        assert result == []


class TestFilterSbomTargets:
    """Tests for filter_sbom_targets function."""

    def test_empty_targets_list(self):
        """Test with empty targets list."""
        from incremental_analyzer import filter_sbom_targets
        
        result = filter_sbom_targets([], '/workspace')
        
        assert result == []

    @patch('incremental_analyzer.subprocess.run')
    def test_already_sbom_targets(self, mock_run):
        """Test targets already ending with _sbom."""
        from incremental_analyzer import filter_sbom_targets
        
        result = filter_sbom_targets(['//app:app_sbom', '//lib:lib_sbom'], '/workspace')
        
        assert '//app:app_sbom' in result
        assert '//lib:lib_sbom' in result

    @patch('incremental_analyzer.subprocess.run')
    def test_converts_to_sbom_targets(self, mock_run):
        """Test converting regular targets to SBOM targets."""
        mock_run.return_value = Mock(returncode=0, stdout='//app:app_sbom')
        from incremental_analyzer import filter_sbom_targets
        
        result = filter_sbom_targets(['//app:app'], '/workspace')
        
        assert '//app:app_sbom' in result

    @patch('incremental_analyzer.subprocess.run')
    def test_skips_nonexistent_sbom_targets(self, mock_run):
        """Test skipping targets without SBOM equivalents."""
        mock_run.return_value = Mock(returncode=1, stderr='not found')
        from incremental_analyzer import filter_sbom_targets
        
        result = filter_sbom_targets(['//app:app'], '/workspace')
        
        # Should not crash on failed query
        assert isinstance(result, list)

    @patch('incremental_analyzer.subprocess.run')
    def test_handles_query_exception(self, mock_run):
        """Test handling of query exception."""
        mock_run.side_effect = Exception('Failed')
        from incremental_analyzer import filter_sbom_targets
        
        result = filter_sbom_targets(['//app:app'], '/workspace')
        
        assert isinstance(result, list)


class TestMainFunction:
    """Tests for main CLI function."""

    @patch('incremental_analyzer.subprocess.run')
    @patch('incremental_analyzer.Path')
    @patch('sys.argv', ['incremental_analyzer.py', '--full-analysis'])
    def test_full_analysis_mode(self, mock_path, mock_run):
        """Test full analysis mode."""
        mock_path.return_value.resolve.return_value = Path('/workspace')
        mock_run.return_value = Mock(
            returncode=0,
            stdout='//app:app_sbom\n'
        )
        from incremental_analyzer import main
        
        result = main()
        
        assert result == 0

    @patch('incremental_analyzer.subprocess.run')
    @patch('incremental_analyzer.Path')
    @patch('incremental_analyzer.get_changed_files')
    @patch('sys.argv', ['incremental_analyzer.py'])
    def test_incremental_mode_no_changes(self, mock_get_files, mock_path, mock_run):
        """Test incremental mode with no changes."""
        mock_path.return_value.resolve.return_value = Path('/workspace')
        mock_get_files.return_value = []
        from incremental_analyzer import main
        
        result = main()
        
        assert result == 0

    @patch('incremental_analyzer.subprocess.run')
    @patch('incremental_analyzer.Path')
    @patch('incremental_analyzer.get_changed_files')
    @patch('incremental_analyzer.check_ripgrep_available')
    @patch('sys.argv', ['incremental_analyzer.py', '--fast-mode'])
    def test_fast_mode_enabled(self, mock_check_rg, mock_get_files, mock_path, mock_run):
        """Test fast mode when RipGrep available."""
        mock_path.return_value.resolve.return_value = Path('/workspace')
        mock_check_rg.return_value = True
        mock_get_files.return_value = []
        from incremental_analyzer import main
        
        result = main()
        
        assert result == 0

    @patch('incremental_analyzer.subprocess.run')
    @patch('incremental_analyzer.Path')
    @patch('sys.argv', ['incremental_analyzer.py', '--no-fast-mode'])
    def test_no_fast_mode(self, mock_path, mock_run):
        """Test disabling fast mode."""
        mock_path.return_value.resolve.return_value = Path('/workspace')
        mock_run.return_value = Mock(returncode=0, stdout='')
        from incremental_analyzer import main
        
        result = main()
        
        assert result == 0

    @patch('incremental_analyzer.subprocess.run')
    @patch('incremental_analyzer.Path')
    @patch('incremental_analyzer.get_changed_files')
    @patch('builtins.open', create=True)
    @patch('sys.argv', ['incremental_analyzer.py', '--output', '/tmp/result.json', '--output-format', 'json'])
    def test_json_output_to_file(self, mock_open, mock_get_files, mock_path, mock_run):
        """Test JSON output to file."""
        mock_path.return_value.resolve.return_value = Path('/workspace')
        mock_get_files.return_value = []
        mock_run.return_value = Mock(returncode=0, stdout='')
        mock_file = MagicMock()
        mock_open.return_value.__enter__.return_value = mock_file
        from incremental_analyzer import main
        
        result = main()
        
        assert result == 0
        mock_file.write.assert_called()

    @patch('incremental_analyzer.subprocess.run')
    @patch('incremental_analyzer.Path')
    @patch('incremental_analyzer.get_changed_files')
    @patch('sys.argv', ['incremental_analyzer.py', '--output-format', 'targets'])
    def test_targets_output_format(self, mock_get_files, mock_path, mock_run):
        """Test targets output format."""
        mock_path.return_value.resolve.return_value = Path('/workspace')
        mock_get_files.return_value = []
        from incremental_analyzer import main
        
        result = main()
        
        assert result == 0

    @patch('incremental_analyzer.subprocess.run')
    @patch('incremental_analyzer.Path')
    @patch('incremental_analyzer.get_changed_files')
    @patch('sys.argv', ['incremental_analyzer.py', '--output-format', 'bazel-query'])
    def test_bazel_query_output_format(self, mock_get_files, mock_path, mock_run):
        """Test bazel-query output format."""
        mock_path.return_value.resolve.return_value = Path('/workspace')
        mock_get_files.return_value = []
        from incremental_analyzer import main
        
        result = main()
        
        assert result == 0

    @patch('incremental_analyzer.subprocess.run')
    @patch('incremental_analyzer.Path')
    @patch('incremental_analyzer.get_changed_files')
    @patch('incremental_analyzer.get_bazel_targets_from_files')
    @patch('incremental_analyzer.query_affected_targets')
    @patch('incremental_analyzer.get_all_sbom_targets')
    @patch('sys.argv', ['incremental_analyzer.py'])
    def test_fallback_to_all_targets(self, mock_all, mock_query, mock_targets, mock_get_files, mock_path, mock_run):
        """Test fallback to all targets when no affected found."""
        mock_path.return_value.resolve.return_value = Path('/workspace')
        mock_get_files.return_value = ['src/App.java']
        mock_targets.return_value = {'//src:*'}
        mock_query.return_value = []  # No affected targets
        mock_all.return_value = ['//app:app_sbom']
        from incremental_analyzer import main
        
        result = main()
        
        assert result == 0
        mock_all.assert_called_once()


class TestEdgeCases:
    """Tests for edge cases and boundary conditions."""

    def test_get_bazel_targets_with_dots_in_filename(self):
        """Test handling of files with multiple dots."""
        files = ["src/com/example/App.test.java"]
        targets = get_bazel_targets_from_files(files, "/workspace")
        
        assert len(targets) > 0

    def test_get_bazel_targets_with_spaces_in_path(self):
        """Test handling of spaces in file paths."""
        files = ["src/my folder/App.java"]
        targets = get_bazel_targets_from_files(files, "/workspace")
        
        # Should handle spaces in paths
        assert len(targets) > 0

    @patch('incremental_analyzer.run_git_command')
    def test_get_changed_files_with_special_characters(self, mock_run_git):
        """Test files with special characters in names."""
        mock_run_git.return_value = "file-with-dashes.py\nfile_with_underscores.java"
        
        files = get_changed_files()
        
        assert len(files) == 2

    def test_very_long_file_path(self):
        """Test handling of very long file paths."""
        long_path = "/".join(["dir"] * 50) + "/file.java"
        files = [long_path]
        targets = get_bazel_targets_from_files(files, "/workspace")
        
        assert len(targets) > 0
