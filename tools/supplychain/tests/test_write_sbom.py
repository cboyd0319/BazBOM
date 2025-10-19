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


def test_main_invalid_json_input(tmp_path):
    """Test handling of malformed JSON input file."""
    # Create file with invalid JSON
    input_file = tmp_path / "invalid.json"
    input_file.write_text("{ not valid json }")
    
    output_file = tmp_path / "output.json"
    
    sys.argv = [
        "write_sbom.py",
        "--input", str(input_file),
        "--output", str(output_file),
        "--name", "test",
        "--format", "spdx"
    ]
    
    result = main()
    assert result == 1


def test_main_output_write_error(sample_input_file, tmp_path, mocker):
    """Test handling of output file write errors."""
    # Mock open to raise IOError when writing output
    mock_open = mocker.patch("builtins.open", mocker.mock_open())
    mock_open.side_effect = [
        # First call: reading input (succeed)
        mocker.mock_open(read_data='{"packages": []}')(),
        # Second call: writing output (fail)
        IOError("Permission denied")
    ]
    
    sys.argv = [
        "write_sbom.py",
        "--input", str(sample_input_file),
        "--output", str(tmp_path / "output.json"),
        "--name", "test",
        "--format", "spdx"
    ]
    
    result = main()
    assert result == 1


def test_spdx_direct_dependencies(sample_packages):
    """Test that direct dependency relationships are created."""
    # Mark first package as direct dependency
    sample_packages[0]["is_direct"] = True
    
    doc = generate_spdx_document(sample_packages, "test-sbom")
    
    # Find DEPENDS_ON relationships from root
    root_deps = [r for r in doc["relationships"] 
                 if r["spdxElementId"] == "SPDXRef-Package-root" 
                 and r["relationshipType"] == "DEPENDS_ON"]
    
    assert len(root_deps) >= 1
    # Check that the direct dependency is in the relationships
    assert any("guava" in r["relatedSpdxElement"] for r in root_deps)


def test_spdx_transitive_dependencies():
    """Test that transitive dependency relationships are created."""
    packages = [
        {
            "name": "app",
            "group": "com.example",
            "version": "1.0.0",
            "dependencies": ["com.google.guava:guava"]
        },
        {
            "name": "guava",
            "group": "com.google.guava",
            "version": "31.1-jre",
            "dependencies": []
        }
    ]
    
    doc = generate_spdx_document(packages, "test-sbom")
    
    # Find transitive DEPENDS_ON relationships
    transitive_deps = [r for r in doc["relationships"]
                      if "app" in r["spdxElementId"]
                      and r["relationshipType"] == "DEPENDS_ON"]
    
    assert len(transitive_deps) >= 1


def test_spdx_checksum_handling(sample_packages):
    """Test that SHA256 checksums are included when available."""
    doc = generate_spdx_document(sample_packages, "test-sbom")
    
    # Find package with checksum
    guava_pkg = next((p for p in doc["packages"] if p.get("name") == "guava"), None)
    assert guava_pkg is not None
    assert "checksums" in guava_pkg
    
    checksum = guava_pkg["checksums"][0]
    assert checksum["algorithm"] == "SHA256"
    assert checksum["checksumValue"] == "abc123"


def test_cyclonedx_empty_packages():
    """Test handling of empty package list in CycloneDX format."""
    doc = generate_cyclonedx_document([], "empty-sbom")
    
    # Should still have valid structure
    assert doc["bomFormat"] == "CycloneDX"
    assert "components" in doc
    assert "dependencies" in doc


