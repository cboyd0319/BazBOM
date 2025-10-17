#!/usr/bin/env python3
"""Comprehensive tests for SBOM signing, Rekor, and attestation modules."""

import json
import os
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch, MagicMock, call
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from sbom_signing import SBOMSigner, verify_sbom_signature
from rekor_integration import RekorClient, RekorEntryFormatter
from intoto_attestation import InTotoAttestationGenerator


class TestSBOMSigner(unittest.TestCase):
    """Test cases for SBOM signing functionality."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.test_sbom_path = os.path.join(self.temp_dir, "test.spdx.json")
        
        # Create a valid SBOM file
        test_sbom = {
            "spdxVersion": "SPDX-2.3",
            "dataLicense": "CC0-1.0",
            "SPDXID": "SPDXRef-DOCUMENT",
            "name": "test-sbom",
            "documentNamespace": "https://example.com/test",
            "creationInfo": {
                "created": "2025-01-17T00:00:00Z",
                "creators": ["Tool: BazBOM"]
            },
            "packages": []
        }
        
        with open(self.test_sbom_path, 'w') as f:
            json.dump(test_sbom, f)
    
    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    @patch('sbom_signing.subprocess.run')
    def test_verify_cosign_available_success(self, mock_run):
        """Test successful cosign availability check."""
        mock_result = Mock()
        mock_result.returncode = 0
        mock_result.stderr = ""
        mock_run.return_value = mock_result
        
        # Should not raise
        signer = SBOMSigner()
        self.assertIsNotNone(signer)
    
    @patch('sbom_signing.subprocess.run')
    def test_verify_cosign_not_found(self, mock_run):
        """Test cosign not found error."""
        mock_run.side_effect = FileNotFoundError("cosign not found")
        
        with self.assertRaises(FileNotFoundError) as ctx:
            SBOMSigner()
        
        self.assertIn("cosign binary not found", str(ctx.exception))
    
    @patch('sbom_signing.subprocess.run')
    def test_sign_sbom_file_not_found(self, mock_run):
        """Test signing with non-existent SBOM file."""
        mock_result = Mock()
        mock_result.returncode = 0
        mock_run.return_value = mock_result
        
        signer = SBOMSigner()
        
        with self.assertRaises(FileNotFoundError) as ctx:
            signer.sign_sbom("/nonexistent/sbom.json")
        
        self.assertIn("SBOM file not found", str(ctx.exception))
    
    @patch('sbom_signing.subprocess.run')
    def test_sign_sbom_empty_file(self, mock_run):
        """Test signing with empty SBOM file."""
        mock_result = Mock()
        mock_result.returncode = 0
        mock_run.return_value = mock_result
        
        empty_sbom = os.path.join(self.temp_dir, "empty.json")
        open(empty_sbom, 'w').close()
        
        signer = SBOMSigner()
        
        with self.assertRaises(ValueError) as ctx:
            signer.sign_sbom(empty_sbom)
        
        self.assertIn("empty", str(ctx.exception))
    
    @patch('sbom_signing.subprocess.run')
    def test_sign_sbom_invalid_json(self, mock_run):
        """Test signing with invalid JSON."""
        mock_result = Mock()
        mock_result.returncode = 0
        mock_run.return_value = mock_result
        
        invalid_sbom = os.path.join(self.temp_dir, "invalid.json")
        with open(invalid_sbom, 'w') as f:
            f.write("not valid json {")
        
        signer = SBOMSigner()
        
        with self.assertRaises(ValueError) as ctx:
            signer.sign_sbom(invalid_sbom)
        
        self.assertIn("not valid JSON", str(ctx.exception))
    
    @patch('sbom_signing.subprocess.run')
    def test_sign_sbom_success(self, mock_run):
        """Test successful SBOM signing."""
        # Mock cosign version check
        version_result = Mock()
        version_result.returncode = 0
        
        # Mock cosign sign-blob
        sign_result = Mock()
        sign_result.returncode = 0
        sign_result.stdout = "MEUCIQBase64SignatureHere=="
        sign_result.stderr = (
            "-----BEGIN CERTIFICATE-----\n"
            "MIICertificateDataHere\n"
            "-----END CERTIFICATE-----\n"
            "tlog entry created with index: 12345\n"
            "https://rekor.sigstore.dev/api/v1/log/entries/abc123"
        )
        
        mock_run.side_effect = [version_result, sign_result]
        
        signer = SBOMSigner()
        result = signer.sign_sbom(self.test_sbom_path)
        
        self.assertIn("signature", result)
        self.assertIn("certificate", result)
        self.assertIn("rekor_entry", result)
        self.assertEqual(result["signature"], "MEUCIQBase64SignatureHere==")
        self.assertIn("BEGIN CERTIFICATE", result["certificate"])
        self.assertIn("rekor.sigstore.dev", result["rekor_entry"])
    
    @patch('sbom_signing.subprocess.run')
    def test_sign_sbom_with_bundle(self, mock_run):
        """Test SBOM signing with bundle output."""
        version_result = Mock()
        version_result.returncode = 0
        
        sign_result = Mock()
        sign_result.returncode = 0
        sign_result.stdout = "SignatureData"
        sign_result.stderr = (
            "-----BEGIN CERTIFICATE-----\nCert\n-----END CERTIFICATE-----\n"
            "https://rekor.sigstore.dev/api/v1/log/entries/xyz789"
        )
        
        mock_run.side_effect = [version_result, sign_result]
        
        bundle_path = os.path.join(self.temp_dir, "bundle.json")
        
        signer = SBOMSigner()
        result = signer.sign_sbom(self.test_sbom_path, bundle_path=bundle_path)
        
        self.assertEqual(result["bundle_path"], bundle_path)
        
        # Verify cosign was called with --bundle argument
        sign_call = mock_run.call_args_list[1]
        self.assertIn("--bundle", sign_call[0][0])
        self.assertIn(bundle_path, sign_call[0][0])
    
    @patch('sbom_signing.subprocess.run')
    def test_sign_sbom_timeout(self, mock_run):
        """Test signing timeout handling."""
        version_result = Mock()
        version_result.returncode = 0
        
        import subprocess
        mock_run.side_effect = [
            version_result,
            subprocess.TimeoutExpired("cosign", 60)
        ]
        
        signer = SBOMSigner()
        
        with self.assertRaises(RuntimeError) as ctx:
            signer.sign_sbom(self.test_sbom_path)
        
        self.assertIn("timed out", str(ctx.exception))
    
    @patch('sbom_signing.subprocess.run')
    def test_sign_multiple_sboms(self, mock_run):
        """Test signing multiple SBOMs."""
        # Create additional test SBOMs
        sbom2_path = os.path.join(self.temp_dir, "test2.spdx.json")
        with open(sbom2_path, 'w') as f:
            json.dump({"spdxVersion": "SPDX-2.3", "packages": []}, f)
        
        version_result = Mock()
        version_result.returncode = 0
        
        sign_result = Mock()
        sign_result.returncode = 0
        sign_result.stdout = "Signature"
        sign_result.stderr = (
            "-----BEGIN CERTIFICATE-----\nCert\n-----END CERTIFICATE-----\n"
            "https://rekor.sigstore.dev/api/v1/log/entries/entry1"
        )
        
        mock_run.side_effect = [version_result, sign_result, sign_result]
        
        signer = SBOMSigner()
        results = signer.sign_multiple_sboms([self.test_sbom_path, sbom2_path])
        
        self.assertEqual(len(results), 2)
        self.assertEqual(results[0]["sbom_path"], self.test_sbom_path)
        self.assertEqual(results[1]["sbom_path"], sbom2_path)
    
    @patch('sbom_signing.subprocess.run')
    def test_sign_multiple_sboms_with_failure(self, mock_run):
        """Test batch signing with one failure."""
        sbom2_path = os.path.join(self.temp_dir, "test2.spdx.json")
        with open(sbom2_path, 'w') as f:
            json.dump({"spdxVersion": "SPDX-2.3", "packages": []}, f)
        
        version_result = Mock()
        version_result.returncode = 0
        
        # First sign succeeds, second fails
        sign_success = Mock()
        sign_success.returncode = 0
        sign_success.stdout = "Signature"
        sign_success.stderr = "Cert\nhttps://rekor.sigstore.dev/entry1"
        
        sign_fail = Mock()
        sign_fail.returncode = 1
        sign_fail.stderr = "Signing failed"
        
        mock_run.side_effect = [version_result, sign_success, sign_fail]
        
        signer = SBOMSigner()
        
        with self.assertRaises(RuntimeError) as ctx:
            signer.sign_multiple_sboms([self.test_sbom_path, sbom2_path])
        
        self.assertIn("Failed to sign", str(ctx.exception))


class TestRekorClient(unittest.TestCase):
    """Test cases for Rekor client."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.client = RekorClient()
    
    @patch('rekor_integration.requests.get')
    def test_get_entry_by_uuid_success(self, mock_get):
        """Test successful entry retrieval."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "test-uuid": {
                "logIndex": 12345,
                "integratedTime": 1642000000,
                "body": "base64data"
            }
        }
        mock_get.return_value = mock_response
        
        entry = self.client.get_entry_by_uuid("test-uuid")
        
        self.assertIsNotNone(entry)
        self.assertEqual(entry["logIndex"], 12345)
    
    @patch('rekor_integration.requests.get')
    def test_get_entry_not_found(self, mock_get):
        """Test entry not found (404)."""
        mock_response = Mock()
        mock_response.status_code = 404
        mock_response.raise_for_status.side_effect = Exception("404")
        
        import requests
        mock_get.return_value = mock_response
        mock_response.raise_for_status.side_effect = requests.HTTPError(response=mock_response)
        
        entry = self.client.get_entry_by_uuid("nonexistent-uuid")
        
        self.assertIsNone(entry)
    
    @patch('rekor_integration.requests.get')
    def test_get_entry_timeout(self, mock_get):
        """Test request timeout."""
        import requests
        mock_get.side_effect = requests.Timeout()
        
        with self.assertRaises(TimeoutError):
            self.client.get_entry_by_uuid("test-uuid")
    
    def test_get_entry_empty_uuid(self):
        """Test empty UUID validation."""
        with self.assertRaises(ValueError) as ctx:
            self.client.get_entry_by_uuid("")
        
        self.assertIn("UUID cannot be empty", str(ctx.exception))
    
    @patch('rekor_integration.requests.post')
    def test_search_by_sha256_success(self, mock_post):
        """Test SHA256 search."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = ["uuid1", "uuid2"]
        mock_post.return_value = mock_response
        
        sha256 = "a" * 64  # Valid 64-char hex
        uuids = self.client.search_by_sha256(sha256)
        
        self.assertEqual(len(uuids), 2)
        self.assertEqual(uuids[0], "uuid1")
    
    def test_search_by_sha256_invalid_hash(self):
        """Test invalid SHA256 hash."""
        with self.assertRaises(ValueError) as ctx:
            self.client.search_by_sha256("tooshort")
        
        self.assertIn("Invalid SHA256 hash", str(ctx.exception))
    
    @patch('rekor_integration.requests.get')
    def test_get_latest_checkpoint(self, mock_get):
        """Test checkpoint retrieval."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "treeSize": "123456",
            "rootHash": "abc123"
        }
        mock_get.return_value = mock_response
        
        checkpoint = self.client.get_latest_checkpoint()
        
        self.assertEqual(checkpoint["treeSize"], "123456")
    
    @patch('rekor_integration.requests.get')
    def test_verify_entry_inclusion_success(self, mock_get):
        """Test entry inclusion verification."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "test-uuid": {"logIndex": 123}
        }
        mock_get.return_value = mock_response
        
        verified = self.client.verify_entry_inclusion("test-uuid")
        
        self.assertTrue(verified)
    
    @patch('rekor_integration.requests.get')
    def test_verify_entry_inclusion_not_found(self, mock_get):
        """Test verification with non-existent entry."""
        import requests
        mock_response = Mock()
        mock_response.status_code = 404
        mock_response.raise_for_status.side_effect = requests.HTTPError(response=mock_response)
        mock_get.return_value = mock_response
        
        verified = self.client.verify_entry_inclusion("nonexistent")
        
        self.assertFalse(verified)


