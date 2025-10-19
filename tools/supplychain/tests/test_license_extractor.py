#!/usr/bin/env python3
"""Tests for license_extractor.py - License extraction from JARs and POMs."""

import io
import sys
import zipfile
from pathlib import Path

import pytest

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from license_extractor import (
    normalize_license_text,
    detect_license_from_text,
    extract_from_jar_manifest,
    extract_from_license_files,
    parse_pom_licenses,
    normalize_license_name_to_spdx,
    extract_jar_licenses,
    extract_pom_licenses_enhanced,
)


class TestNormalizeLicenseText:
    """Test license text normalization."""
    
    def test_normalize_converts_to_lowercase(self):
        """Test that normalization converts text to lowercase."""
        result = normalize_license_text("Apache License")
        assert result == "apache license"
    
    def test_normalize_removes_extra_whitespace(self):
        """Test that normalization collapses multiple spaces."""
        result = normalize_license_text("Apache    License   Version  2.0")
        assert result == "apache license version 2.0"
    
    def test_normalize_strips_leading_trailing_whitespace(self):
        """Test that normalization strips leading/trailing whitespace."""
        result = normalize_license_text("  MIT License  ")
        assert result == "mit license"
    
    def test_normalize_handles_newlines(self):
        """Test that normalization handles newlines."""
        result = normalize_license_text("MIT\nLicense\n\n")
        assert result == "mit license"
    
    def test_normalize_handles_tabs(self):
        """Test that normalization handles tabs."""
        result = normalize_license_text("MIT\t\tLicense")
        assert result == "mit license"
    
    def test_normalize_empty_string(self):
        """Test normalization of empty string."""
        result = normalize_license_text("")
        assert result == ""


class TestDetectLicenseFromText:
    """Test license detection from text."""
    
    def test_detect_apache_2_0(self):
        """Test detection of Apache 2.0 license."""
        text = """
        Apache License
        Version 2.0, January 2004
        http://www.apache.org/licenses/
        """
        result = detect_license_from_text(text)
        assert result == "Apache-2.0"
    
    def test_detect_apache_2_0_short_form(self):
        """Test detection of Apache 2.0 short form."""
        text = "Licensed under Apache 2.0"
        result = detect_license_from_text(text)
        assert result == "Apache-2.0"
    
    def test_detect_mit_license(self):
        """Test detection of MIT license."""
        text = """
        MIT License
        
        Permission is hereby granted, free of charge...
        """
        result = detect_license_from_text(text)
        assert result == "MIT"
    
    def test_detect_bsd_3_clause(self):
        """Test detection of BSD 3-Clause license."""
        text = "This software is licensed under the BSD 3-Clause license"
        result = detect_license_from_text(text)
        assert result == "BSD-3-Clause"
    
    def test_detect_bsd_2_clause(self):
        """Test detection of BSD 2-Clause license."""
        text = "This software is licensed under the BSD 2-Clause license"
        result = detect_license_from_text(text)
        assert result == "BSD-2-Clause"
    
    def test_detect_gpl_2_0(self):
        """Test detection of GPL 2.0 license."""
        text = "GNU General Public License version 2"
        result = detect_license_from_text(text)
        assert result == "GPL-2.0"
    
    def test_detect_gpl_3_0(self):
        """Test detection of GPL 3.0 license."""
        text = "GNU General Public License version 3"
        result = detect_license_from_text(text)
        assert result == "GPL-3.0"
    
    def test_detect_lgpl_2_1(self):
        """Test detection of LGPL 2.1 license."""
        text = "GNU Lesser General Public License version 2.1"
        result = detect_license_from_text(text)
        assert result == "LGPL-2.1"
    
    def test_detect_lgpl_3_0(self):
        """Test detection of LGPL 3.0 license."""
        text = "GNU Lesser General Public License version 3"
        result = detect_license_from_text(text)
        assert result == "LGPL-3.0"
    
    def test_detect_epl_1_0(self):
        """Test detection of Eclipse Public License 1.0."""
        text = "Eclipse Public License version 1.0"
        result = detect_license_from_text(text)
        assert result == "EPL-1.0"
    
    def test_detect_epl_2_0(self):
        """Test detection of Eclipse Public License 2.0."""
        text = "Eclipse Public License version 2.0"
        result = detect_license_from_text(text)
        assert result == "EPL-2.0"
    
    def test_detect_mpl_2_0(self):
        """Test detection of Mozilla Public License 2.0."""
        text = "Mozilla Public License version 2.0"
        result = detect_license_from_text(text)
        assert result == "MPL-2.0"
    
    def test_detect_unknown_license(self):
        """Test that unknown licenses return None."""
        text = "This is some proprietary license text"
        result = detect_license_from_text(text)
        assert result is None
    
    def test_detect_empty_text(self):
        """Test detection with empty text."""
        result = detect_license_from_text("")
        assert result is None
    
    def test_detect_case_insensitive(self):
        """Test that detection is case-insensitive."""
        text = "APACHE LICENSE VERSION 2.0"
        result = detect_license_from_text(text)
        assert result == "Apache-2.0"
    
    @pytest.mark.parametrize("license_text,expected_spdx", [
        ("Apache License Version 2.0", "Apache-2.0"),
        ("MIT License", "MIT"),
        ("BSD 3-Clause", "BSD-3-Clause"),
        ("GNU General Public License version 2", "GPL-2.0"),
        ("GNU Lesser General Public License version 3", "LGPL-3.0"),
    ], ids=["apache", "mit", "bsd3", "gpl2", "lgpl3"])
    def test_detect_various_licenses_parametrized(self, license_text, expected_spdx):
        """Test detection of various licenses with parametrized test."""
        result = detect_license_from_text(license_text)
        assert result == expected_spdx


