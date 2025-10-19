#!/usr/bin/env python3
"""Tests for purl_generator.py - Package URL generation from Maven coordinates.

Tests cover:
- Maven coordinates to PURL conversion (happy path)
- PURL generation with classifiers and packaging types
- Maven coordinate parsing (simple, with packaging, full format)
- Error handling for invalid coordinates
- File processing and batch conversion
"""

import json
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from purl_generator import maven_to_purl, parse_maven_coordinates, process_dependencies, main


class TestMavenToPurl:
    """Test Maven to PURL conversion."""
    
    def test_basic_conversion(self):
        """Test basic Maven to PURL conversion."""
        purl = maven_to_purl("com.google.guava", "guava", "31.1-jre")
        # Maven PURLs use namespace with dots replaced by slashes
        assert purl == "pkg:maven/com/google/guava/guava@31.1-jre"
    
    def test_with_classifier(self):
        """Test Maven to PURL with classifier."""
        purl = maven_to_purl("com.example", "lib", "1.0", classifier="sources")
        assert "classifier=sources" in purl
    
    def test_with_packaging(self):
        """Test Maven to PURL with non-jar packaging."""
        purl = maven_to_purl("com.example", "webapp", "1.0", packaging="war")
        assert "type=war" in purl


@pytest.mark.parametrize("coords,expected_group,expected_artifact,expected_version", [
    ("com.google.guava:guava:31.1-jre", "com.google.guava", "guava", "31.1-jre"),
    ("com.example:lib:1.0", "com.example", "lib", "1.0"),
    ("org.springframework:spring-core:5.3.0", "org.springframework", "spring-core", "5.3.0"),
], ids=["guava", "simple", "spring"])
def test_parse_maven_coordinates_simple(coords, expected_group, expected_artifact, expected_version):
    """Test parsing simple Maven coordinates."""
    result = parse_maven_coordinates(coords)
    assert result["group_id"] == expected_group
    assert result["artifact_id"] == expected_artifact
    assert result["version"] == expected_version


def test_parse_maven_coordinates_with_packaging():
    """Test parsing Maven coordinates with packaging."""
    coords = parse_maven_coordinates("com.example:lib:jar:1.0")
    assert coords["group_id"] == "com.example"
    assert coords["artifact_id"] == "lib"
    assert coords["packaging"] == "jar"
    assert coords["version"] == "1.0"


def test_parse_maven_coordinates_full():
    """Test parsing full Maven coordinates."""
    coords = parse_maven_coordinates("com.example:lib:jar:sources:1.0")
    assert coords["group_id"] == "com.example"
    assert coords["artifact_id"] == "lib"
    assert coords["packaging"] == "jar"
    assert coords["classifier"] == "sources"
    assert coords["version"] == "1.0"


@pytest.mark.parametrize("invalid_coords", [
    "com.example:artifact",  # Too few parts
    "a:b:c:d:e:f",          # Too many parts
], ids=["too_few", "too_many"])
def test_parse_maven_coordinates_invalid(invalid_coords):
    """Test parsing invalid Maven coordinates."""
    with pytest.raises(ValueError):
        parse_maven_coordinates(invalid_coords)


def test_process_dependencies(tmp_path, temp_json_file):
    """Test processing dependencies file."""
    # Arrange
    test_data = {
        "dependencies": [
            {
                "coordinates": "com.google.guava:guava:31.1-jre",
                "name": "guava",
            },
        ]
    }
    
    input_file = temp_json_file(test_data, "input.json")
    output_file = tmp_path / "output.json"
    
    # Act
    process_dependencies(str(input_file), str(output_file))
    
    # Assert
    result = json.loads(output_file.read_text())
    assert "dependencies" in result
    assert len(result["dependencies"]) == 1
    assert "purl" in result["dependencies"][0]
    assert result["dependencies"][0]["purl"] == "pkg:maven/com/google/guava/guava@31.1-jre"


def test_process_dependencies_with_group_artifact_version(tmp_path, temp_json_file):
    """Test processing dependencies with separate group/artifact/version fields."""
    test_data = {
        "dependencies": [
            {
                "group": "com.example",
                "artifact": "lib",
                "version": "1.0",
            },
        ]
    }
    
    input_file = temp_json_file(test_data, "input.json")
    output_file = tmp_path / "output.json"
    
    process_dependencies(str(input_file), str(output_file))
    
    result = json.loads(output_file.read_text())
    assert "purl" in result["dependencies"][0]
    assert result["dependencies"][0]["purl"].startswith("pkg:maven")


def test_process_dependencies_list_format(tmp_path, temp_json_file):
    """Test processing dependencies in list format."""
    test_data = [
        {
            "coordinates": "com.example:lib:1.0",
        },
    ]
    
    input_file = temp_json_file(test_data, "input.json")
    output_file = tmp_path / "output.json"
    
    process_dependencies(str(input_file), str(output_file))
    
    result = json.loads(output_file.read_text())
    assert isinstance(result, list)
    assert "purl" in result[0]


def test_main_single_coordinate():
    """Test main with single coordinate conversion."""
    # Arrange - mock sys.argv
    with patch('sys.argv', ['purl_generator.py', '--input', 'dummy.json', 
                            '--output', 'dummy2.json', '--coordinates', 
                            'com.example:lib:1.0']):
        # Act
        with patch('builtins.print') as mock_print:
            main()
            
            # Assert - PURL was printed (coordinates takes precedence)
            mock_print.assert_called_once()
            args = mock_print.call_args[0]
            assert args[0].startswith("pkg:maven")


def test_main_file_processing():
    """Test main with file processing."""
    with patch('sys.argv', ['purl_generator.py', '--input', 'in.json', '--output', 'out.json']):
        with patch('purl_generator.process_dependencies') as mock_process:
            main()
            
            # Verify process_dependencies was called
            mock_process.assert_called_once_with('in.json', 'out.json')


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
