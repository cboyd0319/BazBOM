#!/usr/bin/env python3
"""Tests for write_sbom.py - SBOM generation functionality."""

import json
import sys
import tempfile
import unittest
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


class TestSanitizeSpdxId(unittest.TestCase):
    """Test SPDX ID sanitization."""
    
    def test_simple_name(self):
        """Test sanitization of simple package names."""
        self.assertEqual(
            sanitize_spdx_id("SPDXRef-Package-guava"),
            "SPDXRef-Package-guava"
        )
    
    def test_dots_in_name(self):
        """Test handling of dots in package names."""
        result = sanitize_spdx_id("SPDXRef-Package-com.google.guava")
        self.assertIn("SPDXRef-Package-", result)
        # Dots are allowed in SPDX IDs
        self.assertIn("com.google.guava", result)
    
    def test_special_characters(self):
        """Test handling of special characters."""
        result = sanitize_spdx_id("SPDXRef-Package-test@1.0-SNAPSHOT")
        self.assertTrue(result.startswith("SPDXRef-Package-"))
        # Should not contain @
        self.assertNotIn("@", result)
    
    def test_empty_string(self):
        """Test handling of empty string."""
        result = sanitize_spdx_id("")
        # Empty string stays empty after sanitization
        self.assertEqual(result, "")


class TestGenerateSpdxDocument(unittest.TestCase):
    """Test SPDX document generation."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.sample_packages = [
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
    
    def test_basic_structure(self):
        """Test that SPDX document has required fields."""
        doc = generate_spdx_document(self.sample_packages, "test-sbom")
        
        # Check required top-level fields
        self.assertEqual(doc["spdxVersion"], "SPDX-2.3")
        self.assertEqual(doc["dataLicense"], "CC0-1.0")
        self.assertEqual(doc["SPDXID"], "SPDXRef-DOCUMENT")
        self.assertEqual(doc["name"], "test-sbom")
        self.assertIn("documentNamespace", doc)
        self.assertIn("creationInfo", doc)
        self.assertIn("packages", doc)
        self.assertIn("relationships", doc)
    
    def test_creation_info(self):
        """Test creation info is properly formatted."""
        doc = generate_spdx_document(self.sample_packages, "test-sbom")
        
        creation_info = doc["creationInfo"]
        self.assertIn("created", creation_info)
        self.assertIn("creators", creation_info)
        self.assertIn("licenseListVersion", creation_info)
        
        # Verify timestamp format (ISO 8601)
        timestamp = creation_info["created"]
        datetime.strptime(timestamp, "%Y-%m-%dT%H:%M:%SZ")
        
        # Verify creators
        self.assertTrue(any("BazBOM" in c for c in creation_info["creators"]))
    
    def test_packages_included(self):
        """Test that all packages are included."""
        doc = generate_spdx_document(self.sample_packages, "test-sbom")
        
        packages = doc["packages"]
        # Should have root package + 2 dependencies
        self.assertGreaterEqual(len(packages), 3)
        
        # Check root package
        root_pkg = packages[0]
        self.assertEqual(root_pkg["SPDXID"], "SPDXRef-Package-root")
        self.assertEqual(root_pkg["name"], "test-sbom")
    
    def test_package_fields(self):
        """Test that packages have required fields."""
        doc = generate_spdx_document(self.sample_packages, "test-sbom")
        
        # Find guava package
        guava_pkg = None
        for pkg in doc["packages"]:
            if pkg.get("name") == "guava":
                guava_pkg = pkg
                break
        
        self.assertIsNotNone(guava_pkg)
        self.assertIn("SPDXID", guava_pkg)
        self.assertEqual(guava_pkg["name"], "guava")
        self.assertEqual(guava_pkg["versionInfo"], "31.1-jre")
        self.assertEqual(guava_pkg["licenseConcluded"], "Apache-2.0")
        self.assertEqual(guava_pkg["licenseDeclared"], "Apache-2.0")
    
    def test_external_refs(self):
        """Test that external references (PURLs) are included."""
        doc = generate_spdx_document(self.sample_packages, "test-sbom")
        
        # Find package with PURL
        for pkg in doc["packages"]:
            if pkg.get("name") == "guava":
                self.assertIn("externalRefs", pkg)
                refs = pkg["externalRefs"]
                self.assertTrue(len(refs) > 0)
                
                # Check PURL reference
                purl_ref = refs[0]
                self.assertEqual(purl_ref["referenceCategory"], "PACKAGE-MANAGER")
                self.assertEqual(purl_ref["referenceType"], "purl")
                self.assertIn("pkg:maven", purl_ref["referenceLocator"])
    
    def test_relationships(self):
        """Test that relationships are created."""
        doc = generate_spdx_document(self.sample_packages, "test-sbom")
        
        relationships = doc["relationships"]
        self.assertTrue(len(relationships) > 0)
        
        # Check DESCRIBES relationship exists
        describes = [r for r in relationships if r["relationshipType"] == "DESCRIBES"]
        self.assertEqual(len(describes), 1)
        self.assertEqual(describes[0]["spdxElementId"], "SPDXRef-DOCUMENT")
        self.assertEqual(describes[0]["relatedSpdxElement"], "SPDXRef-Package-root")
    
    def test_empty_packages(self):
        """Test handling of empty package list."""
        doc = generate_spdx_document([], "empty-sbom")
        
        # Should still have valid structure
        self.assertEqual(doc["spdxVersion"], "SPDX-2.3")
        self.assertGreaterEqual(len(doc["packages"]), 1)  # At least root package
    
    def test_missing_fields(self):
        """Test handling of packages with missing fields."""
        incomplete_packages = [
            {"name": "test-pkg"}  # Missing most fields
        ]
        
        doc = generate_spdx_document(incomplete_packages, "test-sbom")
        
        # Should still generate valid document
        self.assertIn("packages", doc)
        pkg = [p for p in doc["packages"] if p.get("name") == "test-pkg"][0]
        self.assertIn("SPDXID", pkg)
        self.assertEqual(pkg["versionInfo"], "unknown")


class TestGenerateCycloneDxDocument(unittest.TestCase):
    """Test CycloneDX document generation."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.sample_packages = [
            {
                "name": "guava",
                "group": "com.google.guava",
                "version": "31.1-jre",
                "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
                "license": "Apache-2.0",
            }
        ]
    
    def test_basic_structure(self):
        """Test that CycloneDX document has required fields."""
        doc = generate_cyclonedx_document(self.sample_packages, "test-sbom")
        
        # Check required top-level fields
        self.assertEqual(doc["bomFormat"], "CycloneDX")
        self.assertEqual(doc["specVersion"], "1.5")
        self.assertIn("version", doc)
        self.assertIn("metadata", doc)
        self.assertIn("components", doc)
    
    def test_metadata(self):
        """Test metadata section."""
        doc = generate_cyclonedx_document(self.sample_packages, "test-sbom")
        
        metadata = doc["metadata"]
        self.assertIn("timestamp", metadata)
        self.assertIn("tools", metadata)
        
        # Verify timestamp is valid ISO 8601
        timestamp = metadata["timestamp"]
        datetime.fromisoformat(timestamp.replace('Z', '+00:00'))
    
    def test_components(self):
        """Test that components are included."""
        doc = generate_cyclonedx_document(self.sample_packages, "test-sbom")
        
        components = doc["components"]
        self.assertEqual(len(components), 1)
        
        component = components[0]
        self.assertEqual(component["type"], "library")
        self.assertEqual(component["name"], "guava")
        self.assertEqual(component["version"], "31.1-jre")
    
    def test_purl_in_component(self):
        """Test that PURLs are included in components."""
        doc = generate_cyclonedx_document(self.sample_packages, "test-sbom")
        
        component = doc["components"][0]
        self.assertIn("purl", component)
        self.assertEqual(
            component["purl"],
            "pkg:maven/com.google.guava/guava@31.1-jre"
        )
    
    def test_licenses(self):
        """Test that licenses are included."""
        doc = generate_cyclonedx_document(self.sample_packages, "test-sbom")
        
        component = doc["components"][0]
        self.assertIn("licenses", component)
        licenses = component["licenses"]
        self.assertTrue(len(licenses) > 0)
        self.assertIn("license", licenses[0])


