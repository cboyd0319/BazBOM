#!/usr/bin/env python3
"""Tests for SBOM diff engine."""

import json
import sys
import tempfile
import unittest
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from sbom_diff import Package, SBOMDiff, load_sbom


class TestPackage(unittest.TestCase):
    """Test Package class."""
    
    def test_package_creation(self):
        """Test creating a package."""
        pkg = Package(
            name="com.example:mylib",
            version="1.0.0",
            purl="pkg:maven/com.example/mylib@1.0.0",
            license="Apache-2.0"
        )
        
        self.assertEqual(pkg.name, "com.example:mylib")
        self.assertEqual(pkg.version, "1.0.0")
        self.assertEqual(pkg.purl, "pkg:maven/com.example/mylib@1.0.0")
        self.assertEqual(pkg.license, "Apache-2.0")
    
    def test_package_equality(self):
        """Test package equality comparison."""
        pkg1 = Package("lib", "1.0.0")
        pkg2 = Package("lib", "1.0.0")
        pkg3 = Package("lib", "2.0.0")
        pkg4 = Package("other", "1.0.0")
        
        self.assertEqual(pkg1, pkg2)
        self.assertNotEqual(pkg1, pkg3)
        self.assertNotEqual(pkg1, pkg4)
    
    def test_package_hash(self):
        """Test package hashing for set operations."""
        pkg1 = Package("lib", "1.0.0")
        pkg2 = Package("lib", "1.0.0")
        pkg3 = Package("lib", "2.0.0")
        
        # Same name and version should have same hash
        self.assertEqual(hash(pkg1), hash(pkg2))
        
        # Can be used in sets
        pkg_set = {pkg1, pkg2, pkg3}
        self.assertEqual(len(pkg_set), 2)  # pkg1 and pkg2 are duplicates
    
    def test_package_to_dict(self):
        """Test converting package to dictionary."""
        pkg = Package(
            name="mylib",
            version="1.0.0",
            purl="pkg:maven/mylib@1.0.0",
            license="MIT",
            vulnerabilities=["CVE-2023-1234"],
            spdxid="SPDXRef-Package-mylib-1.0.0"
        )
        
        result = pkg.to_dict()
        
        self.assertEqual(result['name'], "mylib")
        self.assertEqual(result['version'], "1.0.0")
        self.assertEqual(result['purl'], "pkg:maven/mylib@1.0.0")
        self.assertEqual(result['license'], "MIT")
        self.assertEqual(result['vulnerabilities'], ["CVE-2023-1234"])
        self.assertEqual(result['spdxid'], "SPDXRef-Package-mylib-1.0.0")


