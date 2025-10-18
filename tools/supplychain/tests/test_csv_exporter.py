#!/usr/bin/env python3
"""Tests for CSV export functionality."""

import csv
import json
import os
import tempfile
import unittest
from pathlib import Path

# Import the module under test
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from csv_exporter import (
    export_sbom_to_csv,
    export_vulnerabilities_to_csv,
    export_license_report_to_csv,
)


class TestCSVExporter(unittest.TestCase):
    """Test CSV export functionality."""
    
    def setUp(self):
        """Create temporary directory for test outputs."""
        self.test_dir = tempfile.mkdtemp()
    
    def tearDown(self):
        """Clean up test directory."""
        import shutil
        shutil.rmtree(self.test_dir, ignore_errors=True)
    
    def test_export_sbom_to_csv_happy_path(self):
        """Test exporting a valid SBOM to CSV."""
        sbom_data = {
            "packages": [
                {
                    "name": "example-package",
                    "versionInfo": "1.0.0",
                    "SPDXID": "SPDXRef-Package-example",
                    "licenseConcluded": "Apache-2.0",
                    "supplier": "Organization: Example Corp",
                    "downloadLocation": "https://example.com/package",
                    "externalRefs": [
                        {
                            "referenceType": "purl",
                            "referenceLocator": "pkg:maven/com.example/example-package@1.0.0"
                        }
                    ],
                    "checksums": [
                        {
                            "algorithm": "SHA256",
                            "checksumValue": "abc123"
                        }
                    ]
                }
            ]
        }
        
        output_path = os.path.join(self.test_dir, "sbom.csv")
        export_sbom_to_csv(sbom_data, output_path)
        
        # Verify file was created
        self.assertTrue(os.path.exists(output_path))
        
        # Verify CSV content
        with open(output_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
            
            self.assertEqual(len(rows), 1)
            self.assertEqual(rows[0]['Name'], 'example-package')
            self.assertEqual(rows[0]['Version'], '1.0.0')
            self.assertEqual(rows[0]['License'], 'Apache-2.0')
            self.assertIn('pkg:maven', rows[0]['Package URL (PURL)'])
    
    def test_export_sbom_missing_packages_field(self):
        """Test error handling for SBOM missing packages field."""
        sbom_data = {}
        
        output_path = os.path.join(self.test_dir, "sbom.csv")
        
        with self.assertRaises(ValueError) as ctx:
            export_sbom_to_csv(sbom_data, output_path)
        
        self.assertIn('packages', str(ctx.exception))
    
    def test_export_vulnerabilities_to_csv_happy_path(self):
        """Test exporting vulnerability findings to CSV."""
        findings_data = {
            "vulnerabilities": [
                {
                    "id": "CVE-2023-12345",
                    "package_name": "vulnerable-lib",
                    "package_version": "1.0.0",
                    "summary": "Remote code execution vulnerability",
                    "references": ["https://nvd.nist.gov/vuln/detail/CVE-2023-12345"],
                    "published": "2023-01-01T00:00:00Z",
                    "modified": "2023-01-02T00:00:00Z",
                    "fixed_versions": ["1.0.1", "1.1.0"],
                    "database_specific": {
                        "severity": [
                            {
                                "type": "CVSS_V3",
                                "score": 9.8,
                                "severity": "CRITICAL"
                            }
                        ]
                    }
                }
            ]
        }
        
        output_path = os.path.join(self.test_dir, "vulnerabilities.csv")
        export_vulnerabilities_to_csv(findings_data, output_path)
        
        # Verify file was created
        self.assertTrue(os.path.exists(output_path))
        
        # Verify CSV content
        with open(output_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
            
            self.assertEqual(len(rows), 1)
            self.assertEqual(rows[0]['CVE ID'], 'CVE-2023-12345')
            self.assertEqual(rows[0]['Package Name'], 'vulnerable-lib')
            self.assertEqual(rows[0]['Severity'], 'CRITICAL')
            self.assertEqual(rows[0]['CVSS Score'], '9.8')
    
    def test_export_vulnerabilities_empty_list(self):
        """Test exporting empty vulnerability list."""
        findings_data = {"vulnerabilities": []}
        
        output_path = os.path.join(self.test_dir, "vulnerabilities.csv")
        export_vulnerabilities_to_csv(findings_data, output_path)
        
        # Verify file was created with header only
        self.assertTrue(os.path.exists(output_path))
        
        with open(output_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
            self.assertEqual(len(rows), 0)
    
    def test_export_license_report_to_csv_happy_path(self):
        """Test exporting license report to CSV."""
        license_data = {
            "packages": [
                {
                    "name": "example-lib",
                    "version": "2.0.0",
                    "license": "MIT",
                    "license_type": "Permissive",
                    "is_copyleft": False,
                    "is_permissive": True,
                    "conflicts": []
                },
                {
                    "name": "gpl-lib",
                    "version": "1.0.0",
                    "license": "GPL-3.0",
                    "license_type": "Copyleft",
                    "is_copyleft": True,
                    "is_permissive": False,
                    "conflicts": ["Apache-2.0"]
                }
            ]
        }
        
        output_path = os.path.join(self.test_dir, "licenses.csv")
        export_license_report_to_csv(license_data, output_path)
        
        # Verify file was created
        self.assertTrue(os.path.exists(output_path))
        
        # Verify CSV content
        with open(output_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
            
            self.assertEqual(len(rows), 2)
            self.assertEqual(rows[0]['Package Name'], 'example-lib')
            self.assertEqual(rows[0]['License'], 'MIT')
            self.assertEqual(rows[0]['Is Copyleft'], 'No')
            self.assertEqual(rows[1]['Is Copyleft'], 'Yes')
            self.assertIn('Apache-2.0', rows[1]['Conflicts'])
    
    def test_export_sbom_handles_missing_optional_fields(self):
        """Test that export handles packages with missing optional fields gracefully."""
        sbom_data = {
            "packages": [
                {
                    "name": "minimal-package",
                    "versionInfo": "1.0.0",
                    "SPDXID": "SPDXRef-Package-minimal",
                    # Missing: license, supplier, download location, external refs, checksums
                }
            ]
        }
        
        output_path = os.path.join(self.test_dir, "sbom_minimal.csv")
        export_sbom_to_csv(sbom_data, output_path)
        
        # Verify file was created and contains expected defaults
        self.assertTrue(os.path.exists(output_path))
        
        with open(output_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
            
            self.assertEqual(len(rows), 1)
            self.assertEqual(rows[0]['Name'], 'minimal-package')
            self.assertEqual(rows[0]['License'], 'NOASSERTION')
            self.assertEqual(rows[0]['Package URL (PURL)'], '')


if __name__ == '__main__':
    unittest.main()
