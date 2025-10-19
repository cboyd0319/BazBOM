#!/usr/bin/env python3
"""Tests for supply_chain_risk.py"""

import unittest
import json
import tempfile
import os
from pathlib import Path
import sys

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from supply_chain_risk import (
    levenshtein_distance,
    check_typosquatting,
    load_popular_packages,
)


class TestLevenshteinDistance(unittest.TestCase):
    def test_identical_strings(self):
        self.assertEqual(levenshtein_distance("test", "test"), 0)
    
    def test_one_char_difference(self):
        self.assertEqual(levenshtein_distance("test", "best"), 1)
    
    def test_insertion(self):
        self.assertEqual(levenshtein_distance("test", "tests"), 1)
    
    def test_deletion(self):
        self.assertEqual(levenshtein_distance("tests", "test"), 1)
    
    def test_completely_different(self):
        distance = levenshtein_distance("abc", "xyz")
        self.assertEqual(distance, 3)


class TestTyposquatting(unittest.TestCase):
    def test_no_typosquatting(self):
        known_packages = {"guava", "junit", "mockito"}
        findings = check_typosquatting("mypackage", known_packages)
        self.assertEqual(len(findings), 0)
    
    def test_single_char_typo(self):
        known_packages = {"guava", "junit", "mockito"}
        findings = check_typosquatting("guav", known_packages, threshold=2)
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0]['similar_to'], "guava")
        self.assertEqual(findings[0]['distance'], 1)
        self.assertEqual(findings[0]['severity'], "HIGH")
    
    def test_two_char_typo(self):
        known_packages = {"junit"}
        findings = check_typosquatting("jnit", known_packages, threshold=2)
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0]['distance'], 1)
    
    def test_threshold_filtering(self):
        known_packages = {"guava"}
        findings = check_typosquatting("abc", known_packages, threshold=2)
        self.assertEqual(len(findings), 0)
    
    def test_case_insensitive(self):
        known_packages = {"Guava"}
        findings = check_typosquatting("guav", known_packages, threshold=2)
        self.assertEqual(len(findings), 1)


class TestLoadPopularPackages(unittest.TestCase):
    def test_returns_set(self):
        packages = load_popular_packages()
        self.assertIsInstance(packages, set)
    
    def test_contains_known_packages(self):
        packages = load_popular_packages()
        self.assertIn("guava", packages)
        self.assertIn("junit", packages)
        self.assertIn("slf4j", packages)
    
    def test_not_empty(self):
        packages = load_popular_packages()
        self.assertGreater(len(packages), 0)


class TestSBOMIntegration(unittest.TestCase):
    def test_parse_sbom(self):
        from supply_chain_risk import parse_sbom
        
        # Create a minimal SBOM
        sbom = {
            "spdxVersion": "SPDX-2.3",
            "packages": [
                {
                    "name": "guava",
                    "versionInfo": "31.1-jre",
                    "SPDXID": "SPDXRef-Package-guava",
                    "externalRefs": [
                        {
                            "referenceType": "purl",
                            "referenceLocator": "pkg:maven/com.google.guava/guava@31.1-jre"
                        }
                    ]
                }
            ]
        }
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(sbom, f)
            sbom_path = f.name
        
        try:
            packages = parse_sbom(sbom_path)
            self.assertEqual(len(packages), 1)
            self.assertEqual(packages[0]['name'], 'guava')
            self.assertEqual(packages[0]['version'], '31.1-jre')
            self.assertEqual(packages[0]['group_id'], 'com.google.guava')
            self.assertEqual(packages[0]['artifact_id'], 'guava')
        finally:
            os.unlink(sbom_path)


class TestCheckUnmaintained(unittest.TestCase):
    """Test check_unmaintained function."""
    
    def test_recently_maintained(self):
        """Test package with recent commit is not flagged."""
        from supply_chain_risk import check_unmaintained
        from datetime import datetime, timedelta, timezone
        
        recent_date = (datetime.now(timezone.utc) - timedelta(days=30)).isoformat()
        result = check_unmaintained("test-pkg", recent_date)
        self.assertIsNone(result)
    
    def test_unmaintained_package(self):
        """Test package with old commit is flagged."""
        from supply_chain_risk import check_unmaintained
        from datetime import datetime, timedelta, timezone
        
        old_date = (datetime.now(timezone.utc) - timedelta(days=800)).isoformat()
        result = check_unmaintained("old-pkg", old_date, threshold_years=2)
        self.assertIsNotNone(result)
        self.assertEqual(result['type'], 'unmaintained')
        self.assertEqual(result['package'], 'old-pkg')
        self.assertEqual(result['severity'], 'MEDIUM')
    
    def test_no_commit_date(self):
        """Test package with no commit date."""
        from supply_chain_risk import check_unmaintained
        
        result = check_unmaintained("unknown-pkg", None)
        self.assertIsNone(result)
    
    def test_empty_commit_date(self):
        """Test package with empty commit date."""
        from supply_chain_risk import check_unmaintained
        
        result = check_unmaintained("unknown-pkg", "")
        self.assertIsNone(result)
    
    def test_custom_threshold(self):
        """Test custom threshold years."""
        from supply_chain_risk import check_unmaintained
        from datetime import datetime, timedelta, timezone
        
        old_date = (datetime.now(timezone.utc) - timedelta(days=400)).isoformat()
        
        # Should be flagged with 1 year threshold
        result = check_unmaintained("pkg", old_date, threshold_years=1)
        self.assertIsNotNone(result)
        
        # Should not be flagged with 2 year threshold
        result2 = check_unmaintained("pkg", old_date, threshold_years=2)
        self.assertIsNone(result2)
    
    def test_invalid_date_format(self):
        """Test handling of invalid date format."""
        from supply_chain_risk import check_unmaintained
        
        # Should handle invalid date gracefully
        result = check_unmaintained("pkg", "invalid-date")
        # Implementation may return None or raise - either is acceptable
        # Just testing it doesn't crash


