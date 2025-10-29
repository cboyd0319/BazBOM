#!/usr/bin/env python3
"""SBOM signing with Sigstore/cosign.

This module provides cryptographic signing of SBOMs using Sigstore's cosign
tool with keyless signing (OIDC-based authentication). Signatures are
automatically logged to Rekor transparency log.
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple


class SBOMSigner:
    """Sign SBOMs using Sigstore cosign with keyless signing."""

    def __init__(
        self,
        cosign_path: str = "cosign",
        experimental_mode: bool = True,
        oidc_issuer: Optional[str] = None,
        oidc_client_id: Optional[str] = None
    ):
        """Initialize the SBOM signer.
        
        Args:
            cosign_path: Path to cosign binary (default: "cosign" from PATH)
            experimental_mode: Enable experimental features (required for keyless)
            oidc_issuer: Custom OIDC issuer URL (default: GitHub OIDC)
            oidc_client_id: Custom OIDC client ID (default: sigstore)
            
        Raises:
            FileNotFoundError: If cosign binary is not found
            RuntimeError: If cosign version check fails
        """
        self.cosign_path = cosign_path
        self.experimental_mode = experimental_mode
        self.oidc_issuer = oidc_issuer or os.getenv("COSIGN_OIDC_ISSUER", "https://token.actions.githubusercontent.com")
        self.oidc_client_id = oidc_client_id or os.getenv("COSIGN_OIDC_CLIENT_ID", "sigstore")
        
        # Verify cosign is available
        self._verify_cosign_available()
    
    def _verify_cosign_available(self) -> None:
        """Verify cosign binary is available and working.
        
        Raises:
            FileNotFoundError: If cosign is not found
            RuntimeError: If cosign version check fails
        """
        try:
            result = subprocess.run(
                [self.cosign_path, "version"],
                capture_output=True,
                text=True,
                timeout=10
            )
            if result.returncode != 0:
                raise RuntimeError(f"cosign version check failed: {result.stderr}")
        except FileNotFoundError:
            raise FileNotFoundError(
                f"cosign binary not found at '{self.cosign_path}'. "
                "Install from: https://docs.sigstore.dev/cosign/installation"
            )
        except subprocess.TimeoutExpired:
            raise RuntimeError("cosign version check timed out")
    
    def sign_sbom(
        self,
        sbom_path: str,
        output_signature_path: Optional[str] = None,
        bundle_path: Optional[str] = None
    ) -> Dict[str, str]:
        """Sign an SBOM file using keyless signing.
        
        Args:
            sbom_path: Path to SBOM file (SPDX JSON or CycloneDX)
            output_signature_path: Path for detached signature (optional)
            bundle_path: Path for signature bundle with Rekor entry (optional)
            
        Returns:
            Dictionary with:
                - signature: Base64-encoded signature
                - certificate: Signing certificate (PEM)
                - rekor_entry: Rekor transparency log entry URL
                - bundle_path: Path to signature bundle (if requested)
                
        Raises:
            FileNotFoundError: If SBOM file doesn't exist
            ValueError: If SBOM file is invalid
            RuntimeError: If signing fails
        """
        # Validate inputs
        if not os.path.exists(sbom_path):
            raise FileNotFoundError(f"SBOM file not found: {sbom_path}")
        
        sbom_path_obj = Path(sbom_path)
        if sbom_path_obj.stat().st_size == 0:
            raise ValueError(f"SBOM file is empty: {sbom_path}")
        
        # Validate SBOM is valid JSON
        try:
            with open(sbom_path, 'r') as f:
                json.load(f)
        except json.JSONDecodeError as e:
            raise ValueError(f"SBOM is not valid JSON: {e}")
        
        # Prepare environment for keyless signing
        env = os.environ.copy()
        if self.experimental_mode:
            env["COSIGN_EXPERIMENTAL"] = "1"
        
        # Build cosign command
        cmd = [
            self.cosign_path,
            "sign-blob",
            sbom_path,
            "--yes"  # Non-interactive mode
        ]
        
        # Add bundle output if requested
        if bundle_path:
            cmd.extend(["--bundle", bundle_path])
        
        # Execute signing
        try:
            print(f"Signing SBOM: {sbom_path}")
            print("Using keyless signing with GitHub OIDC...")
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                env=env,
                timeout=60
            )
            
            if result.returncode != 0:
                raise RuntimeError(
                    f"cosign signing failed (exit code {result.returncode}):\n"
                    f"stderr: {result.stderr}\n"
                    f"stdout: {result.stdout}"
                )
            
            # Parse output
            signature_base64 = result.stdout.strip()
            
            # Extract certificate and Rekor entry from stderr (cosign logs there)
            cert_pem = self._extract_certificate_from_output(result.stderr)
            rekor_entry = self._extract_rekor_entry_from_output(result.stderr)
            
            # Write signature to file if requested
            if output_signature_path:
                with open(output_signature_path, 'w') as f:
                    f.write(signature_base64)
                print(f"Signature written to: {output_signature_path}")
            
            response = {
                "signature": signature_base64,
                "certificate": cert_pem,
                "rekor_entry": rekor_entry,
                "sbom_path": sbom_path
            }
            
            if bundle_path:
                response["bundle_path"] = bundle_path
                print(f"Signature bundle written to: {bundle_path}")
            
            print(f" SBOM signed successfully")
            print(f"  Rekor entry: {rekor_entry}")
            
            return response
            
        except subprocess.TimeoutExpired:
            raise RuntimeError(
                f"cosign signing timed out after 60 seconds. "
                "Check network connectivity and OIDC token availability."
            )
        except Exception as e:
            raise RuntimeError(f"Unexpected error during signing: {str(e)}")
    
    def _extract_certificate_from_output(self, output: str) -> str:
        """Extract signing certificate from cosign output.
        
        Args:
            output: cosign command output (stderr)
            
        Returns:
            PEM-encoded certificate or empty string if not found
        """
        lines = output.split('\n')
        cert_lines = []
        in_cert = False
        
        for line in lines:
            if "-----BEGIN CERTIFICATE-----" in line:
                in_cert = True
            if in_cert:
                cert_lines.append(line)
            if "-----END CERTIFICATE-----" in line:
                in_cert = False
                break
        
        return '\n'.join(cert_lines) if cert_lines else ""
    
    def _extract_rekor_entry_from_output(self, output: str) -> str:
        """Extract Rekor transparency log entry URL from cosign output.
        
        Args:
            output: cosign command output (stderr)
            
        Returns:
            Rekor entry URL or empty string if not found
        """
        lines = output.split('\n')
        for line in lines:
            if "rekor.sigstore.dev" in line.lower() or "tlog entry" in line.lower():
                # Extract URL
                parts = line.split()
                for part in parts:
                    if "rekor.sigstore.dev" in part or part.startswith("https://"):
                        return part.strip()
        
        return ""
    
    def sign_multiple_sboms(
        self,
        sbom_paths: List[str],
        output_dir: Optional[str] = None
    ) -> List[Dict[str, str]]:
        """Sign multiple SBOM files.
        
        Args:
            sbom_paths: List of SBOM file paths
            output_dir: Directory for signature outputs (optional)
            
        Returns:
            List of signing results (one per SBOM)
            
        Raises:
            ValueError: If sbom_paths is empty
            RuntimeError: If any signing fails (after attempting all)
        """
        if not sbom_paths:
            raise ValueError("No SBOM paths provided")
        
        results = []
        failures = []
        
        for sbom_path in sbom_paths:
            try:
                # Generate output paths if directory specified
                sig_path = None
                bundle_path = None
                
                if output_dir:
                    os.makedirs(output_dir, exist_ok=True)
                    base_name = Path(sbom_path).stem
                    sig_path = os.path.join(output_dir, f"{base_name}.sig")
                    bundle_path = os.path.join(output_dir, f"{base_name}.bundle.json")
                
                result = self.sign_sbom(sbom_path, sig_path, bundle_path)
                results.append(result)
                
            except Exception as e:
                error_msg = f"Failed to sign {sbom_path}: {str(e)}"
                print(f"ERROR: {error_msg}", file=sys.stderr)
                failures.append((sbom_path, str(e)))
                results.append({
                    "sbom_path": sbom_path,
                    "error": str(e)
                })
        
        if failures:
            raise RuntimeError(
                f"Failed to sign {len(failures)} of {len(sbom_paths)} SBOMs:\n" +
                "\n".join([f"  - {path}: {err}" for path, err in failures])
            )
        
        return results


def verify_sbom_signature(
    sbom_path: str,
    signature_path: Optional[str] = None,
    bundle_path: Optional[str] = None,
    certificate_identity: Optional[str] = None,
    certificate_oidc_issuer: Optional[str] = None,
    cosign_path: str = "cosign"
) -> bool:
    """Verify an SBOM signature.
    
    Args:
        sbom_path: Path to SBOM file
        signature_path: Path to detached signature (optional if bundle provided)
        bundle_path: Path to signature bundle (optional if signature provided)
        certificate_identity: Expected identity in certificate (e.g., email, workflow)
        certificate_oidc_issuer: Expected OIDC issuer in certificate
        cosign_path: Path to cosign binary
        
    Returns:
        True if signature is valid, False otherwise
        
    Raises:
        FileNotFoundError: If required files don't exist
        ValueError: If neither signature nor bundle provided
    """
    if not os.path.exists(sbom_path):
        raise FileNotFoundError(f"SBOM file not found: {sbom_path}")
    
    if not signature_path and not bundle_path:
        raise ValueError("Either signature_path or bundle_path must be provided")
    
    # Prepare environment
    env = os.environ.copy()
    env["COSIGN_EXPERIMENTAL"] = "1"
    
    # Build verification command
    cmd = [cosign_path, "verify-blob", sbom_path]
    
    if bundle_path:
        if not os.path.exists(bundle_path):
            raise FileNotFoundError(f"Bundle file not found: {bundle_path}")
        cmd.extend(["--bundle", bundle_path])
    elif signature_path:
        if not os.path.exists(signature_path):
            raise FileNotFoundError(f"Signature file not found: {signature_path}")
        cmd.extend(["--signature", signature_path])
    
    # Add identity verification if provided
    if certificate_identity:
        cmd.extend(["--certificate-identity", certificate_identity])
    
    if certificate_oidc_issuer:
        cmd.extend(["--certificate-oidc-issuer", certificate_oidc_issuer])
    
    # Execute verification
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            env=env,
            timeout=30
        )
        
        if result.returncode == 0:
            print(f" Signature verification PASSED for: {sbom_path}")
            return True
        else:
            print(f" Signature verification FAILED for: {sbom_path}", file=sys.stderr)
            print(f"  Error: {result.stderr}", file=sys.stderr)
            return False
            
    except subprocess.TimeoutExpired:
        print(f" Verification timed out for: {sbom_path}", file=sys.stderr)
        return False
    except Exception as e:
        print(f" Verification error for {sbom_path}: {e}", file=sys.stderr)
        return False


def main() -> int:
    """CLI entry point for SBOM signing."""
    parser = argparse.ArgumentParser(
        description="Sign SBOMs with Sigstore/cosign using keyless signing"
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Command to execute")
    
    # Sign command
    sign_parser = subparsers.add_parser("sign", help="Sign one or more SBOMs")
    sign_parser.add_argument(
        "sbom_paths",
        nargs="+",
        help="Path(s) to SBOM file(s) to sign"
    )
    sign_parser.add_argument(
        "--output-dir",
        help="Directory for signature outputs (creates .sig and .bundle.json files)"
    )
    sign_parser.add_argument(
        "--cosign-path",
        default="cosign",
        help="Path to cosign binary (default: cosign from PATH)"
    )
    
    # Verify command
    verify_parser = subparsers.add_parser("verify", help="Verify SBOM signature")
    verify_parser.add_argument(
        "sbom_path",
        help="Path to SBOM file"
    )
    verify_parser.add_argument(
        "--signature",
        help="Path to detached signature file"
    )
    verify_parser.add_argument(
        "--bundle",
        help="Path to signature bundle file"
    )
    verify_parser.add_argument(
        "--cert-identity",
        help="Expected certificate identity (e.g., email or GitHub workflow)"
    )
    verify_parser.add_argument(
        "--cert-oidc-issuer",
        help="Expected OIDC issuer in certificate"
    )
    verify_parser.add_argument(
        "--cosign-path",
        default="cosign",
        help="Path to cosign binary"
    )
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return 1
    
    try:
        if args.command == "sign":
            signer = SBOMSigner(cosign_path=args.cosign_path)
            results = signer.sign_multiple_sboms(
                args.sbom_paths,
                output_dir=args.output_dir
            )
            
            print(f"\n Successfully signed {len(results)} SBOM(s)")
            return 0
            
        elif args.command == "verify":
            valid = verify_sbom_signature(
                args.sbom_path,
                signature_path=args.signature,
                bundle_path=args.bundle,
                certificate_identity=args.cert_identity,
                certificate_oidc_issuer=args.cert_oidc_issuer,
                cosign_path=args.cosign_path
            )
            
            return 0 if valid else 1
            
    except FileNotFoundError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    except RuntimeError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 2
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