class TestMainFunction(unittest.TestCase):
    """Test main entry point and CLI handling."""
    
    def setUp(self):
        """Set up temporary files for testing."""
        self.temp_dir = tempfile.mkdtemp()
        self.temp_dir_path = Path(self.temp_dir)
        
        # Create sample input file
        self.input_file = self.temp_dir_path / "deps.json"
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
        with open(self.input_file, 'w') as f:
            json.dump(sample_data, f)
    
    def tearDown(self):
        """Clean up temporary files."""
        import shutil
        shutil.rmtree(self.temp_dir)
    
    def test_spdx_output(self):
        """Test SPDX format output."""
        output_file = self.temp_dir_path / "output.spdx.json"
        
        sys.argv = [
            "write_sbom.py",
            "--input", str(self.input_file),
            "--output", str(output_file),
            "--name", "test-sbom",
            "--format", "spdx"
        ]
        
        result = main()
        self.assertEqual(result, 0)
        
        # Verify output file exists and is valid JSON
        self.assertTrue(output_file.exists())
        with open(output_file) as f:
            data = json.load(f)
        self.assertEqual(data["spdxVersion"], "SPDX-2.3")
    
    def test_cyclonedx_output(self):
        """Test CycloneDX format output."""
        output_file = self.temp_dir_path / "output.cdx.json"
        
        sys.argv = [
            "write_sbom.py",
            "--input", str(self.input_file),
            "--output", str(output_file),
            "--name", "test-sbom",
            "--format", "cyclonedx"
        ]
        
        result = main()
        self.assertEqual(result, 0)
        
        # Verify output file exists and is valid JSON
        self.assertTrue(output_file.exists())
        with open(output_file) as f:
            data = json.load(f)
        self.assertEqual(data["bomFormat"], "CycloneDX")
    
    def test_invalid_input_file(self):
        """Test handling of missing input file."""
        sys.argv = [
            "write_sbom.py",
            "--input", "/nonexistent/file.json",
            "--output", str(self.temp_dir_path / "output.json"),
            "--name", "test",
            "--format", "spdx"
        ]
        
        result = main()
        self.assertNotEqual(result, 0)


if __name__ == '__main__':
    unittest.main()
