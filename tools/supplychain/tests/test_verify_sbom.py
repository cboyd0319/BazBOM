#!/usr/bin/env python3
"""Comprehensive tests for verify_sbom.py - SBOM signature and attestation verification."""

import json
import sys
from pathlib import Path
from unittest.mock import MagicMock, Mock, patch, call

import pytest

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from verify_sbom import SBOMVerifier


class TestSBOMVerifierInit:
    """Test SBOMVerifier initialization."""

    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_init_with_default_cosign_path(self, mock_attestation, mock_rekor):
        """Test initialization with default cosign path."""
        # Act
        verifier = SBOMVerifier()
        
        # Assert
        assert verifier.cosign_path == "cosign"
        assert verifier.rekor_client is not None
        assert verifier.attestation_generator is not None

    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_init_with_custom_cosign_path(self, mock_attestation, mock_rekor):
        """Test initialization with custom cosign path."""
        # Act
        verifier = SBOMVerifier(cosign_path="/usr/local/bin/cosign")
        
        # Assert
        assert verifier.cosign_path == "/usr/local/bin/cosign"

    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_init_creates_rekor_client(self, mock_attestation, mock_rekor):
        """Test initialization creates RekorClient instance."""
        # Act
        verifier = SBOMVerifier()
        
        # Assert
        mock_rekor.assert_called_once()

    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_init_creates_attestation_generator(self, mock_attestation, mock_rekor):
        """Test initialization creates InTotoAttestationGenerator instance."""
        # Act
        verifier = SBOMVerifier()
        
        # Assert
        mock_attestation.assert_called_once()


class TestVerifySBOMComplete:
    """Test complete SBOM verification."""

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_sbom_complete_signature_fails(self, mock_attestation, mock_rekor, mock_verify):
        """Test verification when signature check fails."""
        # Arrange
        mock_verify.return_value = False
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json",
            signature_path="/path/to/sbom.json.sig"
        )
        
        # Assert
        assert result["signature_valid"] is False
        assert result["overall_status"] == "FAILED"
        assert len(result["errors"]) > 0
        assert "Signature verification failed" in result["errors"]

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_sbom_complete_signature_succeeds(self, mock_attestation, mock_rekor, mock_verify):
        """Test verification when signature check succeeds."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json",
            signature_path="/path/to/sbom.json.sig",
            verify_rekor=False,
            verify_attestation=False
        )
        
        # Assert
        assert result["signature_valid"] is True
        assert result["sbom_path"] == "/path/to/sbom.json"

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_sbom_complete_signature_exception(self, mock_attestation, mock_rekor, mock_verify):
        """Test verification handles signature verification exceptions."""
        # Arrange
        mock_verify.side_effect = Exception("Signature verification error")
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json"
        )
        
        # Assert
        assert result["signature_valid"] is False
        assert result["overall_status"] == "FAILED"
        assert any("Signature verification error" in err for err in result["errors"])

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_sbom_complete_with_bundle_path(self, mock_attestation, mock_rekor, mock_verify):
        """Test verification with bundle path."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json",
            bundle_path="/path/to/bundle.json",
            verify_rekor=False,
            verify_attestation=False
        )
        
        # Assert
        assert result["signature_valid"] is True
        mock_verify.assert_called_once()

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_sbom_complete_with_certificate_identity(self, mock_attestation, mock_rekor, mock_verify):
        """Test verification with certificate identity."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json",
            signature_path="/path/to/sbom.json.sig",
            certificate_identity="user@example.com",
            certificate_oidc_issuer="https://github.com/login/oauth",
            verify_rekor=False,
            verify_attestation=False
        )
        
        # Assert
        assert result["signature_valid"] is True


class TestResultsStructure:
    """Test verification results structure."""

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_results_contains_required_fields(self, mock_attestation, mock_rekor, mock_verify):
        """Test results dictionary contains all required fields."""
        # Arrange
        mock_verify.return_value = False
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json"
        )
        
        # Assert
        assert "sbom_path" in result
        assert "signature_valid" in result
        assert "rekor_verified" in result
        assert "attestation_valid" in result
        assert "overall_status" in result
        assert "errors" in result
        assert "warnings" in result

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_results_errors_is_list(self, mock_attestation, mock_rekor, mock_verify):
        """Test results errors field is a list."""
        # Arrange
        mock_verify.return_value = False
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json"
        )
        
        # Assert
        assert isinstance(result["errors"], list)

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_results_warnings_is_list(self, mock_attestation, mock_rekor, mock_verify):
        """Test results warnings field is a list."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json",
            verify_rekor=False,
            verify_attestation=False
        )
        
        # Assert
        assert isinstance(result["warnings"], list)


