#!/usr/bin/env python3
"""Tests for kev_enrichment.py - KEV (Known Exploited Vulnerabilities) enrichment.

Comprehensive test suite covering:
- KEV catalog fetching and caching
- KEV index building
- CVE lookup in KEV catalog
- Finding enrichment with KEV data
- Error handling for invalid inputs
- Cache TTL and staleness handling
- CLI functionality
"""

import json
import sys
from datetime import datetime, timedelta
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
import pytest

sys.path.insert(0, str(Path(__file__).parent.parent))

from kev_enrichment import KEVEnricher, main


@pytest.fixture
def kev_catalog_data():
    """Sample KEV catalog data matching CISA format."""
    return {
        "catalogVersion": "2025.01.17",
        "dateReleased": "2025-01-17T00:00:00Z",
        "count": 2,
        "vulnerabilities": [
            {
                "cveID": "CVE-2021-44228",
                "vendorProject": "Apache",
                "product": "Log4j",
                "vulnerabilityName": "Log4Shell",
                "dateAdded": "2021-12-10",
                "shortDescription": "Remote code execution via JNDI",
                "requiredAction": "Apply updates immediately",
                "dueDate": "2021-12-24",
                "notes": "Actively exploited in the wild"
            },
            {
                "cveID": "CVE-2023-12345",
                "vendorProject": "TestVendor",
                "product": "TestProduct",
                "vulnerabilityName": "Test Vuln",
                "dateAdded": "2023-06-15",
                "shortDescription": "Test vulnerability",
                "requiredAction": "Test action",
                "dueDate": "2023-07-15",
                "notes": "Test notes"
            }
        ]
    }


@pytest.fixture
def enricher(tmp_path):
    """Create KEVEnricher instance with temp cache dir."""
    return KEVEnricher(cache_dir=str(tmp_path / "kev_cache"))


class TestKEVEnricherInit:
    """Test KEVEnricher initialization."""
    
    def test_init_with_default_cache_dir(self):
        """Test initialization with default cache directory."""
        enricher = KEVEnricher()
        assert enricher.cache_dir == ".bazel-cache/kev"
        assert enricher._kev_catalog is None
        assert enricher._kev_index is None
    
    def test_init_with_custom_cache_dir(self):
        """Test initialization with custom cache directory."""
        enricher = KEVEnricher(cache_dir="/custom/path")
        assert enricher.cache_dir == "/custom/path"


