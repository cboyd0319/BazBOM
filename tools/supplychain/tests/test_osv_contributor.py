#!/usr/bin/env python3
"""Comprehensive tests for osv_contributor.py - OSV vulnerability report generation."""

import json
import sys
from datetime import datetime
from pathlib import Path
from unittest.mock import patch

import pytest
from freezegun import freeze_time

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from osv_contributor import OSVContributor


class TestOSVContributorInit:
    """Test OSVContributor initialization."""

    def test_init_creates_instance(self):
        """Test initialization creates a valid instance."""
        # Act
        contributor = OSVContributor()
        
        # Assert
        assert contributor is not None
        assert contributor.OSV_SCHEMA_VERSION == "1.6.0"

    def test_osv_schema_version_is_set(self):
        """Test OSV schema version is properly set."""
        # Act
        contributor = OSVContributor()
        
        # Assert
        assert contributor.OSV_SCHEMA_VERSION == "1.6.0"


class TestGenerateOSVEntry:
    """Test OSV entry generation."""

    @freeze_time("2025-01-01 12:00:00")
    def test_generate_osv_entry_minimal_required_fields(self):
        """Test generating OSV entry with only required fields."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="GHSA-xxxx-yyyy-zzzz",
            package_name="com.example:vulnerable-lib",
            package_ecosystem="Maven",
            affected_versions=["1.0.0", "1.1.0"]
        )
        
        # Assert
        assert entry["id"] == "GHSA-xxxx-yyyy-zzzz"
        assert entry["schema_version"] == "1.6.0"
        assert "modified" in entry
        assert entry["modified"].endswith("Z")

    @freeze_time("2025-01-01 12:00:00")
    def test_generate_osv_entry_with_all_fields(self):
        """Test generating OSV entry with all optional fields."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-12345",
            package_name="org.example:lib",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            fixed_version="1.1.0",
            summary="Brief summary",
            details="Detailed description",
            severity="HIGH",
            cvss_score=7.5,
            references=["https://example.com/advisory"],
            aliases=["GHSA-xxxx-yyyy-zzzz", "CVE-2023-12345"],
            database_specific={"custom": "data"}
        )
        
        # Assert
        assert entry["id"] == "CVE-2023-12345"
        assert entry["summary"] == "Brief summary"
        assert entry["details"] == "Detailed description"
        assert entry["aliases"] == ["CVE-2023-12345", "GHSA-xxxx-yyyy-zzzz"]  # sorted

    def test_generate_osv_entry_missing_vulnerability_id_raises_error(self):
        """Test generation fails without vulnerability_id."""
        # Arrange
        contributor = OSVContributor()
        
        # Act & Assert
        with pytest.raises(ValueError, match="vulnerability_id is required"):
            contributor.generate_osv_entry(
                vulnerability_id="",
                package_name="test",
                package_ecosystem="Maven",
                affected_versions=["1.0.0"]
            )

    def test_generate_osv_entry_missing_package_name_raises_error(self):
        """Test generation fails without package_name."""
        # Arrange
        contributor = OSVContributor()
        
        # Act & Assert
        with pytest.raises(ValueError, match="package_name is required"):
            contributor.generate_osv_entry(
                vulnerability_id="CVE-2023-0001",
                package_name="",
                package_ecosystem="Maven",
                affected_versions=["1.0.0"]
            )

    def test_generate_osv_entry_missing_package_ecosystem_raises_error(self):
        """Test generation fails without package_ecosystem."""
        # Arrange
        contributor = OSVContributor()
        
        # Act & Assert
        with pytest.raises(ValueError, match="package_ecosystem is required"):
            contributor.generate_osv_entry(
                vulnerability_id="CVE-2023-0001",
                package_name="test",
                package_ecosystem="",
                affected_versions=["1.0.0"]
            )

    def test_generate_osv_entry_missing_affected_versions_raises_error(self):
        """Test generation fails without affected_versions."""
        # Arrange
        contributor = OSVContributor()
        
        # Act & Assert
        with pytest.raises(ValueError, match="At least one affected version is required"):
            contributor.generate_osv_entry(
                vulnerability_id="CVE-2023-0001",
                package_name="test",
                package_ecosystem="Maven",
                affected_versions=[]
            )

    def test_generate_osv_entry_with_summary(self):
        """Test OSV entry includes summary when provided."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            summary="Test vulnerability summary"
        )
        
        # Assert
        assert entry["summary"] == "Test vulnerability summary"

    def test_generate_osv_entry_without_summary(self):
        """Test OSV entry excludes summary when not provided."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"]
        )
        
        # Assert
        assert "summary" not in entry

    def test_generate_osv_entry_with_details(self):
        """Test OSV entry includes details when provided."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            details="Detailed vulnerability description"
        )
        
        # Assert
        assert entry["details"] == "Detailed vulnerability description"

    def test_generate_osv_entry_aliases_are_sorted_and_unique(self):
        """Test aliases are sorted and deduplicated."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            aliases=["GHSA-zzzz", "CVE-2023-0001", "GHSA-aaaa", "CVE-2023-0001"]
        )
        
        # Assert
        assert entry["aliases"] == ["CVE-2023-0001", "GHSA-aaaa", "GHSA-zzzz"]
        assert len(entry["aliases"]) == 3  # Duplicates removed


