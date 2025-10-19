#!/usr/bin/env python3
"""Comprehensive tests for rekor_integration module."""

import json
import pytest
from unittest.mock import Mock, patch, MagicMock
from pathlib import Path

# Import the module under test
import sys
sys.path.insert(0, str(Path(__file__).parent.parent))
from rekor_integration import RekorClient, RekorEntryFormatter, main


class TestRekorClient:
    """Test suite for RekorClient class."""
    
    def test_initialization_default_url(self):
        """Test RekorClient initialization with default URL."""
        client = RekorClient()
        assert client.rekor_url == "https://rekor.sigstore.dev"
        assert client.timeout == 30
    
    def test_initialization_custom_url(self):
        """Test RekorClient initialization with custom URL."""
        client = RekorClient(rekor_url="https://custom.rekor.io", timeout=60)
        assert client.rekor_url == "https://custom.rekor.io"
        assert client.timeout == 60
    
    def test_initialization_strips_trailing_slash(self):
        """Test that trailing slash is stripped from URL."""
        client = RekorClient(rekor_url="https://rekor.sigstore.dev/")
        assert client.rekor_url == "https://rekor.sigstore.dev"
    
    @patch('rekor_integration.requests.get')
    def test_get_entry_by_uuid_success(self, mock_get):
        """Test successful retrieval of entry by UUID."""
        # Arrange
        uuid = "24296fb24b8ad77a123456789abcdef"
        expected_entry = {
            "body": "base64data",
            "integratedTime": 1234567890,
            "logID": "logid123"
        }
        mock_response = Mock()
        mock_response.json.return_value = {uuid: expected_entry}
        mock_response.raise_for_status = Mock()
        mock_get.return_value = mock_response
        
        client = RekorClient()
        
        # Act
        result = client.get_entry_by_uuid(uuid)
        
        # Assert
        assert result == expected_entry
        mock_get.assert_called_once_with(
            f"https://rekor.sigstore.dev/api/v1/log/entries/{uuid}",
            timeout=30
        )
    
    @patch('rekor_integration.requests.get')
    def test_get_entry_by_uuid_direct_response(self, mock_get):
        """Test retrieval when response is entry directly (not wrapped)."""
        # Arrange
        uuid = "24296fb24b8ad77a"
        entry_data = {
            "body": "base64data",
            "integratedTime": 1234567890
        }
        mock_response = Mock()
        mock_response.json.return_value = entry_data
        mock_response.raise_for_status = Mock()
        mock_get.return_value = mock_response
        
        client = RekorClient()
        
        # Act
        result = client.get_entry_by_uuid(uuid)
        
        # Assert
        assert result == entry_data
    
    def test_get_entry_by_uuid_empty_uuid(self):
        """Test that empty UUID raises ValueError."""
        client = RekorClient()
        
        with pytest.raises(ValueError, match="UUID cannot be empty"):
            client.get_entry_by_uuid("")
    
    @patch('rekor_integration.requests.get')
    def test_get_entry_by_uuid_not_found(self, mock_get):
        """Test handling of 404 not found."""
        # Arrange
        uuid = "notfound123"
        mock_response = Mock()
        mock_response.status_code = 404
        mock_response.raise_for_status.side_effect = Exception("404")
        
        import requests
        http_error = requests.HTTPError()
        http_error.response = mock_response
        mock_response.raise_for_status.side_effect = http_error
        mock_get.return_value = mock_response
        
        client = RekorClient()
        
        # Act
        result = client.get_entry_by_uuid(uuid)
        
        # Assert
        assert result is None
    
    @patch('rekor_integration.requests.get')
    def test_get_entry_by_uuid_timeout(self, mock_get):
        """Test handling of request timeout."""
        # Arrange
        import requests
        mock_get.side_effect = requests.Timeout("Connection timeout")
        client = RekorClient()
        
        # Act & Assert
        with pytest.raises(TimeoutError, match="timed out"):
            client.get_entry_by_uuid("uuid123")
    
    @patch('rekor_integration.requests.get')
    def test_get_entry_by_uuid_request_exception(self, mock_get):
        """Test handling of general request exception."""
        # Arrange
        import requests
        mock_get.side_effect = requests.RequestException("Network error")
        client = RekorClient()
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="Failed to retrieve entry"):
            client.get_entry_by_uuid("uuid123")
    
    def test_search_by_sha256_invalid_hash_empty(self):
        """Test that empty SHA256 hash raises ValueError."""
        client = RekorClient()
        
        with pytest.raises(ValueError, match="Invalid SHA256 hash"):
            client.search_by_sha256("")
    
    def test_search_by_sha256_invalid_hash_wrong_length(self):
        """Test that wrong length SHA256 hash raises ValueError."""
        client = RekorClient()
        
        with pytest.raises(ValueError, match="Invalid SHA256 hash"):
            client.search_by_sha256("tooshort")
    
    @patch('rekor_integration.requests.post')
    def test_search_by_sha256_success(self, mock_post):
        """Test successful SHA256 search."""
        # Arrange
        sha256 = "a" * 64  # Valid 64-character hash
        expected_uuids = ["uuid1", "uuid2"]
        mock_response = Mock()
        mock_response.json.return_value = expected_uuids
        mock_response.raise_for_status = Mock()
        mock_post.return_value = mock_response
        
        client = RekorClient()
        
        # Act
        result = client.search_by_sha256(sha256)
        
        # Assert
        assert result == expected_uuids
        mock_post.assert_called_once()
        call_args = mock_post.call_args
        assert "hash" in call_args[1]["json"]
        assert call_args[1]["json"]["hash"] == f"sha256:{sha256}"
    
    @patch('rekor_integration.requests.post')
    def test_search_by_sha256_no_results(self, mock_post):
        """Test SHA256 search with no results."""
        # Arrange
        sha256 = "b" * 64
        mock_response = Mock()
        mock_response.json.return_value = []
        mock_response.raise_for_status = Mock()
        mock_post.return_value = mock_response
        
        client = RekorClient()
        
        # Act
        result = client.search_by_sha256(sha256)
        
        # Assert
        assert result == []
    
    @patch('rekor_integration.requests.post')
    def test_search_by_sha256_timeout(self, mock_post):
        """Test SHA256 search timeout handling."""
        # Arrange
        import requests
        mock_post.side_effect = requests.Timeout("Timeout")
        client = RekorClient()
        
        # Act & Assert
        with pytest.raises(TimeoutError):
            client.search_by_sha256("c" * 64)


