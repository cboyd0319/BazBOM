#!/usr/bin/env python3
"""Tests for ghsa_enrichment.py - GitHub Security Advisory enrichment.

Test suite covering:
- GHSA GraphQL query execution
- Advisory data parsing
- Finding enrichment with GHSA data
- Error handling for API errors
- Caching behavior
- Token authentication
"""

import json
import sys
from pathlib import Path
from unittest.mock import Mock, patch
import pytest

sys.path.insert(0, str(Path(__file__).parent.parent))

from ghsa_enrichment import GHSAEnricher


@pytest.fixture
def enricher_with_token():
    """Create GHSAEnricher with GitHub token."""
    return GHSAEnricher(github_token="test-token")


@pytest.fixture
def enricher_no_token():
    """Create GHSAEnricher without token."""
    return GHSAEnricher()


@pytest.fixture
def ghsa_api_response():
    """Sample GHSA GraphQL response."""
    return {
        "data": {
            "securityAdvisories": {
                "nodes": [
                    {
                        "ghsaId": "GHSA-jfh8-c2jp-5v3q",
                        "summary": "Remote Code Execution in Apache Log4j",
                        "description": "Detailed description of the vulnerability",
                        "severity": "CRITICAL",
                        "publishedAt": "2021-12-10T00:00:00Z",
                        "updatedAt": "2021-12-14T00:00:00Z",
                        "withdrawnAt": None,
                        "permalink": "https://github.com/advisories/GHSA-jfh8-c2jp-5v3q",
                        "vulnerabilities": {
                            "nodes": [
                                {
                                    "package": {
                                        "name": "org.apache.logging.log4j:log4j-core",
                                        "ecosystem": "MAVEN"
                                    },
                                    "vulnerableVersionRange": ">= 2.0-beta9, < 2.15.0",
                                    "firstPatchedVersion": {
                                        "identifier": "2.15.0"
                                    }
                                }
                            ]
                        },
                        "references": [
                            {"url": "https://nvd.nist.gov/vuln/detail/CVE-2021-44228"}
                        ]
                    }
                ]
            }
        }
    }


class TestGHSAEnricherInit:
    """Test GHSAEnricher initialization."""
    
    def test_init_with_token(self):
        """Test initialization with explicit token."""
        enricher = GHSAEnricher(github_token="test-token")
        assert enricher.token == "test-token"
        assert enricher._cache == {}
    
    def test_init_without_token(self, monkeypatch):
        """Test initialization without token."""
        monkeypatch.delenv("GITHUB_TOKEN", raising=False)
        enricher = GHSAEnricher()
        assert enricher.token is None
    
    def test_init_reads_env_var(self, monkeypatch):
        """Test initialization reads from environment variable."""
        monkeypatch.setenv("GITHUB_TOKEN", "env-token")
        enricher = GHSAEnricher()
        assert enricher.token == "env-token"


