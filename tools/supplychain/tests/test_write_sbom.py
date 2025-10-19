#!/usr/bin/env python3
"""Tests for write_sbom.py - SBOM generation functionality.

Tests cover:
- SPDX ID sanitization
- SPDX 2.3 document generation
- CycloneDX 1.5 document generation
- Package field handling and relationships
- External references (PURLs)
- Edge cases (empty packages, missing fields)
"""

import json
import sys
import pytest
from pathlib import Path
from datetime import datetime

# Add parent directory to path to import write_sbom
sys.path.insert(0, str(Path(__file__).parent.parent))

from write_sbom import (
    generate_spdx_document,
    generate_cyclonedx_document,
    sanitize_spdx_id,
    main,
)


# SPDX ID Sanitization Tests

@pytest.mark.parametrize("input_id,expected", [
    ("SPDXRef-Package-guava", "SPDXRef-Package-guava"),
    ("SPDXRef-Package-com.google.guava", "SPDXRef-Package-com.google.guava"),
], ids=["simple", "with_dots"])
def test_sanitize_spdx_id_valid(input_id, expected):
    """Test sanitization of valid SPDX IDs."""
    assert sanitize_spdx_id(input_id) == expected or input_id in sanitize_spdx_id(input_id)


def test_sanitize_spdx_id_special_characters():
    """Test handling of special characters."""
    result = sanitize_spdx_id("SPDXRef-Package-test@1.0-SNAPSHOT")
    assert result.startswith("SPDXRef-Package-")
    # Should not contain @
    assert "@" not in result


def test_sanitize_spdx_id_empty():
    """Test handling of empty string."""
    result = sanitize_spdx_id("")
    # Empty string stays empty after sanitization
    assert result == ""


# SPDX Document Generation Tests

@pytest.fixture
def sample_packages():
    """Provide sample package data for SPDX tests."""
    return [
        {
            "name": "guava",
            "group": "com.google.guava",
            "version": "31.1-jre",
            "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
            "license": "Apache-2.0",
            "url": "https://github.com/google/guava",
            "sha256": "abc123",
        },
        {
            "name": "gson",
            "group": "com.google.code.gson",
            "version": "2.10.1",
            "purl": "pkg:maven/com.google.code.gson/gson@2.10.1",
            "license": "Apache-2.0",
        }
    ]


def test_spdx_basic_structure(sample_packages):
    """Test that SPDX document has required fields."""
    doc = generate_spdx_document(sample_packages, "test-sbom")
    
    # Check required top-level fields
    assert doc["spdxVersion"] == "SPDX-2.3"
    assert doc["dataLicense"] == "CC0-1.0"
    assert doc["SPDXID"] == "SPDXRef-DOCUMENT"
    assert doc["name"] == "test-sbom"
    assert "documentNamespace" in doc
    assert "creationInfo" in doc
    assert "packages" in doc
    assert "relationships" in doc


def test_spdx_creation_info(sample_packages):
    """Test creation info is properly formatted."""
    doc = generate_spdx_document(sample_packages, "test-sbom")
    
    creation_info = doc["creationInfo"]
    assert "created" in creation_info
    assert "creators" in creation_info
    assert "licenseListVersion" in creation_info
    
    # Verify timestamp format (ISO 8601)
    timestamp = creation_info["created"]
    datetime.strptime(timestamp, "%Y-%m-%dT%H:%M:%SZ")
    
    # Verify creators
    assert any("BazBOM" in c for c in creation_info["creators"])


def test_spdx_packages_included(sample_packages):
    """Test that all packages are included."""
    doc = generate_spdx_document(sample_packages, "test-sbom")
    
    packages = doc["packages"]
    # Should have root package + 2 dependencies
    assert len(packages) >= 3
    
    # Check root package
    root_pkg = packages[0]
    assert root_pkg["SPDXID"] == "SPDXRef-Package-root"
    assert root_pkg["name"] == "test-sbom"


def test_spdx_package_fields(sample_packages):
    """Test that packages have required fields."""
    doc = generate_spdx_document(sample_packages, "test-sbom")
    
    # Find guava package
    guava_pkg = next((p for p in doc["packages"] if p.get("name") == "guava"), None)
    
    assert guava_pkg is not None
    assert "SPDXID" in guava_pkg
    assert guava_pkg["name"] == "guava"
    assert guava_pkg["versionInfo"] == "31.1-jre"
    assert guava_pkg["licenseConcluded"] == "Apache-2.0"
    assert guava_pkg["licenseDeclared"] == "Apache-2.0"


def test_spdx_external_refs(sample_packages):
    """Test that external references (PURLs) are included."""
    doc = generate_spdx_document(sample_packages, "test-sbom")
    
    # Find package with PURL
    guava_pkg = next((p for p in doc["packages"] if p.get("name") == "guava"), None)
    assert guava_pkg is not None
    assert "externalRefs" in guava_pkg
    
    refs = guava_pkg["externalRefs"]
    assert len(refs) > 0
    
    # Check PURL reference
    purl_ref = refs[0]
    assert purl_ref["referenceCategory"] == "PACKAGE-MANAGER"
    assert purl_ref["referenceType"] == "purl"
    assert "pkg:maven" in purl_ref["referenceLocator"]