class TestRekorVerification:
    """Test Rekor transparency log verification."""

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_rekor_skipped_when_disabled(self, mock_attestation, mock_rekor, mock_verify):
        """Test Rekor verification is skipped when verify_rekor=False."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json",
            verify_rekor=False,
            verify_attestation=False
        )
        
        # Assert
        assert result["rekor_verified"] is False

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_rekor_requires_bundle(self, mock_attestation, mock_rekor, mock_verify):
        """Test Rekor verification requires bundle_path."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json",
            verify_rekor=True,  # Enable but no bundle
            verify_attestation=False
        )
        
        # Assert
        # Should still pass signature but not verify Rekor without bundle
        assert result["signature_valid"] is True


class TestAttestationVerification:
    """Test in-toto attestation verification."""

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_attestation_skipped_when_disabled(self, mock_attestation, mock_rekor, mock_verify):
        """Test attestation verification is skipped when verify_attestation=False."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json",
            verify_rekor=False,
            verify_attestation=False
        )
        
        # Assert
        assert result["attestation_valid"] is False


class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_with_empty_sbom_path(self, mock_attestation, mock_rekor, mock_verify):
        """Test verification with empty SBOM path."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="",
            verify_rekor=False,
            verify_attestation=False
        )
        
        # Assert
        assert result["sbom_path"] == ""

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_with_unicode_paths(self, mock_attestation, mock_rekor, mock_verify):
        """Test verification with Unicode characters in paths."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        unicode_path = "/path/tø/sbøm-日本語.json"
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path=unicode_path,
            verify_rekor=False,
            verify_attestation=False
        )
        
        # Assert
        assert result["sbom_path"] == unicode_path

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_with_very_long_paths(self, mock_attestation, mock_rekor, mock_verify):
        """Test verification with very long file paths."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        long_path = "/path/" + ("subdir/" * 100) + "sbom.json"
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path=long_path,
            verify_rekor=False,
            verify_attestation=False
        )
        
        # Assert
        assert result["sbom_path"] == long_path


class TestCosignPathConfiguration:
    """Test cosign binary path configuration."""

    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_custom_cosign_path(self, mock_attestation, mock_rekor):
        """Test using custom cosign binary path."""
        # Act
        verifier = SBOMVerifier(cosign_path="/custom/path/to/cosign")
        
        # Assert
        assert verifier.cosign_path == "/custom/path/to/cosign"

    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_cosign_path_with_spaces(self, mock_attestation, mock_rekor):
        """Test cosign path with spaces."""
        # Act
        verifier = SBOMVerifier(cosign_path="/path with spaces/cosign")
        
        # Assert
        assert verifier.cosign_path == "/path with spaces/cosign"


class TestVerificationParameters:
    """Test various verification parameter combinations."""

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_all_checks_enabled(self, mock_attestation, mock_rekor, mock_verify):
        """Test verification with all checks enabled."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json",
            bundle_path="/path/to/bundle.json",
            verify_rekor=True,
            verify_attestation=True
        )
        
        # Assert
        assert result["signature_valid"] is True

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_verify_all_checks_disabled(self, mock_attestation, mock_rekor, mock_verify):
        """Test verification with all optional checks disabled."""
        # Arrange
        mock_verify.return_value = True
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json",
            verify_rekor=False,
            verify_attestation=False
        )
        
        # Assert
        assert result["signature_valid"] is True
        assert result["rekor_verified"] is False
        assert result["attestation_valid"] is False


class TestOverallStatus:
    """Test overall verification status determination."""

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_overall_status_failed_when_signature_invalid(self, mock_attestation, mock_rekor, mock_verify):
        """Test overall status is FAILED when signature is invalid."""
        # Arrange
        mock_verify.return_value = False
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json"
        )
        
        # Assert
        assert result["overall_status"] == "FAILED"


class TestErrorHandling:
    """Test error handling and reporting."""

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_errors_list_populated_on_signature_failure(self, mock_attestation, mock_rekor, mock_verify):
        """Test errors list is populated when signature verification fails."""
        # Arrange
        mock_verify.return_value = False
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json"
        )
        
        # Assert
        assert len(result["errors"]) > 0
        assert "Signature verification failed" in result["errors"]

    @patch('verify_sbom.verify_sbom_signature')
    @patch('verify_sbom.RekorClient')
    @patch('verify_sbom.InTotoAttestationGenerator')
    def test_exception_message_captured_in_errors(self, mock_attestation, mock_rekor, mock_verify):
        """Test exception messages are captured in errors list."""
        # Arrange
        error_msg = "Custom error message"
        mock_verify.side_effect = Exception(error_msg)
        verifier = SBOMVerifier()
        
        # Act
        result = verifier.verify_sbom_complete(
            sbom_path="/path/to/sbom.json"
        )
        
        # Assert
        assert any(error_msg in err for err in result["errors"])
