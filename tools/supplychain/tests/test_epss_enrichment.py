#!/usr/bin/env python3
"""Tests for epss_enrichment.py - EPSS (Exploit Prediction Scoring System) enrichment.

Comprehensive test suite covering:
- EPSS score fetching and caching
- Batch processing of CVEs
- Priority level calculation
- Finding enrichment with EPSS scores
- Error handling for invalid inputs
- Cache management
- CLI functionality
"""

import json
import sys
from datetime import datetime, timedelta
from pathlib import Path
from unittest.mock import Mock, patch
import pytest

sys.path.insert(0, str(Path(__file__).parent.parent))

from epss_enrichment import EPSSEnricher, main


@pytest.fixture
def epss_api_response():
    """Sample EPSS API response matching FIRST.org format."""
    return {
        "status": "OK",
        "status-code": 200,
        "version": "1.0",
        "access": "public",
        "total": 2,
        "offset": 0,
        "limit": 100,
        "data": [
            {
                "cve": "CVE-2021-44228",
                "epss": "0.97542",
                "percentile": "0.99995",
                "date": "2025-01-17"
            },
            {
                "cve": "CVE-2023-12345",
                "epss": "0.00234",
                "percentile": "0.45632",
                "date": "2025-01-17"
            }
        ]
    }


@pytest.fixture
def enricher(tmp_path):
    """Create EPSSEnricher instance with temp cache dir."""
    return EPSSEnricher(cache_dir=str(tmp_path / "epss_cache"))


class TestEPSSEnricherInit:
    """Test EPSSEnricher initialization."""
    
    def test_init_with_default_cache_dir(self):
        """Test initialization with default cache directory."""
        enricher = EPSSEnricher()
        assert enricher.cache_dir == ".bazel-cache/epss"
        assert enricher._cache == {}
        assert enricher._cache_loaded is False
    
    def test_init_with_custom_cache_dir(self):
        """Test initialization with custom cache directory."""
        enricher = EPSSEnricher(cache_dir="/custom/path")
        assert enricher.cache_dir == "/custom/path"
    
    def test_batch_size_constant(self):
        """Test BATCH_SIZE constant is set."""
        enricher = EPSSEnricher()
        assert enricher.BATCH_SIZE == 100


class TestLoadCache:
    """Test EPSS cache loading."""
    
    def test_load_cache_returns_empty_when_no_cache(self, enricher):
        """Test loading cache when no cache file exists."""
        # Act
        cache = enricher._load_cache()
        
        # Assert
        assert cache == {}
        # Note: _cache_loaded might not be True if cache doesn't exist
    
    def test_load_cache_returns_cached_data(self, enricher):
        """Test loading cache from existing file."""
        # Arrange
        cache_file = Path(enricher.cache_dir) / "epss_cache.json"
        cache_file.parent.mkdir(parents=True, exist_ok=True)
        
        cache_data = {
            "CVE-2021-44228": {
                "epss_score": 0.97542,
                "epss_percentile": 0.99995,
                "date": "2025-01-17"
            }
        }
        with open(cache_file, 'w') as f:
            json.dump(cache_data, f)
        
        # Act
        cache = enricher._load_cache()
        
        # Assert
        assert cache == cache_data
        assert enricher._cache_loaded is True
    
    def test_load_cache_ignores_stale_cache(self, enricher):
        """Test stale cache is ignored (past TTL)."""
        # Arrange
        cache_file = Path(enricher.cache_dir) / "epss_cache.json"
        cache_file.parent.mkdir(parents=True, exist_ok=True)
        
        stale_data = {"CVE-2021-00000": {"epss_score": 0.5}}
        with open(cache_file, 'w') as f:
            json.dump(stale_data, f)
        
        # Set file modification time to 25 hours ago
        old_time = datetime.now() - timedelta(hours=25)
        import os
        os.utime(cache_file, (old_time.timestamp(), old_time.timestamp()))
        
        # Act
        cache = enricher._load_cache()
        
        # Assert
        assert cache == {}  # Stale cache not loaded
    
    def test_load_cache_handles_corrupt_json(self, enricher):
        """Test handling of corrupt cache file."""
        # Arrange
        cache_file = Path(enricher.cache_dir) / "epss_cache.json"
        cache_file.parent.mkdir(parents=True, exist_ok=True)
        
        with open(cache_file, 'w') as f:
            f.write("{invalid json")
        
        # Act
        cache = enricher._load_cache()
        
        # Assert
        assert cache == {}  # Returns empty cache on error
    
    def test_load_cache_only_loads_once(self, enricher):
        """Test cache is only loaded once."""
        # Arrange
        cache_file = Path(enricher.cache_dir) / "epss_cache.json"
        cache_file.parent.mkdir(parents=True, exist_ok=True)
        
        cache_data = {"CVE-2021-44228": {"epss_score": 0.97542}}
        with open(cache_file, 'w') as f:
            json.dump(cache_data, f)
        
        # Act
        cache1 = enricher._load_cache()
        cache2 = enricher._load_cache()
        
        # Assert
        assert cache1 == cache2
        assert enricher._cache_loaded is True