def test_spdx_relationships(sample_packages):
    """Test that relationships are created."""
    doc = generate_spdx_document(sample_packages, "test-sbom")
    
    relationships = doc["relationships"]
    assert len(relationships) > 0
    
    # Check DESCRIBES relationship exists
    describes = [r for r in relationships if r["relationshipType"] == "DESCRIBES"]
    assert len(describes) == 1
    assert describes[0]["spdxElementId"] == "SPDXRef-DOCUMENT"
    assert describes[0]["relatedSpdxElement"] == "SPDXRef-Package-root"


def test_spdx_empty_packages():
    """Test handling of empty package list."""
    doc = generate_spdx_document([], "empty-sbom")
    
    # Should still have valid structure
    assert doc["spdxVersion"] == "SPDX-2.3"
    assert len(doc["packages"]) >= 1  # At least root package


def test_spdx_missing_fields():
    """Test handling of packages with missing fields."""
    incomplete_packages = [
        {"name": "test-pkg"}  # Missing most fields
    ]
    
    doc = generate_spdx_document(incomplete_packages, "test-sbom")
    
    # Should still generate valid document
    assert "packages" in doc
    pkg = next((p for p in doc["packages"] if p.get("name") == "test-pkg"), None)
    assert pkg is not None
    assert "SPDXID" in pkg
    assert pkg["versionInfo"] == "unknown"


# CycloneDX Document Generation Tests

@pytest.fixture
def cyclonedx_sample_packages():
    """Provide sample package data for CycloneDX tests."""
    return [
        {
            "name": "guava",
            "group": "com.google.guava",
            "version": "31.1-jre",
            "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
            "license": "Apache-2.0",
        }
    ]


def test_cyclonedx_basic_structure(cyclonedx_sample_packages):
    """Test that CycloneDX document has required fields."""
    doc = generate_cyclonedx_document(cyclonedx_sample_packages, "test-sbom")
    
    # Check required top-level fields
    assert doc["bomFormat"] == "CycloneDX"
    assert doc["specVersion"] == "1.5"
    assert "version" in doc
    assert "metadata" in doc
    assert "components" in doc


def test_cyclonedx_metadata(cyclonedx_sample_packages):
    """Test metadata section."""
    doc = generate_cyclonedx_document(cyclonedx_sample_packages, "test-sbom")
    
    metadata = doc["metadata"]
    assert "timestamp" in metadata
    assert "tools" in metadata
    
    # Verify timestamp is valid ISO 8601
    timestamp = metadata["timestamp"]
    datetime.fromisoformat(timestamp.replace('Z', '+00:00'))


def test_cyclonedx_components(cyclonedx_sample_packages):
    """Test that components are included."""
    doc = generate_cyclonedx_document(cyclonedx_sample_packages, "test-sbom")
    
    components = doc["components"]
    assert len(components) == 1
    
    component = components[0]
    assert component["type"] == "library"
    assert component["name"] == "guava"
    assert component["version"] == "31.1-jre"


def test_cyclonedx_purl_in_component(cyclonedx_sample_packages):
    """Test that PURLs are included in components."""
    doc = generate_cyclonedx_document(cyclonedx_sample_packages, "test-sbom")
    
    component = doc["components"][0]
    assert "purl" in component
    assert component["purl"] == "pkg:maven/com.google.guava/guava@31.1-jre"


def test_cyclonedx_licenses(cyclonedx_sample_packages):
    """Test that licenses are included."""
    doc = generate_cyclonedx_document(cyclonedx_sample_packages, "test-sbom")
    
    component = doc["components"][0]
    assert "licenses" in component
    licenses = component["licenses"]
    assert len(licenses) > 0
    assert "license" in licenses[0]


# Main Function Tests

@pytest.fixture
def sample_input_file(tmp_path, temp_json_file):
    """Create sample input file for main function tests."""
    sample_data = {
        "packages": [
            {
                "name": "guava",
                "group": "com.google.guava",
                "version": "31.1-jre",
                "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
            }
        ]
    }
    return temp_json_file(sample_data, "deps.json")


def test_main_spdx_output(sample_input_file, tmp_path):
    """Test SPDX format output."""
    output_file = tmp_path / "output.spdx.json"
    
    sys.argv = [
        "write_sbom.py",
        "--input", str(sample_input_file),
        "--output", str(output_file),
        "--name", "test-sbom",
        "--format", "spdx"
    ]
    
    result = main()
    assert result == 0
    
    # Verify output file exists and is valid JSON
    assert output_file.exists()
    data = json.loads(output_file.read_text())
    assert data["spdxVersion"] == "SPDX-2.3"


def test_main_cyclonedx_output(sample_input_file, tmp_path):
    """Test CycloneDX format output."""
    output_file = tmp_path / "output.cdx.json"
    
    sys.argv = [
        "write_sbom.py",
        "--input", str(sample_input_file),
        "--output", str(output_file),
        "--name", "test-sbom",
        "--format", "cyclonedx"
    ]
    
    result = main()
    assert result == 0
    
    # Verify output file exists and is valid JSON
    assert output_file.exists()
    data = json.loads(output_file.read_text())
    assert data["bomFormat"] == "CycloneDX"


def test_main_invalid_input_file(tmp_path):
    """Test handling of missing input file."""
    sys.argv = [
        "write_sbom.py",
        "--input", "/nonexistent/file.json",
        "--output", str(tmp_path / "output.json"),
        "--name", "test",
        "--format", "spdx"
    ]
    
    result = main()
    assert result != 0


if __name__ == '__main__':
    pytest.main([__file__, "-v"])
