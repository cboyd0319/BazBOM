#!/usr/bin/env python3
"""Tests for CSV export functionality."""

import csv
import json
import os
import sys
from pathlib import Path

import pytest

# Import the module under test
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from csv_exporter import (
    export_sbom_to_csv,
    export_vulnerabilities_to_csv,
    export_license_report_to_csv,
)


class TestCSVExporter:
    """Test CSV export functionality."""
    
    def test_export_sbom_to_csv_happy_path(self, tmp_path):
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
        
        output_path = tmp_path / "sbom.csv"
        export_sbom_to_csv(sbom_data, str(output_path))
        
        # Verify file was created
        assert output_path.exists()
        
        # Verify CSV content
        with open(output_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
            
            assert len(rows) == 1
            assert rows[0]['Name'] == 'example-package'
            assert rows[0]['Version'] == '1.0.0'
            assert rows[0]['License'] == 'Apache-2.0'
            assert 'pkg:maven' in rows[0]['Package URL (PURL)']
    
    def test_export_sbom_missing_packages_field(self, tmp_path):
        """Test error handling for SBOM missing packages field."""
        sbom_data = {}
        
        output_path = tmp_path / "sbom.csv"
        
        with pytest.raises(ValueError) as ctx:
            export_sbom_to_csv(sbom_data, str(output_path))
        
        assert 'packages' in str(ctx.value)
    
    def test_export_vulnerabilities_to_csv_happy_path(self, tmp_path):
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
        
        output_path = tmp_path / "vulnerabilities.csv"
        export_vulnerabilities_to_csv(findings_data, str(output_path))
        
        # Verify file was created
        assert output_path.exists()
        
        # Verify CSV content
        with open(output_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
            
            assert len(rows) == 1
            assert rows[0]['CVE ID'] == 'CVE-2023-12345'
            assert rows[0]['Package Name'] == 'vulnerable-lib'
            assert rows[0]['Severity'] == 'CRITICAL'
            assert rows[0]['CVSS Score'] == '9.8'
    
    def test_export_vulnerabilities_empty_list(self, tmp_path):
        """Test exporting empty vulnerability list."""
        findings_data = {"vulnerabilities": []}
        
        output_path = tmp_path / "vulnerabilities.csv"
        export_vulnerabilities_to_csv(findings_data, str(output_path))
        
        # Verify file was created with header only
        assert output_path.exists()
        
        with open(output_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
            assert len(rows) == 0
    
    def test_export_license_report_to_csv_happy_path(self, tmp_path):
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
        
        output_path = tmp_path / "licenses.csv"
        export_license_report_to_csv(license_data, str(output_path))
        
        # Verify file was created
        assert output_path.exists()
        
        # Verify CSV content
        with open(output_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
            
            assert len(rows) == 2
            assert rows[0]['Package Name'] == 'example-lib'
            assert rows[0]['License'] == 'MIT'
            assert rows[0]['Is Copyleft'] == 'No'
            assert rows[1]['Is Copyleft'] == 'Yes'
            assert 'Apache-2.0' in rows[1]['Conflicts']
    
    def test_export_sbom_handles_missing_optional_fields(self, tmp_path):
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
        
        output_path = tmp_path / "sbom_minimal.csv"
        export_sbom_to_csv(sbom_data, str(output_path))
        
        # Verify file was created and contains expected defaults
        assert output_path.exists()
        
        with open(output_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
            
            assert len(rows) == 1
            assert rows[0]['Name'] == 'minimal-package'
            assert rows[0]['License'] == 'NOASSERTION'
            assert rows[0]['Package URL (PURL)'] == ''


if __name__ == '__main__':
    pytest.main([__file__])
