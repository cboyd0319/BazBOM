#!/usr/bin/env python3
"""Shared pytest fixtures and configuration for BazBOM test suite."""

import json
import os
import random
import tempfile
from pathlib import Path
from typing import Any, Dict

import pytest


@pytest.fixture(autouse=True)
def _seed_rng():
    """Seed random number generators for deterministic tests."""
    random.seed(1337)
    # Seed numpy if available
    try:
        import numpy as np
        np.random.seed(1337)
    except ImportError:
        pass


@pytest.fixture
def tmp_dir(tmp_path):
    """Provide a temporary directory for test files."""
    return tmp_path


@pytest.fixture
def sample_sbom_data() -> Dict[str, Any]:
    """Provide sample SPDX SBOM data for tests."""
    return {
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": "test-sbom",
        "documentNamespace": "https://example.com/test-sbom",
        "creationInfo": {
            "created": "2025-01-01T00:00:00Z",
            "creators": ["Tool: BazBOM-1.0.0"],
        },
        "packages": [
            {
                "SPDXID": "SPDXRef-Package-guava",
                "name": "guava",
                "versionInfo": "31.1-jre",
                "supplier": "Organization: Google",
                "downloadLocation": "https://repo1.maven.org/maven2/com/google/guava/guava/31.1-jre/guava-31.1-jre.jar",
                "filesAnalyzed": False,
                "licenseConcluded": "Apache-2.0",
                "licenseDeclared": "Apache-2.0",
                "externalRefs": [
                    {
                        "referenceCategory": "PACKAGE-MANAGER",
                        "referenceType": "purl",
                        "referenceLocator": "pkg:maven/com.google.guava/guava@31.1-jre",
                    }
                ],
            },
            {
                "SPDXID": "SPDXRef-Package-log4j-core",
                "name": "log4j-core",
                "versionInfo": "2.17.0",
                "supplier": "Organization: Apache",
                "downloadLocation": "https://repo1.maven.org/maven2/org/apache/logging/log4j/log4j-core/2.17.0/log4j-core-2.17.0.jar",
                "filesAnalyzed": False,
                "licenseConcluded": "Apache-2.0",
                "licenseDeclared": "Apache-2.0",
                "externalRefs": [
                    {
                        "referenceCategory": "PACKAGE-MANAGER",
                        "referenceType": "purl",
                        "referenceLocator": "pkg:maven/org.apache.logging.log4j/log4j-core@2.17.0",
                    },
                    {
                        "referenceCategory": "SECURITY",
                        "referenceType": "cpe23Type",
                        "referenceLocator": "cpe:2.3:a:apache:log4j:2.17.0:*:*:*:*:*:*:*",
                    },
                ],
            },
        ],
        "relationships": [
            {
                "spdxElementId": "SPDXRef-DOCUMENT",
                "relatedSpdxElement": "SPDXRef-Package-guava",
                "relationshipType": "DESCRIBES",
            },
            {
                "spdxElementId": "SPDXRef-DOCUMENT",
                "relatedSpdxElement": "SPDXRef-Package-log4j-core",
                "relationshipType": "DESCRIBES",
            },
        ],
    }


@pytest.fixture
def sample_vulnerability_data() -> Dict[str, Any]:
    """Provide sample vulnerability data for tests."""
    return {
        "vulnerabilities": [
            {
                "id": "CVE-2021-44228",
                "package": {
                    "name": "log4j-core",
                    "ecosystem": "Maven",
                    "purl": "pkg:maven/org.apache.logging.log4j/log4j-core@2.14.1",
                },
                "summary": "Apache Log4j2 JNDI features do not protect against attacker controlled LDAP and other JNDI related endpoints.",
                "details": "Apache Log4j2 2.0-beta9 through 2.15.0 (excluding security releases 2.12.2, 2.12.3, and 2.3.1) JNDI features used in configuration, log messages, and parameters do not protect against attacker controlled LDAP and other JNDI related endpoints.",
                "database_specific": {
                    "severity": [
                        {
                            "type": "CVSS_V3",
                            "score": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:C/C:H/I:H/A:H",
                            "severity": "CRITICAL",
                        }
                    ],
                },
                "references": [
                    {
                        "type": "ADVISORY",
                        "url": "https://nvd.nist.gov/vuln/detail/CVE-2021-44228",
                    }
                ],
                "affected": [
                    {
                        "package": {
                            "ecosystem": "Maven",
                            "name": "org.apache.logging.log4j:log4j-core",
                        },
                        "ranges": [
                            {
                                "type": "ECOSYSTEM",
                                "events": [
                                    {"introduced": "2.0-beta9"},
                                    {"fixed": "2.15.0"},
                                ],
                            }
                        ],
                    }
                ],
            },
            {
                "id": "CVE-2023-12345",
                "package": {
                    "name": "guava",
                    "ecosystem": "Maven",
                    "purl": "pkg:maven/com.google.guava/guava@30.0",
                },
                "summary": "Guava hypothetical vulnerability for testing",
                "details": "This is a test vulnerability for the guava package.",
                "database_specific": {
                    "severity": [
                        {
                            "type": "CVSS_V3",
                            "score": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:L/I:L/A:N",
                            "severity": "MEDIUM",
                        }
                    ],
                },
            },
        ]
    }


@pytest.fixture
def sample_maven_coordinates() -> Dict[str, Any]:
    """Provide sample Maven coordinates data."""
    return {
        "dependencies": [
            {
                "coordinates": "com.google.guava:guava:31.1-jre",
                "name": "guava",
                "version": "31.1-jre",
                "group": "com.google.guava",
            },
            {
                "coordinates": "org.apache.logging.log4j:log4j-core:2.17.0",
                "name": "log4j-core",
                "version": "2.17.0",
                "group": "org.apache.logging.log4j",
            },
        ]
    }


@pytest.fixture
def env_vars(monkeypatch):
    """Fixture to set environment variables for tests."""
    def _set(**kwargs):
        for key, value in kwargs.items():
            monkeypatch.setenv(key, str(value))
    return _set


@pytest.fixture
def mock_http_response():
    """Factory fixture for creating mock HTTP responses."""
    class MockResponse:
        def __init__(self, json_data=None, status_code=200, text=""):
            self.json_data = json_data
            self.status_code = status_code
            self.text = text
            self.content = text.encode() if isinstance(text, str) else text

        def json(self):
            if self.json_data is None:
                raise ValueError("No JSON data")
            return self.json_data

        def raise_for_status(self):
            if 400 <= self.status_code < 600:
                raise Exception(f"HTTP {self.status_code}")

    return MockResponse


@pytest.fixture
def temp_json_file(tmp_path):
    """Factory fixture for creating temporary JSON files."""
    def _create(data: Dict[str, Any], filename: str = "test.json") -> Path:
        file_path = tmp_path / filename
        with open(file_path, "w") as f:
            json.dump(data, f)
        return file_path
    return _create
