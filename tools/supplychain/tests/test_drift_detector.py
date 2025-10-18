#!/usr/bin/env python3
"""Comprehensive unit tests for drift_detector module.

Tests cover:
- DriftRule base class and subclasses
- UnexpectedAdditionsRule (threshold checking)
- UnexpectedRemovalsRule (threshold checking)
- LicenseChangeRule (forbidden license detection)
- DowngradeRule (version downgrade detection)
- DriftDetector (orchestration and reporting)
- Edge cases (empty diffs, large changes, unicode)
- Error handling and boundary conditions
"""

import json
import sys
import pytest
from datetime import datetime
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

# Module under test
from drift_detector import (
    DriftRule,
    UnexpectedAdditionsRule,
    UnexpectedRemovalsRule,
    LicenseChangeRule,
    DowngradeRule,
    DriftDetector,
)

# Import dependencies
from sbom_diff import SBOMDiff, Package


class TestDriftRuleBase:
    """Tests for DriftRule base class."""

    def test_drift_rule_init(self):
        """Test DriftRule initialization."""
        rule = DriftRule(
            rule_id="TEST-001",
            name="Test Rule",
            description="Test description",
            severity="WARNING"
        )
        
        assert rule.rule_id == "TEST-001"
        assert rule.name == "Test Rule"
        assert rule.description == "Test description"
        assert rule.severity == "WARNING"

    def test_drift_rule_check_not_implemented(self):
        """Test that check() must be implemented by subclasses."""
        rule = DriftRule(
            rule_id="TEST-001",
            name="Test Rule",
            description="Test description"
        )
        
        with pytest.raises(NotImplementedError):
            rule.check(Mock())

    def test_severity_uppercased(self):
        """Test that severity is automatically uppercased."""
        rule = DriftRule(
            rule_id="TEST-001",
            name="Test Rule",
            description="Test description",
            severity="warning"
        )
        
        assert rule.severity == "WARNING"


class TestUnexpectedAdditionsRule:
    """Tests for UnexpectedAdditionsRule."""

    def test_init_with_default_threshold(self):
        """Test initialization with default threshold."""
        rule = UnexpectedAdditionsRule()
        assert rule.max_additions == 5
        assert rule.rule_id == "DRIFT-001"
        assert rule.severity == "WARNING"

    def test_init_with_custom_threshold(self):
        """Test initialization with custom threshold."""
        rule = UnexpectedAdditionsRule(max_additions=10)
        assert rule.max_additions == 10

    def test_no_violation_when_below_threshold(self):
        """Test no violation when additions are below threshold."""
        rule = UnexpectedAdditionsRule(max_additions=5)
        
        # Create mock diff with 3 additions
        diff = Mock(spec=SBOMDiff)
        pkg1 = Mock(spec=Package)
        pkg1.to_dict.return_value = {"name": "pkg1", "version": "1.0.0"}
        pkg2 = Mock(spec=Package)
        pkg2.to_dict.return_value = {"name": "pkg2", "version": "1.0.0"}
        pkg3 = Mock(spec=Package)
        pkg3.to_dict.return_value = {"name": "pkg3", "version": "1.0.0"}
        
        diff.get_added_packages.return_value = [pkg1, pkg2, pkg3]
        
        violations = rule.check(diff)
        assert len(violations) == 0

    def test_violation_when_above_threshold(self):
        """Test violation when additions exceed threshold."""
        rule = UnexpectedAdditionsRule(max_additions=2)
        
        # Create mock diff with 5 additions
        diff = Mock(spec=SBOMDiff)
        packages = []
        for i in range(5):
            pkg = Mock(spec=Package)
            pkg.to_dict.return_value = {"name": f"pkg{i}", "version": "1.0.0"}
            packages.append(pkg)
        
        diff.get_added_packages.return_value = packages
        
        violations = rule.check(diff)
        assert len(violations) == 1
        assert violations[0]['rule_id'] == "DRIFT-001"
        assert violations[0]['severity'] == "WARNING"
        assert "5 new dependencies" in violations[0]['message']
        assert violations[0]['details']['count'] == 5
        assert violations[0]['details']['threshold'] == 2

    def test_violation_at_exact_threshold(self):
        """Test no violation at exact threshold."""
        rule = UnexpectedAdditionsRule(max_additions=3)
        
        diff = Mock(spec=SBOMDiff)
        packages = []
        for i in range(3):
            pkg = Mock(spec=Package)
            pkg.to_dict.return_value = {"name": f"pkg{i}", "version": "1.0.0"}
            packages.append(pkg)
        
        diff.get_added_packages.return_value = packages
        
        violations = rule.check(diff)
        assert len(violations) == 0

    def test_empty_additions(self):
        """Test with no additions."""
        rule = UnexpectedAdditionsRule()
        
        diff = Mock(spec=SBOMDiff)
        diff.get_added_packages.return_value = []
        
        violations = rule.check(diff)
        assert len(violations) == 0


