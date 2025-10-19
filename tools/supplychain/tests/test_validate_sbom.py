#!/usr/bin/env python3
"""Tests for validators/validate_sbom.py - SPDX SBOM validation."""

import json
import sys
from pathlib import Path
from unittest.mock import patch, mock_open

import pytest

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from validators.validate_sbom import (
    validate_spdx_required_fields,
    validate_package,
    validate_sbom_file,
    main,
)


class TestValidateSpdxRequiredFields:
    """Test validation of SPDX required fields."""
    
    def test_validate_valid_sbom(self, sample_sbom_data):
        """Test validation of a valid SBOM."""
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert errors == []
    
    def test_validate_missing_spdx_version(self, sample_sbom_data):
        """Test detection of missing spdxVersion field."""
        del sample_sbom_data["spdxVersion"]
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("spdxVersion" in error for error in errors)
    
    def test_validate_invalid_spdx_version(self, sample_sbom_data):
        """Test detection of invalid spdxVersion."""
        sample_sbom_data["spdxVersion"] = "SPDX-2.2"
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("Invalid spdxVersion" in error for error in errors)
    
    def test_validate_missing_data_license(self, sample_sbom_data):
        """Test detection of missing dataLicense field."""
        del sample_sbom_data["dataLicense"]
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("dataLicense" in error for error in errors)
    
    def test_validate_invalid_data_license(self, sample_sbom_data):
        """Test detection of invalid dataLicense."""
        sample_sbom_data["dataLicense"] = "MIT"
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("Invalid dataLicense" in error for error in errors)
    
    def test_validate_missing_spdxid(self, sample_sbom_data):
        """Test detection of missing SPDXID field."""
        del sample_sbom_data["SPDXID"]
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("SPDXID" in error for error in errors)
    
    def test_validate_invalid_spdxid(self, sample_sbom_data):
        """Test detection of invalid SPDXID."""
        sample_sbom_data["SPDXID"] = "InvalidID"
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("Invalid SPDXID" in error for error in errors)
    
    def test_validate_missing_name(self, sample_sbom_data):
        """Test detection of missing name field."""
        del sample_sbom_data["name"]
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("name" in error for error in errors)
    
    def test_validate_missing_document_namespace(self, sample_sbom_data):
        """Test detection of missing documentNamespace field."""
        del sample_sbom_data["documentNamespace"]
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("documentNamespace" in error for error in errors)
    
    def test_validate_missing_creation_info(self, sample_sbom_data):
        """Test detection of missing creationInfo field."""
        del sample_sbom_data["creationInfo"]
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("creationInfo" in error for error in errors)
    
    def test_validate_creation_info_missing_created(self, sample_sbom_data):
        """Test detection of missing created field in creationInfo."""
        del sample_sbom_data["creationInfo"]["created"]
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("creationInfo.created" in error for error in errors)
    
    def test_validate_creation_info_missing_creators(self, sample_sbom_data):
        """Test detection of missing creators field in creationInfo."""
        del sample_sbom_data["creationInfo"]["creators"]
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("creationInfo.creators" in error for error in errors)
    
    def test_validate_creation_info_empty_creators(self, sample_sbom_data):
        """Test detection of empty creators list in creationInfo."""
        sample_sbom_data["creationInfo"]["creators"] = []
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("creationInfo.creators" in error for error in errors)
    
    def test_validate_missing_packages(self, sample_sbom_data):
        """Test detection of missing packages field."""
        del sample_sbom_data["packages"]
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("packages" in error for error in errors)
    
    def test_validate_packages_not_list(self, sample_sbom_data):
        """Test detection of packages field not being a list."""
        sample_sbom_data["packages"] = "not a list"
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("packages" in error and "list" in error for error in errors)
    
    def test_validate_missing_relationships(self, sample_sbom_data):
        """Test detection of missing relationships field."""
        del sample_sbom_data["relationships"]
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("relationships" in error for error in errors)
    
    def test_validate_relationships_not_list(self, sample_sbom_data):
        """Test detection of relationships field not being a list."""
        sample_sbom_data["relationships"] = "not a list"
        
        errors = validate_spdx_required_fields(sample_sbom_data)
        
        assert any("relationships" in error and "list" in error for error in errors)
    
    def test_validate_multiple_errors(self):
        """Test detection of multiple validation errors."""
        invalid_sbom = {
            "spdxVersion": "SPDX-2.2",  # Invalid
            "dataLicense": "MIT",  # Invalid
            "SPDXID": "WrongID",  # Invalid
        }
        
        errors = validate_spdx_required_fields(invalid_sbom)
        
        # Should have multiple errors
        assert len(errors) > 3


