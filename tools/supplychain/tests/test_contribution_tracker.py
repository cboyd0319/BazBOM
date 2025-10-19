#!/usr/bin/env python3
"""Tests for contribution_tracker.py"""

import json
import sys
from pathlib import Path
from unittest.mock import Mock, patch, mock_open

import pytest

sys.path.insert(0, str(Path(__file__).parent.parent))

from contribution_tracker import ContributionTracker


class TestContributionTracker:
    """Tests for ContributionTracker class."""

    def test_init_creates_new_tracker(self, tmp_path):
        """Test initialization creates new tracker file."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        assert tracker.tracker_file == str(tracker_file)
        assert tracker.contributions is not None
        assert "version" in tracker.contributions

    def test_load_contributions_creates_default_structure(self, tmp_path):
        """Test loading creates default structure when file doesn't exist."""
        tracker_file = tmp_path / "nonexistent.json"
        tracker = ContributionTracker(str(tracker_file))
        
        contrib = tracker.contributions
        assert contrib["version"] == "1.0"
        assert contrib["total_contributions"] == 0
        assert "contributions" in contrib
        assert "statistics" in contrib

    def test_load_contributions_from_existing_file(self, tmp_path):
        """Test loading contributions from existing file."""
        tracker_file = tmp_path / "tracker.json"
        existing_data = {
            "version": "1.0",
            "total_contributions": 5,
            "contributions": [],
            "statistics": {}
        }
        tracker_file.write_text(json.dumps(existing_data))
        
        tracker = ContributionTracker(str(tracker_file))
        
        assert tracker.contributions["total_contributions"] == 5

    def test_load_contributions_handles_corrupted_file(self, tmp_path):
        """Test loading handles corrupted JSON file."""
        tracker_file = tmp_path / "tracker.json"
        tracker_file.write_text("invalid json{")
        
        tracker = ContributionTracker(str(tracker_file))
        
        # Should create fresh tracker
        assert tracker.contributions["total_contributions"] == 0

    def test_add_contribution_basic(self, tmp_path):
        """Test adding a basic contribution."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution(
            vulnerability_id="CVE-2024-1234",
            package_name="test-package",
            ecosystem="Maven"
        )
        
        tracker._save_contributions()
        
        # Verify saved
        assert tracker_file.exists()
        data = json.loads(tracker_file.read_text())
        assert data["total_contributions"] == 1
        assert len(data["contributions"]) == 1

    def test_add_contribution_with_severity(self, tmp_path):
        """Test adding contribution with severity."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution(
            vulnerability_id="CVE-2024-5678",
            package_name="vuln-pkg",
            ecosystem="npm",
            severity="CRITICAL",
            contributor="test-user",
            notes="Test notes"
        )
        
        assert tracker.contributions["total_contributions"] == 1

    def test_save_contributions_writes_json(self, tmp_path):
        """Test save_contributions writes valid JSON."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution(
            vulnerability_id="TEST-001",
            package_name="pkg",
            ecosystem="PyPI"
        )
        
        tracker._save_contributions()
        
        # Verify file is valid JSON
        with open(tracker_file) as f:
            data = json.load(f)
            assert "total_contributions" in data

    def test_save_contributions_io_error(self, tmp_path, mocker):
        """Test save_contributions handles IO errors."""
        tracker_file = "/invalid/path/tracker.json"
        tracker = ContributionTracker(tracker_file)
        
        with pytest.raises(IOError):
            tracker._save_contributions()

    def test_get_statistics_empty_tracker(self, tmp_path):
        """Test getting statistics from empty tracker."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        stats = tracker.get_statistics()
        
        assert "by_ecosystem" in stats
        assert "by_severity" in stats
        assert "by_year" in stats
        assert "by_contributor" in stats

    def test_get_statistics_with_contributions(self, tmp_path):
        """Test getting statistics with contributions."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        tracker.add_contribution("CVE-2", "pkg2", "Maven", "CRITICAL")
        tracker.add_contribution("CVE-3", "pkg3", "npm", "MEDIUM")
        
        stats = tracker.get_statistics()
        
        assert stats["by_ecosystem"]["Maven"] == 2
        assert stats["by_ecosystem"]["npm"] == 1
        assert tracker.contributions["total_contributions"] == 3

    def test_add_contribution_missing_vulnerability_id(self, tmp_path):
        """Test add_contribution raises error for missing vulnerability_id."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        with pytest.raises(ValueError, match="vulnerability_id is required"):
            tracker.add_contribution("", "pkg", "Maven")

    def test_add_contribution_missing_package_name(self, tmp_path):
        """Test add_contribution raises error for missing package_name."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        with pytest.raises(ValueError, match="package_name is required"):
            tracker.add_contribution("CVE-123", "", "Maven")

    def test_add_contribution_missing_ecosystem(self, tmp_path):
        """Test add_contribution raises error for missing ecosystem."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        with pytest.raises(ValueError, match="ecosystem is required"):
            tracker.add_contribution("CVE-123", "pkg", "")

    def test_add_contribution_duplicate(self, tmp_path):
        """Test add_contribution raises error for duplicates."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-123", "pkg", "Maven")
        
        with pytest.raises(ValueError, match="already recorded"):
            tracker.add_contribution("CVE-123", "pkg2", "npm")

    def test_get_contributions_no_filters(self, tmp_path):
        """Test get_contributions with no filters."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        tracker.add_contribution("CVE-2", "pkg2", "npm", "CRITICAL")
        
        contributions = tracker.get_contributions()
        
        assert len(contributions) == 2

    def test_get_contributions_filter_by_ecosystem(self, tmp_path):
        """Test get_contributions filtered by ecosystem."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        tracker.add_contribution("CVE-2", "pkg2", "npm", "CRITICAL")
        
        contributions = tracker.get_contributions(ecosystem="Maven")
        
        assert len(contributions) == 1
        assert contributions[0]["id"] == "CVE-1"

    def test_get_contributions_filter_by_contributor(self, tmp_path):
        """Test get_contributions filtered by contributor."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH", "alice")
        tracker.add_contribution("CVE-2", "pkg2", "npm", "CRITICAL", "bob")
        
        contributions = tracker.get_contributions(contributor="alice")
        
        assert len(contributions) == 1
        assert contributions[0]["contributor"] == "alice"

    def test_get_contributions_filter_by_severity(self, tmp_path):
        """Test get_contributions filtered by severity."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        tracker.add_contribution("CVE-2", "pkg2", "npm", "CRITICAL")
        
        contributions = tracker.get_contributions(severity="CRITICAL")
        
        assert len(contributions) == 1
        assert contributions[0]["severity"] == "CRITICAL"

    def test_get_contributions_multiple_filters(self, tmp_path):
        """Test get_contributions with multiple filters."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH", "alice")
        tracker.add_contribution("CVE-2", "pkg2", "Maven", "CRITICAL", "bob")
        tracker.add_contribution("CVE-3", "pkg3", "npm", "HIGH", "alice")
        
        contributions = tracker.get_contributions(ecosystem="Maven", contributor="alice")
        
        assert len(contributions) == 1
        assert contributions[0]["id"] == "CVE-1"

    def test_generate_report_empty(self, tmp_path):
        """Test generate_report for empty tracker."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        report = tracker.generate_report()
        
        assert "No contributions recorded yet" in report
        assert "Total Contributions: 0" in report

    def test_generate_report_with_contributions(self, tmp_path):
        """Test generate_report with contributions."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH", "alice")
        tracker.add_contribution("CVE-2", "pkg2", "npm", "CRITICAL", "bob")
        
        report = tracker.generate_report()
        
        assert "Total Contributions: 2" in report
        assert "BY ECOSYSTEM" in report
        assert "BY SEVERITY" in report
        assert "BY YEAR" in report
        assert "TOP CONTRIBUTORS" in report
        assert "ACHIEVEMENT BADGES" in report

    def test_calculate_badges_first_contribution(self, tmp_path):
        """Test badge calculation for first contribution."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        
        badges = tracker._calculate_badges(1, tracker.contributions['statistics'])
        
        first_badge = next(b for b in badges if b['name'] == "First Contribution")
        assert first_badge['achieved'] is True

    def test_calculate_badges_active_contributor(self, tmp_path):
        """Test badge calculation for active contributor."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        for i in range(10):
            tracker.add_contribution(f"CVE-{i}", f"pkg{i}", "Maven", "HIGH")
        
        badges = tracker._calculate_badges(10, tracker.contributions['statistics'])
        
        active_badge = next(b for b in badges if b['name'] == "Active Contributor")
        assert active_badge['achieved'] is True

    def test_calculate_badges_multi_ecosystem(self, tmp_path):
        """Test badge calculation for multi-ecosystem."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        tracker.add_contribution("CVE-2", "pkg2", "npm", "MEDIUM")
        tracker.add_contribution("CVE-3", "pkg3", "PyPI", "LOW")
        
        badges = tracker._calculate_badges(3, tracker.contributions['statistics'])
        
        multi_badge = next(b for b in badges if b['name'] == "Multi-Ecosystem")
        assert multi_badge['achieved'] is True

    def test_calculate_badges_critical_finder(self, tmp_path):
        """Test badge calculation for critical finder."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        for i in range(5):
            tracker.add_contribution(f"CVE-{i}", f"pkg{i}", "Maven", "CRITICAL")
        
        badges = tracker._calculate_badges(5, tracker.contributions['statistics'])
        
        critical_badge = next(b for b in badges if b['name'] == "Critical Finder")
        assert critical_badge['achieved'] is True

    def test_update_statistics_by_ecosystem(self, tmp_path):
        """Test statistics update for ecosystem."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        tracker.add_contribution("CVE-2", "pkg2", "Maven", "MEDIUM")
        
        stats = tracker.get_statistics()
        assert stats['by_ecosystem']['Maven'] == 2

    def test_update_statistics_by_severity(self, tmp_path):
        """Test statistics update for severity."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        tracker.add_contribution("CVE-2", "pkg2", "npm", "HIGH")
        
        stats = tracker.get_statistics()
        assert stats['by_severity']['HIGH'] == 2

    def test_update_statistics_by_year(self, tmp_path):
        """Test statistics update for year."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        
        stats = tracker.get_statistics()
        assert len(stats['by_year']) > 0

    def test_update_statistics_by_contributor(self, tmp_path):
        """Test statistics update for contributor."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH", "alice")
        tracker.add_contribution("CVE-2", "pkg2", "npm", "MEDIUM", "alice")
        
        stats = tracker.get_statistics()
        assert stats['by_contributor']['alice'] == 2

    def test_contribution_default_severity(self, tmp_path):
        """Test contribution with default severity."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven")
        
        contrib = tracker.contributions['contributions'][0]
        assert contrib['severity'] == "UNKNOWN"

    def test_contribution_stores_notes(self, tmp_path):
        """Test contribution stores notes."""
        tracker_file = tmp_path / "tracker.json"
        tracker = ContributionTracker(str(tracker_file))
        
        tracker.add_contribution("CVE-1", "pkg1", "Maven", notes="Test note")
        
        contrib = tracker.contributions['contributions'][0]
        assert contrib['notes'] == "Test note"


class TestContributionTrackerCLI:
    """Tests for contribution_tracker CLI."""

    def test_main_add_action(self, tmp_path, monkeypatch, capsys):
        """Test main with add action."""
        from contribution_tracker import main
        
        tracker_file = tmp_path / "tracker.json"
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'add',
             '--tracker-file', str(tracker_file),
             '--id', 'CVE-2024-1234',
             '--package', 'test-pkg',
             '--ecosystem', 'Maven',
             '--severity', 'HIGH']
        )
        
        result = main()
        
        assert result == 0
        captured = capsys.readouterr()
        assert "Added contribution" in captured.err

    def test_main_add_action_missing_required_fields(self, tmp_path, monkeypatch, capsys):
        """Test main with add action missing required fields."""
        from contribution_tracker import main
        
        tracker_file = tmp_path / "tracker.json"
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'add',
             '--tracker-file', str(tracker_file),
             '--id', 'CVE-2024-1234']
        )
        
        result = main()
        
        assert result == 1
        captured = capsys.readouterr()
        assert "ERROR" in captured.err

    def test_main_list_action(self, tmp_path, monkeypatch, capsys):
        """Test main with list action."""
        from contribution_tracker import main
        
        # Setup tracker with data
        tracker_file = tmp_path / "tracker.json"
        from contribution_tracker import ContributionTracker
        tracker = ContributionTracker(str(tracker_file))
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'list',
             '--tracker-file', str(tracker_file)]
        )
        
        result = main()
        
        assert result == 0
        captured = capsys.readouterr()
        assert "Found" in captured.out
        assert "CVE-1" in captured.out

    def test_main_list_action_with_filters(self, tmp_path, monkeypatch, capsys):
        """Test main with list action and filters."""
        from contribution_tracker import main
        
        # Setup tracker with data
        tracker_file = tmp_path / "tracker.json"
        from contribution_tracker import ContributionTracker
        tracker = ContributionTracker(str(tracker_file))
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH", "alice")
        tracker.add_contribution("CVE-2", "pkg2", "npm", "CRITICAL", "bob")
        
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'list',
             '--tracker-file', str(tracker_file),
             '--ecosystem', 'Maven']
        )
        
        result = main()
        
        assert result == 0
        captured = capsys.readouterr()
        assert "CVE-1" in captured.out

    def test_main_report_action(self, tmp_path, monkeypatch, capsys):
        """Test main with report action."""
        from contribution_tracker import main
        
        # Setup tracker with data
        tracker_file = tmp_path / "tracker.json"
        from contribution_tracker import ContributionTracker
        tracker = ContributionTracker(str(tracker_file))
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'report',
             '--tracker-file', str(tracker_file)]
        )
        
        result = main()
        
        assert result == 0
        captured = capsys.readouterr()
        assert "CONTRIBUTION REPORT" in captured.out

    def test_main_export_action_to_stdout(self, tmp_path, monkeypatch, capsys):
        """Test main with export action to stdout."""
        from contribution_tracker import main
        
        # Setup tracker with data
        tracker_file = tmp_path / "tracker.json"
        from contribution_tracker import ContributionTracker
        tracker = ContributionTracker(str(tracker_file))
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'export',
             '--tracker-file', str(tracker_file)]
        )
        
        result = main()
        
        assert result == 0
        captured = capsys.readouterr()
        output = json.loads(captured.out)
        assert "metadata" in output
        assert "contributions" in output
        assert "statistics" in output

    def test_main_export_action_to_file(self, tmp_path, monkeypatch, capsys):
        """Test main with export action to file."""
        from contribution_tracker import main
        
        # Setup tracker with data
        tracker_file = tmp_path / "tracker.json"
        output_file = tmp_path / "export.json"
        from contribution_tracker import ContributionTracker
        tracker = ContributionTracker(str(tracker_file))
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'export',
             '--tracker-file', str(tracker_file),
             '--output', str(output_file)]
        )
        
        result = main()
        
        assert result == 0
        assert output_file.exists()
        data = json.loads(output_file.read_text())
        assert "metadata" in data

    def test_main_value_error_handling(self, tmp_path, monkeypatch, capsys):
        """Test main handles ValueError."""
        from contribution_tracker import main
        
        # Setup tracker with duplicate
        tracker_file = tmp_path / "tracker.json"
        from contribution_tracker import ContributionTracker
        tracker = ContributionTracker(str(tracker_file))
        tracker.add_contribution("CVE-1", "pkg1", "Maven")
        
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'add',
             '--tracker-file', str(tracker_file),
             '--id', 'CVE-1',
             '--package', 'pkg2',
             '--ecosystem', 'npm']
        )
        
        result = main()
        
        assert result == 2
        captured = capsys.readouterr()
        assert "ERROR" in captured.err

    def test_main_exception_handling(self, tmp_path, monkeypatch, capsys):
        """Test main handles unexpected exceptions."""
        from contribution_tracker import main
        
        # Create invalid tracker file setup
        tracker_file = tmp_path / "tracker.json"
        
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'report',
             '--tracker-file', str(tracker_file)]
        )
        
        # Force exception by patching
        with patch('contribution_tracker.ContributionTracker') as mock_tracker:
            mock_tracker.side_effect = RuntimeError("Test error")
            result = main()
        
        assert result == 3

    def test_main_add_with_notes(self, tmp_path, monkeypatch, capsys):
        """Test main add action with notes."""
        from contribution_tracker import main
        
        tracker_file = tmp_path / "tracker.json"
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'add',
             '--tracker-file', str(tracker_file),
             '--id', 'CVE-2024-1234',
             '--package', 'test-pkg',
             '--ecosystem', 'Maven',
             '--notes', 'Important security fix']
        )
        
        result = main()
        
        assert result == 0

    def test_main_list_with_notes_display(self, tmp_path, monkeypatch, capsys):
        """Test main list action displays notes."""
        from contribution_tracker import main
        
        # Setup tracker with data including notes
        tracker_file = tmp_path / "tracker.json"
        from contribution_tracker import ContributionTracker
        tracker = ContributionTracker(str(tracker_file))
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH", notes="Test note")
        
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'list',
             '--tracker-file', str(tracker_file)]
        )
        
        result = main()
        
        assert result == 0
        captured = capsys.readouterr()
        assert "Test note" in captured.out

    def test_main_file_not_found_error(self, tmp_path, monkeypatch, capsys):
        """Test main handles FileNotFoundError during export."""
        from contribution_tracker import main
        
        # Setup a valid tracker
        tracker_file = tmp_path / "tracker.json"
        from contribution_tracker import ContributionTracker
        tracker = ContributionTracker(str(tracker_file))
        tracker.add_contribution("CVE-1", "pkg1", "Maven", "HIGH")
        
        # Try to export to an invalid directory
        output_file = tmp_path / "nonexistent" / "output.json"
        
        monkeypatch.setattr(
            sys, 'argv',
            ['contribution_tracker.py', 'export',
             '--tracker-file', str(tracker_file),
             '--output', str(output_file)]
        )
        
        result = main()
        
        # Should return 1 due to FileNotFoundError
        assert result == 1
        captured = capsys.readouterr()
        assert "ERROR" in captured.err