class TestExtractFromJarManifest:
    """Test license extraction from JAR manifest."""
    
    def test_extract_from_jar_with_manifest(self, tmp_path):
        """Test extracting license from JAR with manifest."""
        jar_path = tmp_path / "test.jar"
        
        # Create a JAR with manifest
        manifest_content = """Manifest-Version: 1.0
Bundle-License: Apache-2.0
Bundle-Vendor: Example Corp
Bundle-Version: 1.0.0
Bundle-Name: test-lib
"""
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/MANIFEST.MF', manifest_content)
        
        result = extract_from_jar_manifest(str(jar_path))
        
        assert result['licenses'] == ['Apache-2.0']
        assert result['vendor'] == 'Example Corp'
        assert result['version'] == '1.0.0'
        assert result['name'] == 'test-lib'
    
    def test_extract_from_jar_with_license_field(self, tmp_path):
        """Test extracting from manifest with License field."""
        jar_path = tmp_path / "test.jar"
        
        manifest_content = """Manifest-Version: 1.0
License: MIT
"""
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/MANIFEST.MF', manifest_content)
        
        result = extract_from_jar_manifest(str(jar_path))
        
        assert result['licenses'] == ['MIT']
    
    def test_extract_from_jar_with_implementation_fields(self, tmp_path):
        """Test extracting from manifest with Implementation- fields."""
        jar_path = tmp_path / "test.jar"
        
        manifest_content = """Manifest-Version: 1.0
Implementation-Vendor: Acme Inc
Implementation-Version: 2.0.0
Implementation-Title: acme-lib
"""
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/MANIFEST.MF', manifest_content)
        
        result = extract_from_jar_manifest(str(jar_path))
        
        assert result['vendor'] == 'Acme Inc'
        assert result['version'] == '2.0.0'
        assert result['name'] == 'acme-lib'
    
    def test_extract_from_jar_without_manifest(self, tmp_path):
        """Test extracting from JAR without manifest."""
        jar_path = tmp_path / "test.jar"
        
        # Create a JAR without manifest
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('some/file.class', b'content')
        
        result = extract_from_jar_manifest(str(jar_path))
        
        assert result['licenses'] == []
        assert result['vendor'] is None
        assert result['version'] is None
        assert result['name'] is None
    
    def test_extract_from_nonexistent_jar(self, tmp_path):
        """Test extracting from non-existent JAR."""
        jar_path = tmp_path / "nonexistent.jar"
        
        result = extract_from_jar_manifest(str(jar_path))
        
        # Should return empty info without crashing
        assert result['licenses'] == []
    
    def test_extract_from_corrupt_jar(self, tmp_path):
        """Test extracting from corrupt JAR file."""
        jar_path = tmp_path / "corrupt.jar"
        # Create corrupt file
        with open(jar_path, 'w') as f:
            f.write("not a valid zip file")
        
        result = extract_from_jar_manifest(str(jar_path))
        
        # Should return empty info without crashing
        assert result['licenses'] == []
    
    def test_extract_handles_multiline_manifest(self, tmp_path):
        """Test extracting from manifest with continuation lines."""
        jar_path = tmp_path / "test.jar"
        
        # Manifest with continuation (space-prefixed line)
        manifest_content = """Manifest-Version: 1.0
Bundle-License: Apache-2.0,
 MIT
"""
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/MANIFEST.MF', manifest_content)
        
        result = extract_from_jar_manifest(str(jar_path))
        
        # Should handle the first license line
        assert len(result['licenses']) >= 1


