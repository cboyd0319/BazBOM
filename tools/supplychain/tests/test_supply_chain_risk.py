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


if __name__ == '__main__':
    unittest.main()