class TestQueryAdvisory:
    """Test GHSA advisory querying."""
    
    def test_query_advisory_success(self, enricher_with_token, ghsa_api_response, mocker):
        """Test successful advisory query."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = ghsa_api_response
        mock_response.raise_for_status.return_value = None
        mock_post = mocker.patch('ghsa_enrichment.requests.post', return_value=mock_response)
        
        # Act
        result = enricher_with_token.query_advisory("CVE-2021-44228")
        
        # Assert
        assert result["ghsa_id"] == "GHSA-jfh8-c2jp-5v3q"
        assert result["severity"] == "CRITICAL"
        assert result["summary"] == "Remote Code Execution in Apache Log4j"
        mock_post.assert_called_once()
        # Check authorization header
        call_args = mock_post.call_args
        assert "Authorization" in call_args[1]["headers"]
        assert "bearer test-token" in call_args[1]["headers"]["Authorization"]
    
    def test_query_advisory_not_found(self, enricher_with_token, mocker):
        """Test advisory not found in GHSA."""
        # Arrange
        response = {
            "data": {
                "securityAdvisories": {
                    "nodes": []
                }
            }
        }
        mock_response = Mock()
        mock_response.json.return_value = response
        mock_response.raise_for_status.return_value = None
        mocker.patch('ghsa_enrichment.requests.post', return_value=mock_response)
        
        # Act
        result = enricher_with_token.query_advisory("CVE-9999-99999")
        
        # Assert - returns empty result structure
        assert result["ghsa_id"] == ""
        assert result["summary"] == ""
        assert result["vulnerabilities"] == []
    
    def test_query_advisory_validates_empty_cve(self, enricher_with_token):
        """Test validation of empty CVE ID."""
        # Act & Assert
        with pytest.raises(ValueError, match="CVE ID cannot be empty"):
            enricher_with_token.query_advisory("")
    
    def test_query_advisory_validates_cve_type(self, enricher_with_token):
        """Test validation of CVE ID type."""
        # Act & Assert
        with pytest.raises(TypeError, match="CVE ID must be string"):
            enricher_with_token.query_advisory(12345)
    
    def test_query_advisory_validates_cve_format(self, enricher_with_token):
        """Test validation of CVE format."""
        # Act & Assert
        with pytest.raises(ValueError, match="Invalid CVE format"):
            enricher_with_token.query_advisory("not-a-cve")
    
    def test_query_advisory_uses_cache(self, enricher_with_token, ghsa_api_response, mocker):
        """Test that results are cached."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = ghsa_api_response
        mock_response.raise_for_status.return_value = None
        mock_post = mocker.patch('ghsa_enrichment.requests.post', return_value=mock_response)
        
        # Act - first call
        result1 = enricher_with_token.query_advisory("CVE-2021-44228")
        # Second call should use cache
        result2 = enricher_with_token.query_advisory("CVE-2021-44228")
        
        # Assert
        assert result1 == result2
        mock_post.assert_called_once()  # Only one API call
    
    def test_query_advisory_handles_graphql_errors(self, enricher_with_token, mocker):
        """Test handling of GraphQL errors."""
        # Arrange
        response = {
            "errors": [
                {"message": "Rate limit exceeded"},
                {"message": "Service unavailable"}
            ]
        }
        mock_response = Mock()
        mock_response.json.return_value = response
        mock_response.raise_for_status.return_value = None
        mocker.patch('ghsa_enrichment.requests.post', return_value=mock_response)
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="GraphQL errors"):
            enricher_with_token.query_advisory("CVE-2021-44228")
    
    def test_query_advisory_without_token(self, enricher_no_token, ghsa_api_response, mocker):
        """Test query without authentication token."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = ghsa_api_response
        mock_response.raise_for_status.return_value = None
        mock_post = mocker.patch('ghsa_enrichment.requests.post', return_value=mock_response)
        
        # Act
        result = enricher_no_token.query_advisory("CVE-2021-44228")
        
        # Assert
        assert result["ghsa_id"] == "GHSA-jfh8-c2jp-5v3q"
        # Check that no authorization header was sent
        call_args = mock_post.call_args
        assert "Authorization" not in call_args[1]["headers"]
    
    def test_query_advisory_handles_network_error(self, enricher_with_token, mocker):
        """Test handling of network errors."""
        # Arrange
        import requests
        mocker.patch('ghsa_enrichment.requests.post',
                    side_effect=requests.RequestException("Network error"))
        
        # Act
        result = enricher_with_token.query_advisory("CVE-2021-44228")
        
        # Assert - returns error result structure
        assert result["ghsa_id"] == ""
        assert "error" in result


class TestEnrichFinding:
    """Test finding enrichment."""
    
    def test_enrich_finding_adds_ghsa_data(self, enricher_with_token, ghsa_api_response, mocker):
        """Test adding GHSA data to finding."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = ghsa_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('ghsa_enrichment.requests.post', return_value=mock_response)
        
        finding = {"cve": "CVE-2021-44228", "severity": "HIGH"}
        
        # Act
        result = enricher_with_token.enrich_finding(finding)
        
        # Assert
        assert "ghsa" in result
        assert result["ghsa"]["ghsa_id"] == "GHSA-jfh8-c2jp-5v3q"
        assert result["ghsa"]["severity"] == "CRITICAL"
    
    def test_enrich_finding_extracts_cve_from_id(self, enricher_with_token, ghsa_api_response, mocker):
        """Test CVE extraction from 'id' field."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = ghsa_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('ghsa_enrichment.requests.post', return_value=mock_response)
        
        finding = {"id": "CVE-2021-44228"}
        
        # Act
        result = enricher_with_token.enrich_finding(finding)
        
        # Assert
        assert "ghsa" in result
        assert result["ghsa"]["ghsa_id"] == "GHSA-jfh8-c2jp-5v3q"
    
    def test_enrich_finding_handles_non_cve(self, enricher_with_token):
        """Test handling of non-CVE identifiers."""
        # Arrange
        finding = {"id": "GHSA-1234-5678-9abc"}
        
        # Act
        result = enricher_with_token.enrich_finding(finding)
        
        # Assert - Should have ghsa with empty ghsa_id
        assert "ghsa" in result
        assert result["ghsa"].get("ghsa_id", "") == ""
    
    def test_enrich_finding_handles_missing_id(self, enricher_with_token):
        """Test handling of finding without ID."""
        # Arrange
        finding = {"severity": "HIGH"}
        
        # Act
        result = enricher_with_token.enrich_finding(finding)
        
        # Assert - Should have ghsa with empty ghsa_id
        assert "ghsa" in result
        assert result["ghsa"].get("ghsa_id", "") == ""
    
    def test_enrich_finding_validates_finding_type(self, enricher_with_token):
        """Test validation of finding type."""
        # Act & Assert
        with pytest.raises(TypeError, match="Finding must be dict"):
            enricher_with_token.enrich_finding("not a dict")
    
    def test_enrich_finding_handles_query_errors(self, enricher_with_token, mocker):
        """Test handling of query errors during enrichment."""
        # Arrange
        import requests
        mocker.patch('ghsa_enrichment.requests.post',
                    side_effect=requests.RequestException("Network error"))
        
        finding = {"cve": "CVE-2021-44228"}
        
        # Act - should not raise
        result = enricher_with_token.enrich_finding(finding)
        
        # Assert - GHSA data should be empty or not enriched properly
        assert "ghsa" in result
        assert result["ghsa"].get("ghsa_id", "") == ""


@pytest.mark.parametrize("cve_id,has_advisory", [
    ("CVE-2021-44228", True),
    ("CVE-9999-99999", False),
], ids=["known_advisory", "unknown_cve"])
def test_advisory_lookup_parametrized(enricher_with_token, mocker, cve_id, has_advisory):
    """Parametrized test for advisory lookups."""
    # Arrange
    if has_advisory:
        response_data = {
            "data": {
                "securityAdvisories": {
                    "nodes": [
                        {
                            "ghsaId": "GHSA-test-1234",
                            "severity": "HIGH",
                            "summary": "Test advisory"
                        }
                    ]
                }
            }
        }
    else:
        response_data = {
            "data": {
                "securityAdvisories": {
                    "nodes": []
                }
            }
        }
    
    mock_response = Mock()
    mock_response.json.return_value = response_data
    mock_response.raise_for_status.return_value = None
    mocker.patch('ghsa_enrichment.requests.post', return_value=mock_response)
    
    # Act
    result = enricher_with_token.query_advisory(cve_id)
    
    # Assert
    if has_advisory:
        assert "ghsa_id" in result
        assert result["ghsa_id"] == "GHSA-test-1234"
    else:
        assert result["ghsa_id"] == ""
