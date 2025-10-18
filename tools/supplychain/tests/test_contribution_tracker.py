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