class TestValidatePackage:
    """Test validation of individual packages."""
    
    def test_validate_valid_package(self):
        """Test validation of a valid package."""
        package = {
            "SPDXID": "SPDXRef-Package-guava",
            "name": "guava",
            "downloadLocation": "https://repo1.maven.org/maven2/com/google/guava/guava/31.1-jre/guava-31.1-jre.jar",
        }
        
        errors = validate_package(package, 0)
        
        assert errors == []
    
    def test_validate_package_missing_spdxid(self):
        """Test detection of missing SPDXID in package."""
        package = {
            "name": "guava",
            "downloadLocation": "https://example.com/guava.jar",
        }
        
        errors = validate_package(package, 0)
        
        assert any("SPDXID" in error for error in errors)
    
    def test_validate_package_missing_name(self):
        """Test detection of missing name in package."""
        package = {
            "SPDXID": "SPDXRef-Package-test",
            "downloadLocation": "https://example.com/test.jar",
        }
        
        errors = validate_package(package, 0)
        
        assert any("name" in error for error in errors)
    
    def test_validate_package_missing_download_location(self):
        """Test detection of missing downloadLocation in package."""
        package = {
            "SPDXID": "SPDXRef-Package-test",
            "name": "test",
        }
        
        errors = validate_package(package, 0)
        
        assert any("downloadLocation" in error for error in errors)
    
    def test_validate_package_invalid_spdxid_format(self):
        """Test detection of invalid SPDXID format."""
        package = {
            "SPDXID": "InvalidID",
            "name": "test",
            "downloadLocation": "NOASSERTION",
        }
        
        errors = validate_package(package, 0)
        
        assert any("Invalid SPDXID format" in error for error in errors)
    
    def test_validate_package_valid_download_location_noassertion(self):
        """Test that NOASSERTION is valid for downloadLocation."""
        package = {
            "SPDXID": "SPDXRef-Package-test",
            "name": "test",
            "downloadLocation": "NOASSERTION",
        }
        
        errors = validate_package(package, 0)
        
        assert not any("downloadLocation" in error for error in errors)
    
    def test_validate_package_valid_download_location_none(self):
        """Test that NONE is valid for downloadLocation."""
        package = {
            "SPDXID": "SPDXRef-Package-test",
            "name": "test",
            "downloadLocation": "NONE",
        }
        
        errors = validate_package(package, 0)
        
        assert not any("downloadLocation" in error for error in errors)
    
    @pytest.mark.parametrize("url_prefix", [
        "http://",
        "https://",
        "git://",
        "ftp://",
    ], ids=["http", "https", "git", "ftp"])
    def test_validate_package_valid_download_location_urls(self, url_prefix):
        """Test that various URL schemes are valid for downloadLocation."""
        package = {
            "SPDXID": "SPDXRef-Package-test",
            "name": "test",
            "downloadLocation": f"{url_prefix}example.com/file.jar",
        }
        
        errors = validate_package(package, 0)
        
        assert not any("downloadLocation" in error for error in errors)
    
    def test_validate_package_invalid_download_location(self):
        """Test detection of invalid downloadLocation."""
        package = {
            "SPDXID": "SPDXRef-Package-test",
            "name": "test",
            "downloadLocation": "invalid-location",
        }
        
        errors = validate_package(package, 0)
        
        assert any("Invalid downloadLocation" in error for error in errors)
    
    def test_validate_package_error_includes_index(self):
        """Test that package validation errors include the package index."""
        package = {
            "name": "test",
            "downloadLocation": "https://example.com/test.jar",
        }
        
        errors = validate_package(package, 5)
        
        # Error should include "Package 5"
        assert any("Package 5" in error for error in errors)