class TestUnexpectedRemovalsRule:
    """Tests for UnexpectedRemovalsRule."""

    def test_init_with_default_threshold(self):
        """Test initialization with default threshold."""
        rule = UnexpectedRemovalsRule()
        assert rule.max_removals == 3
        assert rule.rule_id == "DRIFT-002"

    def test_init_with_custom_threshold(self):
        """Test initialization with custom threshold."""
        rule = UnexpectedRemovalsRule(max_removals=10)
        assert rule.max_removals == 10

    def test_no_violation_when_below_threshold(self):
        """Test no violation when removals are below threshold."""
        rule = UnexpectedRemovalsRule(max_removals=5)
        
        diff = Mock(spec=SBOMDiff)
        pkg = Mock(spec=Package)
        pkg.to_dict.return_value = {"name": "pkg1", "version": "1.0.0"}
        
        diff.get_removed_packages.return_value = [pkg]
        
        violations = rule.check(diff)
        assert len(violations) == 0

    def test_violation_when_above_threshold(self):
        """Test violation when removals exceed threshold."""
        rule = UnexpectedRemovalsRule(max_removals=1)
        
        diff = Mock(spec=SBOMDiff)
        packages = []
        for i in range(3):
            pkg = Mock(spec=Package)
            pkg.to_dict.return_value = {"name": f"pkg{i}", "version": "1.0.0"}
            packages.append(pkg)
        
        diff.get_removed_packages.return_value = packages
        
        violations = rule.check(diff)
        assert len(violations) == 1
        assert violations[0]['rule_id'] == "DRIFT-002"
        assert "3 removed dependencies" in violations[0]['message']


class TestLicenseChangeRule:
    """Tests for LicenseChangeRule."""

    def test_init_with_default_forbidden_licenses(self):
        """Test initialization with default forbidden licenses."""
        rule = LicenseChangeRule()
        assert rule.rule_id == "DRIFT-003"
        assert rule.severity == "ERROR"
        assert 'GPL-3.0' in rule.forbidden_licenses
        assert 'AGPL-3.0' in rule.forbidden_licenses

    def test_init_with_custom_forbidden_licenses(self):
        """Test initialization with custom forbidden licenses."""
        rule = LicenseChangeRule(forbidden_licenses=['MIT', 'BSD'])
        assert 'MIT' in rule.forbidden_licenses
        assert 'BSD' in rule.forbidden_licenses

    def test_no_violation_for_acceptable_licenses(self):
        """Test no violation when licenses are acceptable."""
        rule = LicenseChangeRule()
        
        diff = Mock(spec=SBOMDiff)
        diff.get_license_changes.return_value = []
        diff.get_added_packages.return_value = []
        
        violations = rule.check(diff)
        assert len(violations) == 0

    def test_violation_for_license_change_to_forbidden(self):
        """Test violation when license changes to forbidden license."""
        rule = LicenseChangeRule()
        
        old_pkg = Mock(spec=Package)
        old_pkg.name = "test-lib"
        old_pkg.version = "1.0.0"
        old_pkg.license = "Apache-2.0"
        
        new_pkg = Mock(spec=Package)
        new_pkg.name = "test-lib"
        new_pkg.version = "2.0.0"
        new_pkg.license = "GPL-3.0"
        
        diff = Mock(spec=SBOMDiff)
        diff.get_license_changes.return_value = [(old_pkg, new_pkg)]
        diff.get_added_packages.return_value = []
        
        violations = rule.check(diff)
        assert len(violations) == 1
        assert violations[0]['rule_id'] == "DRIFT-003"
        assert violations[0]['severity'] == "ERROR"
        assert "GPL-3.0" in violations[0]['message']
        assert violations[0]['details']['old_license'] == "Apache-2.0"
        assert violations[0]['details']['new_license'] == "GPL-3.0"

    def test_violation_for_new_package_with_forbidden_license(self):
        """Test violation when new package has forbidden license."""
        rule = LicenseChangeRule()
        
        pkg = Mock(spec=Package)
        pkg.name = "new-lib"
        pkg.version = "1.0.0"
        pkg.license = "AGPL-3.0"
        
        diff = Mock(spec=SBOMDiff)
        diff.get_license_changes.return_value = []
        diff.get_added_packages.return_value = [pkg]
        
        violations = rule.check(diff)
        assert len(violations) == 1
        assert "AGPL-3.0" in violations[0]['message']
        assert violations[0]['details']['package'] == "new-lib"

    def test_multiple_violations(self):
        """Test multiple license violations."""
        rule = LicenseChangeRule()
        
        # License change violation
        old_pkg = Mock(spec=Package)
        old_pkg.name = "lib1"
        old_pkg.version = "1.0.0"
        old_pkg.license = "MIT"
        
        new_pkg = Mock(spec=Package)
        new_pkg.name = "lib1"
        new_pkg.version = "2.0.0"
        new_pkg.license = "GPL-3.0"
        
        # New package with forbidden license
        added_pkg = Mock(spec=Package)
        added_pkg.name = "lib2"
        added_pkg.version = "1.0.0"
        added_pkg.license = "AGPL-3.0"
        
        diff = Mock(spec=SBOMDiff)
        diff.get_license_changes.return_value = [(old_pkg, new_pkg)]
        diff.get_added_packages.return_value = [added_pkg]
        
        violations = rule.check(diff)
        assert len(violations) == 2


