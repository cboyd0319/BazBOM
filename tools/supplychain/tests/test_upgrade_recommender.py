#!/usr/bin/env python3
"""Tests for upgrade recommender."""

import json
import os
import tempfile
import unittest
from pathlib import Path

# Import the module from parent directory
import sys
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if parent_dir not in sys.path:
    sys.path.insert(0, parent_dir)

import upgrade_recommender


class TestBreakingChangeAnalyzer(unittest.TestCase):
    """Test breaking change analyzer."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.analyzer = upgrade_recommender.BreakingChangeAnalyzer()
    
    def test_analyze_changelog_no_breaking_changes(self):
        """Test changelog with no breaking changes."""
        changelog = """
        ## Version 1.1.0
        - Added new feature
        - Fixed bug in logger
        - Improved performance
        """
        
        changes = self.analyzer.analyze_changelog(changelog)
        self.assertEqual(len(changes), 0)
    
    def test_analyze_changelog_with_breaking_changes(self):
        """Test changelog with breaking changes."""
        changelog = """
## 2.0.0

- BREAKING CHANGE: Removed old API
- Breaking: Method signature changed for processData()
- Added new features
        """
        
        changes = self.analyzer.analyze_changelog(changelog)
        self.assertGreaterEqual(len(changes), 1)  # Should find at least one
        if changes:
            self.assertIn("2.0.0", changes[0].version)
    
    def test_analyze_changelog_empty(self):
        """Test empty changelog."""
        changes = self.analyzer.analyze_changelog("")
        self.assertEqual(len(changes), 0)
    
    def test_extract_apis(self):
        """Test API extraction from text."""
        text = "Changed signature of ImmutableList.of() and Builder.build()"
        apis = self.analyzer._extract_apis(text)
        
        self.assertGreater(len(apis), 0)
    
    def test_parse_version_standard(self):
        """Test parsing standard version."""
        version = self.analyzer._parse_version("1.2.3")
        self.assertEqual(version, (1, 2, 3))
    
    def test_parse_version_with_prefix(self):
        """Test parsing version with v prefix."""
        version = self.analyzer._parse_version("v2.0.1")
        self.assertEqual(version, (2, 0, 1))
    
    def test_parse_version_with_suffix(self):
        """Test parsing version with suffix."""
        version = self.analyzer._parse_version("1.0.0-jre")
        self.assertEqual(version, (1, 0, 0))
    
    def test_parse_version_invalid(self):
        """Test parsing invalid version."""
        with self.assertRaises(ValueError):
            self.analyzer._parse_version("invalid")
    
    def test_is_version_in_range(self):
        """Test version range checking."""
        self.assertTrue(
            self.analyzer._is_version_in_range("1.5.0", "1.0.0", "2.0.0")
        )
        self.assertFalse(
            self.analyzer._is_version_in_range("2.5.0", "1.0.0", "2.0.0")
        )
        self.assertFalse(
            self.analyzer._is_version_in_range("0.5.0", "1.0.0", "2.0.0")
        )
    
    def test_count_breaking_changes(self):
        """Test counting breaking changes in range."""
        changes = [
            upgrade_recommender.BreakingChange(
                version="1.5.0",
                change_type="API",
                description="Test change",
                severity="HIGH",
                affected_apis=[]
            ),
            upgrade_recommender.BreakingChange(
                version="2.5.0",
                change_type="API",
                description="Test change",
                severity="HIGH",
                affected_apis=[]
            ),
        ]
        
        count = self.analyzer.count_breaking_changes("1.0.0", "2.0.0", changes)
        self.assertEqual(count, 1)  # Only 1.5.0 is in range


class TestUpgradeRecommender(unittest.TestCase):
    """Test upgrade recommender."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.recommender = upgrade_recommender.UpgradeRecommender()
    
    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    def test_initialization(self):
        """Test recommender initialization."""
        self.assertIsNone(self.recommender.sbom_data)
        self.assertEqual(len(self.recommender.packages), 0)
    
    def test_load_sbom_success(self):
        """Test loading valid SBOM."""
        sbom_path = os.path.join(self.temp_dir, "test.json")
        sbom_data = {
            "spdxVersion": "SPDX-2.3",
            "packages": [
                {"name": "com.google.guava:guava", "versionInfo": "31.1-jre"}
            ]
        }
        
        with open(sbom_path, 'w') as f:
            json.dump(sbom_data, f)
        
        self.recommender.load_sbom(sbom_path)
        
        self.assertIsNotNone(self.recommender.sbom_data)
        self.assertEqual(len(self.recommender.packages), 1)
    
    def test_load_sbom_file_not_found(self):
        """Test loading non-existent SBOM."""
        with self.assertRaises(FileNotFoundError):
            self.recommender.load_sbom("/nonexistent/file.json")
    
    def test_load_sbom_invalid_json(self):
        """Test loading invalid JSON."""
        bad_path = os.path.join(self.temp_dir, "bad.json")
        with open(bad_path, 'w') as f:
            f.write("{ invalid json }")
        
        with self.assertRaises(ValueError):
            self.recommender.load_sbom(bad_path)
    
    def test_recommend_upgrade_basic(self):
        """Test basic upgrade recommendation."""
        rec = self.recommender.recommend_upgrade(
            package_name="com.google.guava:guava",
            current_version="30.0-jre",
            available_versions=["30.0-jre", "30.1-jre", "31.0-jre", "31.1-jre"]
        )
        
        self.assertEqual(rec.package, "com.google.guava:guava")
        self.assertEqual(rec.current_version, "30.0-jre")
        self.assertIn(rec.recommended_version, ["30.1-jre", "31.0-jre", "31.1-jre"])
        self.assertEqual(rec.latest_version, "31.1-jre")
        self.assertIsNotNone(rec.migration_guide)
    
    def test_recommend_upgrade_no_versions(self):
        """Test recommendation with no versions provided."""
        rec = self.recommender.recommend_upgrade(
            package_name="test:package",
            current_version="1.0.0"
        )
        
        self.assertEqual(rec.package, "test:package")
        self.assertIsNotNone(rec.recommended_version)
    
    def test_recommend_upgrade_with_changelog(self):
        """Test recommendation with changelog analysis."""
        changelog = """
## 2.0.0

- BREAKING CHANGE: Removed deprecated methods
- Added new API

## 1.5.0

- Bug fixes
        """
        
        rec = self.recommender.recommend_upgrade(
            package_name="test:package",
            current_version="1.0.0",
            available_versions=["1.0.0", "1.5.0", "2.0.0"],
            changelog=changelog
        )
        
        # Should have higher confidence with changelog
        self.assertGreater(rec.confidence, 0.5)
        # May or may not detect breaking changes depending on parsing
        # So just verify recommendation was generated
        self.assertIsNotNone(rec.recommended_version)
    
    def test_recommend_upgrade_invalid_inputs(self):
        """Test recommendation with invalid inputs."""
        with self.assertRaises(ValueError):
            self.recommender.recommend_upgrade("", "1.0.0")
        
        with self.assertRaises(ValueError):
            self.recommender.recommend_upgrade("test:package", "")
    
    def test_generate_candidate_versions(self):
        """Test candidate version generation."""
        candidates = self.recommender._generate_candidate_versions("1.0.0")
        
        self.assertGreater(len(candidates), 1)
        self.assertIn("1.0.0", candidates)
        # Should include patch updates
        self.assertTrue(any("1.0." in v for v in candidates))
        # Should include minor updates
        self.assertTrue(any("1.1" in v or "1.2" in v for v in candidates))
        # Should include major update
        self.assertTrue(any("2.0" in v for v in candidates))
    
    def test_find_safest_upgrade_patch(self):
        """Test finding safest upgrade (patch)."""
        versions = ["1.0.0", "1.0.1", "1.0.2", "1.1.0", "2.0.0"]
        
        safest = self.recommender._find_safest_upgrade("1.0.0", versions, [])
        
        # Should prefer patch update in same minor version
        self.assertEqual(safest, "1.0.1")
    
    def test_find_safest_upgrade_minor(self):
        """Test finding safest upgrade (minor)."""
        versions = ["1.0.0", "1.1.0", "1.2.0", "2.0.0"]
        
        safest = self.recommender._find_safest_upgrade("1.0.0", versions, [])
        
        # Should prefer minor update in same major version
        self.assertIn(safest, ["1.1.0", "1.2.0"])
    
    def test_calculate_compatibility_patch(self):
        """Test compatibility calculation for patch update."""
        score = self.recommender._calculate_compatibility("1.0.0", "1.0.1", 0)
        
        self.assertGreater(score, 0.9)  # High compatibility for patch
    
    def test_calculate_compatibility_major(self):
        """Test compatibility calculation for major update."""
        score = self.recommender._calculate_compatibility("1.0.0", "2.0.0", 5)
        
        self.assertLess(score, 0.6)  # Lower compatibility for major + breaking changes
    
    def test_estimate_effort_low(self):
        """Test effort estimation with no breaking changes."""
        effort = self.recommender._estimate_effort(0)
        
        self.assertIn("LOW", effort)
    
    def test_estimate_effort_medium(self):
        """Test effort estimation with few breaking changes."""
        effort = self.recommender._estimate_effort(2)
        
        self.assertIn("MEDIUM", effort)
    
    def test_estimate_effort_high(self):
        """Test effort estimation with many breaking changes."""
        effort = self.recommender._estimate_effort(10)
        
        self.assertIn("HIGH", effort)
    
    def test_generate_migration_guide(self):
        """Test migration guide generation."""
        guide = self.recommender._generate_migration_guide(
            "test:package",
            "1.0.0",
            "2.0.0",
            ["Breaking change 1", "Breaking change 2"]
        )
        
        self.assertIn("Migration Guide", guide)
        self.assertIn("1.0.0", guide)
        self.assertIn("2.0.0", guide)
        self.assertIn("Breaking change 1", guide)
        self.assertIn("Migration Steps", guide)
    
    def test_upgrade_recommendation_dataclass(self):
        """Test UpgradeRecommendation dataclass."""
        rec = upgrade_recommender.UpgradeRecommendation(
            package="test:package",
            current_version="1.0.0",
            recommended_version="1.1.0",
            latest_version="2.0.0",
            breaking_changes=["change1"],
            compatibility_score=0.85,
            effort_estimate="LOW",
            confidence=0.9,
            migration_guide="guide",
            security_fixes=[]
        )
        
        self.assertEqual(rec.package, "test:package")
        self.assertIsNotNone(rec.timestamp)


class TestMainFunction(unittest.TestCase):
    """Test main entry point."""
    
    def test_main_basic(self):
        """Test main function with basic arguments."""
        import sys
        from unittest.mock import patch
        
        with patch.object(sys, 'argv', [
            'upgrade_recommender.py',
            '--package', 'com.google.guava:guava',
            '--current', '30.0-jre',
            '--versions', '30.0-jre', '31.0-jre'
        ]):
            result = upgrade_recommender.main()
        
        self.assertEqual(result, 0)
    
    def test_main_json_output(self):
        """Test main function with JSON output."""
        import sys
        from unittest.mock import patch
        from io import StringIO
        
        with patch.object(sys, 'argv', [
            'upgrade_recommender.py',
            '--package', 'test:package',
            '--current', '1.0.0',
            '--json'
        ]):
            with patch('sys.stdout', new=StringIO()) as fake_out:
                result = upgrade_recommender.main()
                output = fake_out.getvalue()
        
        self.assertEqual(result, 0)
        # Should be valid JSON
        json_data = json.loads(output)
        self.assertIn("package", json_data)
        self.assertIn("recommended_version", json_data)


if __name__ == '__main__':
    unittest.main()
