#!/usr/bin/env python3
"""Tests for purl_generator.py"""

import json
import os
import sys
import tempfile
import unittest
from unittest.mock import patch, MagicMock

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from purl_generator import maven_to_purl, parse_maven_coordinates, process_dependencies, main


class TestPurlGenerator(unittest.TestCase):
    """Test cases for PURL generation."""
    
    def test_basic_maven_to_purl(self):
        """Test basic Maven to PURL conversion."""
        purl = maven_to_purl("com.google.guava", "guava", "31.1-jre")
        # Maven PURLs use namespace with dots replaced by slashes
        self.assertEqual(purl, "pkg:maven/com/google/guava/guava@31.1-jre")
    
    def test_maven_to_purl_with_classifier(self):
        """Test Maven to PURL with classifier."""
        purl = maven_to_purl("com.example", "lib", "1.0", classifier="sources")
        self.assertTrue("classifier=sources" in purl)
    
    def test_maven_to_purl_with_packaging(self):
        """Test Maven to PURL with non-jar packaging."""
        purl = maven_to_purl("com.example", "webapp", "1.0", packaging="war")
        self.assertTrue("type=war" in purl)
    
    def test_parse_maven_coordinates_simple(self):
        """Test parsing simple Maven coordinates."""
        coords = parse_maven_coordinates("com.google.guava:guava:31.1-jre")
        self.assertEqual(coords["group_id"], "com.google.guava")
        self.assertEqual(coords["artifact_id"], "guava")
        self.assertEqual(coords["version"], "31.1-jre")
    
    def test_parse_maven_coordinates_with_packaging(self):
        """Test parsing Maven coordinates with packaging."""
        coords = parse_maven_coordinates("com.example:lib:jar:1.0")
        self.assertEqual(coords["group_id"], "com.example")
        self.assertEqual(coords["artifact_id"], "lib")
        self.assertEqual(coords["packaging"], "jar")
        self.assertEqual(coords["version"], "1.0")
    
    def test_parse_maven_coordinates_full(self):
        """Test parsing full Maven coordinates."""
        coords = parse_maven_coordinates("com.example:lib:jar:sources:1.0")
        self.assertEqual(coords["group_id"], "com.example")
        self.assertEqual(coords["artifact_id"], "lib")
        self.assertEqual(coords["packaging"], "jar")
        self.assertEqual(coords["classifier"], "sources")
        self.assertEqual(coords["version"], "1.0")
    
    def test_parse_maven_coordinates_invalid(self):
        """Test parsing invalid Maven coordinates."""
        # Too few parts
        with self.assertRaises(ValueError):
            parse_maven_coordinates("com.example:artifact")
        
        # Too many parts
        with self.assertRaises(ValueError):
            parse_maven_coordinates("a:b:c:d:e:f")
    
    def test_process_dependencies(self):
        """Test processing dependencies file."""
        # Create test data
        test_data = {
            "dependencies": [
                {
                    "coordinates": "com.google.guava:guava:31.1-jre",
                    "name": "guava",
                },
            ]
        }
        
        # Create temporary files
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(test_data, f)
            input_file = f.name
        
        output_file = input_file.replace('.json', '_output.json')
        
        try:
            # Process
            process_dependencies(input_file, output_file)
            
            # Verify output
            with open(output_file, 'r') as f:
                result = json.load(f)
            
            self.assertIn("dependencies", result)
            self.assertEqual(len(result["dependencies"]), 1)
            self.assertIn("purl", result["dependencies"][0])
            self.assertEqual(
                result["dependencies"][0]["purl"],
                "pkg:maven/com/google/guava/guava@31.1-jre"
            )
        finally:
            # Cleanup
            if os.path.exists(input_file):
                os.unlink(input_file)
            if os.path.exists(output_file):
                os.unlink(output_file)
    
    def test_process_dependencies_with_group_artifact_version(self):
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
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(test_data, f)
            input_file = f.name
        
        output_file = input_file.replace('.json', '_output.json')
        
        try:
            process_dependencies(input_file, output_file)
            
            with open(output_file, 'r') as f:
                result = json.load(f)
            
            self.assertIn("purl", result["dependencies"][0])
            self.assertTrue(result["dependencies"][0]["purl"].startswith("pkg:maven"))
        finally:
            if os.path.exists(input_file):
                os.unlink(input_file)
            if os.path.exists(output_file):
                os.unlink(output_file)
    
    def test_process_dependencies_list_format(self):
        """Test processing dependencies in list format."""
        test_data = [
            {
                "coordinates": "com.example:lib:1.0",
            },
        ]
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(test_data, f)
            input_file = f.name
        
        output_file = input_file.replace('.json', '_output.json')
        
        try:
            process_dependencies(input_file, output_file)
            
            with open(output_file, 'r') as f:
                result = json.load(f)
            
            self.assertIsInstance(result, list)
            self.assertIn("purl", result[0])
        finally:
            if os.path.exists(input_file):
                os.unlink(input_file)
            if os.path.exists(output_file):
                os.unlink(output_file)
    
    def test_main_single_coordinate(self):
        """Test main with single coordinate conversion."""
        # Need to provide --input and --output even with --coordinates due to argparse
        with patch('sys.argv', ['purl_generator.py', '--input', 'dummy.json', '--output', 'dummy2.json', '--coordinates', 'com.example:lib:1.0']):
            with patch('builtins.print') as mock_print:
                main()
                
                # Verify PURL was printed (coordinates takes precedence)
                mock_print.assert_called_once()
                args = mock_print.call_args[0]
                self.assertTrue(args[0].startswith("pkg:maven"))
    
    def test_main_file_processing(self):
        """Test main with file processing."""
        with patch('sys.argv', ['purl_generator.py', '--input', 'in.json', '--output', 'out.json']):
            with patch('purl_generator.process_dependencies') as mock_process:
                main()
                
                # Verify process_dependencies was called
                mock_process.assert_called_once_with('in.json', 'out.json')


if __name__ == "__main__":
    unittest.main()