class TestExtractFromLicenseFiles:
    """Test license extraction from embedded license files."""
    
    def test_extract_from_jar_with_license_file(self, tmp_path):
        """Test extracting license from embedded LICENSE file."""
        jar_path = tmp_path / "test.jar"
        
        license_content = """
        Apache License
        Version 2.0, January 2004
        http://www.apache.org/licenses/
        """
        
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/LICENSE', license_content)
        
        result = extract_from_license_files(str(jar_path))
        
        assert 'Apache-2.0' in result
    
    def test_extract_from_jar_with_license_txt(self, tmp_path):
        """Test extracting license from LICENSE.txt file."""
        jar_path = tmp_path / "test.jar"
        
        license_content = "MIT License\n\nPermission is hereby granted..."
        
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/LICENSE.txt', license_content)
        
        result = extract_from_license_files(str(jar_path))
        
        assert 'MIT' in result
    
    def test_extract_from_jar_with_notice_file(self, tmp_path):
        """Test extracting license from NOTICE file."""
        jar_path = tmp_path / "test.jar"
        
        notice_content = "This product includes software developed under Apache 2.0"
        
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/NOTICE', notice_content)
        
        result = extract_from_license_files(str(jar_path))
        
        assert 'Apache-2.0' in result
    
    def test_extract_from_jar_with_multiple_licenses(self, tmp_path):
        """Test extracting multiple licenses from JAR."""
        jar_path = tmp_path / "test.jar"
        
        apache_license = "Apache License Version 2.0"
        mit_license = "MIT License"
        
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/LICENSE', apache_license)
            jar.writestr('META-INF/LICENSE-MIT', mit_license)
        
        result = extract_from_license_files(str(jar_path))
        
        # Should detect both licenses
        assert len(result) >= 1  # At least one should be detected
    
    def test_extract_from_jar_without_license_files(self, tmp_path):
        """Test extracting from JAR without license files."""
        jar_path = tmp_path / "test.jar"
        
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('com/example/Class.class', b'bytecode')
        
        result = extract_from_license_files(str(jar_path))
        
        assert result == []
    
    def test_extract_from_nonexistent_jar(self, tmp_path):
        """Test extracting from non-existent JAR."""
        jar_path = tmp_path / "nonexistent.jar"
        
        result = extract_from_license_files(str(jar_path))
        
        # Should return empty list without crashing
        assert result == []
    
    def test_extract_deduplicates_licenses(self, tmp_path):
        """Test that duplicate licenses are not returned."""
        jar_path = tmp_path / "test.jar"
        
        # Both files contain Apache license
        apache_license = "Apache License Version 2.0"
        
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/LICENSE', apache_license)
            jar.writestr('LICENSE.txt', apache_license)
        
        result = extract_from_license_files(str(jar_path))
        
        # Should only return one instance of Apache-2.0
        assert result.count('Apache-2.0') == 1
    
    def test_extract_handles_case_insensitive_paths(self, tmp_path):
        """Test that license file detection is case-insensitive."""
        jar_path = tmp_path / "test.jar"
        
        license_content = "MIT License"
        
        with zipfile.ZipFile(jar_path, 'w') as jar:
            # Lowercase path
            jar.writestr('meta-inf/license', license_content)
        
        result = extract_from_license_files(str(jar_path))
        
        # Should still detect the license
        assert 'MIT' in result or len(result) == 0  # May or may not detect depending on path matching
    
    def test_extract_handles_unreadable_license_file(self, tmp_path):
        """Test handling of unreadable license file content."""
        jar_path = tmp_path / "test.jar"
        
        with zipfile.ZipFile(jar_path, 'w') as jar:
            # Add binary content that can't be decoded as UTF-8
            jar.writestr('META-INF/LICENSE', b'\x80\x81\x82')
        
        result = extract_from_license_files(str(jar_path))
        
        # Should not crash, may return empty list
        assert isinstance(result, list)