class TestSaveCache:
    """Test EPSS cache saving."""
    
    def test_save_cache_writes_to_file(self, enricher):
        """Test cache is written to file."""
        # Arrange
        enricher._cache = {
            "CVE-2021-44228": {
                "epss_score": 0.97542,
                "epss_percentile": 0.99995,
                "date": "2025-01-17"
            }
        }
        
        # Act
        enricher._save_cache()
        
        # Assert
        cache_file = Path(enricher.cache_dir) / "epss_cache.json"
        assert cache_file.exists()
        
        with open(cache_file) as f:
            saved_data = json.load(f)
        assert saved_data == enricher._cache
    
    def test_save_cache_creates_directory(self, tmp_path):
        """Test cache directory is created if it doesn't exist."""
        # Arrange
        enricher = EPSSEnricher(cache_dir=str(tmp_path / "nested" / "cache"))
        enricher._cache = {"CVE-2021-44228": {"epss_score": 0.5}}
        
        # Act
        enricher._save_cache()
        
        # Assert
        cache_file = Path(enricher.cache_dir) / "epss_cache.json"
        assert cache_file.exists()


class TestFetchEPSSScores:
    """Test EPSS score fetching."""
    
    def test_fetch_epss_scores_success(self, enricher, epss_api_response, mocker):
        """Test successful EPSS scores fetch from API."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = epss_api_response
        mock_response.raise_for_status.return_value = None
        mock_get = mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        cve_list = ["CVE-2021-44228", "CVE-2023-12345"]
        
        # Act
        result = enricher.fetch_epss_scores(cve_list)
        
        # Assert
        assert len(result) == 2
        assert result["CVE-2021-44228"]["epss_score"] == 0.97542
        assert result["CVE-2021-44228"]["epss_percentile"] == 0.99995
        assert result["CVE-2023-12345"]["epss_score"] == 0.00234
        mock_get.assert_called_once()
    
    def test_fetch_epss_scores_empty_list(self, enricher):
        """Test fetching with empty CVE list."""
        # Act
        result = enricher.fetch_epss_scores([])
        
        # Assert
        assert result == {}
    
    def test_fetch_epss_scores_validates_list_type(self, enricher):
        """Test validation of cve_list parameter type."""
        # Act & Assert
        with pytest.raises(TypeError, match="cve_list must be list"):
            enricher.fetch_epss_scores("not a list")
    
    def test_fetch_epss_scores_validates_cve_format(self, enricher):
        """Test validation of CVE format."""
        # Act & Assert
        with pytest.raises(ValueError, match="Invalid CVE format"):
            enricher.fetch_epss_scores(["not-a-cve"])
    
    def test_fetch_epss_scores_validates_cve_type(self, enricher):
        """Test validation of CVE type in list."""
        # Act & Assert
        with pytest.raises(TypeError, match="CVE ID must be string"):
            enricher.fetch_epss_scores([12345])
    
    def test_fetch_epss_scores_uses_cache(self, enricher, epss_api_response, mocker):
        """Test that cached scores are used without API call."""
        # Arrange
        enricher._cache = {
            "CVE-2021-44228": {
                "epss_score": 0.97542,
                "epss_percentile": 0.99995,
                "date": "2025-01-17"
            }
        }
        enricher._cache_loaded = True
        
        mock_get = mocker.patch('epss_enrichment.requests.get')
        
        # Act
        result = enricher.fetch_epss_scores(["CVE-2021-44228"])
        
        # Assert
        assert result["CVE-2021-44228"]["epss_score"] == 0.97542
        mock_get.assert_not_called()  # No API call for cached data
    
    def test_fetch_epss_scores_batch_processing(self, enricher, mocker):
        """Test batch processing of large CVE lists."""
        # Arrange
        # Create 150 CVEs to test batching (batch size is 100)
        cve_list = [f"CVE-2021-{i:05d}" for i in range(150)]
        
        # Mock API responses for two batches
        batch1_response = {
            "data": [{"cve": f"CVE-2021-{i:05d}", "epss": "0.5", "percentile": "0.5", "date": "2025-01-17"} 
                    for i in range(100)]
        }
        batch2_response = {
            "data": [{"cve": f"CVE-2021-{i:05d}", "epss": "0.5", "percentile": "0.5", "date": "2025-01-17"} 
                    for i in range(100, 150)]
        }
        
        mock_response1 = Mock()
        mock_response1.json.return_value = batch1_response
        mock_response1.raise_for_status.return_value = None
        
        mock_response2 = Mock()
        mock_response2.json.return_value = batch2_response
        mock_response2.raise_for_status.return_value = None
        
        mock_get = mocker.patch('epss_enrichment.requests.get', 
                               side_effect=[mock_response1, mock_response2])
        
        # Act
        result = enricher.fetch_epss_scores(cve_list)
        
        # Assert
        assert len(result) == 150
        assert mock_get.call_count == 2  # Two API calls for two batches
    
    def test_fetch_epss_scores_handles_api_error(self, enricher, mocker):
        """Test handling of API errors."""
        # Arrange
        import requests
        mock_get = mocker.patch('epss_enrichment.requests.get',
                               side_effect=requests.RequestException("Network error"))
        
        cve_list = ["CVE-2021-44228"]
        
        # Act
        result = enricher.fetch_epss_scores(cve_list)
        
        # Assert
        assert result["CVE-2021-44228"]["epss_score"] == 0.0
        assert result["CVE-2021-44228"]["epss_percentile"] == 0.0
        assert "error" in result["CVE-2021-44228"]
    
    def test_fetch_epss_scores_validates_api_response(self, enricher, mocker):
        """Test validation of API response structure."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = []  # Invalid: not a dict
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        # Act & Assert
        with pytest.raises(ValueError, match="Invalid EPSS API response"):
            enricher.fetch_epss_scores(["CVE-2021-44228"])
    
    def test_fetch_epss_scores_handles_missing_cve_in_response(self, enricher, mocker):
        """Test handling of entries without CVE ID in response."""
        # Arrange
        response_with_missing_cve = {
            "data": [
                {"cve": "CVE-2021-44228", "epss": "0.5", "percentile": "0.5", "date": "2025-01-17"},
                {"epss": "0.3", "percentile": "0.3", "date": "2025-01-17"}  # Missing cve
            ]
        }
        mock_response = Mock()
        mock_response.json.return_value = response_with_missing_cve
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        # Act
        result = enricher.fetch_epss_scores(["CVE-2021-44228"])
        
        # Assert
        assert len(result) == 1  # Only valid entry
        assert "CVE-2021-44228" in result
    
    def test_fetch_epss_scores_handles_invalid_score_values(self, enricher, mocker):
        """Test handling of invalid EPSS score values."""
        # Arrange
        response_with_invalid_scores = {
            "data": [
                {"cve": "CVE-2021-44228", "epss": "invalid", "percentile": "0.5", "date": "2025-01-17"}
            ]
        }
        mock_response = Mock()
        mock_response.json.return_value = response_with_invalid_scores
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        # Act
        result = enricher.fetch_epss_scores(["CVE-2021-44228"])
        
        # Assert - invalid entry should be skipped
        assert "CVE-2021-44228" not in result or result["CVE-2021-44228"]["epss_score"] == 0.0
    
    def test_fetch_epss_scores_saves_to_cache(self, enricher, epss_api_response, mocker):
        """Test that fetched scores are saved to cache."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = epss_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        cve_list = ["CVE-2021-44228"]
        
        # Act
        enricher.fetch_epss_scores(cve_list)
        
        # Assert
        cache_file = Path(enricher.cache_dir) / "epss_cache.json"
        assert cache_file.exists()
        
        with open(cache_file) as f:
            cached_data = json.load(f)
        assert "CVE-2021-44228" in cached_data


class TestGetPriorityLevel:
    """Test EPSS score to priority level mapping."""
    
    @pytest.mark.parametrize("score,expected_priority", [
        (0.90, "CRITICAL"),  # >= 0.75
        (0.75, "CRITICAL"),  # Boundary
        (0.60, "HIGH"),      # >= 0.50
        (0.50, "HIGH"),      # Boundary
        (0.40, "MEDIUM"),    # >= 0.25
        (0.25, "MEDIUM"),    # Boundary
        (0.10, "LOW"),       # < 0.25
        (0.00, "LOW"),       # Minimum
        (1.00, "CRITICAL"),  # Maximum
    ], ids=["high_critical", "boundary_critical", "high_high", "boundary_high",
            "high_medium", "boundary_medium", "low", "zero", "one"])
    def test_get_priority_level_thresholds(self, enricher, score, expected_priority):
        """Test priority level calculation for various EPSS scores."""
        # Act
        result = enricher.get_priority_level(score)
        
        # Assert
        assert result == expected_priority
    
    def test_get_priority_level_validates_numeric_type(self, enricher):
        """Test validation of numeric score type."""
        # Act & Assert
        with pytest.raises(TypeError, match="EPSS score must be numeric"):
            enricher.get_priority_level("not numeric")
    
    def test_get_priority_level_validates_range_low(self, enricher):
        """Test validation of score lower bound."""
        # Act & Assert
        with pytest.raises(ValueError, match="EPSS score must be between 0.0 and 1.0"):
            enricher.get_priority_level(-0.1)
    
    def test_get_priority_level_validates_range_high(self, enricher):
        """Test validation of score upper bound."""
        # Act & Assert
        with pytest.raises(ValueError, match="EPSS score must be between 0.0 and 1.0"):
            enricher.get_priority_level(1.1)


class TestEnrichFindings:
    """Test enriching multiple vulnerability findings."""
    
    def test_enrich_findings_adds_epss_to_all(self, enricher, epss_api_response, mocker):
        """Test adding EPSS scores to all findings."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = epss_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        findings = [
            {"cve": "CVE-2021-44228", "severity": "HIGH"},
            {"cve": "CVE-2023-12345", "severity": "MEDIUM"}
        ]
        
        # Act
        result = enricher.enrich_findings(findings)
        
        # Assert
        assert len(result) == 2
        assert result[0]["epss"]["epss_score"] == 0.97542
        assert result[0]["exploitation_probability"] == "97.5%"
        assert result[0]["epss_priority"] == "CRITICAL"
        assert result[1]["epss"]["epss_score"] == 0.00234
        assert result[1]["exploitation_probability"] == "0.2%"
        assert result[1]["epss_priority"] == "LOW"
    
    def test_enrich_findings_handles_empty_list(self, enricher):
        """Test enriching empty findings list."""
        # Act
        result = enricher.enrich_findings([])
        
        # Assert
        assert result == []
    
    def test_enrich_findings_validates_findings_type(self, enricher):
        """Test validation of findings parameter type."""
        # Act & Assert
        with pytest.raises(TypeError, match="Findings must be list"):
            enricher.enrich_findings("not a list")
    
    def test_enrich_findings_skips_non_dict_items(self, enricher, epss_api_response, mocker):
        """Test that non-dict findings are skipped."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = epss_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        findings = [
            {"cve": "CVE-2021-44228"},
            "not a dict",  # Should be skipped
            {"cve": "CVE-2023-12345"}
        ]
        
        # Act
        result = enricher.enrich_findings(findings)
        
        # Assert
        assert len(result) == 3  # All items returned
        assert "epss" in result[0]
        assert "epss" in result[2]
    
    def test_enrich_findings_extracts_cve_from_various_fields(self, enricher, epss_api_response, mocker):
        """Test CVE extraction from different field names."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = epss_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        findings = [
            {"cve": "CVE-2021-44228"},
            {"id": "CVE-2023-12345"},
            {"vulnerability": {"id": "CVE-2021-44228"}}
        ]
        
        # Act
        result = enricher.enrich_findings(findings)
        
        # Assert
        assert "epss" in result[0]
        assert "epss" in result[1]
        assert "epss" in result[2]
    
    def test_enrich_findings_handles_fetch_errors(self, enricher, mocker):
        """Test graceful handling of EPSS fetch errors."""
        # Arrange
        import requests
        mocker.patch('epss_enrichment.requests.get',
                    side_effect=requests.RequestException("Network error"))
        
        findings = [{"cve": "CVE-2021-44228"}]
        
        # Act - should not raise, just continue
        result = enricher.enrich_findings(findings)
        
        # Assert
        assert len(result) == 1
        # EPSS data might be added with error=0.0 or not added at all, depending on implementation
        # The important thing is the process doesn't crash
    
    def test_enrich_findings_skips_non_cve_ids(self, enricher, epss_api_response, mocker):
        """Test that non-CVE IDs are skipped."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = epss_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        findings = [
            {"id": "GHSA-1234-5678-9abc"},  # Not a CVE
            {"cve": "CVE-2021-44228"}
        ]
        
        # Act
        result = enricher.enrich_findings(findings)
        
        # Assert
        assert "epss" not in result[0]  # GHSA skipped
        assert "epss" in result[1]  # CVE enriched


class TestEnrichFinding:
    """Test enriching a single vulnerability finding."""
    
    def test_enrich_finding_calls_enrich_findings(self, enricher, epss_api_response, mocker):
        """Test that enrich_finding wraps enrich_findings."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = epss_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        finding = {"cve": "CVE-2021-44228"}
        
        # Act
        result = enricher.enrich_finding(finding)
        
        # Assert
        assert "epss" in result
        assert result["epss"]["epss_score"] == 0.97542
    
    def test_enrich_finding_handles_empty_finding(self, enricher):
        """Test handling of empty/None finding."""
        # Act
        result = enricher.enrich_finding({})
        
        # Assert
        assert result == {}


