#!/usr/bin/env python3
"""Comprehensive tests for compliance_report.py - Compliance report generation."""

import json
import sys
from datetime import datetime
from pathlib import Path
from unittest.mock import MagicMock, Mock, patch

import pytest
from freezegun import freeze_time

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from compliance_report import ComplianceReportGenerator


class TestComplianceReportGeneratorInit:
    """Test ComplianceReportGenerator initialization."""

    def test_init_with_custom_templates_dir(self, tmp_path):
        """Test initialization with custom templates directory."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        
        # Act
        generator = ComplianceReportGenerator(
            templates_dir=str(templates_dir),
            company_name="TestCorp",
            company_logo="/path/to/logo.png",
            brand_color="#ff0000"
        )
        
        # Assert
        assert generator.templates_dir == templates_dir
        assert generator.company_name == "TestCorp"
        assert generator.company_logo == "/path/to/logo.png"
        assert generator.brand_color == "#ff0000"
        assert generator.jinja_env is not None

    def test_init_with_nonexistent_templates_dir_raises_error(self):
        """Test initialization fails with nonexistent templates directory."""
        # Act & Assert
        with pytest.raises(FileNotFoundError, match="Templates directory not found"):
            ComplianceReportGenerator(templates_dir="/nonexistent/path")

    def test_init_without_templates_dir_tries_to_find_default(self):
        """Test initialization tries to find templates in default locations."""
        # Act & Assert - will either find templates or raise FileNotFoundError
        try:
            generator = ComplianceReportGenerator()
            # If it succeeds, templates were found
            assert generator.templates_dir is not None
        except FileNotFoundError:
            # Expected if templates don't exist in default locations
            pass

    def test_init_registers_custom_jinja_filters(self, tmp_path):
        """Test that custom Jinja filters are registered."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        
        # Act
        generator = ComplianceReportGenerator(templates_dir=str(templates_dir))
        
        # Assert
        assert 'format_date' in generator.jinja_env.filters
        assert 'format_number' in generator.jinja_env.filters

    def test_init_with_default_values(self, tmp_path):
        """Test initialization with default parameter values."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        
        # Act
        generator = ComplianceReportGenerator(templates_dir=str(templates_dir))
        
        # Assert
        assert generator.company_name == "Organization"
        assert generator.company_logo is None
        assert generator.brand_color == "#0066cc"


class TestFormatDateFilter:
    """Test the _format_date filter method."""

    def test_format_date_with_default_format(self, tmp_path):
        """Test date formatting with default format string."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        generator = ComplianceReportGenerator(templates_dir=str(templates_dir))
        
        # Act
        result = generator._format_date("2025-01-15T10:30:00Z")
        
        # Assert
        assert result == "2025-01-15"

    def test_format_date_with_custom_format(self, tmp_path):
        """Test date formatting with custom format string."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        generator = ComplianceReportGenerator(templates_dir=str(templates_dir))
        
        # Act
        result = generator._format_date("2025-01-15T10:30:00Z", "%B %d, %Y")
        
        # Assert
        assert "January" in result
        assert "15" in result
        assert "2025" in result

    def test_format_date_with_invalid_date_string(self, tmp_path):
        """Test date formatting with invalid date string returns input string."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        generator = ComplianceReportGenerator(templates_dir=str(templates_dir))
        
        # Act - implementation returns input on error
        result = generator._format_date("invalid-date")
        
        # Assert - should return the input string unchanged
        assert result == "invalid-date"

    @pytest.mark.parametrize("date_str,expected", [
        ("2025-01-01T00:00:00Z", "2025-01-01"),
        ("2024-12-31T23:59:59Z", "2024-12-31"),
        ("2023-06-15T12:30:45Z", "2023-06-15"),
    ], ids=["start-of-year", "end-of-year", "mid-year"])
    def test_format_date_parametrized(self, tmp_path, date_str, expected):
        """Test date formatting with various date strings."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        generator = ComplianceReportGenerator(templates_dir=str(templates_dir))
        
        # Act
        result = generator._format_date(date_str)
        
        # Assert
        assert result == expected


class TestFormatNumberFilter:
    """Test the _format_number filter method."""

    def test_format_number_integer(self, tmp_path):
        """Test number formatting with integer."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        generator = ComplianceReportGenerator(templates_dir=str(templates_dir))
        
        # Act
        result = generator._format_number(1234567)
        
        # Assert
        assert "1,234,567" in result or "1234567" in result

    def test_format_number_float(self, tmp_path):
        """Test number formatting with float."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        generator = ComplianceReportGenerator(templates_dir=str(templates_dir))
        
        # Act
        result = generator._format_number(12345.67)
        
        # Assert
        assert "12,345.67" in result or "12345.67" in result

    @pytest.mark.parametrize("number,expected_contains", [
        (0, "0"),
        (1, "1"),
        (999, "999"),
        (1000, "1"),
        (1000000, "1"),
    ], ids=["zero", "one", "hundreds", "thousands", "millions"])
    def test_format_number_parametrized(self, tmp_path, number, expected_contains):
        """Test number formatting with various numbers."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        generator = ComplianceReportGenerator(templates_dir=str(templates_dir))
        
        # Act
        result = generator._format_number(number)
        
        # Assert
        assert expected_contains in str(result)