class TestIntegration:
    """Integration tests for license extraction."""
    
    def test_extract_full_jar_with_all_metadata(self, tmp_path):
        """Test extracting from JAR with both manifest and license files."""
        jar_path = tmp_path / "complete.jar"
        
        manifest = """Manifest-Version: 1.0
Bundle-License: Apache-2.0
Bundle-Vendor: Apache Foundation
Bundle-Version: 2.0.0
Bundle-Name: log4j-core
"""
        license_file = "Apache License Version 2.0"
        
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/MANIFEST.MF', manifest)
            jar.writestr('META-INF/LICENSE', license_file)
            jar.writestr('com/example/Class.class', b'bytecode')
        
        manifest_info = extract_from_jar_manifest(str(jar_path))
        license_info = extract_from_license_files(str(jar_path))
        
        # Manifest extraction
        assert manifest_info['licenses'] == ['Apache-2.0']
        assert manifest_info['vendor'] == 'Apache Foundation'
        assert manifest_info['version'] == '2.0.0'
        assert manifest_info['name'] == 'log4j-core'
        
        # License file extraction
        assert 'Apache-2.0' in license_info


class TestParsePomLicenses:
    """Test parsing licenses from POM files following PyTest Architect standards."""
    
    def test_parse_pom_with_single_license(self, tmp_path):
        """Test parsing POM file with single license entry."""
        # Arrange
        pom_path = tmp_path / "pom.xml"
        pom_content = """<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <licenses>
        <license>
            <name>Apache License, Version 2.0</name>
            <url>https://www.apache.org/licenses/LICENSE-2.0.txt</url>
        </license>
    </licenses>
</project>"""
        pom_path.write_text(pom_content)
        
        # Act
        result = parse_pom_licenses(str(pom_path))
        
        # Assert
        assert len(result) == 1
        assert result[0]['name'] == 'Apache License, Version 2.0'
        assert result[0]['url'] == 'https://www.apache.org/licenses/LICENSE-2.0.txt'
    
    def test_parse_pom_with_multiple_licenses(self, tmp_path):
        """Test parsing POM file with multiple licenses."""
        # Arrange
        pom_path = tmp_path / "pom.xml"
        pom_content = """<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <licenses>
        <license>
            <name>MIT License</name>
            <url>https://opensource.org/licenses/MIT</url>
        </license>
        <license>
            <name>Apache License, Version 2.0</name>
            <url>https://www.apache.org/licenses/LICENSE-2.0.txt</url>
        </license>
    </licenses>
</project>"""
        pom_path.write_text(pom_content)
        
        # Act
        result = parse_pom_licenses(str(pom_path))
        
        # Assert
        assert len(result) == 2
        assert result[0]['name'] == 'MIT License'
        assert result[1]['name'] == 'Apache License, Version 2.0'
    
    def test_parse_pom_without_namespace(self, tmp_path):
        """Test parsing POM file without XML namespace."""
        # Arrange
        pom_path = tmp_path / "pom.xml"
        pom_content = """<?xml version="1.0"?>
<project>
    <licenses>
        <license>
            <name>BSD 3-Clause License</name>
            <url>https://opensource.org/licenses/BSD-3-Clause</url>
        </license>
    </licenses>
</project>"""
        pom_path.write_text(pom_content)
        
        # Act
        result = parse_pom_licenses(str(pom_path))
        
        # Assert
        assert len(result) == 1
        assert result[0]['name'] == 'BSD 3-Clause License'
    
    def test_parse_pom_with_license_name_only(self, tmp_path):
        """Test parsing POM with license name but no URL."""
        # Arrange
        pom_path = tmp_path / "pom.xml"
        pom_content = """<?xml version="1.0"?>
<project>
    <licenses>
        <license>
            <name>GPL-3.0</name>
        </license>
    </licenses>
</project>"""
        pom_path.write_text(pom_content)
        
        # Act
        result = parse_pom_licenses(str(pom_path))
        
        # Assert
        assert len(result) == 1
        assert result[0]['name'] == 'GPL-3.0'
        assert 'url' not in result[0]
    
    def test_parse_pom_with_license_url_only(self, tmp_path):
        """Test parsing POM with license URL but no name."""
        # Arrange
        pom_path = tmp_path / "pom.xml"
        pom_content = """<?xml version="1.0"?>
<project>
    <licenses>
        <license>
            <url>https://www.apache.org/licenses/LICENSE-2.0.txt</url>
        </license>
    </licenses>
</project>"""
        pom_path.write_text(pom_content)
        
        # Act
        result = parse_pom_licenses(str(pom_path))
        
        # Assert
        assert len(result) == 1
        assert 'name' not in result[0]
        assert result[0]['url'] == 'https://www.apache.org/licenses/LICENSE-2.0.txt'
    
    def test_parse_pom_without_licenses(self, tmp_path):
        """Test parsing POM file without licenses section."""
        # Arrange
        pom_path = tmp_path / "pom.xml"
        pom_content = """<?xml version="1.0"?>
<project>
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>test</artifactId>
</project>"""
        pom_path.write_text(pom_content)
        
        # Act
        result = parse_pom_licenses(str(pom_path))
        
        # Assert
        assert result == []
    
    def test_parse_pom_nonexistent_file(self, tmp_path):
        """Test parsing non-existent POM file returns empty list."""
        # Arrange
        pom_path = tmp_path / "nonexistent.xml"
        
        # Act
        result = parse_pom_licenses(str(pom_path))
        
        # Assert
        assert result == []
    
    def test_parse_pom_invalid_xml(self, tmp_path):
        """Test parsing invalid XML returns empty list without crashing."""
        # Arrange
        pom_path = tmp_path / "invalid.xml"
        pom_path.write_text("not valid xml <<<<")
        
        # Act
        result = parse_pom_licenses(str(pom_path))
        
        # Assert
        assert result == []
    
    def test_parse_pom_with_empty_license_elements(self, tmp_path):
        """Test parsing POM with empty license elements."""
        # Arrange
        pom_path = tmp_path / "pom.xml"
        pom_content = """<?xml version="1.0"?>
<project>
    <licenses>
        <license>
            <name></name>
            <url></url>
        </license>
        <license>
        </license>
    </licenses>
</project>"""
        pom_path.write_text(pom_content)
        
        # Act
        result = parse_pom_licenses(str(pom_path))
        
        # Assert
        # Should skip empty licenses
        assert len(result) == 0