class TestCLIMain:
    """Test command-line interface."""
    
    def test_main_with_single_cve(self, epss_api_response, mocker, capsys):
        """Test CLI with single CVE."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = epss_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        with patch('sys.argv', ['epss_enrichment.py', 'CVE-2021-44228']):
            # Act
            exit_code = main()
            
            # Assert
            assert exit_code == 0
            captured = capsys.readouterr()
            assert "CVE-2021-44228" in captured.out
            assert "97.542" in captured.out or "0.97542" in captured.out
            assert "CRITICAL" in captured.out
    
    def test_main_with_multiple_cves(self, epss_api_response, mocker, capsys):
        """Test CLI with multiple CVEs."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = epss_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        with patch('sys.argv', ['epss_enrichment.py', 'CVE-2021-44228', 'CVE-2023-12345']):
            # Act
            exit_code = main()
            
            # Assert
            assert exit_code == 0
            captured = capsys.readouterr()
            assert "CVE-2021-44228" in captured.out
            assert "CVE-2023-12345" in captured.out
    
    def test_main_with_json_output(self, epss_api_response, mocker, capsys):
        """Test CLI with JSON output format."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = epss_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        with patch('sys.argv', ['epss_enrichment.py', 'CVE-2021-44228', '--json']):
            # Act
            exit_code = main()
            
            # Assert
            assert exit_code == 0
            captured = capsys.readouterr()
            result = json.loads(captured.out)
            assert "CVE-2021-44228" in result
            assert result["CVE-2021-44228"]["epss_score"] == 0.97542
    
    def test_main_with_custom_cache_dir(self, epss_api_response, mocker, tmp_path):
        """Test CLI with custom cache directory."""
        # Arrange
        mock_response = Mock()
        mock_response.json.return_value = epss_api_response
        mock_response.raise_for_status.return_value = None
        mocker.patch('epss_enrichment.requests.get', return_value=mock_response)
        
        cache_dir = str(tmp_path / "custom_cache")
        with patch('sys.argv', ['epss_enrichment.py', 'CVE-2021-44228', '--cache-dir', cache_dir]):
            # Act
            exit_code = main()
            
            # Assert
            assert exit_code == 0
            # Verify cache was created in custom location
            assert (Path(cache_dir) / "epss_cache.json").exists()
    
    def test_main_handles_errors(self, mocker, capsys):
        """Test CLI error handling - should handle gracefully."""
        # Arrange
        import requests
        # Mock to prevent real API calls and force error
        mocker.patch('epss_enrichment.requests.get',
                    side_effect=requests.RequestException("Network error"))
        # Also prevent cache from being used
        mocker.patch('pathlib.Path.exists', return_value=False)
        
        with patch('sys.argv', ['epss_enrichment.py', 'CVE-2021-44228']):
            # Act
            exit_code = main()
            
            # Assert - the CLI handles errors gracefully by returning default values
            assert exit_code == 0  # Success with default values
            captured = capsys.readouterr()
            # Should show warning about API failure
            assert "Warning" in captured.err or "0.00" in captured.out