class TestRekorEntryFormatter(unittest.TestCase):
    """Test cases for Rekor entry formatting."""
    
    def test_format_entry_summary(self):
        """Test entry summary formatting."""
        entry = {
            "logIndex": 12345,
            "integratedTime": 1642000000,
            "body": {
                "kind": "hashedrekord",
                "spec": {
                    "data": {
                        "hash": {
                            "sha256": "abc123"
                        }
                    }
                }
            }
        }
        
        summary = RekorEntryFormatter.format_entry_summary(entry, "test-uuid")
        
        self.assertIn("test-uuid", summary)
        self.assertIn("12345", summary)
        self.assertIn("sha256", summary.lower())
    
    def test_format_timestamp(self):
        """Test timestamp formatting."""
        timestamp = 1642000000  # 2022-01-12T18:13:20
        formatted = RekorEntryFormatter._format_timestamp(timestamp)
        
        self.assertIn("2022-01-12", formatted)
    
    def test_format_timestamp_none(self):
        """Test None timestamp handling."""
        formatted = RekorEntryFormatter._format_timestamp(None)
        self.assertEqual(formatted, "unknown")
    
    def test_to_json(self):
        """Test JSON formatting."""
        entry = {"logIndex": 123}
        json_str = RekorEntryFormatter.to_json(entry, "test-uuid")
        
        data = json.loads(json_str)
        self.assertEqual(data["uuid"], "test-uuid")
        self.assertEqual(data["entry"]["logIndex"], 123)