class TestFetchKEVCatalog:
    """Test KEV catalog fetching and caching."""
    
    def test_fetch_kev_catalog_success(self, enricher, kev_catalog_data, mocker):
        """Test successful KEV catalog fetch from API."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mock_get = mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act
        result = enricher.fetch_kev_catalog()
        
        # Assert
        assert result == kev_catalog_data
        mock_get.assert_called_once_with(enricher.KEV_CATALOG_URL, timeout=30)
        assert result["catalogVersion"] == "2025.01.17"
        assert len(result["vulnerabilities"]) == 2
    
    def test_fetch_kev_catalog_caching(self, enricher, kev_catalog_data, tmp_path, mocker):
        """Test that KEV catalog is cached after first fetch."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mock_get = mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act - first fetch should call API
        result1 = enricher.fetch_kev_catalog()
        
        # Second fetch should use cache
        result2 = enricher.fetch_kev_catalog()
        
        # Assert
        assert result1 == result2
        mock_get.assert_called_once()  # Only called once, not twice
        
        # Check cache file exists
        cache_file = Path(enricher.cache_dir) / "kev_catalog.json"
        assert cache_file.exists()
        with open(cache_file) as f:
            cached_data = json.load(f)
        assert cached_data == kev_catalog_data
    
    def test_fetch_kev_catalog_cache_ttl_expired(self, enricher, kev_catalog_data, mocker):
        """Test cache TTL expiration triggers fresh fetch."""
        # Arrange
        cache_file = Path(enricher.cache_dir) / "kev_catalog.json"
        cache_file.parent.mkdir(parents=True, exist_ok=True)
        
        # Write stale cache (older than TTL)
        stale_data = {"catalogVersion": "2024.01.01", "vulnerabilities": []}
        with open(cache_file, 'w') as f:
            json.dump(stale_data, f)
        
        # Set file modification time to 25 hours ago (past TTL)
        old_time = datetime.now() - timedelta(hours=25)
        cache_file.touch()
        import os
        os.utime(cache_file, (old_time.timestamp(), old_time.timestamp()))
        
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mock_get = mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act
        result = enricher.fetch_kev_catalog()
        
        # Assert
        assert result == kev_catalog_data  # Fresh data, not stale
        mock_get.assert_called_once()  # API was called due to expired cache
    
    def test_fetch_kev_catalog_uses_stale_cache_on_api_failure(self, enricher, mocker):
        """Test stale cache is used as fallback when API fails."""
        # Arrange
        cache_file = Path(enricher.cache_dir) / "kev_catalog.json"
        cache_file.parent.mkdir(parents=True, exist_ok=True)
        
        stale_data = {"catalogVersion": "2024.01.01", "vulnerabilities": []}
        with open(cache_file, 'w') as f:
            json.dump(stale_data, f)
        
        # Set file to old time
        old_time = datetime.now() - timedelta(hours=48)
        import os
        os.utime(cache_file, (old_time.timestamp(), old_time.timestamp()))
        
        # Mock API failure
        import requests
        mock_get = mocker.patch('kev_enrichment.requests.get', 
                               side_effect=requests.RequestException("Network error"))
        
        # Act
        result = enricher.fetch_kev_catalog()
        
        # Assert
        assert result == stale_data  # Used stale cache as fallback
        mock_get.assert_called_once()
    
    def test_fetch_kev_catalog_raises_when_no_cache_and_api_fails(self, enricher, mocker):
        """Test error raised when API fails and no cache exists."""
        # Arrange
        import requests
        mock_get = mocker.patch('kev_enrichment.requests.get',
                               side_effect=requests.RequestException("Network error"))
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="Failed to fetch KEV catalog and no cache available"):
            enricher.fetch_kev_catalog()
    
    def test_fetch_kev_catalog_validates_data_structure(self, enricher, mocker):
        """Test validation of KEV catalog data structure."""
        # Arrange - invalid data type
        mock_response = Mock()
        mock_response.json.return_value = []  # List instead of dict
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act & Assert
        with pytest.raises(ValueError, match="Invalid KEV catalog format: expected dict"):
            enricher.fetch_kev_catalog()
    
    def test_fetch_kev_catalog_validates_vulnerabilities_field(self, enricher, mocker):
        """Test validation that vulnerabilities field exists."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = {"catalogVersion": "2025.01.01"}  # Missing vulnerabilities
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act & Assert
        with pytest.raises(ValueError, match="Invalid KEV catalog: missing 'vulnerabilities' field"):
            enricher.fetch_kev_catalog()
    
    def test_fetch_kev_catalog_handles_corrupt_cache(self, enricher, kev_catalog_data, mocker):
        """Test handling of corrupt cache file."""
        # Arrange
        cache_file = Path(enricher.cache_dir) / "kev_catalog.json"
        cache_file.parent.mkdir(parents=True, exist_ok=True)
        
        # Write corrupt JSON
        with open(cache_file, 'w') as f:
            f.write("{invalid json")
        
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mock_get = mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act
        result = enricher.fetch_kev_catalog()
        
        # Assert
        assert result == kev_catalog_data
        mock_get.assert_called_once()  # Fetched fresh data due to corrupt cache


class TestBuildKEVIndex:
    """Test KEV index building."""
    
    def test_build_kev_index_creates_cve_lookup(self, enricher, kev_catalog_data, mocker):
        """Test building KEV index for fast CVE lookups."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act
        index = enricher._build_kev_index()
        
        # Assert
        assert "CVE-2021-44228" in index
        assert "CVE-2023-12345" in index
        assert len(index) == 2
        assert index["CVE-2021-44228"]["vulnerabilityName"] == "Log4Shell"
    
    def test_build_kev_index_handles_empty_catalog(self, enricher, mocker):
        """Test building index from empty catalog."""
        # Arrange
        empty_catalog = {"catalogVersion": "2025.01.01", "vulnerabilities": []}
        mock_response = Mock()
        mock_response.json.return_value = empty_catalog
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act
        index = enricher._build_kev_index()
        
        # Assert
        assert index == {}
    
    def test_build_kev_index_skips_entries_without_cve_id(self, enricher, mocker):
        """Test that index building skips vulnerabilities without CVE ID."""
        # Arrange
        catalog_with_missing_cve = {
            "vulnerabilities": [
                {"cveID": "CVE-2021-44228", "vulnerabilityName": "Log4Shell"},
                {"vulnerabilityName": "No CVE ID"},  # Missing cveID
            ]
        }
        mock_response = Mock()
        mock_response.json.return_value = catalog_with_missing_cve
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act
        index = enricher._build_kev_index()
        
        # Assert
        assert len(index) == 1
        assert "CVE-2021-44228" in index