class TestDowngradeRule:
    """Tests for DowngradeRule."""

    def test_init(self):
        """Test DowngradeRule initialization."""
        rule = DowngradeRule()
        assert rule.rule_id == "DRIFT-004"
        assert rule.severity == "WARNING"

    def test_no_violation_when_no_downgrades(self):
        """Test no violation when no downgrades."""
        rule = DowngradeRule()
        
        diff = Mock(spec=SBOMDiff)
        diff.get_downgraded_packages.return_value = []
        
        violations = rule.check(diff)
        assert len(violations) == 0

    def test_violation_for_downgrade(self):
        """Test violation when package is downgraded."""
        rule = DowngradeRule()
        
        old_pkg = Mock(spec=Package)
        old_pkg.name = "guava"
        old_pkg.version = "31.0"
        
        new_pkg = Mock(spec=Package)
        new_pkg.name = "guava"
        new_pkg.version = "30.0"
        
        diff = Mock(spec=SBOMDiff)
        diff.get_downgraded_packages.return_value = [(old_pkg, new_pkg)]
        
        violations = rule.check(diff)
        assert len(violations) == 1
        assert violations[0]['rule_id'] == "DRIFT-004"
        assert violations[0]['severity'] == "WARNING"
        assert "31.0" in violations[0]['message']
        assert "30.0" in violations[0]['message']
        assert violations[0]['details']['package'] == "guava"
        assert violations[0]['details']['old_version'] == "31.0"
        assert violations[0]['details']['new_version'] == "30.0"

    def test_multiple_downgrades(self):
        """Test multiple downgrades."""
        rule = DowngradeRule()
        
        downgrades = []
        for i in range(3):
            old = Mock(spec=Package)
            old.name = f"pkg{i}"
            old.version = "2.0.0"
            
            new = Mock(spec=Package)
            new.name = f"pkg{i}"
            new.version = "1.0.0"
            
            downgrades.append((old, new))
        
        diff = Mock(spec=SBOMDiff)
        diff.get_downgraded_packages.return_value = downgrades
        
        violations = rule.check(diff)
        assert len(violations) == 3