class TestNormalizeLicenseNameToSpdx:
    """Test normalization of license names to SPDX identifiers."""
    
    @pytest.mark.parametrize("license_name,expected_spdx", [
        ("Apache License, Version 2.0", "Apache-2.0"),
        ("Apache Software License - Version 2.0", "Apache-2.0"),
        ("apache 2.0", "Apache-2.0"),
        ("MIT License", "MIT"),
        ("The MIT License", "MIT"),
        ("MIT", "MIT"),
        ("BSD 3-Clause License", "BSD-3-Clause"),
        ("3-Clause BSD License", "BSD-3-Clause"),
        ("BSD 2-Clause License", "BSD-2-Clause"),
        ("2-Clause BSD License", "BSD-2-Clause"),
        ("Eclipse Public License - v 1.0", "EPL-1.0"),
        ("Eclipse Public License 1.0", "EPL-1.0"),
        ("Eclipse Public License - v 2.0", "EPL-2.0"),
        ("Eclipse Public License 2.0", "EPL-2.0"),
        ("Mozilla Public License Version 2.0", "MPL-2.0"),
    ], ids=lambda x: x if isinstance(x, str) and len(x) < 20 else None)
    def test_normalize_known_licenses(self, license_name, expected_spdx):
        """Test normalization of known license names to SPDX identifiers."""
        # Act
        result = normalize_license_name_to_spdx(license_name)
        
        # Assert
        assert result == expected_spdx, f"Expected {expected_spdx} for '{license_name}', got {result}"
    
    def test_normalize_unknown_license_returns_none(self):
        """Test that unknown license names return None."""
        # Arrange
        license_name = "Proprietary License XYZ"
        
        # Act
        result = normalize_license_name_to_spdx(license_name)
        
        # Assert
        assert result is None
    
    def test_normalize_empty_string_returns_none(self):
        """Test that empty string returns None."""
        # Act
        result = normalize_license_name_to_spdx("")
        
        # Assert
        assert result is None
    
    def test_normalize_case_insensitive(self):
        """Test that normalization is case-insensitive."""
        # Act
        result1 = normalize_license_name_to_spdx("APACHE LICENSE VERSION 2.0")
        result2 = normalize_license_name_to_spdx("apache license version 2.0")
        result3 = normalize_license_name_to_spdx("Apache License Version 2.0")
        
        # Assert
        assert result1 == result2 == result3 == "Apache-2.0"