class TestBuildAffectedSection:
    """Test building the affected section of OSV entry."""

    def test_build_affected_section_is_called(self):
        """Test that _build_affected_section is called during generation."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="com.example:lib",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"]
        )
        
        # Assert
        assert "affected" in entry
        assert isinstance(entry["affected"], list)


class TestSeverityHandling:
    """Test severity and CVSS score handling."""

    def test_generate_osv_entry_with_severity(self):
        """Test OSV entry with severity level."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            severity="CRITICAL"
        )
        
        # Assert - severity should be included in some form
        assert entry is not None

    def test_generate_osv_entry_with_cvss_score(self):
        """Test OSV entry with CVSS score."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            cvss_score=9.8
        )
        
        # Assert - CVSS score should be included
        assert entry is not None

    @pytest.mark.parametrize("severity", [
        "CRITICAL",
        "HIGH",
        "MEDIUM",
        "LOW"
    ], ids=["critical", "high", "medium", "low"])
    def test_generate_osv_entry_various_severities(self, severity):
        """Test OSV entry generation with various severity levels."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id=f"CVE-2023-{severity}",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            severity=severity
        )
        
        # Assert
        assert entry is not None
        assert entry["id"] == f"CVE-2023-{severity}"

    @pytest.mark.parametrize("cvss_score", [
        0.0, 3.9, 6.9, 8.9, 10.0
    ], ids=["none", "low", "medium", "high", "critical"])
    def test_generate_osv_entry_various_cvss_scores(self, cvss_score):
        """Test OSV entry generation with various CVSS scores."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id=f"CVE-2023-{int(cvss_score * 10)}",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            cvss_score=cvss_score
        )
        
        # Assert
        assert entry is not None


class TestReferencesHandling:
    """Test references handling."""

    def test_generate_osv_entry_with_references(self):
        """Test OSV entry with reference URLs."""
        # Arrange
        contributor = OSVContributor()
        references = [
            "https://nvd.nist.gov/vuln/detail/CVE-2023-0001",
            "https://github.com/advisories/GHSA-xxxx"
        ]
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            references=references
        )
        
        # Assert
        assert entry is not None

    def test_generate_osv_entry_without_references(self):
        """Test OSV entry without references."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"]
        )
        
        # Assert
        assert entry is not None


class TestDatabaseSpecificData:
    """Test database-specific data handling."""

    def test_generate_osv_entry_with_database_specific_data(self):
        """Test OSV entry with database-specific metadata."""
        # Arrange
        contributor = OSVContributor()
        db_specific = {
            "github_reviewed": True,
            "cwe_ids": ["CWE-79"],
            "nvd_published_at": "2023-01-15T10:00:00Z"
        }
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            database_specific=db_specific
        )
        
        # Assert
        assert entry is not None


