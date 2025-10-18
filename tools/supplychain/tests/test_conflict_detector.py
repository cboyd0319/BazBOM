#!/usr/bin/env python3
"""Comprehensive unit tests for conflict_detector module.

Tests cover:
- PURL parsing (happy path and edge cases)
- Conflict detection with PURLs
- Conflict detection with coordinates
- Resolution suggestion generation
- Reporting and output formatting
- Edge cases (empty deps, malformed PURLs, unicode)
"""

import json
import os
import sys
import tempfile
import pytest
from pathlib import Path
from unittest.mock import Mock, patch

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from conflict_detector import (
    detect_conflicts,
    parse_purl,
    generate_resolution_suggestions,
)


class TestParsePurl:
    """Tests for parse_purl function."""

    def test_parse_maven_purl_with_version(self):
        """Test parsing Maven PURL with version."""
        purl = "pkg:maven/com.google.guava/guava@31.1-jre"
        parsed = parse_purl(purl)
        
        assert parsed["namespace"] == "com.google.guava"
        assert parsed["name"] == "guava"
        assert parsed["package"] == "com.google.guava/guava"
        assert parsed["version"] == "31.1-jre"

    def test_parse_maven_purl_without_version(self):
        """Test parsing Maven PURL without version."""
        purl = "pkg:maven/com.example/lib"
        parsed = parse_purl(purl)
        
        assert parsed["package"] == "com.example/lib"
        assert parsed["version"] == "unknown"

    def test_parse_non_maven_purl_returns_empty(self):
        """Test that non-Maven PURLs return empty dict."""
        purl = "pkg:npm/lodash@4.17.21"
        parsed = parse_purl(purl)
        
        assert parsed == {}

    def test_parse_purl_with_qualifiers(self):
        """Test parsing PURL with qualifiers."""
        purl = "pkg:maven/com.example/lib@1.0.0?type=jar&classifier=sources"
        parsed = parse_purl(purl)
        
        assert parsed["version"] == "1.0.0"
        assert parsed["package"] == "com.example/lib"

    def test_parse_purl_without_namespace(self):
        """Test parsing PURL without namespace."""
        purl = "pkg:maven/lib@1.0.0"
        parsed = parse_purl(purl)
        
        assert parsed["namespace"] == ""
        assert parsed["name"] == "lib"

    def test_parse_empty_purl(self):
        """Test parsing empty PURL."""
        purl = ""
        parsed = parse_purl(purl)
        
        assert parsed == {}

    def test_parse_malformed_purl(self):
        """Test parsing malformed PURL."""
        purl = "not-a-valid-purl"
        parsed = parse_purl(purl)
        
        assert parsed == {}


class TestDetectConflicts:
    """Tests for detect_conflicts function."""

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
        assert len(conflicts) == 0

    def test_detect_version_conflicts(self):
        """Test detection with version conflicts."""
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
        assert len(conflicts) == 1
        assert "com.google.guava/guava" in conflicts
        assert len(conflicts["com.google.guava/guava"]) == 2

    def test_detect_conflicts_with_coordinates(self):
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
        assert len(conflicts) >= 1
        assert "com.google.guava/guava" in conflicts

    def test_detect_conflicts_mixed_purl_and_coordinates(self):
        """Test detection with mixed PURL and coordinate dependencies."""
        dependencies = [
            {
                "purl": "pkg:maven/com.example/lib@1.0.0",
                "version": "1.0.0",
            },
            {
                "coordinates": "com.example:lib:2.0.0",
                "version": "2.0.0",
            },
        ]
        
        conflicts = detect_conflicts(dependencies)
        assert "com.example/lib" in conflicts

    def test_detect_conflicts_multiple_packages(self):
        """Test detection with multiple packages having conflicts."""
        dependencies = [
            {"purl": "pkg:maven/com.example/lib1@1.0.0"},
            {"purl": "pkg:maven/com.example/lib1@2.0.0"},
            {"purl": "pkg:maven/com.example/lib2@1.0.0"},
            {"purl": "pkg:maven/com.example/lib2@2.0.0"},
        ]
        
        conflicts = detect_conflicts(dependencies)
        assert len(conflicts) == 2
        assert "com.example/lib1" in conflicts
        assert "com.example/lib2" in conflicts

    def test_detect_conflicts_empty_dependencies(self):
        """Test detection with empty dependencies list."""
        conflicts = detect_conflicts([])
        assert len(conflicts) == 0

    def test_detect_conflicts_single_dependency(self):
        """Test detection with single dependency."""
        dependencies = [
            {"purl": "pkg:maven/com.example/lib@1.0.0"}
        ]
        
        conflicts = detect_conflicts(dependencies)
        assert len(conflicts) == 0

    def test_detect_conflicts_same_version_multiple_times(self):
        """Test that same version multiple times is not a conflict."""
        dependencies = [
            {"purl": "pkg:maven/com.example/lib@1.0.0"},
            {"purl": "pkg:maven/com.example/lib@1.0.0"},
            {"purl": "pkg:maven/com.example/lib@1.0.0"},
        ]
        
        conflicts = detect_conflicts(dependencies)
        assert len(conflicts) == 0

    def test_detect_conflicts_without_purl_or_coordinates(self):
        """Test dependencies without PURL or coordinates."""
        dependencies = [
            {"name": "lib1", "version": "1.0.0"},
            {"name": "lib1", "version": "2.0.0"},
        ]
        
        # Should not detect conflicts without PURL or coordinates
        conflicts = detect_conflicts(dependencies)
        assert len(conflicts) == 0