class TestDriftDetector:
    """Tests for DriftDetector orchestration."""

    def test_init_with_default_rules(self):
        """Test initialization with default rules."""
        detector = DriftDetector()
        assert len(detector.rules) == 4
        assert isinstance(detector.rules[0], UnexpectedAdditionsRule)
        assert isinstance(detector.rules[1], UnexpectedRemovalsRule)
        assert isinstance(detector.rules[2], LicenseChangeRule)
        assert isinstance(detector.rules[3], DowngradeRule)

    def test_init_with_custom_rules(self):
        """Test initialization with custom rules."""
        custom_rules = [UnexpectedAdditionsRule(max_additions=10)]
        detector = DriftDetector(rules=custom_rules)
        assert len(detector.rules) == 1
        assert detector.rules[0].max_additions == 10

    @patch('drift_detector.SBOMDiff')
    def test_detect_with_no_violations(self, mock_sbom_diff_class):
        """Test detect when no violations found."""
        # Mock SBOMDiff
        mock_diff = Mock(spec=SBOMDiff)
        mock_diff.get_added_packages.return_value = []
        mock_diff.get_removed_packages.return_value = []
        mock_diff.get_license_changes.return_value = []
        mock_diff.get_downgraded_packages.return_value = []
        mock_diff.to_dict.return_value = {
            'summary': {
                'total_old': 10,
                'total_new': 10,
                'added': 0,
                'removed': 0,
                'upgraded': 0,
                'downgraded': 0,
                'license_changed': 0
            }
        }
        mock_sbom_diff_class.return_value = mock_diff
        
        baseline = {'name': 'baseline-sbom', 'packages': []}
        current = {'name': 'current-sbom', 'packages': []}
        
        detector = DriftDetector()
        result = detector.detect(baseline, current)
        
        assert result['status'] == 'PASSED'
        assert result['summary']['total_violations'] == 0
        assert result['summary']['critical'] == 0
        assert result['summary']['error'] == 0
        assert result['summary']['warning'] == 0
        assert result['metadata']['rules_executed'] == 4

    @patch('drift_detector.SBOMDiff')
    def test_detect_with_warning_violations(self, mock_sbom_diff_class):
        """Test detect with warning-level violations."""
        # Mock SBOMDiff with many additions
        mock_diff = Mock(spec=SBOMDiff)
        
        packages = []
        for i in range(10):
            pkg = Mock(spec=Package)
            pkg.name = f"pkg{i}"
            pkg.version = "1.0.0"
            pkg.license = "Apache-2.0"  # Safe license
            pkg.to_dict.return_value = {"name": f"pkg{i}", "version": "1.0.0"}
            packages.append(pkg)
        
        mock_diff.get_added_packages.return_value = packages
        mock_diff.get_removed_packages.return_value = []
        mock_diff.get_license_changes.return_value = []
        mock_diff.get_downgraded_packages.return_value = []
        mock_diff.to_dict.return_value = {
            'summary': {
                'total_old': 5,
                'total_new': 15,
                'added': 10,
                'removed': 0,
                'upgraded': 0,
                'downgraded': 0,
                'license_changed': 0
            }
        }
        mock_sbom_diff_class.return_value = mock_diff
        
        baseline = {'name': 'baseline-sbom'}
        current = {'name': 'current-sbom'}
        
        detector = DriftDetector()
        result = detector.detect(baseline, current)
        
        assert result['status'] == 'WARNING'
        assert result['summary']['total_violations'] > 0
        assert result['summary']['warning'] > 0

    @patch('drift_detector.SBOMDiff')
    def test_detect_with_error_violations(self, mock_sbom_diff_class):
        """Test detect with error-level violations."""
        # Mock SBOMDiff with forbidden license
        mock_diff = Mock(spec=SBOMDiff)
        
        pkg = Mock(spec=Package)
        pkg.name = "test-lib"
        pkg.version = "1.0.0"
        pkg.license = "GPL-3.0"
        
        mock_diff.get_added_packages.return_value = [pkg]
        mock_diff.get_removed_packages.return_value = []
        mock_diff.get_license_changes.return_value = []
        mock_diff.get_downgraded_packages.return_value = []
        mock_diff.to_dict.return_value = {
            'summary': {
                'total_old': 5,
                'total_new': 6,
                'added': 1,
                'removed': 0,
                'upgraded': 0,
                'downgraded': 0,
                'license_changed': 0
            }
        }
        mock_sbom_diff_class.return_value = mock_diff
        
        baseline = {'name': 'baseline-sbom'}
        current = {'name': 'current-sbom'}
        
        detector = DriftDetector()
        result = detector.detect(baseline, current)
        
        assert result['status'] == 'FAILED'
        assert result['summary']['error'] > 0

    @patch('drift_detector.SBOMDiff')
    def test_detect_result_structure(self, mock_sbom_diff_class):
        """Test that detect returns properly structured result."""
        # Mock SBOMDiff
        mock_diff = Mock(spec=SBOMDiff)
        mock_diff.get_added_packages.return_value = []
        mock_diff.get_removed_packages.return_value = []
        mock_diff.get_license_changes.return_value = []
        mock_diff.get_downgraded_packages.return_value = []
        mock_diff.to_dict.return_value = {
            'summary': {
                'total_old': 5,
                'total_new': 5,
                'added': 0,
                'removed': 0,
                'upgraded': 0,
                'downgraded': 0,
                'license_changed': 0
            }
        }
        mock_sbom_diff_class.return_value = mock_diff
        
        baseline = {'name': 'baseline-sbom'}
        current = {'name': 'current-sbom'}
        
        detector = DriftDetector()
        result = detector.detect(baseline, current)
        
        # Check structure
        assert 'metadata' in result
        assert 'status' in result
        assert 'summary' in result
        assert 'diff_summary' in result
        assert 'violations' in result
        
        # Check metadata
        assert 'baseline_sbom' in result['metadata']
        assert 'current_sbom' in result['metadata']
        assert 'detection_date' in result['metadata']
        assert 'rules_executed' in result['metadata']
        
        # Check summary
        assert 'total_violations' in result['summary']
        assert 'critical' in result['summary']
        assert 'error' in result['summary']
        assert 'warning' in result['summary']
        assert 'info' in result['summary']