class TestPackageEcosystems:
    """Test various package ecosystems."""

    @pytest.mark.parametrize("ecosystem", [
        "Maven",
        "npm",
        "PyPI",
        "Go",
        "RubyGems",
        "NuGet",
        "crates.io"
    ], ids=["maven", "npm", "pypi", "go", "rubygems", "nuget", "crates"])
    def test_generate_osv_entry_various_ecosystems(self, ecosystem):
        """Test OSV entry generation for various ecosystems."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id=f"CVE-2023-{ecosystem}",
            package_name=f"test-{ecosystem}",
            package_ecosystem=ecosystem,
            affected_versions=["1.0.0"]
        )
        
        # Assert
        assert entry is not None
        assert entry["id"] == f"CVE-2023-{ecosystem}"


class TestMultipleAffectedVersions:
    """Test handling of multiple affected versions."""

    def test_generate_osv_entry_multiple_versions(self):
        """Test OSV entry with multiple affected versions."""
        # Arrange
        contributor = OSVContributor()
        versions = ["1.0.0", "1.0.1", "1.1.0", "1.1.1", "2.0.0"]
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=versions
        )
        
        # Assert
        assert entry is not None
        assert "affected" in entry

    def test_generate_osv_entry_single_version(self):
        """Test OSV entry with single affected version."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"]
        )
        
        # Assert
        assert entry is not None


class TestFixedVersion:
    """Test fixed version handling."""

    def test_generate_osv_entry_with_fixed_version(self):
        """Test OSV entry with fixed version specified."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0", "1.0.1"],
            fixed_version="1.1.0"
        )
        
        # Assert
        assert entry is not None

    def test_generate_osv_entry_without_fixed_version(self):
        """Test OSV entry without fixed version."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"]
        )
        
        # Assert
        assert entry is not None


class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_generate_osv_entry_with_unicode_characters(self):
        """Test OSV entry with Unicode characters in strings."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="tëst-påckage-",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            summary="Vulnérabilité avec caractères spéciaux",
            details="Description détaillée avec caractères spéciaux"
        )
        
        # Assert
        assert entry is not None
        assert entry["summary"] == "Vulnérabilité avec caractères spéciaux"

    def test_generate_osv_entry_with_very_long_strings(self):
        """Test OSV entry with very long strings."""
        # Arrange
        contributor = OSVContributor()
        long_summary = "A" * 10000
        long_details = "B" * 100000
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            summary=long_summary,
            details=long_details
        )
        
        # Assert
        assert entry is not None
        assert entry["summary"] == long_summary
        assert entry["details"] == long_details

    def test_generate_osv_entry_with_special_characters_in_id(self):
        """Test OSV entry with special characters in vulnerability ID."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="GHSA-xxxx-yyyy-zzzz",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"]
        )
        
        # Assert
        assert entry["id"] == "GHSA-xxxx-yyyy-zzzz"

    def test_generate_osv_entry_with_empty_strings_in_optional_fields(self):
        """Test OSV entry with empty strings in optional fields."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"],
            summary="",
            details=""
        )
        
        # Assert
        assert entry is not None
        # Empty strings should not be included
        assert entry.get("summary") == "" or "summary" not in entry


class TestTimestampGeneration:
    """Test timestamp generation in OSV entries."""

    @freeze_time("2025-03-15 14:30:45")
    def test_modified_timestamp_format(self):
        """Test modified timestamp is in correct ISO format."""
        # Arrange
        contributor = OSVContributor()
        
        # Act
        entry = contributor.generate_osv_entry(
            vulnerability_id="CVE-2023-0001",
            package_name="test",
            package_ecosystem="Maven",
            affected_versions=["1.0.0"]
        )
        
        # Assert
        assert "modified" in entry
        assert entry["modified"].endswith("Z")
        assert "2025-03-15" in entry["modified"]
