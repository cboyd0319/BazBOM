#!/usr/bin/env python3
"""Tests for vulncheck_enrichment.py - VulnCheck exploit intelligence enrichment.

Test suite covering:
- VulnCheck API initialization
- Exploit status fetching
- API key validation
- Caching behavior
- Error handling
- Finding enrichment
"""

import json
import sys
from pathlib import Path
from unittest.mock import Mock, patch
import pytest

sys.path.insert(0, str(Path(__file__).parent.parent))

from vulncheck_enrichment import VulnCheckEnricher


@pytest.fixture
def enricher_with_key():
    """Create VulnCheckEnricher with API key."""
    return VulnCheckEnricher(api_key="test-api-key")


@pytest.fixture
def enricher_no_key():
    """Create VulnCheckEnricher without API key."""
    return VulnCheckEnricher()


@pytest.fixture
def vulncheck_api_response():
    """Sample VulnCheck API response."""
    return {
        "data": [{
            "cve_id": "CVE-2021-44228",
            "exploit_available": True,
            "exploit_maturity": "functional",
            "attack_vector": "network",
            "weaponized": True,
            "exploit_public_date": "2021-12-10",
            "exploit_sources": ["metasploit", "github"]
        }]
    }


class TestVulnCheckEnricherInit:
    """Test VulnCheckEnricher initialization."""
    
    def test_init_with_api_key(self):
        """Test initialization with explicit API key."""
        enricher = VulnCheckEnricher(api_key="test-key")
        assert enricher.api_key == "test-key"
        assert enricher._cache == {}
    
    def test_init_without_api_key(self, monkeypatch):
        """Test initialization without API key."""
        monkeypatch.delenv("VULNCHECK_API_KEY", raising=False)
        enricher = VulnCheckEnricher()
        assert enricher.api_key is None
    
    def test_init_reads_env_var(self, monkeypatch):
        """Test initialization reads from environment variable."""
        monkeypatch.setenv("VULNCHECK_API_KEY", "env-key")
        enricher = VulnCheckEnricher()
        assert enricher.api_key == "env-key"


class TestGetExploitStatus:
    """Test exploit status fetching."""
    
    def test_get_exploit_status_success(self, enricher_with_key, vulncheck_api_response, mocker):
        """Test successful exploit status fetch."""
        # Arrange
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = vulncheck_api_response
        mocker.patch('vulncheck_enrichment.requests.get', return_value=mock_response)
        
        # Act
        result = enricher_with_key.get_exploit_status("CVE-2021-44228")
        
        # Assert
        assert result["exploit_available"] is True
        assert result["exploit_maturity"] == "functional"
        assert result["weaponized"] is True
    
    def test_get_exploit_status_no_api_key(self, enricher_no_key):
        """Test behavior when API key is not configured."""
        # Act
        result = enricher_no_key.get_exploit_status("CVE-2021-44228")
        
        # Assert
        assert result["exploit_available"] is False
        assert result["exploit_maturity"] == "unknown"
        assert "API key not configured" in result["note"]
    
    def test_get_exploit_status_validates_empty_cve(self, enricher_with_key):
        """Test validation of empty CVE ID."""
        # Act & Assert
        with pytest.raises(ValueError, match="CVE ID cannot be empty"):
            enricher_with_key.get_exploit_status("")
    
    def test_get_exploit_status_validates_cve_type(self, enricher_with_key):
        """Test validation of CVE ID type."""
        # Act & Assert
        with pytest.raises(TypeError, match="CVE ID must be string"):
            enricher_with_key.get_exploit_status(12345)
    
    def test_get_exploit_status_validates_cve_format(self, enricher_with_key):
        """Test validation of CVE format."""
        # Act & Assert
        with pytest.raises(ValueError, match="Invalid CVE format"):
            enricher_with_key.get_exploit_status("not-a-cve")
    
    def test_get_exploit_status_uses_cache(self, enricher_with_key, vulncheck_api_response, mocker):
        """Test that results are cached."""
        # Arrange
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = vulncheck_api_response
        mock_get = mocker.patch('vulncheck_enrichment.requests.get', return_value=mock_response)
        
        # Act - first call
        result1 = enricher_with_key.get_exploit_status("CVE-2021-44228")
        # Second call should use cache
        result2 = enricher_with_key.get_exploit_status("CVE-2021-44228")
        
        # Assert
        assert result1 == result2
        mock_get.assert_called_once()  # Only one API call
    
    def test_get_exploit_status_handles_401(self, enricher_with_key, mocker):
        """Test handling of 401 Unauthorized response."""
        # Arrange
        mock_response = Mock()
        mock_response.status_code = 401
        mocker.patch('vulncheck_enrichment.requests.get', return_value=mock_response)
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="authentication failed"):
            enricher_with_key.get_exploit_status("CVE-2021-44228")
    
    def test_get_exploit_status_handles_403(self, enricher_with_key, mocker):
        """Test handling of 403 Forbidden response."""
        # Arrange
        mock_response = Mock()
        mock_response.status_code = 403
        mocker.patch('vulncheck_enrichment.requests.get', return_value=mock_response)
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="access forbidden"):
            enricher_with_key.get_exploit_status("CVE-2021-44228")
    
    def test_get_exploit_status_handles_429(self, enricher_with_key, mocker, capsys):
        """Test handling of 429 Rate Limit response."""
        # Arrange
        mock_response = Mock()
        mock_response.status_code = 429
        mocker.patch('vulncheck_enrichment.requests.get', return_value=mock_response)
        
        # Act
        result = enricher_with_key.get_exploit_status("CVE-2021-44228")
        
        # Assert
        assert result["exploit_available"] is False
        assert "Rate limit exceeded" in result.get("error", "")
        captured = capsys.readouterr()
        assert "rate limit" in captured.err.lower()
    
    def test_get_exploit_status_handles_404(self, enricher_with_key, mocker):
        """Test handling of 404 Not Found response."""
        # Arrange
        mock_response = Mock()
        mock_response.status_code = 404
        mock_response.json.return_value = {}
        mocker.patch('vulncheck_enrichment.requests.get', return_value=mock_response)
        
        # Act
        result = enricher_with_key.get_exploit_status("CVE-9999-99999")
        
        # Assert
        assert result["exploit_available"] is False
    
    def test_get_exploit_status_handles_network_error(self, enricher_with_key, mocker):
        """Test handling of network errors."""
        # Arrange
        import requests
        mocker.patch('vulncheck_enrichment.requests.get',
                    side_effect=requests.RequestException("Network error"))
        
        # Act
        result = enricher_with_key.get_exploit_status("CVE-2021-44228")
        
        # Assert
        assert result["exploit_available"] is False
        assert "error" in result
    
    def test_get_exploit_status_handles_invalid_response_type(self, enricher_with_key, mocker):
        """Test handling of invalid response type (non-dict)."""
        # Arrange
        mock_response = Mock()
        mock_response.status_code = 200
        # API returns a list instead of dict
        mock_response.json.return_value = ["invalid", "response"]
        mock_response.raise_for_status.return_value = None
        mocker.patch('vulncheck_enrichment.requests.get', return_value=mock_response)
        
        # Act & Assert
        with pytest.raises(ValueError, match="Invalid VulnCheck response: expected dict"):
            enricher_with_key.get_exploit_status("CVE-2021-44228")