class TestIsKnownExploited:
    """Test CVE lookup in KEV catalog."""
    
    def test_is_known_exploited_returns_kev_data_when_found(self, enricher, kev_catalog_data, mocker):
        """Test returning KEV data for CVE in catalog."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act
        result = enricher.is_known_exploited("CVE-2021-44228")
        
        # Assert
        assert result["in_kev"] is True
        assert result["date_added"] == "2021-12-10"
        assert result["due_date"] == "2021-12-24"
        assert result["required_action"] == "Apply updates immediately"
        assert result["vulnerability_name"] == "Log4Shell"
        assert result["vendor_project"] == "Apache"
        assert result["product"] == "Log4j"
    
    def test_is_known_exploited_returns_false_when_not_found(self, enricher, kev_catalog_data, mocker):
        """Test returning in_kev=False for CVE not in catalog."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act
        result = enricher.is_known_exploited("CVE-9999-99999")
        
        # Assert
        assert result == {"in_kev": False}
    
    def test_is_known_exploited_validates_empty_cve_id(self, enricher):
        """Test error on empty CVE ID."""
        # Act & Assert
        with pytest.raises(ValueError, match="CVE ID cannot be empty"):
            enricher.is_known_exploited("")
    
    def test_is_known_exploited_validates_cve_id_type(self, enricher):
        """Test error on non-string CVE ID."""
        # Act & Assert
        with pytest.raises(TypeError, match="CVE ID must be string"):
            enricher.is_known_exploited(12345)
    
    def test_is_known_exploited_caches_index(self, enricher, kev_catalog_data, mocker):
        """Test that KEV index is cached for multiple lookups."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mock_get = mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Act - multiple lookups
        enricher.is_known_exploited("CVE-2021-44228")
        enricher.is_known_exploited("CVE-2023-12345")
        enricher.is_known_exploited("CVE-9999-99999")
        
        # Assert - API called only once for all lookups
        mock_get.assert_called_once()


class TestEnrichFinding:
    """Test enriching vulnerability findings with KEV data."""
    
    def test_enrich_finding_adds_kev_data(self, enricher, kev_catalog_data, mocker):
        """Test adding KEV data to finding with CVE in catalog."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        finding = {
            "cve": "CVE-2021-44228",
            "severity": "HIGH"
        }
        
        # Act
        result = enricher.enrich_finding(finding)
        
        # Assert
        assert result["kev"]["in_kev"] is True
        assert result["kev"]["vulnerability_name"] == "Log4Shell"
        assert result["effective_severity"] == "CRITICAL"
        assert result["priority"] == "IMMEDIATE"
        assert "ACTIVELY EXPLOITED" in result["kev_context"]
    
    def test_enrich_finding_handles_missing_kev_entry(self, enricher, kev_catalog_data, mocker):
        """Test enriching finding with CVE not in KEV catalog."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        finding = {"cve": "CVE-9999-99999"}
        
        # Act
        result = enricher.enrich_finding(finding)
        
        # Assert
        assert result["kev"]["in_kev"] is False
        assert "effective_severity" not in result
        assert "priority" not in result
    
    def test_enrich_finding_extracts_cve_from_id_field(self, enricher, kev_catalog_data, mocker):
        """Test CVE extraction from 'id' field."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        finding = {"id": "CVE-2021-44228"}
        
        # Act
        result = enricher.enrich_finding(finding)
        
        # Assert
        assert result["kev"]["in_kev"] is True
    
    def test_enrich_finding_extracts_cve_from_nested_vulnerability(self, enricher, kev_catalog_data, mocker):
        """Test CVE extraction from nested vulnerability.id field."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        finding = {"vulnerability": {"id": "CVE-2021-44228"}}
        
        # Act
        result = enricher.enrich_finding(finding)
        
        # Assert
        assert result["kev"]["in_kev"] is True
    
    def test_enrich_finding_handles_non_cve_id(self, enricher):
        """Test handling of non-CVE identifiers."""
        # Arrange
        finding = {"id": "GHSA-1234-5678-9abc"}
        
        # Act
        result = enricher.enrich_finding(finding)
        
        # Assert
        assert result["kev"]["in_kev"] is False
    
    def test_enrich_finding_handles_missing_id(self, enricher):
        """Test handling of finding without any ID field."""
        # Arrange
        finding = {"severity": "HIGH"}
        
        # Act
        result = enricher.enrich_finding(finding)
        
        # Assert
        assert result["kev"]["in_kev"] is False
    
    def test_enrich_finding_validates_finding_type(self, enricher):
        """Test error when finding is not a dictionary."""
        # Act & Assert
        with pytest.raises(TypeError, match="Finding must be dict"):
            enricher.enrich_finding("not a dict")
    
    def test_enrich_finding_handles_lookup_errors(self, enricher, kev_catalog_data, mocker):
        """Test graceful handling of errors during KEV lookup."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        # Force an error in is_known_exploited by providing None
        finding = {"cve": None}
        
        # Act
        result = enricher.enrich_finding(finding)
        
        # Assert - should add in_kev=False rather than raise
        assert result["kev"]["in_kev"] is False