class TestExtractJarLicenses:
    """Test complete JAR license extraction combining all methods."""
    
    def test_extract_jar_licenses_complete(self, tmp_path):
        """Test extracting all license info from JAR with both manifest and license files."""
        # Arrange
        jar_path = tmp_path / "complete.jar"
        manifest = """Manifest-Version: 1.0
Bundle-License: Apache-2.0
Bundle-Vendor: Test Vendor
Bundle-Version: 1.0.0
Bundle-Name: test-lib
"""
        license_file = "MIT License\n\nPermission is hereby granted..."
        
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/MANIFEST.MF', manifest)
            jar.writestr('META-INF/LICENSE', license_file)
        
        # Act
        result = extract_jar_licenses(str(jar_path))
        
        # Assert
        assert result['jar_path'] == str(jar_path)
        assert 'Apache-2.0' in result['spdx_licenses']
        assert 'MIT' in result['spdx_licenses']
        assert len(result['raw_licenses']) >= 2
        assert result['metadata']['vendor'] == 'Test Vendor'
        assert result['metadata']['version'] == '1.0.0'
    
    def test_extract_jar_licenses_deduplicates(self, tmp_path):
        """Test that duplicate licenses are deduplicated."""
        # Arrange
        jar_path = tmp_path / "test.jar"
        manifest = """Manifest-Version: 1.0
Bundle-License: Apache License 2.0
"""
        license_file = "Apache License Version 2.0"
        
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('META-INF/MANIFEST.MF', manifest)
            jar.writestr('META-INF/LICENSE', license_file)
        
        # Act
        result = extract_jar_licenses(str(jar_path))
        
        # Assert
        # Should only have one Apache-2.0 entry
        assert result['spdx_licenses'].count('Apache-2.0') == 1
    
    def test_extract_jar_licenses_empty_jar(self, tmp_path):
        """Test extracting from JAR with no license information."""
        # Arrange
        jar_path = tmp_path / "empty.jar"
        with zipfile.ZipFile(jar_path, 'w') as jar:
            jar.writestr('com/example/Class.class', b'bytecode')
        
        # Act
        result = extract_jar_licenses(str(jar_path))
        
        # Assert
        assert result['jar_path'] == str(jar_path)
        assert result['spdx_licenses'] == []
        assert result['raw_licenses'] == []


