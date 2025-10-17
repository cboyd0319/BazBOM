#!/usr/bin/env python3
"""Unit tests for extract_maven_deps.py"""

import json
import os
import tempfile
import unittest
from pathlib import Path

# Add parent directory to path to import the module
import sys
sys.path.insert(0, str(Path(__file__).parent.parent))

from extract_maven_deps import (
    extract_from_maven_install_json,
    extract_maven_artifacts,
)


class TestExtractMavenDeps(unittest.TestCase):
    """Test cases for Maven dependency extraction."""

    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        
    def tearDown(self):
        """Clean up temporary files."""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_extract_from_maven_install_json_basic(self):
        """Test basic extraction from maven_install.json."""
        lockfile_data = {
            "artifacts": {
                "com.google.guava:guava": {
                    "shasums": {
                        "jar": "abc123"
                    },
                    "version": "31.1-jre"
                }
            },
            "dependencies": {
                "com.google.guava:guava": []
            }
        }
        
        lockfile_path = os.path.join(self.temp_dir, "maven_install.json")
        with open(lockfile_path, "w") as f:
            json.dump(lockfile_data, f)
        
        result = extract_from_maven_install_json(lockfile_path)
        
        self.assertIsNotNone(result)
        self.assertEqual(result["source"], "maven_install.json")
        self.assertTrue(result["transitive_included"])
        self.assertEqual(len(result["packages"]), 1)
        
        pkg = result["packages"][0]
        self.assertEqual(pkg["name"], "guava")
        self.assertEqual(pkg["group"], "com.google.guava")
        self.assertEqual(pkg["version"], "31.1-jre")
        self.assertEqual(pkg["sha256"], "abc123")
        self.assertEqual(pkg["purl"], "pkg:maven/com.google.guava/guava@31.1-jre")

    def test_extract_from_maven_install_json_with_transitive(self):
        """Test extraction with transitive dependencies."""
        lockfile_data = {
            "artifacts": {
                "com.google.guava:guava": {
                    "shasums": {"jar": "sha_guava"},
                    "version": "31.1-jre"
                },
                "com.google.guava:failureaccess": {
                    "shasums": {"jar": "sha_failure"},
                    "version": "1.0.1"
                }
            },
            "dependencies": {
                "com.google.guava:guava": [
                    "com.google.guava:failureaccess"
                ]
            }
        }
        
        lockfile_path = os.path.join(self.temp_dir, "maven_install.json")
        with open(lockfile_path, "w") as f:
            json.dump(lockfile_data, f)
        
        result = extract_from_maven_install_json(lockfile_path)
        
        self.assertEqual(len(result["packages"]), 2)
        
        guava = [p for p in result["packages"] if p["name"] == "guava"][0]
        self.assertEqual(len(guava["dependencies"]), 1)
        self.assertEqual(guava["dependencies"][0], "com.google.guava:failureaccess")
        self.assertTrue(guava["is_direct"])
        
        failure = [p for p in result["packages"] if p["name"] == "failureaccess"][0]
        self.assertEqual(len(failure["dependencies"]), 0)
        self.assertFalse(failure["is_direct"])

    def test_extract_from_maven_install_json_missing_file(self):
        """Test handling of missing maven_install.json."""
        result = extract_from_maven_install_json("/nonexistent/file.json")
        self.assertIsNone(result)

    def test_extract_from_maven_install_json_invalid_json(self):
        """Test handling of invalid JSON."""
        lockfile_path = os.path.join(self.temp_dir, "invalid.json")
        with open(lockfile_path, "w") as f:
            f.write("not valid json {")
        
        result = extract_from_maven_install_json(lockfile_path)
        self.assertIsNone(result)

    def test_extract_maven_artifacts_from_workspace(self):
        """Test extraction from WORKSPACE file."""
        workspace_content = '''
load("@rules_jvm_external//:defs.bzl", "maven_install")

maven_install(
    artifacts = [
        "com.google.guava:guava:31.1-jre",
        "org.slf4j:slf4j-api:1.7.32",
    ],
    repositories = [
        "https://repo1.maven.org/maven2",
    ],
)
'''
        
        result = extract_maven_artifacts(workspace_content)
        
        self.assertEqual(result["source"], "WORKSPACE")
        self.assertFalse(result["transitive_included"])
        self.assertEqual(len(result["packages"]), 2)
        
        guava = [p for p in result["packages"] if p["name"] == "guava"][0]
        self.assertEqual(guava["group"], "com.google.guava")
        self.assertEqual(guava["version"], "31.1-jre")
        self.assertEqual(guava["purl"], "pkg:maven/com.google.guava/guava@31.1-jre")
        self.assertTrue(guava["is_direct"])
        
        slf4j = [p for p in result["packages"] if p["name"] == "slf4j-api"][0]
        self.assertEqual(slf4j["group"], "org.slf4j")

    def test_extract_maven_artifacts_empty_workspace(self):
        """Test extraction from empty WORKSPACE."""
        result = extract_maven_artifacts("")
        
        self.assertEqual(result["source"], "WORKSPACE")
        self.assertEqual(len(result["packages"]), 0)

    def test_extract_maven_artifacts_no_maven_install(self):
        """Test extraction from WORKSPACE without maven_install."""
        workspace_content = '''
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "some_repo",
    urls = ["https://example.com/repo.tar.gz"],
)
'''
        
        result = extract_maven_artifacts(workspace_content)
        self.assertEqual(len(result["packages"]), 0)

    def test_purl_format(self):
        """Test that PURL format is correct."""
        lockfile_data = {
            "artifacts": {
                "org.example:my-artifact": {
                    "shasums": {"jar": "abc"},
                    "version": "1.0.0"
                }
            },
            "dependencies": {}
        }
        
        lockfile_path = os.path.join(self.temp_dir, "maven_install.json")
        with open(lockfile_path, "w") as f:
            json.dump(lockfile_data, f)
        
        result = extract_from_maven_install_json(lockfile_path)
        pkg = result["packages"][0]
        
        # PURL format: pkg:maven/{group}/{artifact}@{version}
        self.assertEqual(pkg["purl"], "pkg:maven/org.example/my-artifact@1.0.0")

    def test_download_url_format(self):
        """Test that download URL is correctly formatted."""
        lockfile_data = {
            "artifacts": {
                "org.example:my-artifact": {
                    "shasums": {"jar": "abc"},
                    "version": "1.0.0"
                }
            },
            "dependencies": {}
        }
        
        lockfile_path = os.path.join(self.temp_dir, "maven_install.json")
        with open(lockfile_path, "w") as f:
            json.dump(lockfile_data, f)
        
        result = extract_from_maven_install_json(lockfile_path)
        pkg = result["packages"][0]
        
        expected_url = "https://repo1.maven.org/maven2/org/example/my-artifact/1.0.0/my-artifact-1.0.0.jar"
        self.assertEqual(pkg["url"], expected_url)

    def test_all_required_fields_present(self):
        """Test that all required fields are present in output."""
        lockfile_data = {
            "artifacts": {
                "org.example:test": {
                    "shasums": {"jar": "sha"},
                    "version": "1.0"
                }
            },
            "dependencies": {}
        }
        
        lockfile_path = os.path.join(self.temp_dir, "maven_install.json")
        with open(lockfile_path, "w") as f:
            json.dump(lockfile_data, f)
        
        result = extract_from_maven_install_json(lockfile_path)
        pkg = result["packages"][0]
        
        required_fields = ["name", "group", "version", "purl", "type", 
                          "license", "url", "sha256", "dependencies", "is_direct"]
        for field in required_fields:
            self.assertIn(field, pkg, f"Missing required field: {field}")


if __name__ == "__main__":
    unittest.main()