class TestInTotoAttestationGenerator(unittest.TestCase):
    """Test cases for in-toto attestation generation."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.generator = InTotoAttestationGenerator()
        
        # Create test SBOM
        self.test_sbom_path = os.path.join(self.temp_dir, "test.spdx.json")
        with open(self.test_sbom_path, 'w') as f:
            json.dump({"spdxVersion": "SPDX-2.3"}, f)
    
    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    def test_generate_sbom_attestation_success(self):
        """Test successful attestation generation."""
        attestation = self.generator.generate_sbom_attestation(self.test_sbom_path)
        
        self.assertEqual(attestation["_type"], InTotoAttestationGenerator.STATEMENT_TYPE)
        self.assertEqual(attestation["predicateType"], InTotoAttestationGenerator.SLSA_PROVENANCE_TYPE)
        self.assertIn("subject", attestation)
        self.assertIn("predicate", attestation)
        
        # Verify subject structure
        subject = attestation["subject"][0]
        self.assertEqual(subject["name"], "test.spdx.json")
        self.assertIn("sha256", subject["digest"])
    
    def test_generate_sbom_attestation_file_not_found(self):
        """Test attestation with non-existent file."""
        with self.assertRaises(FileNotFoundError):
            self.generator.generate_sbom_attestation("/nonexistent/sbom.json")
    
    def test_calculate_sha256(self):
        """Test SHA256 calculation."""
        sha256 = self.generator._calculate_sha256(self.test_sbom_path)
        
        self.assertEqual(len(sha256), 64)
        self.assertTrue(all(c in "0123456789abcdef" for c in sha256))
    
    def test_build_predicate_with_materials(self):
        """Test predicate building with materials."""
        materials = [
            {
                "uri": "git+https://github.com/example/repo@abc123",
                "digest": {"sha1": "abc123"},
                "name": "source"
            }
        ]
        
        predicate = self.generator._build_predicate(
            builder_id="test-builder",
            build_invocation_id="build-123",
            build_config={"key": "value"},
            materials=materials
        )
        
        self.assertIn("buildDefinition", predicate)
        self.assertIn("runDetails", predicate)
        self.assertEqual(predicate["runDetails"]["builder"]["id"], "test-builder")
        self.assertEqual(predicate["runDetails"]["metadata"]["invocationId"], "build-123")
    
    def test_create_attestation_bundle(self):
        """Test attestation bundle creation."""
        bundle = self.generator.create_attestation_bundle(
            self.test_sbom_path,
            signature="base64sig",
            certificate="PEM cert",
            rekor_entry_url="https://rekor.sigstore.dev/entry123"
        )
        
        self.assertIn("attestation", bundle)
        self.assertIn("signature", bundle)
        self.assertIn("signing_cert", bundle)
        self.assertIn("rekor_entry", bundle)
        self.assertEqual(bundle["signature"]["sig"], "base64sig")
        self.assertEqual(bundle["signing_cert"], "PEM cert")
    
    def test_create_attestation_bundle_empty_signature(self):
        """Test bundle creation with empty signature."""
        with self.assertRaises(ValueError) as ctx:
            self.generator.create_attestation_bundle(
                self.test_sbom_path,
                signature="",
                certificate="cert"
            )
        
        self.assertIn("Signature cannot be empty", str(ctx.exception))
    
    def test_verify_attestation_structure_valid(self):
        """Test attestation structure validation (valid)."""
        attestation = self.generator.generate_sbom_attestation(self.test_sbom_path)
        
        valid = self.generator.verify_attestation_structure(attestation)
        
        self.assertTrue(valid)
    
    def test_verify_attestation_structure_invalid_type(self):
        """Test validation with invalid _type."""
        attestation = {
            "_type": "wrong-type",
            "subject": [],
            "predicateType": "type",
            "predicate": {}
        }
        
        valid = self.generator.verify_attestation_structure(attestation)
        
        self.assertFalse(valid)
    
    def test_verify_attestation_structure_missing_fields(self):
        """Test validation with missing fields."""
        attestation = {
            "_type": InTotoAttestationGenerator.STATEMENT_TYPE,
            "subject": []
            # Missing predicateType and predicate
        }
        
        valid = self.generator.verify_attestation_structure(attestation)
        
        self.assertFalse(valid)
    
    def test_format_materials(self):
        """Test materials formatting."""
        materials = [
            {"uri": "git+repo", "digest": {"sha1": "abc"}, "name": "source"},
            {"uri": "http://dep", "digest": {"sha256": "def"}}
        ]
        
        formatted = self.generator._format_materials(materials)
        
        self.assertEqual(len(formatted), 2)
        self.assertEqual(formatted[0]["uri"], "git+repo")
        self.assertIn("name", formatted[0])
        self.assertNotIn("name", formatted[1])  # Second material has no name


if __name__ == "__main__":
    unittest.main()
