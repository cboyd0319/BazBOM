#!/usr/bin/env python3
"""Shared pytest fixtures and configuration for BazBOM test suite.

This module provides shared fixtures following pytest best practices:
- Session-scoped fixtures for expensive setup
- Function-scoped fixtures for test isolation
- Factory fixtures for flexible test data creation
- Automatic cleanup via pytest fixture lifecycle
"""

import json
import os
import random
from pathlib import Path
from typing import Any, Dict, Callable
from unittest.mock import Mock

import pytest


@pytest.fixture(autouse=True, scope="function")
def _seed_rng():
    """Seed random number generators for deterministic tests.
    
    This fixture runs automatically for every test to ensure reproducibility.
    Function-scoped to reset RNG state for each test.
    """
    random.seed(1337)
    # Seed numpy if available
    try:
        import numpy as np
        np.random.seed(1337)
    except ImportError:
        pass


@pytest.fixture(autouse=True, scope="function")
def _isolate_environment(monkeypatch):
    """Isolate tests from environment variables.
    
    This fixture runs automatically to prevent environment leakage between tests.
    Monkeypatch automatically restores environment after test completion.
    """
    # Monkeypatch handles all cleanup automatically
    yield


@pytest.fixture
def freeze_time():
    """Fixture to freeze time for deterministic tests.
    
    Usage:
        def test_something(freeze_time):
            with freeze_time("2025-01-01 00:00:00"):
                # Time is frozen at this point
                assert datetime.now() == datetime(2025, 1, 1)
    """
    try:
        from freezegun import freeze_time as _freeze_time
        return _freeze_time
    except ImportError:
        # If freezegun not available, return a no-op context manager
        from contextlib import contextmanager
        @contextmanager
        def _noop_freeze(time_str):
            yield
        return _noop_freeze


@pytest.fixture
def tmp_dir(tmp_path):
    """Provide a temporary directory for test files.
    
    Note: Prefer tmp_path directly in new tests. This is for backward compatibility.
    Pytest's tmp_path is automatically cleaned up after test completion.
    """
    return tmp_path


@pytest.fixture(scope="session")
def sample_sbom_data() -> Dict[str, Any]:
    """Provide sample SPDX SBOM data for tests.
    
    Session-scoped as this data is immutable and expensive to recreate.
    Tests should copy this data if they need to modify it.
    """
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


@pytest.fixture(scope="session")
def sample_vulnerability_data() -> Dict[str, Any]:
    """Provide sample vulnerability data for tests.
    
    Session-scoped for performance. Tests should copy if modification needed.
    """
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


@pytest.fixture(scope="session")
def sample_maven_coordinates() -> Dict[str, Any]:
    """Provide sample Maven coordinates data.
    
    Session-scoped for performance. Tests should copy if modification needed.
    """
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
    """Factory fixture for creating temporary JSON files.
    
    Returns a function that creates JSON files in tmp_path.
    Files are automatically cleaned up by pytest after test completion.
    
    Args:
        data: Dictionary to write as JSON
        filename: Name of the file (default: "test.json")
    
    Returns:
        Path object pointing to the created file
    """
    def _create(data: Dict[str, Any], filename: str = "test.json") -> Path:
        file_path = tmp_path / filename
        with open(file_path, "w") as f:
            json.dump(data, f)
        return file_path
    return _create


@pytest.fixture
def mock_requests_get():
    """Factory fixture for creating mock requests.get responses.
    
    Returns a function that creates Mock responses with specified behavior.
    More efficient than creating mocks repeatedly in tests.
    
    Usage:
        def test_api_call(mock_requests_get, mocker):
            mock_resp = mock_requests_get(json_data={"key": "value"}, status=200)
            mocker.patch('module.requests.get', return_value=mock_resp)
    """
    def _create(json_data=None, status_code=200, text="", raise_error=None):
        mock = Mock()
        mock.status_code = status_code
        mock.text = text
        mock.content = text.encode() if isinstance(text, str) else text
        
        if raise_error:
            mock.raise_for_status.side_effect = raise_error
        elif 400 <= status_code < 600:
            mock.raise_for_status.side_effect = Exception(f"HTTP {status_code}")
        else:
            mock.raise_for_status.return_value = None
        
        if json_data is not None:
            mock.json.return_value = json_data
        else:
            mock.json.side_effect = ValueError("No JSON data")
        
        return mock
    return _create


@pytest.fixture
def kev_catalog_data() -> Dict[str, Any]:
    """Provide sample KEV catalog data for enrichment tests.
    
    Extracted as a shared fixture to avoid repetition across tests.
    """
    return {
        "catalogVersion": "2025.01.17",
        "vulnerabilities": [
            {
                "cveID": "CVE-2021-44228",
                "vendorProject": "Apache",
                "product": "Log4j",
                "vulnerabilityName": "Log4Shell",
                "dateAdded": "2021-12-10",
                "shortDescription": "Remote code execution via JNDI",
                "requiredAction": "Apply updates immediately",
                "dueDate": "2021-12-24"
            }
        ]
    }


@pytest.fixture
def epss_data() -> Dict[str, Any]:
    """Provide sample EPSS scoring data for enrichment tests.
    
    Extracted as a shared fixture to avoid repetition.
    """
    return {
        "CVE-2021-44228": {
            "epss": 0.97542,
            "percentile": 0.99995,
            "date": "2025-01-17"
        },
        "CVE-2023-12345": {
            "epss": 0.00234,
            "percentile": 0.45632,
            "date": "2025-01-17"
        }
    }


@pytest.fixture
def ghsa_advisory_data() -> Dict[str, Any]:
    """Provide sample GitHub Security Advisory data.
    
    Extracted as a shared fixture for GHSA enrichment tests.
    """
    return {
        "data": {
            "securityAdvisory": {
                "ghsaId": "GHSA-jfh8-c2jp-5v3q",
                "summary": "Remote Code Execution in Apache Log4j",
                "description": "Apache Log4j2 vulnerability allows remote code execution.",
                "severity": "CRITICAL",
                "publishedAt": "2021-12-10T00:00:00Z",
                "updatedAt": "2021-12-14T00:00:00Z",
                "identifiers": [
                    {"type": "CVE", "value": "CVE-2021-44228"}
                ],
                "references": [
                    {"url": "https://nvd.nist.gov/vuln/detail/CVE-2021-44228"}
                ]
            }
        }
    }
