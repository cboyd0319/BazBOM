#!/usr/bin/env python3
"""Comprehensive unit tests for changelog_generator module.

Tests cover:
- ChangelogGenerator initialization and configuration
- Markdown changelog generation (happy path and edge cases)
- HTML changelog generation
- Text changelog generation
- Security section generation with vulnerability data
- Error handling for invalid formats
- Edge cases (empty diffs, large diffs, unicode, etc.)
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
from changelog_generator import ChangelogGenerator

# Import dependencies (will be mocked as needed)
from sbom_diff import SBOMDiff, Package


class TestChangelogGeneratorInit:
    """Tests for ChangelogGenerator initialization."""

    def test_init_without_vulnerability_data(self):
        """Test initialization without vulnerability data."""
        generator = ChangelogGenerator()
        assert generator.vulnerability_data == {}

    def test_init_with_empty_vulnerability_data(self):
        """Test initialization with empty vulnerability data."""
        generator = ChangelogGenerator(vulnerability_data={})
        assert generator.vulnerability_data == {}

    def test_init_with_vulnerability_data(self):
        """Test initialization with vulnerability data."""
        vuln_data = {
            "guava": {
                "30.0": ["CVE-2023-1234"],
                "31.0": []
            }
        }
        generator = ChangelogGenerator(vulnerability_data=vuln_data)
        assert generator.vulnerability_data == vuln_data


class TestGenerateMethod:
    """Tests for the generate() method."""

    @pytest.fixture
    def mock_diff(self):
        """Create a mock SBOMDiff object."""
        diff = Mock(spec=SBOMDiff)
        diff.to_dict.return_value = {
            'summary': {
                'total_old': 10,
                'total_new': 12,
                'added': 3,
                'removed': 1,
                'upgraded': 2,
                'downgraded': 0,
                'license_changed': 1
            }
        }
        diff.get_added_packages.return_value = []
        diff.get_removed_packages.return_value = []
        diff.get_upgraded_packages.return_value = []
        diff.get_downgraded_packages.return_value = []
        diff.get_license_changes.return_value = []
        return diff

    def test_generate_markdown_format(self, mock_diff):
        """Test generate with markdown format."""
        generator = ChangelogGenerator()
        result = generator.generate(mock_diff, "v1.0.0", "v1.1.0", format="markdown")
        
        assert isinstance(result, str)
        assert "# Release Notes: v1.0.0 → v1.1.0" in result
        assert "## 📊 Summary" in result
        assert "Total Dependencies (old)" in result

    def test_generate_html_format(self, mock_diff):
        """Test generate with HTML format."""
        generator = ChangelogGenerator()
        result = generator.generate(mock_diff, "v1.0.0", "v1.1.0", format="html")
        
        assert isinstance(result, str)
        assert "<!DOCTYPE html>" in result
        assert "<html>" in result

    def test_generate_text_format(self, mock_diff):
        """Test generate with text format."""
        generator = ChangelogGenerator()
        result = generator.generate(mock_diff, "v1.0.0", "v1.1.0", format="text")
        
        assert isinstance(result, str)
        assert "RELEASE NOTES: v1.0.0" in result and "v1.1.0" in result

    def test_generate_invalid_format_raises_error(self, mock_diff):
        """Test generate with invalid format raises ValueError."""
        generator = ChangelogGenerator()
        
        with pytest.raises(ValueError, match="Unsupported format: invalid"):
            generator.generate(mock_diff, "v1.0.0", "v1.1.0", format="invalid")

    def test_generate_default_parameters(self, mock_diff):
        """Test generate with default version labels."""
        generator = ChangelogGenerator()
        result = generator.generate(mock_diff)
        
        assert "previous → current" in result


class TestGenerateMarkdown:
    """Tests for markdown changelog generation."""

    @pytest.fixture
    def generator(self):
        """Create a ChangelogGenerator instance."""
        return ChangelogGenerator()

    @pytest.fixture
    def mock_diff_with_changes(self):
        """Create a mock diff with various changes."""
        diff = Mock(spec=SBOMDiff)
        diff.to_dict.return_value = {
            'summary': {
                'total_old': 5,
                'total_new': 7,
                'added': 2,
                'removed': 1,
                'upgraded': 2,
                'downgraded': 1,
                'license_changed': 1
            }
        }
        
        # Create mock packages
        pkg_added_1 = Mock(spec=Package)
        pkg_added_1.name = "new-lib"
        pkg_added_1.version = "1.0.0"
        pkg_added_1.license = "Apache-2.0"
        pkg_added_1.purl = "pkg:maven/com.example/new-lib@1.0.0"
        
        pkg_added_2 = Mock(spec=Package)
        pkg_added_2.name = "another-lib"
        pkg_added_2.version = "2.0.0"
        pkg_added_2.license = "NOASSERTION"
        pkg_added_2.purl = None
        
        pkg_removed = Mock(spec=Package)
        pkg_removed.name = "old-lib"
        pkg_removed.version = "0.9.0"
        pkg_removed.license = "MIT"
        
        pkg_old = Mock(spec=Package)
        pkg_old.name = "guava"
        pkg_old.version = "30.0"
        pkg_old.license = "Apache-2.0"
        
        pkg_new = Mock(spec=Package)
        pkg_new.name = "guava"
        pkg_new.version = "31.0"
        pkg_new.license = "Apache-2.0"
        
        pkg_downgrade_old = Mock(spec=Package)
        pkg_downgrade_old.name = "commons-io"
        pkg_downgrade_old.version = "2.11.0"
        
        pkg_downgrade_new = Mock(spec=Package)
        pkg_downgrade_new.name = "commons-io"
        pkg_downgrade_new.version = "2.10.0"
        
        pkg_license_old = Mock(spec=Package)
        pkg_license_old.name = "some-lib"
        pkg_license_old.version = "1.0.0"
        pkg_license_old.license = "MIT"
        
        pkg_license_new = Mock(spec=Package)
        pkg_license_new.name = "some-lib"
        pkg_license_new.version = "1.0.0"
        pkg_license_new.license = "Apache-2.0"
        
        diff.get_added_packages.return_value = [pkg_added_1, pkg_added_2]
        diff.get_removed_packages.return_value = [pkg_removed]
        diff.get_upgraded_packages.return_value = [(pkg_old, pkg_new)]
        diff.get_downgraded_packages.return_value = [(pkg_downgrade_old, pkg_downgrade_new)]
        diff.get_license_changes.return_value = [(pkg_license_old, pkg_license_new)]
        
        return diff

    def test_markdown_includes_header(self, generator, mock_diff_with_changes):
        """Test markdown includes proper header."""
        result = generator._generate_markdown(
            mock_diff_with_changes,
            "v1.0.0",
            "v2.0.0"
        )
        
        assert "# Release Notes: v1.0.0 → v2.0.0" in result or "# Release Notes: v1.0.0 \u2192 v2.0.0" in result
        assert "**Generated:**" in result

    def test_markdown_includes_summary_table(self, generator, mock_diff_with_changes):
        """Test markdown includes summary table."""
        result = generator._generate_markdown(
            mock_diff_with_changes,
            "v1.0.0",
            "v2.0.0"
        )
        
        assert "## 📊 Summary" in result
        assert "| Total Dependencies (old) | 5 |" in result
        assert "| Total Dependencies (new) | 7 |" in result
        assert "| ➕ Added | 2 |" in result
        assert "| ➖ Removed | 1 |" in result
        assert "| ⬆️ Upgraded | 2 |" in result
        assert "| ⬇️ Downgraded | 1 |" in result
        assert "| 📄 License Changed | 1 |" in result

    def test_markdown_includes_added_packages(self, generator, mock_diff_with_changes):
        """Test markdown includes added packages section."""
        result = generator._generate_markdown(
            mock_diff_with_changes,
            "v1.0.0",
            "v2.0.0"
        )
        
        assert "## ➕ New Dependencies (2)" in result
        assert "**new-lib** `1.0.0`" in result
        assert "License: `Apache-2.0`" in result
        assert "PURL: `pkg:maven/com.example/new-lib@1.0.0`" in result
        assert "**another-lib** `2.0.0`" in result

    def test_markdown_includes_removed_packages(self, generator, mock_diff_with_changes):
        """Test markdown includes removed packages section."""
        result = generator._generate_markdown(
            mock_diff_with_changes,
            "v1.0.0",
            "v2.0.0"
        )
        
        assert "## ➖ Removed Dependencies (1)" in result
        assert "**old-lib** `0.9.0`" in result

    def test_markdown_includes_upgraded_packages(self, generator, mock_diff_with_changes):
        """Test markdown includes upgraded packages section."""
        result = generator._generate_markdown(
            mock_diff_with_changes,
            "v1.0.0",
            "v2.0.0"
        )
        
        assert "## ⬆️ Upgraded Dependencies (1)" in result
        assert "**guava**: `30.0` → `31.0`" in result

    def test_markdown_includes_downgraded_packages(self, generator, mock_diff_with_changes):
        """Test markdown includes downgraded packages section."""
        result = generator._generate_markdown(
            mock_diff_with_changes,
            "v1.0.0",
            "v2.0.0"
        )
        
        assert "## ⚠️ Downgraded Dependencies (1)" in result
        assert "**Warning:** Downgrades may reintroduce known vulnerabilities." in result
        assert "**commons-io**: `2.11.0` → `2.10.0`" in result

    def test_markdown_includes_license_changes(self, generator, mock_diff_with_changes):
        """Test markdown includes license changes section."""
        result = generator._generate_markdown(
            mock_diff_with_changes,
            "v1.0.0",
            "v2.0.0"
        )
        
        assert "## 📄 License Changes (1)" in result
        assert "**some-lib** `1.0.0`" in result
        assert "`MIT` → `Apache-2.0`" in result

    def test_markdown_includes_footer(self, generator, mock_diff_with_changes):
        """Test markdown includes footer."""
        result = generator._generate_markdown(
            mock_diff_with_changes,
            "v1.0.0",
            "v2.0.0"
        )
        
        assert "*This changelog was automatically generated from SBOM comparison.*" in result

    def test_markdown_with_empty_diff(self, generator):
        """Test markdown generation with empty diff."""
        diff = Mock(spec=SBOMDiff)
        diff.to_dict.return_value = {
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
        diff.get_added_packages.return_value = []
        diff.get_removed_packages.return_value = []
        diff.get_upgraded_packages.return_value = []
        diff.get_downgraded_packages.return_value = []
        diff.get_license_changes.return_value = []
        
        result = generator._generate_markdown(diff, "v1.0.0", "v1.0.0")
        
        # Should still have header and summary, but no change sections
        assert "# Release Notes" in result
        assert "## 📊 Summary" in result
        assert "| ➕ Added | 0 |" in result
        # Should not have specific change sections
        assert "## ➕ New Dependencies" not in result
        assert "## ➖ Removed Dependencies" not in result


class TestSecuritySection:
    """Tests for security section generation."""

    @pytest.fixture
    def generator_with_vuln_data(self):
        """Create generator with vulnerability data."""
        vuln_data = {
            "guava": {
                "30.0": ["CVE-2023-1234", "CVE-2023-5678"],
                "31.0": ["CVE-2023-5678"],  # One fixed, one remains
                "32.0": []  # All fixed
            },
            "commons-io": {
                "2.10.0": ["CVE-2022-1111"],
                "2.11.0": []
            },
            "new-lib": {
                "1.0.0": ["CVE-2024-9999"]
            }
        }
        return ChangelogGenerator(vulnerability_data=vuln_data)

    @pytest.fixture
    def mock_diff_with_security_changes(self):
        """Create diff with security-relevant changes."""
        diff = Mock(spec=SBOMDiff)
        
        # Upgraded package (fixes CVE)
        pkg_old = Mock(spec=Package)
        pkg_old.name = "guava"
        pkg_old.version = "30.0"
        
        pkg_new = Mock(spec=Package)
        pkg_new.name = "guava"
        pkg_new.version = "31.0"
        
        # Downgraded package (reintroduces CVE)
        pkg_downgrade_old = Mock(spec=Package)
        pkg_downgrade_old.name = "commons-io"
        pkg_downgrade_old.version = "2.11.0"
        
        pkg_downgrade_new = Mock(spec=Package)
        pkg_downgrade_new.name = "commons-io"
        pkg_downgrade_new.version = "2.10.0"
        
        # New package with vulnerability
        pkg_added = Mock(spec=Package)
        pkg_added.name = "new-lib"
        pkg_added.version = "1.0.0"
        
        diff.get_upgraded_packages.return_value = [(pkg_old, pkg_new)]
        diff.get_downgraded_packages.return_value = [(pkg_downgrade_old, pkg_downgrade_new)]
        diff.get_added_packages.return_value = [pkg_added]
        
        return diff

    def test_security_section_with_fixed_vulnerabilities(
        self, generator_with_vuln_data, mock_diff_with_security_changes
    ):
        """Test security section shows fixed vulnerabilities."""
        result = generator_with_vuln_data._generate_security_section(
            mock_diff_with_security_changes
        )
        
        assert "✅ Vulnerabilities Fixed" in result
        assert "CVE-2023-1234" in result

    def test_security_section_with_introduced_vulnerabilities(
        self, generator_with_vuln_data, mock_diff_with_security_changes
    ):
        """Test security section shows introduced vulnerabilities."""
        result = generator_with_vuln_data._generate_security_section(
            mock_diff_with_security_changes
        )
        
        assert "⚠️ New Vulnerabilities Introduced" in result or "\u26a0\ufe0f New Vulnerabilities Introduced" in result
        # Note: The current implementation only checks new packages, not downgrades
        assert "CVE-2024-9999" in result  # From new package

    def test_security_section_without_changes(self):
        """Test security section when no security changes."""
        generator = ChangelogGenerator(vulnerability_data={})
        diff = Mock(spec=SBOMDiff)
        diff.get_upgraded_packages.return_value = []
        diff.get_added_packages.return_value = []
        
        result = generator._generate_security_section(diff)
        
        assert "No significant security changes detected." in result

    def test_markdown_includes_security_section_when_vuln_data_present(self):
        """Test markdown includes security section when vulnerability data available."""
        vuln_data = {"guava": {"30.0": ["CVE-2023-1234"]}}
        generator = ChangelogGenerator(vulnerability_data=vuln_data)
        
        diff = Mock(spec=SBOMDiff)
        diff.to_dict.return_value = {
            'summary': {
                'total_old': 1,
                'total_new': 1,
                'added': 0,
                'removed': 0,
                'upgraded': 0,
                'downgraded': 0,
                'license_changed': 0
            }
        }
        diff.get_added_packages.return_value = []
        diff.get_removed_packages.return_value = []
        diff.get_upgraded_packages.return_value = []
        diff.get_downgraded_packages.return_value = []
        diff.get_license_changes.return_value = []
        
        result = generator._generate_markdown(diff, "v1.0.0", "v2.0.0")
        
        assert "## 🔐 Security Impact" in result

    def test_markdown_omits_security_section_when_no_vuln_data(self):
        """Test markdown omits security section when no vulnerability data."""
        generator = ChangelogGenerator()
        
        diff = Mock(spec=SBOMDiff)
        diff.to_dict.return_value = {
            'summary': {
                'total_old': 1,
                'total_new': 1,
                'added': 0,
                'removed': 0,
                'upgraded': 0,
                'downgraded': 0,
                'license_changed': 0
            }
        }
        diff.get_added_packages.return_value = []
        diff.get_removed_packages.return_value = []
        diff.get_upgraded_packages.return_value = []
        diff.get_downgraded_packages.return_value = []
        diff.get_license_changes.return_value = []
        
        result = generator._generate_markdown(diff, "v1.0.0", "v2.0.0")
        
        assert "## 🔐 Security Impact" not in result


class TestGenerateHTML:
    """Tests for HTML changelog generation."""

    @pytest.fixture
    def generator(self):
        """Create a ChangelogGenerator instance."""
        return ChangelogGenerator()

    @pytest.fixture
    def simple_diff(self):
        """Create a simple mock diff."""
        diff = Mock(spec=SBOMDiff)
        diff.to_dict.return_value = {
            'summary': {
                'total_old': 1,
                'total_new': 1,
                'added': 0,
                'removed': 0,
                'upgraded': 0,
                'downgraded': 0,
                'license_changed': 0
            }
        }
        diff.get_added_packages.return_value = []
        diff.get_removed_packages.return_value = []
        diff.get_upgraded_packages.return_value = []
        diff.get_downgraded_packages.return_value = []
        diff.get_license_changes.return_value = []
        return diff

    def test_html_includes_doctype(self, generator, simple_diff):
        """Test HTML output includes DOCTYPE."""
        result = generator._generate_html(simple_diff, "v1.0.0", "v2.0.0")
        assert "<!DOCTYPE html>" in result

    def test_html_includes_structure(self, generator, simple_diff):
        """Test HTML output has proper structure."""
        result = generator._generate_html(simple_diff, "v1.0.0", "v2.0.0")
        assert "<html>" in result
        assert "<head>" in result
        assert "<body>" in result
        assert "</body>" in result
        assert "</html>" in result

    def test_html_includes_title(self, generator, simple_diff):
        """Test HTML output includes title."""
        result = generator._generate_html(simple_diff, "v1.0.0", "v2.0.0")
        assert "<title>Release Notes: v1.0.0 → v2.0.0</title>" in result

    def test_html_includes_styling(self, generator, simple_diff):
        """Test HTML output includes CSS styling."""
        result = generator._generate_html(simple_diff, "v1.0.0", "v2.0.0")
        assert "<style>" in result
        assert "font-family" in result


class TestGenerateText:
    """Tests for text changelog generation."""

    @pytest.fixture
    def generator(self):
        """Create a ChangelogGenerator instance."""
        return ChangelogGenerator()

    @pytest.fixture
    def mock_diff(self):
        """Create a simple mock diff."""
        diff = Mock(spec=SBOMDiff)
        diff.to_dict.return_value = {
            'summary': {
                'total_old': 5,
                'total_new': 7,
                'added': 2,
                'removed': 0,
                'upgraded': 1,
                'downgraded': 0,
                'license_changed': 0
            }
        }
        
        pkg_added = Mock(spec=Package)
        pkg_added.name = "new-lib"
        pkg_added.version = "1.0.0"
        pkg_added.license = "MIT"
        
        diff.get_added_packages.return_value = [pkg_added]
        diff.get_removed_packages.return_value = []
        diff.get_upgraded_packages.return_value = []
        diff.get_downgraded_packages.return_value = []
        diff.get_license_changes.return_value = []
        return diff

    def test_text_includes_header(self, generator, mock_diff):
        """Test text output includes header."""
        result = generator._generate_text(mock_diff, "v1.0.0", "v2.0.0")
        assert "RELEASE NOTES: v1.0.0 → v2.0.0" in result
        assert "=" * 80 in result

    def test_text_includes_summary(self, generator, mock_diff):
        """Test text output includes summary."""
        result = generator._generate_text(mock_diff, "v1.0.0", "v2.0.0")
        assert "SUMMARY" in result
        assert "Total Dependencies (old):  5" in result
        assert "Total Dependencies (new):  7" in result
        assert "Added:                     2" in result

    def test_text_includes_changes(self, generator, mock_diff):
        """Test text output includes change details."""
        result = generator._generate_text(mock_diff, "v1.0.0", "v2.0.0")
        # Check that NEW DEPENDENCIES section exists with count
        assert "NEW DEPENDENCIES" in result
        assert "new-lib@1.0.0" in result
        assert "(MIT)" in result


class TestEdgeCases:
    """Tests for edge cases and boundary conditions."""

    def test_unicode_in_package_names(self):
        """Test handling of unicode characters in package names."""
        generator = ChangelogGenerator()
        
        diff = Mock(spec=SBOMDiff)
        diff.to_dict.return_value = {
            'summary': {
                'total_old': 0,
                'total_new': 1,
                'added': 1,
                'removed': 0,
                'upgraded': 0,
                'downgraded': 0,
                'license_changed': 0
            }
        }
        
        pkg = Mock(spec=Package)
        pkg.name = "test-lib-日本語"
        pkg.version = "1.0.0"
        pkg.license = "MIT"
        pkg.purl = None
        
        diff.get_added_packages.return_value = [pkg]
        diff.get_removed_packages.return_value = []
        diff.get_upgraded_packages.return_value = []
        diff.get_downgraded_packages.return_value = []
        diff.get_license_changes.return_value = []
        
        # Should not raise exception
        result = generator.generate(diff, format="markdown")
        assert "test-lib-日本語" in result

    def test_large_number_of_changes(self):
        """Test handling of large number of changes."""
        generator = ChangelogGenerator()
        
        diff = Mock(spec=SBOMDiff)
        diff.to_dict.return_value = {
            'summary': {
                'total_old': 1000,
                'total_new': 2000,
                'added': 1000,
                'removed': 0,
                'upgraded': 0,
                'downgraded': 0,
                'license_changed': 0
            }
        }
        
        # Create 1000 packages
        packages = []
        for i in range(1000):
            pkg = Mock(spec=Package)
            pkg.name = f"lib-{i}"
            pkg.version = "1.0.0"
            pkg.license = "MIT"
            pkg.purl = None
            packages.append(pkg)
        
        diff.get_added_packages.return_value = packages
        diff.get_removed_packages.return_value = []
        diff.get_upgraded_packages.return_value = []
        diff.get_downgraded_packages.return_value = []
        diff.get_license_changes.return_value = []
        
        # Should complete without error
        result = generator.generate(diff, format="markdown")
        assert "NEW DEPENDENCIES (1000)" in result or "New Dependencies (1000)" in result

    def test_special_characters_in_versions(self):
        """Test handling of special characters in version strings."""
        generator = ChangelogGenerator()
        
        diff = Mock(spec=SBOMDiff)
        diff.to_dict.return_value = {
            'summary': {
                'total_old': 0,
                'total_new': 1,
                'added': 1,
                'removed': 0,
                'upgraded': 0,
                'downgraded': 0,
                'license_changed': 0
            }
        }
        
        pkg = Mock(spec=Package)
        pkg.name = "test-lib"
        pkg.version = "1.0.0-SNAPSHOT+build.123"
        pkg.license = "MIT"
        pkg.purl = None
        
        diff.get_added_packages.return_value = [pkg]
        diff.get_removed_packages.return_value = []
        diff.get_upgraded_packages.return_value = []
        diff.get_downgraded_packages.return_value = []
        diff.get_license_changes.return_value = []
        
        result = generator.generate(diff, format="markdown")
        assert "1.0.0-SNAPSHOT+build.123" in result