class TestCLIMain:
    """Test command-line interface."""
    
    def test_main_with_cve_in_kev(self, kev_catalog_data, mocker, capsys):
        """Test CLI output for CVE in KEV catalog."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-2021-44228']):
            # Act
            exit_code = main()
            
            # Assert
            assert exit_code == 0
            captured = capsys.readouterr()
            assert "IS in CISA KEV catalog" in captured.out
            # Check for vulnerability name (may vary in real KEV catalog)
            assert "Apache" in captured.out or "Log4j" in captured.out
    
    def test_main_with_cve_not_in_kev(self, kev_catalog_data, mocker, capsys):
        """Test CLI output for CVE not in KEV catalog."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-9999-99999']):
            # Act
            exit_code = main()
            
            # Assert
            assert exit_code == 0
            captured = capsys.readouterr()
            assert "is NOT in CISA KEV catalog" in captured.out
    
    def test_main_with_json_output(self, kev_catalog_data, mocker, capsys):
        """Test CLI with JSON output format."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-2021-44228', '--json']):
            # Act
            exit_code = main()
            
            # Assert
            assert exit_code == 0
            captured = capsys.readouterr()
            result = json.loads(captured.out)
            assert result["in_kev"] is True
            # Check for vulnerability name (may vary in real KEV catalog)
            assert "vulnerability_name" in result
            assert result["vulnerability_name"]  # Non-empty
    
    def test_main_with_custom_cache_dir(self, kev_catalog_data, mocker, tmp_path):
        """Test CLI with custom cache directory."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = kev_catalog_data
        mock_response.raise_for_status.return_value = None
        mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
        
        cache_dir = str(tmp_path / "custom_cache")
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-2021-44228', '--cache-dir', cache_dir]):
            # Act
            exit_code = main()
            
            # Assert
            assert exit_code == 0
            # Verify cache was created in custom location
            assert (Path(cache_dir) / "kev_catalog.json").exists()
    
    def test_main_handles_errors(self, mocker, capsys):
        """Test CLI error handling."""
        # Arrange
        import requests
        # Mock both the get call and ensure no cache exists
        mocker.patch('kev_enrichment.requests.get',
                    side_effect=requests.RequestException("Network error"))
        mocker.patch('pathlib.Path.exists', return_value=False)
        
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-2021-44228']):
            # Act
            exit_code = main()
            
            # Assert
            assert exit_code == 1
            captured = capsys.readouterr()
            assert "Error:" in captured.err


@pytest.mark.parametrize("cve_id,expected_in_kev", [
    ("CVE-2021-44228", True),
    ("CVE-2023-12345", True),
    ("CVE-9999-99999", False),
], ids=["log4shell", "test_vuln", "not_in_kev"])
def test_kev_lookup_parametrized(enricher, kev_catalog_data, mocker, cve_id, expected_in_kev):
    """Parametrized test for KEV lookups."""
    # Arrange
    mock_response = Mock()
    mock_response.json.return_value = kev_catalog_data
    mock_response.raise_for_status.return_value = None
    mocker.patch('kev_enrichment.requests.get', return_value=mock_response)
    
    # Act
    result = enricher.is_known_exploited(cve_id)
    
    # Assert
    assert result["in_kev"] == expected_in_kev