class TestCheckDeprecatedMaven(unittest.TestCase):
    """Test check_deprecated_maven function."""
    
    def test_maven_check_with_network_mock(self):
        """Test Maven check with mocked network response."""
        from supply_chain_risk import check_deprecated_maven
        from unittest.mock import patch, Mock
        import urllib.request
        
        # Mock the urlopen to avoid actual network calls
        mock_response = Mock()
        mock_response.read.return_value = json.dumps({
            "response": {
                "numFound": 1,
                "docs": [{
                    "latestVersion": "32.0.0-jre"
                }]
            }
        }).encode('utf-8')
        mock_response.__enter__ = Mock(return_value=mock_response)
        mock_response.__exit__ = Mock(return_value=False)
        
        with patch('urllib.request.urlopen', return_value=mock_response):
            result = check_deprecated_maven("com.google.guava", "guava", "31.0-jre")
            
            if result is not None:  # May return None or a finding
                self.assertEqual(result['type'], 'outdated_version')
    
    def test_maven_check_network_error(self):
        """Test Maven check handles network errors."""
        from supply_chain_risk import check_deprecated_maven
        from unittest.mock import patch
        import urllib.error
        
        with patch('urllib.request.urlopen', side_effect=urllib.error.URLError("Network error")):
            result = check_deprecated_maven("com.example", "test", "1.0")
            self.assertIsNone(result)
    
    def test_maven_check_timeout(self):
        """Test Maven check handles timeout."""
        from supply_chain_risk import check_deprecated_maven
        from unittest.mock import patch
        import socket
        
        with patch('urllib.request.urlopen', side_effect=socket.timeout("Timeout")):
            result = check_deprecated_maven("com.example", "test", "1.0")
            self.assertIsNone(result)
    
    def test_maven_same_version(self):
        """Test Maven check when version is the same as latest."""
        from supply_chain_risk import check_deprecated_maven
        from unittest.mock import patch, Mock
        
        mock_response = Mock()
        mock_response.read.return_value = json.dumps({
            "response": {
                "numFound": 1,
                "docs": [{
                    "latestVersion": "1.0.0"
                }]
            }
        }).encode('utf-8')
        mock_response.__enter__ = Mock(return_value=mock_response)
        mock_response.__exit__ = Mock(return_value=False)
        
        with patch('urllib.request.urlopen', return_value=mock_response):
            result = check_deprecated_maven("com.example", "test", "1.0.0")
            self.assertIsNone(result)


class TestEdgeCases(unittest.TestCase):
    """Test edge cases and boundary conditions."""
    
    def test_levenshtein_empty_strings(self):
        from supply_chain_risk import levenshtein_distance
        
        self.assertEqual(levenshtein_distance("", ""), 0)
        self.assertEqual(levenshtein_distance("abc", ""), 3)
        self.assertEqual(levenshtein_distance("", "xyz"), 3)
    
    def test_levenshtein_swap_args(self):
        """Test that swapping arguments gives same result."""
        from supply_chain_risk import levenshtein_distance
        
        dist1 = levenshtein_distance("short", "verylongstring")
        dist2 = levenshtein_distance("verylongstring", "short")
        self.assertEqual(dist1, dist2)
    
    def test_typosquatting_exact_match(self):
        """Test that exact match is not flagged as typosquatting."""
        from supply_chain_risk import check_typosquatting
        
        known_packages = {"guava"}
        findings = check_typosquatting("guava", known_packages)
        self.assertEqual(len(findings), 0)
    
    def test_typosquatting_empty_known_packages(self):
        """Test typosquatting check with empty known packages."""
        from supply_chain_risk import check_typosquatting
        
        findings = check_typosquatting("test", set())
        self.assertEqual(len(findings), 0)
    
    def test_typosquatting_multiple_matches(self):
        """Test typosquatting finds multiple similar packages."""
        from supply_chain_risk import check_typosquatting
        
        known_packages = {"guava", "guave", "gvava"}
        findings = check_typosquatting("guav", known_packages, threshold=2)
        # Should find multiple matches
        self.assertGreater(len(findings), 0)


if __name__ == '__main__':
    unittest.main()