class TestGenerateResolutionSuggestions:
    """Tests for generate_resolution_suggestions function."""

    def test_generate_suggestions_for_conflicts(self):
        """Test generating resolution suggestions."""
        conflicts = {
            "com.google.guava/guava": [
                {"purl": "pkg:maven/com.google.guava/guava@31.1-jre", "version": "31.1-jre"},
                {"purl": "pkg:maven/com.google.guava/guava@30.0-jre", "version": "30.0-jre"},
            ]
        }
        
        suggestions = generate_resolution_suggestions(conflicts)
        
        assert len(suggestions) > 0
        assert any("com.google.guava/guava" in str(s) for s in suggestions)

    def test_generate_suggestions_empty_conflicts(self):
        """Test generating suggestions for empty conflicts."""
        suggestions = generate_resolution_suggestions({})
        
        assert isinstance(suggestions, list)

    def test_suggestions_include_versions(self):
        """Test that suggestions include version information."""
        conflicts = {
            "com.example/lib": [
                {"purl": "pkg:maven/com.example/lib@1.0.0", "version": "1.0.0"},
                {"purl": "pkg:maven/com.example/lib@2.0.0", "version": "2.0.0"},
                {"purl": "pkg:maven/com.example/lib@3.0.0", "version": "3.0.0"},
            ]
        }
        
        suggestions = generate_resolution_suggestions(conflicts)
        
        # Suggestions should reference the versions
        suggestion_text = str(suggestions)
        assert "1.0.0" in suggestion_text or "2.0.0" in suggestion_text or "3.0.0" in suggestion_text


class TestEdgeCases:
    """Tests for edge cases and boundary conditions."""

    def test_unicode_in_package_name(self):
        """Test handling of unicode in package names."""
        dependencies = [
            {"purl": "pkg:maven/com.example/lib-日本語@1.0.0"},
            {"purl": "pkg:maven/com.example/lib-日本語@2.0.0"},
        ]
        
        # Should handle unicode without errors
        conflicts = detect_conflicts(dependencies)
        assert len(conflicts) >= 1

    def test_very_long_version_string(self):
        """Test handling of very long version strings."""
        long_version = "1.0.0-" + "x" * 1000
        dependencies = [
            {"purl": f"pkg:maven/com.example/lib@{long_version}"},
            {"purl": "pkg:maven/com.example/lib@1.0.0"},
        ]
        
        conflicts = detect_conflicts(dependencies)
        assert "com.example/lib" in conflicts

    def test_special_characters_in_version(self):
        """Test handling of special characters in versions."""
        dependencies = [
            {"purl": "pkg:maven/com.example/lib@1.0.0-SNAPSHOT+build.123"},
            {"purl": "pkg:maven/com.example/lib@1.0.0-RC1"},
        ]
        
        conflicts = detect_conflicts(dependencies)
        assert "com.example/lib" in conflicts

    def test_malformed_coordinates(self):
        """Test handling of malformed coordinates."""
        dependencies = [
            {"coordinates": "invalid:format"},
            {"coordinates": "com.example:lib:1.0.0"},
        ]
        
        # Should not crash
        conflicts = detect_conflicts(dependencies)
        assert isinstance(conflicts, dict)

    def test_mixed_valid_and_invalid_purls(self):
        """Test mix of valid and invalid PURLs."""
        dependencies = [
            {"purl": "pkg:maven/com.example/lib@1.0.0"},
            {"purl": "not-a-valid-purl"},
            {"purl": "pkg:npm/lodash@4.17.21"},
            {"purl": "pkg:maven/com.example/lib@2.0.0"},
        ]
        
        conflicts = detect_conflicts(dependencies)
        assert "com.example/lib" in conflicts


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