class TestRekorEntryFormatter:
    """Test suite for RekorEntryFormatter class."""
    
    def test_formatter_initialization(self):
        """Test RekorEntryFormatter initialization."""
        formatter = RekorEntryFormatter()
        # Basic smoke test
        assert formatter is not None


@pytest.mark.parametrize("args,expected_exit", [
    (["--uuid", "test123"], 0),
    (["--sha256", "a" * 64], 0),
])
def test_main_command_variations(args, expected_exit, monkeypatch):
    """Test main function with various command line arguments."""
    # This would require more complex mocking
    # Placeholder for full integration tests
    pass


class TestRekorClientEdgeCases:
    """Test edge cases and error conditions."""
    
    @patch('rekor_integration.requests.get')
    def test_get_entry_empty_response(self, mock_get):
        """Test handling of empty response."""
        mock_response = Mock()
        mock_response.json.return_value = {}
        mock_response.raise_for_status = Mock()
        mock_get.return_value = mock_response
        
        client = RekorClient()
        result = client.get_entry_by_uuid("uuid123")
        
        assert result is None
    
    @patch('rekor_integration.requests.get')
    def test_custom_timeout_used(self, mock_get):
        """Test that custom timeout is passed to requests."""
        mock_response = Mock()
        mock_response.json.return_value = {"uuid": {"data": "test"}}
        mock_response.raise_for_status = Mock()
        mock_get.return_value = mock_response
        
        client = RekorClient(timeout=120)
        client.get_entry_by_uuid("uuid123")
        
        # Verify timeout was passed
        call_kwargs = mock_get.call_args[1]
        assert call_kwargs['timeout'] == 120
    
    def test_multiple_clients_independent(self):
        """Test that multiple client instances are independent."""
        client1 = RekorClient(rekor_url="https://server1.com", timeout=10)
        client2 = RekorClient(rekor_url="https://server2.com", timeout=20)
        
        assert client1.rekor_url != client2.rekor_url
        assert client1.timeout != client2.timeout
    
    @patch('rekor_integration.requests.post')
    def test_search_with_malformed_response(self, mock_post):
        """Test handling of malformed search response."""
        mock_response = Mock()
        mock_response.json.return_value = None
        mock_response.raise_for_status = Mock()
        mock_post.return_value = mock_response
        
        client = RekorClient()
        result = client.search_by_sha256("d" * 64)
        
        # Should handle gracefully
        assert result is None or result == []


class TestRekorIntegration:
    """Integration-style tests for complete workflows."""
    
    @patch('rekor_integration.requests.get')
    @patch('rekor_integration.requests.post')
    def test_search_and_retrieve_workflow(self, mock_post, mock_get):
        """Test complete workflow: search by hash, then retrieve entry."""
        # Arrange
        sha256 = "e" * 64
        uuid = "workflow_uuid123"
        entry_data = {"body": "data", "logIndex": 12345}
        
        # Mock search response
        mock_search_response = Mock()
        mock_search_response.json.return_value = [uuid]
        mock_search_response.raise_for_status = Mock()
        mock_post.return_value = mock_search_response
        
        # Mock get entry response
        mock_get_response = Mock()
        mock_get_response.json.return_value = {uuid: entry_data}
        mock_get_response.raise_for_status = Mock()
        mock_get.return_value = mock_get_response
        
        client = RekorClient()
        
        # Act
        uuids = client.search_by_sha256(sha256)
        entry = client.get_entry_by_uuid(uuids[0])
        
        # Assert
        assert len(uuids) == 1
        assert uuids[0] == uuid
        assert entry == entry_data