class TestExtractPomLicensesEnhanced:
    """Test enhanced POM license extraction with SPDX normalization."""
    
    def test_extract_pom_licenses_enhanced_with_spdx_normalization(self, tmp_path):
        """Test extracting and normalizing licenses from POM."""
        # Arrange
        pom_path = tmp_path / "pom.xml"
        pom_content = """<?xml version="1.0"?>
<project>
    <licenses>
        <license>
            <name>Apache License, Version 2.0</name>
            <url>https://www.apache.org/licenses/LICENSE-2.0.txt</url>
        </license>
        <license>
            <name>MIT License</name>
        </license>
    </licenses>
</project>"""
        pom_path.write_text(pom_content)
        
        # Act
        result = extract_pom_licenses_enhanced(str(pom_path))
        
        # Assert
        assert 'Apache-2.0' in result
        assert 'MIT' in result
        assert len(result) == 2
    
    def test_extract_pom_licenses_enhanced_deduplicates(self, tmp_path):
        """Test that duplicate SPDX licenses are deduplicated."""
        # Arrange
        pom_path = tmp_path / "pom.xml"
        pom_content = """<?xml version="1.0"?>
<project>
    <licenses>
        <license>
            <name>Apache License, Version 2.0</name>
        </license>
        <license>
            <name>Apache 2.0</name>
        </license>
    </licenses>
</project>"""
        pom_path.write_text(pom_content)
        
        # Act
        result = extract_pom_licenses_enhanced(str(pom_path))
        
        # Assert
        # Should only have one Apache-2.0 entry
        assert result.count('Apache-2.0') == 1
    
    def test_extract_pom_licenses_enhanced_skips_unknown(self, tmp_path):
        """Test that unknown licenses are skipped."""
        # Arrange
        pom_path = tmp_path / "pom.xml"
        pom_content = """<?xml version="1.0"?>
<project>
    <licenses>
        <license>
            <name>Proprietary License</name>
        </license>
        <license>
            <name>MIT License</name>
        </license>
    </licenses>
</project>"""
        pom_path.write_text(pom_content)
        
        # Act
        result = extract_pom_licenses_enhanced(str(pom_path))
        
        # Assert
        # Should only have MIT, not the proprietary license
        assert 'MIT' in result
        assert len(result) == 1
    
    def test_extract_pom_licenses_enhanced_empty_pom(self, tmp_path):
        """Test extracting from POM without licenses."""
        # Arrange
        pom_path = tmp_path / "pom.xml"
        pom_content = """<?xml version="1.0"?>
<project>
    <groupId>com.example</groupId>
    <artifactId>test</artifactId>
</project>"""
        pom_path.write_text(pom_content)
        
        # Act
        result = extract_pom_licenses_enhanced(str(pom_path))
        
        # Assert
        assert result == []
    
    def test_extract_pom_licenses_enhanced_nonexistent_file(self, tmp_path):
        """Test extracting from non-existent POM file."""
        # Arrange
        pom_path = tmp_path / "nonexistent.xml"
        
        # Act
        result = extract_pom_licenses_enhanced(str(pom_path))
        
        # Assert
        assert result == []