def test_cyclonedx_missing_license():
    """Test handling of packages without license information."""
    packages = [
        {
            "name": "test-pkg",
            "group": "com.test",
            "version": "1.0.0",
            "license": "NOASSERTION"  # No license
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    component = doc["components"][0]
    # Should not have licenses field when license is NOASSERTION
    assert "licenses" not in component


def test_cyclonedx_with_sha256():
    """Test that SHA256 hashes are included in CycloneDX format."""
    packages = [
        {
            "name": "test-pkg",
            "group": "com.test",
            "version": "1.0.0",
            "sha256": "abc123def456"
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    component = doc["components"][0]
    assert "hashes" in component
    
    hash_entry = component["hashes"][0]
    assert hash_entry["alg"] == "SHA-256"
    assert hash_entry["content"] == "abc123def456"


def test_cyclonedx_with_url():
    """Test that external references (URLs) are included in CycloneDX format."""
    packages = [
        {
            "name": "test-pkg",
            "group": "com.test",
            "version": "1.0.0",
            "url": "https://example.com/package"
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    component = doc["components"][0]
    assert "externalReferences" in component
    
    ref = component["externalReferences"][0]
    assert ref["type"] == "distribution"
    assert ref["url"] == "https://example.com/package"


def test_cyclonedx_without_url():
    """Test handling of packages without URL in CycloneDX format."""
    packages = [
        {
            "name": "test-pkg",
            "group": "com.test",
            "version": "1.0.0",
            "url": "NOASSERTION"  # No URL
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    component = doc["components"][0]
    # Should not have externalReferences when URL is NOASSERTION
    assert "externalReferences" not in component


def test_cyclonedx_direct_dependencies():
    """Test that direct dependencies are included in CycloneDX root dependencies."""
    packages = [
        {
            "name": "guava",
            "group": "com.google.guava",
            "version": "31.1-jre",
            "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
            "is_direct": True
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    # Check root dependency has the direct dependency
    root_dep = doc["dependencies"][0]
    assert root_dep["ref"] == "pkg:generic/application@1.0.0"
    assert "pkg:maven/com.google.guava/guava@31.1-jre" in root_dep["dependsOn"]


def test_cyclonedx_transitive_dependencies():
    """Test that component dependencies are tracked in CycloneDX format."""
    packages = [
        {
            "name": "app",
            "group": "com.example",
            "version": "1.0.0",
            "purl": "pkg:maven/com.example/app@1.0.0",
            "dependencies": ["com.google.guava:guava:31.1-jre"]
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    # Find component dependency entry (not root)
    comp_deps = [d for d in doc["dependencies"] if d["ref"] != "pkg:generic/application@1.0.0"]
    
    assert len(comp_deps) >= 1
    # Check that dependency PURLs are constructed
    assert any("guava" in dep for d in comp_deps for dep in d.get("dependsOn", []))


def test_cyclonedx_dependency_with_version():
    """Test dependency PURL construction with version."""
    packages = [
        {
            "name": "app",
            "group": "com.example",
            "version": "1.0.0",
            "purl": "pkg:maven/com.example/app@1.0.0",
            "dependencies": ["com.google.guava:guava:31.1-jre"]
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    # Find the app's dependency entry
    app_dep = next((d for d in doc["dependencies"] 
                   if "pkg:maven/com.example/app@1.0.0" in d["ref"]), None)
    
    assert app_dep is not None
    assert len(app_dep["dependsOn"]) > 0
    # Check version is included in PURL
    assert any("@31.1-jre" in purl for purl in app_dep["dependsOn"])


def test_cyclonedx_group_field():
    """Test that group/publisher field is included when available."""
    packages = [
        {
            "name": "test-pkg",
            "group": "com.example",
            "version": "1.0.0",
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    component = doc["components"][0]
    assert "group" in component
    assert component["group"] == "com.example"


def test_cyclonedx_without_group_field():
    """Test that components without group field work correctly."""
    packages = [
        {
            "name": "test-pkg",
            # No group field
            "version": "1.0.0",
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    component = doc["components"][0]
    # Should not have group field when not provided
    assert "group" not in component or component.get("group") == "unknown"


def test_cyclonedx_purl_fallback():
    """Test PURL generation when not provided in package data."""
    packages = [
        {
            "name": "test-pkg",
            "group": "com.example",
            "version": "1.0.0",
            # No purl field
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    component = doc["components"][0]
    assert "purl" in component
    # Should construct PURL from group/name/version
    assert component["purl"] == "pkg:maven/com.example/test-pkg@1.0.0"


def test_cyclonedx_dependencies_without_version():
    """Test dependency PURL construction without version (edge case)."""
    packages = [
        {
            "name": "app",
            "group": "com.example",
            "version": "1.0.0",
            "purl": "pkg:maven/com.example/app@1.0.0",
            "dependencies": ["com.google.guava:guava"]  # Only 2 parts, no version
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    # Find the app's dependency entry
    app_dep = next((d for d in doc["dependencies"] 
                   if "pkg:maven/com.example/app@1.0.0" in d["ref"]), None)
    
    assert app_dep is not None
    assert len(app_dep["dependsOn"]) > 0
    # Check PURL is constructed without version
    assert any("guava" in purl for purl in app_dep["dependsOn"])


def test_spdx_dependencies_not_in_map():
    """Test handling of dependencies not found in package map."""
    packages = [
        {
            "name": "app",
            "group": "com.example",
            "version": "1.0.0",
            "dependencies": ["com.unknown:missing-dep"]  # This dep is not in packages
        }
    ]
    
    doc = generate_spdx_document(packages, "test-sbom")
    
    # Should not crash, just skip the missing dependency
    assert "relationships" in doc
    # Verify document is still valid
    assert doc["spdxVersion"] == "SPDX-2.3"


def test_cyclonedx_dependencies_empty_list():
    """Test that components with empty dependencies list don't create dependency entries."""
    packages = [
        {
            "name": "app",
            "group": "com.example",
            "version": "1.0.0",
            "purl": "pkg:maven/com.example/app@1.0.0",
            "dependencies": []  # Empty dependencies
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    # Component with empty dependencies should not have a dependency entry
    # (beyond the root dependency)
    non_root_deps = [d for d in doc["dependencies"] 
                     if d["ref"] != "pkg:generic/application@1.0.0"]
    
    # Should not create an entry for component with empty dependencies
    assert len(non_root_deps) == 0


def test_cyclonedx_dependency_coordinate_with_one_part():
    """Test dependency coordinate with only 1 part (edge case)."""
    packages = [
        {
            "name": "app",
            "group": "com.example",
            "version": "1.0.0",
            "purl": "pkg:maven/com.example/app@1.0.0",
            "dependencies": ["single"]  # Only 1 part
        }
    ]
    
    doc = generate_cyclonedx_document(packages, "test-sbom")
    
    # Should handle gracefully, skipping malformed coordinates
    # or not adding dependencies that can't be parsed
    assert "dependencies" in doc


if __name__ == '__main__':
    pytest.main([__file__, "-v"])