class TestValidateSbomFile:
    """Test validation of SBOM files."""
    
    def test_validate_valid_sbom_file(self, sample_sbom_data, temp_json_file):
        """Test validation of a valid SBOM file."""
        sbom_file = temp_json_file(sample_sbom_data, "valid.spdx.json")
        
        is_valid, errors = validate_sbom_file(str(sbom_file))
        
        assert is_valid
        assert errors == []
    
    def test_validate_invalid_sbom_file(self, temp_json_file):
        """Test validation of an invalid SBOM file."""
        invalid_sbom = {
            "spdxVersion": "SPDX-2.2",  # Invalid
            "name": "test",
        }
        sbom_file = temp_json_file(invalid_sbom, "invalid.spdx.json")
        
        is_valid, errors = validate_sbom_file(str(sbom_file))
        
        assert not is_valid
        assert len(errors) > 0
    
    def test_validate_nonexistent_file(self):
        """Test validation of non-existent file."""
        is_valid, errors = validate_sbom_file("/nonexistent/file.json")
        
        assert not is_valid
        assert any("File not found" in error for error in errors)
    
    def test_validate_invalid_json_file(self, tmp_path):
        """Test validation of file with invalid JSON."""
        sbom_file = tmp_path / "invalid.json"
        with open(sbom_file, "w") as f:
            f.write("not valid json {{{")
        
        is_valid, errors = validate_sbom_file(str(sbom_file))
        
        assert not is_valid
        assert any("Invalid JSON" in error for error in errors)
    
    def test_validate_empty_json_file(self, temp_json_file):
        """Test validation of empty JSON file."""
        sbom_file = temp_json_file({}, "empty.json")
        
        is_valid, errors = validate_sbom_file(str(sbom_file))
        
        assert not is_valid
        assert len(errors) > 0


class TestMainFunction:
    """Test suite for main CLI function."""
    
    @patch('sys.argv', ['validate_sbom.py', 'test.json'])
    @patch('validators.validate_sbom.validate_sbom_file')
    def test_main_single_valid_file(self, mock_validate):
        """Test main with single valid file."""
        # Arrange
        mock_validate.return_value = (True, [])
        
        # Act
        result = main()
        
        # Assert
        assert result == 0
        mock_validate.assert_called_once_with('test.json')
    
    @patch('sys.argv', ['validate_sbom.py', 'invalid.json'])
    @patch('validators.validate_sbom.validate_sbom_file')
    def test_main_single_invalid_file(self, mock_validate):
        """Test main with single invalid file."""
        # Arrange
        mock_validate.return_value = (False, ['Error 1', 'Error 2'])
        
        # Act
        result = main()
        
        # Assert
        assert result == 1
    
    @patch('sys.argv', ['validate_sbom.py', 'file1.json', 'file2.json', 'file3.json'])
    @patch('validators.validate_sbom.validate_sbom_file')
    def test_main_multiple_files_all_valid(self, mock_validate):
        """Test main with multiple valid files."""
        # Arrange
        mock_validate.return_value = (True, [])
        
        # Act
        result = main()
        
        # Assert
        assert result == 0
        assert mock_validate.call_count == 3
    
    @patch('sys.argv', ['validate_sbom.py', 'valid.json', 'invalid.json'])
    @patch('validators.validate_sbom.validate_sbom_file')
    def test_main_multiple_files_some_invalid(self, mock_validate):
        """Test main with mix of valid and invalid files."""
        # Arrange
        mock_validate.side_effect = [
            (True, []),
            (False, ['Error in file'])
        ]
        
        # Act
        result = main()
        
        # Assert
        assert result == 1
    
    @patch('sys.argv', ['validate_sbom.py', '--verbose', 'test.json'])
    @patch('validators.validate_sbom.validate_sbom_file')
    @patch('builtins.print')
    def test_main_verbose_mode(self, mock_print, mock_validate):
        """Test main with verbose flag."""
        # Arrange
        mock_validate.return_value = (True, [])
        
        # Act
        result = main()
        
        # Assert
        assert result == 0
        # Verify verbose output was printed
        assert mock_print.call_count >= 2  # At least summary and verbose message