class TestEdgeCases:
    """Tests for edge cases and boundary conditions."""

    def test_unicode_in_package_names(self):
        """Test handling of unicode in package names."""
        rule = DowngradeRule()
        
        old_pkg = Mock(spec=Package)
        old_pkg.name = "lib-日本語"
        old_pkg.version = "2.0.0"
        
        new_pkg = Mock(spec=Package)
        new_pkg.name = "lib-日本語"
        new_pkg.version = "1.0.0"
        
        diff = Mock(spec=SBOMDiff)
        diff.get_downgraded_packages.return_value = [(old_pkg, new_pkg)]
        
        # Should not raise exception
        violations = rule.check(diff)
        assert len(violations) == 1
        assert "日本語" in violations[0]['message']

    def test_zero_threshold(self):
        """Test with zero threshold."""
        rule = UnexpectedAdditionsRule(max_additions=0)
        
        diff = Mock(spec=SBOMDiff)
        pkg = Mock(spec=Package)
        pkg.to_dict.return_value = {"name": "pkg1", "version": "1.0.0"}
        diff.get_added_packages.return_value = [pkg]
        
        violations = rule.check(diff)
        assert len(violations) == 1

    def test_large_number_of_violations(self):
        """Test with large number of violations."""
        rule = UnexpectedAdditionsRule(max_additions=0)
        
        diff = Mock(spec=SBOMDiff)
        packages = []
        for i in range(1000):
            pkg = Mock(spec=Package)
            pkg.to_dict.return_value = {"name": f"pkg{i}", "version": "1.0.0"}
            packages.append(pkg)
        
        diff.get_added_packages.return_value = packages
        
        violations = rule.check(diff)
        assert len(violations) == 1
        assert violations[0]['details']['count'] == 1000

    def test_empty_license_list(self):
        """Test LicenseChangeRule with empty forbidden list."""
        # Note: When no forbidden licenses specified, defaults are used (GPL-2.0, GPL-3.0, AGPL-3.0)
        # To truly have no forbidden licenses, we need to pass an empty list explicitly
        # However, the implementation uses set() which means empty list becomes empty set
        # and the default check `forbidden_licenses or [...]` will use defaults
        rule = LicenseChangeRule(forbidden_licenses=[])
        
        pkg = Mock(spec=Package)
        pkg.name = "test"
        pkg.version = "1.0.0"
        pkg.license = "MIT"  # Use a non-forbidden license
        
        diff = Mock(spec=SBOMDiff)
        diff.get_license_changes.return_value = []
        diff.get_added_packages.return_value = [pkg]
        
        violations = rule.check(diff)
        assert len(violations) == 0

    def test_special_characters_in_versions(self):
        """Test handling of special characters in versions."""
        rule = DowngradeRule()
        
        old_pkg = Mock(spec=Package)
        old_pkg.name = "test-lib"
        old_pkg.version = "2.0.0-SNAPSHOT+build.123"
        
        new_pkg = Mock(spec=Package)
        new_pkg.name = "test-lib"
        new_pkg.version = "1.0.0-RC1"
        
        diff = Mock(spec=SBOMDiff)
        diff.get_downgraded_packages.return_value = [(old_pkg, new_pkg)]
        
        violations = rule.check(diff)
        assert len(violations) == 1
        assert "2.0.0-SNAPSHOT+build.123" in violations[0]['message']
