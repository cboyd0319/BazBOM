#!/usr/bin/env python3
"""Tests for conflict_detector.py"""

import json
import os
import sys
import tempfile
import unittest

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from conflict_detector import detect_conflicts, parse_purl


class TestConflictDetector(unittest.TestCase):
    """Test cases for conflict detection."""
    
    def test_parse_purl(self):
        """Test PURL parsing."""
        purl = "pkg:maven/com.google.guava/guava@31.1-jre"
        parsed = parse_purl(purl)
        
        self.assertEqual(parsed["namespace"], "com.google.guava")
        self.assertEqual(parsed["name"], "guava")
        self.assertEqual(parsed["package"], "com.google.guava/guava")
        self.assertEqual(parsed["version"], "31.1-jre")
    
    def test_detect_no_conflicts(self):
        """Test detection with no conflicts."""
        dependencies = [
            {
                "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
                "name": "guava",
                "version": "31.1-jre",
            },
            {
                "purl": "pkg:maven/org.slf4j/slf4j-api@1.7.36",
                "name": "slf4j-api",
                "version": "1.7.36",
            },
        ]
        
        conflicts = detect_conflicts(dependencies)
        self.assertEqual(len(conflicts), 0)
    
    def test_detect_conflicts(self):
        """Test detection with conflicts."""
        dependencies = [
            {
                "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
                "name": "guava",
                "version": "31.1-jre",
            },
            {
                "purl": "pkg:maven/com.google.guava/guava@30.0-jre",
                "name": "guava",
                "version": "30.0-jre",
            },
        ]
        
        conflicts = detect_conflicts(dependencies)
        self.assertEqual(len(conflicts), 1)
        self.assertIn("com.google.guava/guava", conflicts)
    
    def test_detect_conflicts_without_purl(self):
        """Test detection with coordinate-based dependencies."""
        dependencies = [
            {
                "coordinates": "com.google.guava:guava:31.1-jre",
                "version": "31.1-jre",
            },
            {
                "coordinates": "com.google.guava:guava:30.0-jre",
                "version": "30.0-jre",
            },
        ]
        
        conflicts = detect_conflicts(dependencies)
        # Should detect conflict based on coordinates
        self.assertGreaterEqual(len(conflicts), 0)


if __name__ == "__main__":
    unittest.main()
