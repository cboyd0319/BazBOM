#!/usr/bin/env python3
"""Verify SBOM signatures and attestations.

This script provides end-to-end verification of signed SBOMs including:
- Signature verification with cosign
- Rekor transparency log verification
- in-toto attestation validation
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple

try:
    from sbom_signing import verify_sbom_signature
    from rekor_integration import RekorClient
    from intoto_attestation import InTotoAttestationGenerator
except ImportError as e:
    print(f"Error importing modules: {e}", file=sys.stderr)
    print("Ensure sbom_signing.py, rekor_integration.py, and intoto_attestation.py are in PYTHONPATH", file=sys.stderr)
    sys.exit(1)


class SBOMVerifier:
    """Comprehensive SBOM verification."""
    
    def __init__(self, cosign_path: str = "cosign"):
        """Initialize the verifier.
        
        Args:
            cosign_path: Path to cosign binary
        """
        self.cosign_path = cosign_path
        self.rekor_client = RekorClient()
        self.attestation_generator = InTotoAttestationGenerator()
    
    def verify_sbom_complete(
        self,
        sbom_path: str,
        bundle_path: Optional[str] = None,
        signature_path: Optional[str] = None,
        certificate_identity: Optional[str] = None,
        certificate_oidc_issuer: Optional[str] = None,
        verify_rekor: bool = True,
        verify_attestation: bool = True
    ) -> Dict[str, any]:
        """Perform complete SBOM verification.
        
        Args:
            sbom_path: Path to SBOM file
            bundle_path: Path to signature bundle (optional)
            signature_path: Path to detached signature (optional if bundle provided)
            certificate_identity: Expected identity in certificate
            certificate_oidc_issuer: Expected OIDC issuer
            verify_rekor: Verify Rekor transparency log entry
            verify_attestation: Verify in-toto attestation structure
            
        Returns:
            Verification results dictionary with status and details
        """
        results = {
            "sbom_path": sbom_path,
            "signature_valid": False,
            "rekor_verified": False,
            "attestation_valid": False,
            "overall_status": "FAILED",
            "errors": [],
            "warnings": []
        }
        
        # Step 1: Verify signature
        print("Step 1: Verifying signature with cosign...")
        try:
            sig_valid = verify_sbom_signature(
                sbom_path,
                signature_path=signature_path,
                bundle_path=bundle_path,
                certificate_identity=certificate_identity,
                certificate_oidc_issuer=certificate_oidc_issuer,
                cosign_path=self.cosign_path
            )
            results["signature_valid"] = sig_valid
            
            if not sig_valid:
                results["errors"].append("Signature verification failed")
                return results
            
            print("  ✓ Signature valid")
            
        except Exception as e:
            results["errors"].append(f"Signature verification error: {e}")
            return results
        
        # Step 2: Verify Rekor entry (if requested and bundle available)
        if verify_rekor and bundle_path:
            print("\nStep 2: Verifying Rekor transparency log entry...")
            try:
                rekor_verified = self._verify_rekor_from_bundle(bundle_path)
                results["rekor_verified"] = rekor_verified
                
                if rekor_verified:
                    print("  ✓ Rekor entry verified")
                else:
                    results["warnings"].append("Rekor entry not found or invalid")
                    
            except Exception as e:
                results["warnings"].append(f"Rekor verification warning: {e}")
        else:
            results["warnings"].append("Rekor verification skipped (no bundle provided)")
        
        # Step 3: Verify attestation structure (if bundle available)
        if verify_attestation and bundle_path:
            print("\nStep 3: Verifying in-toto attestation...")
            try:
                attestation_valid = self._verify_attestation_from_bundle(bundle_path)
                results["attestation_valid"] = attestation_valid
                
                if attestation_valid:
                    print("  ✓ Attestation structure valid")
                else:
                    results["warnings"].append("Attestation structure invalid")
                    
            except Exception as e:
                results["warnings"].append(f"Attestation verification warning: {e}")
        else:
            results["warnings"].append("Attestation verification skipped (no bundle provided)")
        
        # Determine overall status
        if results["signature_valid"]:
            if verify_rekor and not results["rekor_verified"]:
                results["overall_status"] = "PASSED_WITH_WARNINGS"
            else:
                results["overall_status"] = "PASSED"
        
        return results
    
    def _verify_rekor_from_bundle(self, bundle_path: str) -> bool:
        """Extract and verify Rekor entry from bundle.
        
        Args:
            bundle_path: Path to signature bundle JSON
            
        Returns:
            True if Rekor entry is verified
        """
        try:
            with open(bundle_path, 'r') as f:
                bundle = json.load(f)
            
            rekor_entry = bundle.get("rekor_entry", "")
            if not rekor_entry:
                print("  Warning: No Rekor entry in bundle", file=sys.stderr)
                return False
            
            # Extract UUID from URL
            uuid = self._extract_uuid_from_url(rekor_entry)
            if not uuid:
                print(f"  Warning: Could not extract UUID from: {rekor_entry}", file=sys.stderr)
                return False
            
            # Verify entry exists in Rekor
            verified = self.rekor_client.verify_entry_inclusion(uuid)
            return verified
            
        except Exception as e:
            print(f"  Error verifying Rekor entry: {e}", file=sys.stderr)
            return False
    
    def _verify_attestation_from_bundle(self, bundle_path: str) -> bool:
        """Extract and verify attestation from bundle.
        
        Args:
            bundle_path: Path to signature bundle JSON
            
        Returns:
            True if attestation is valid
        """
        try:
            with open(bundle_path, 'r') as f:
                bundle = json.load(f)
            
            attestation = bundle.get("attestation")
            if not attestation:
                print("  Warning: No attestation in bundle", file=sys.stderr)
                return False
            
            valid = self.attestation_generator.verify_attestation_structure(attestation)
            return valid
            
        except Exception as e:
            print(f"  Error verifying attestation: {e}", file=sys.stderr)
            return False
    
    def _extract_uuid_from_url(self, url: str) -> Optional[str]:
        """Extract UUID from Rekor entry URL.
        
        Args:
            url: Rekor entry URL
            
        Returns:
            UUID string or None
        """
        # URL format: https://rekor.sigstore.dev/api/v1/log/entries/{uuid}
        parts = url.split('/')
        if len(parts) > 0:
            return parts[-1]
        return None
    
    def print_verification_summary(self, results: Dict[str, any]) -> None:
        """Print human-readable verification summary.
        
        Args:
            results: Verification results dictionary
        """
        print("\n" + "=" * 60)
        print("SBOM Verification Summary")
        print("=" * 60)
        print(f"SBOM: {results['sbom_path']}")
        print(f"Overall Status: {results['overall_status']}")
        print()
        
        print("Verification Steps:")
        print(f"  Signature:    {'✓ PASS' if results['signature_valid'] else '✗ FAIL'}")
        print(f"  Rekor Log:    {'✓ PASS' if results['rekor_verified'] else '⚠ SKIP/WARN'}")
        print(f"  Attestation:  {'✓ PASS' if results['attestation_valid'] else '⚠ SKIP/WARN'}")
        
        if results['errors']:
            print("\nErrors:")
            for error in results['errors']:
                print(f"  ✗ {error}")
        
        if results['warnings']:
            print("\nWarnings:")
            for warning in results['warnings']:
                print(f"  ⚠ {warning}")
        
        print("=" * 60)


def main() -> int:
    """CLI entry point for SBOM verification."""
    parser = argparse.ArgumentParser(
        description="Verify SBOM signatures and attestations"
    )
    
    parser.add_argument(
        "sbom_path",
        help="Path to SBOM file"
    )
    parser.add_argument(
        "--bundle",
        help="Path to signature bundle JSON"
    )
    parser.add_argument(
        "--signature",
        help="Path to detached signature (if no bundle)"
    )
    parser.add_argument(
        "--cert-identity",
        help="Expected certificate identity (e.g., GitHub workflow)"
    )
    parser.add_argument(
        "--cert-oidc-issuer",
        help="Expected OIDC issuer in certificate"
    )
    parser.add_argument(
        "--skip-rekor",
        action="store_true",
        help="Skip Rekor transparency log verification"
    )
    parser.add_argument(
        "--skip-attestation",
        action="store_true",
        help="Skip in-toto attestation verification"
    )
    parser.add_argument(
        "--cosign-path",
        default="cosign",
        help="Path to cosign binary"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output results as JSON"
    )
    
    args = parser.parse_args()
    
    try:
        verifier = SBOMVerifier(cosign_path=args.cosign_path)
        
        results = verifier.verify_sbom_complete(
            args.sbom_path,
            bundle_path=args.bundle,
            signature_path=args.signature,
            certificate_identity=args.cert_identity,
            certificate_oidc_issuer=args.cert_oidc_issuer,
            verify_rekor=not args.skip_rekor,
            verify_attestation=not args.skip_attestation
        )
        
        if args.json:
            print(json.dumps(results, indent=2))
        else:
            verifier.print_verification_summary(results)
        
        # Return appropriate exit code
        if results["overall_status"] == "PASSED":
            return 0
        elif results["overall_status"] == "PASSED_WITH_WARNINGS":
            return 0
        else:
            return 1
            
    except FileNotFoundError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nInterrupted by user", file=sys.stderr)
        return 130
    except Exception as e:
        print(f"UNEXPECTED ERROR: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return 2


if __name__ == "__main__":
    sys.exit(main())