class TestEnrichFinding:
    """Test finding enrichment."""
    
    def test_enrich_finding_adds_exploit_data(self, enricher_with_key, vulncheck_api_response, mocker):
        """Test adding exploit data to finding."""
        # Arrange
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = vulncheck_api_response
        mocker.patch('vulncheck_enrichment.requests.get', return_value=mock_response)
        
        finding = {"cve": "CVE-2021-44228", "severity": "HIGH"}
        
        # Act
        result = enricher_with_key.enrich_finding(finding)
        
        # Assert
        assert "exploit" in result
        assert result["exploit"]["exploit_available"] is True
        assert result["exploit"]["weaponized"] is True
    
    def test_enrich_finding_extracts_cve_from_id(self, enricher_with_key, vulncheck_api_response, mocker):
        """Test CVE extraction from 'id' field."""
        # Arrange
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = vulncheck_api_response
        mocker.patch('vulncheck_enrichment.requests.get', return_value=mock_response)
        
        finding = {"id": "CVE-2021-44228"}
        
        # Act
        result = enricher_with_key.enrich_finding(finding)
        
        # Assert
        assert "exploit" in result
        assert result["exploit"]["exploit_available"] is True
    
    def test_enrich_finding_handles_non_cve(self, enricher_with_key):
        """Test handling of non-CVE identifiers."""
        # Arrange
        finding = {"id": "GHSA-1234-5678-9abc"}
        
        # Act
        result = enricher_with_key.enrich_finding(finding)
        
        # Assert - Should have exploit with False availability
        assert "exploit" in result
        assert result["exploit"]["exploit_available"] is False
    
    def test_enrich_finding_handles_missing_id(self, enricher_with_key):
        """Test handling of finding without ID."""
        # Arrange
        finding = {"severity": "HIGH"}
        
        # Act
        result = enricher_with_key.enrich_finding(finding)
        
        # Assert - Should have exploit with False availability
        assert "exploit" in result
        assert result["exploit"]["exploit_available"] is False
    
    def test_enrich_finding_validates_finding_type(self, enricher_with_key):
        """Test validation of finding type."""
        # Act & Assert
        with pytest.raises(TypeError, match="Finding must be dict"):
            enricher_with_key.enrich_finding("not a dict")


@pytest.mark.parametrize("cve_id,expected_available", [
    ("CVE-2021-44228", True),
    ("CVE-9999-99999", False),
], ids=["known_exploit", "unknown_cve"])
def test_exploit_lookup_parametrized(enricher_with_key, mocker, cve_id, expected_available):
    """Parametrized test for exploit lookups."""
    # Arrange
    if expected_available:
        response_data = {
            "data": [{
                "cve_id": cve_id,
                "exploit_available": True,
                "exploit_maturity": "functional"
            }]
        }
    else:
        response_data = {"data": []}
    
    mock_response = Mock()
    mock_response.status_code = 200 if expected_available else 404
    mock_response.json.return_value = response_data
    mocker.patch('vulncheck_enrichment.requests.get', return_value=mock_response)
    
    # Act
    result = enricher_with_key.get_exploit_status(cve_id)
    
    # Assert
    assert result["exploit_available"] == expected_available