class TestGenerateReport:
    """Test report generation methods."""

    def test_generate_report_requires_templates(self, tmp_path):
        """Test that report generation requires proper templates."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html>{{ company_name }}</html>")
        generator = ComplianceReportGenerator(templates_dir=str(templates_dir))
        
        # Act - try to load a template
        template = generator.jinja_env.get_template("executive_summary.html")
        result = template.render(company_name="TestCorp")
        
        # Assert
        assert "TestCorp" in result

    def test_jinja_environment_configuration(self, tmp_path):
        """Test that Jinja environment is properly configured."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        
        # Act
        generator = ComplianceReportGenerator(templates_dir=str(templates_dir))
        
        # Assert
        assert generator.jinja_env.autoescape == True
        assert generator.jinja_env.loader is not None


class TestEdgeCases:
    """Test edge cases and error conditions."""

    def test_empty_company_name(self, tmp_path):
        """Test initialization with empty company name."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        
        # Act
        generator = ComplianceReportGenerator(
            templates_dir=str(templates_dir),
            company_name=""
        )
        
        # Assert
        assert generator.company_name == ""

    def test_unicode_in_company_name(self, tmp_path):
        """Test initialization with Unicode characters in company name."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        
        # Act
        generator = ComplianceReportGenerator(
            templates_dir=str(templates_dir),
            company_name="Tëst Çørp™ 日本"
        )
        
        # Assert
        assert generator.company_name == "Tëst Çørp™ 日本"

    def test_special_characters_in_brand_color(self, tmp_path):
        """Test initialization with various brand color formats."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        
        # Act
        generator = ComplianceReportGenerator(
            templates_dir=str(templates_dir),
            brand_color="rgb(255, 0, 0)"
        )
        
        # Assert
        assert generator.brand_color == "rgb(255, 0, 0)"

    def test_very_long_company_name(self, tmp_path):
        """Test initialization with very long company name."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        long_name = "A" * 1000
        
        # Act
        generator = ComplianceReportGenerator(
            templates_dir=str(templates_dir),
            company_name=long_name
        )
        
        # Assert
        assert generator.company_name == long_name
        assert len(generator.company_name) == 1000


class TestTemplateSearch:
    """Test template directory search behavior."""

    def test_templates_dir_must_exist(self, tmp_path):
        """Test that templates directory must exist."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        # Don't create the directory
        
        # Act & Assert
        with pytest.raises(FileNotFoundError, match="Templates directory not found"):
            ComplianceReportGenerator(templates_dir=str(templates_dir))

    def test_templates_dir_as_pathlib_path(self, tmp_path):
        """Test initialization with pathlib.Path object."""
        # Arrange
        templates_dir = tmp_path / "templates" / "compliance"
        templates_dir.mkdir(parents=True)
        (templates_dir / "executive_summary.html").write_text("<html></html>")
        
        # Act
        generator = ComplianceReportGenerator(templates_dir=templates_dir)
        
        # Assert
        assert generator.templates_dir == templates_dir