class TestSBOMDiff(unittest.TestCase):
    """Test SBOM diff functionality."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.sbom_old = {
            'spdxVersion': 'SPDX-2.3',
            'name': 'test-sbom-old',
            'creationInfo': {'created': '2025-01-01T00:00:00Z'},
            'packages': [
                {
                    'SPDXID': 'SPDXRef-Package-guava-30.0',
                    'name': 'com.google.guava:guava',
                    'versionInfo': '30.0',
                    'licenseConcluded': 'Apache-2.0',
                    'externalRefs': [
                        {
                            'referenceType': 'purl',
                            'referenceLocator': 'pkg:maven/com.google.guava/guava@30.0'
                        }
                    ]
                },
                {
                    'SPDXID': 'SPDXRef-Package-commons-text-1.9',
                    'name': 'org.apache.commons:commons-text',
                    'versionInfo': '1.9',
                    'licenseConcluded': 'Apache-2.0',
                },
                {
                    'SPDXID': 'SPDXRef-Package-oldlib-1.0',
                    'name': 'com.example:oldlib',
                    'versionInfo': '1.0.0',
                    'licenseConcluded': 'MIT',
                }
            ]
        }
        
        self.sbom_new = {
            'spdxVersion': 'SPDX-2.3',
            'name': 'test-sbom-new',
            'creationInfo': {'created': '2025-01-02T00:00:00Z'},
            'packages': [
                {
                    'SPDXID': 'SPDXRef-Package-guava-31.1',
                    'name': 'com.google.guava:guava',
                    'versionInfo': '31.1',
                    'licenseConcluded': 'Apache-2.0',
                    'externalRefs': [
                        {
                            'referenceType': 'purl',
                            'referenceLocator': 'pkg:maven/com.google.guava/guava@31.1'
                        }
                    ]
                },
                {
                    'SPDXID': 'SPDXRef-Package-commons-text-1.9',
                    'name': 'org.apache.commons:commons-text',
                    'versionInfo': '1.9',
                    'licenseConcluded': 'GPL-3.0',  # License changed!
                },
                {
                    'SPDXID': 'SPDXRef-Package-newlib-2.0',
                    'name': 'com.example:newlib',
                    'versionInfo': '2.0.0',
                    'licenseConcluded': 'BSD-3-Clause',
                }
            ]
        }
    
    def test_validate_sbom_valid(self):
        """Test SBOM validation with valid input."""
        # Should not raise
        diff = SBOMDiff(self.sbom_old, self.sbom_new)
        self.assertIsNotNone(diff)
    
    def test_validate_sbom_missing_version(self):
        """Test SBOM validation with missing spdxVersion."""
        invalid_sbom = {'packages': []}
        
        with self.assertRaises(ValueError) as ctx:
            SBOMDiff(invalid_sbom, self.sbom_new)
        
        self.assertIn('spdxVersion', str(ctx.exception))
    
    def test_validate_sbom_missing_packages(self):
        """Test SBOM validation with missing packages field."""
        invalid_sbom = {'spdxVersion': 'SPDX-2.3'}
        
        with self.assertRaises(ValueError) as ctx:
            SBOMDiff(invalid_sbom, self.sbom_new)
        
        self.assertIn('packages', str(ctx.exception))
    
    def test_validate_sbom_packages_not_list(self):
        """Test SBOM validation with packages not being a list."""
        invalid_sbom = {'spdxVersion': 'SPDX-2.3', 'packages': 'not a list'}
        
        with self.assertRaises(ValueError) as ctx:
            SBOMDiff(invalid_sbom, self.sbom_new)
        
        self.assertIn('must be a list', str(ctx.exception))
    
    def test_extract_packages(self):
        """Test extracting packages from SBOM."""
        diff = SBOMDiff(self.sbom_old, self.sbom_new)
        
        self.assertEqual(len(diff.packages_old), 3)
        self.assertEqual(len(diff.packages_new), 3)
        
        # Check package names
        old_names = {pkg.name for pkg in diff.packages_old}
        self.assertIn('com.google.guava:guava', old_names)
        self.assertIn('org.apache.commons:commons-text', old_names)
        self.assertIn('com.example:oldlib', old_names)
    
    def test_get_added_packages(self):
        """Test detecting added packages."""
        diff = SBOMDiff(self.sbom_old, self.sbom_new)
        
        added = diff.get_added_packages()
        
        self.assertEqual(len(added), 1)
        self.assertEqual(added[0].name, 'com.example:newlib')
        self.assertEqual(added[0].version, '2.0.0')
    
    def test_get_removed_packages(self):
        """Test detecting removed packages."""
        diff = SBOMDiff(self.sbom_old, self.sbom_new)
        
        removed = diff.get_removed_packages()
        
        self.assertEqual(len(removed), 1)
        self.assertEqual(removed[0].name, 'com.example:oldlib')
        self.assertEqual(removed[0].version, '1.0.0')
    
    def test_get_upgraded_packages(self):
        """Test detecting upgraded packages."""
        diff = SBOMDiff(self.sbom_old, self.sbom_new)
        
        upgraded = diff.get_upgraded_packages()
        
        self.assertEqual(len(upgraded), 1)
        old, new = upgraded[0]
        self.assertEqual(old.name, 'com.google.guava:guava')
        self.assertEqual(old.version, '30.0')
        self.assertEqual(new.version, '31.1')
    
    def test_get_license_changes(self):
        """Test detecting license changes."""
        diff = SBOMDiff(self.sbom_old, self.sbom_new)
        
        license_changes = diff.get_license_changes()
        
        self.assertEqual(len(license_changes), 1)
        old, new = license_changes[0]
        self.assertEqual(old.name, 'org.apache.commons:commons-text')
        self.assertEqual(old.license, 'Apache-2.0')
        self.assertEqual(new.license, 'GPL-3.0')
    
    def test_version_parsing(self):
        """Test version string parsing."""
        diff = SBOMDiff(self.sbom_old, self.sbom_new)
        
        # Test semantic version parsing
        v1 = diff._parse_version('1.2.3')
        self.assertEqual(v1, (1, 2, 3))
        
        v2 = diff._parse_version('2.0.0')
        self.assertEqual(v2, (2, 0, 0))
        
        # Test with pre-release tag
        v3 = diff._parse_version('1.2.3-beta')
        self.assertEqual(v3, (1, 2, 3))
        
        # Test with build metadata
        v4 = diff._parse_version('1.2.3+build.123')
        self.assertEqual(v4, (1, 2, 3))
    
    def test_is_upgrade(self):
        """Test upgrade detection."""
        diff = SBOMDiff(self.sbom_old, self.sbom_new)
        
        # Clear upgrades
        self.assertTrue(diff._is_upgrade('1.0.0', '2.0.0'))
        self.assertTrue(diff._is_upgrade('1.0.0', '1.1.0'))
        self.assertTrue(diff._is_upgrade('1.0.0', '1.0.1'))
        
        # Downgrades
        self.assertFalse(diff._is_upgrade('2.0.0', '1.0.0'))
        self.assertFalse(diff._is_upgrade('1.1.0', '1.0.0'))
        
        # Same version
        self.assertFalse(diff._is_upgrade('1.0.0', '1.0.0'))
    
    def test_to_dict(self):
        """Test converting diff to dictionary."""
        diff = SBOMDiff(self.sbom_old, self.sbom_new)
        
        result = diff.to_dict()
        
        # Check structure
        self.assertIn('metadata', result)
        self.assertIn('summary', result)
        self.assertIn('changes', result)
        
        # Check summary
        summary = result['summary']
        self.assertEqual(summary['total_old'], 3)
        self.assertEqual(summary['total_new'], 3)
        self.assertEqual(summary['added'], 1)
        self.assertEqual(summary['removed'], 1)
        self.assertEqual(summary['upgraded'], 1)
        self.assertEqual(summary['license_changed'], 1)
    
    def test_to_human_readable(self):
        """Test generating human-readable report."""
        diff = SBOMDiff(self.sbom_old, self.sbom_new)
        
        report = diff.to_human_readable()
        
        # Check report contains key sections
        self.assertIn('SBOM DIFF REPORT', report)
        self.assertIn('Summary:', report)
        self.assertIn('NEW DEPENDENCIES', report)
        self.assertIn('REMOVED DEPENDENCIES', report)
        self.assertIn('UPGRADED DEPENDENCIES', report)
        self.assertIn('LICENSE CHANGES', report)
        
        # Check specific changes are mentioned
        self.assertIn('com.example:newlib', report)
        self.assertIn('com.example:oldlib', report)
        self.assertIn('com.google.guava:guava', report)
        self.assertIn('30.0', report)
        self.assertIn('31.1', report)
    
    def test_empty_diff(self):
        """Test diff with no changes."""
        sbom_same = self.sbom_old.copy()
        diff = SBOMDiff(self.sbom_old, sbom_same)
        
        self.assertEqual(len(diff.get_added_packages()), 0)
        self.assertEqual(len(diff.get_removed_packages()), 0)
        self.assertEqual(len(diff.get_upgraded_packages()), 0)
        self.assertEqual(len(diff.get_license_changes()), 0)
        
        # All packages should be unchanged
        self.assertEqual(len(diff.get_unchanged_packages()), 3)


class TestLoadSBOM(unittest.TestCase):
    """Test SBOM loading functionality."""
    
    def test_load_valid_sbom(self):
        """Test loading valid SBOM from file."""
        sbom_data = {
            'spdxVersion': 'SPDX-2.3',
            'name': 'test-sbom',
            'packages': []
        }
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(sbom_data, f)
            temp_path = f.name
        
        try:
            sbom = load_sbom(temp_path)
            self.assertEqual(sbom['name'], 'test-sbom')
        finally:
            Path(temp_path).unlink()
    
    def test_load_missing_file(self):
        """Test loading non-existent file."""
        with self.assertRaises(FileNotFoundError):
            load_sbom('/nonexistent/path/to/sbom.json')
    
    def test_load_invalid_json(self):
        """Test loading file with invalid JSON."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write('{ invalid json }')
            temp_path = f.name
        
        try:
            with self.assertRaises(json.JSONDecodeError):
                load_sbom(temp_path)
        finally:
            Path(temp_path).unlink()
    
    def test_load_non_dict_json(self):
        """Test loading JSON that's not a dictionary."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(['not', 'a', 'dict'], f)
            temp_path = f.name
        
        try:
            with self.assertRaises(ValueError) as ctx:
                load_sbom(temp_path)
            self.assertIn('must be a JSON object', str(ctx.exception))
        finally:
            Path(temp_path).unlink()


class TestEdgeCases(unittest.TestCase):
    """Test edge cases and boundary conditions."""
    
    def test_diff_empty_sboms(self):
        """Test diff with empty SBOMs."""
        sbom_empty = {
            'spdxVersion': 'SPDX-2.3',
            'name': 'empty',
            'packages': []
        }
        
        diff = SBOMDiff(sbom_empty, sbom_empty)
        
        self.assertEqual(len(diff.get_added_packages()), 0)
        self.assertEqual(len(diff.get_removed_packages()), 0)
        self.assertEqual(len(diff.get_upgraded_packages()), 0)
    
    def test_diff_large_sbom(self):
        """Test diff with large number of packages."""
        # Create SBOM with 1000 packages
        packages_old = [
            {
                'name': f'package-{i}',
                'versionInfo': '1.0.0',
                'licenseConcluded': 'MIT'
            }
            for i in range(1000)
        ]
        
        # Deep copy and modify
        packages_new = [pkg.copy() for pkg in packages_old]
        # Modify some packages
        packages_new[0] = {
            'name': 'package-0',
            'versionInfo': '2.0.0',  # Upgrade
            'licenseConcluded': 'MIT'
        }
        packages_new.pop(1)  # Remove package-1
        packages_new.append({
            'name': 'new-package',
            'versionInfo': '1.0.0',
            'licenseConcluded': 'Apache-2.0'
        })  # Add
        
        sbom_old = {
            'spdxVersion': 'SPDX-2.3',
            'name': 'large-old',
            'packages': packages_old
        }
        
        sbom_new = {
            'spdxVersion': 'SPDX-2.3',
            'name': 'large-new',
            'packages': packages_new
        }
        
        diff = SBOMDiff(sbom_old, sbom_new)
        
        self.assertEqual(len(diff.get_added_packages()), 1)
        self.assertEqual(len(diff.get_removed_packages()), 1)
        self.assertEqual(len(diff.get_upgraded_packages()), 1)
    
    def test_package_missing_fields(self):
        """Test handling packages with missing optional fields."""
        sbom = {
            'spdxVersion': 'SPDX-2.3',
            'name': 'minimal',
            'packages': [
                {
                    'name': 'minimal-package',
                    'versionInfo': '1.0.0'
                    # No license, no PURL, no SPDXID
                }
            ]
        }
        
        diff = SBOMDiff(sbom, sbom)
        
        self.assertEqual(len(diff.packages_old), 1)
        pkg = diff.packages_old[0]
        self.assertEqual(pkg.license, 'NOASSERTION')
        self.assertEqual(pkg.purl, '')
    
    def test_unicode_in_package_names(self):
        """Test handling Unicode characters in package names."""
        sbom = {
            'spdxVersion': 'SPDX-2.3',
            'name': 'unicode-test',
            'packages': [
                {
                    'name': 'café-lib',
                    'versionInfo': '1.0.0',
                    'licenseConcluded': 'MIT'
                },
                {
                    'name': '日本語-package',
                    'versionInfo': '2.0.0',
                    'licenseConcluded': 'Apache-2.0'
                }
            ]
        }
        
        diff = SBOMDiff(sbom, sbom)
        
        self.assertEqual(len(diff.packages_old), 2)
        names = {pkg.name for pkg in diff.packages_old}
        self.assertIn('café-lib', names)
        self.assertIn('日本語-package', names)


if __name__ == '__main__':
    unittest.main()
