#!/usr/bin/env python3
"""in-toto attestation generation for SBOMs.

This module generates in-toto attestation bundles in SLSA Provenance v1.0
format for SBOMs, combining provenance data with signing artifacts.
"""

import argparse
import hashlib
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional


class InTotoAttestationGenerator:
    """Generate in-toto attestation bundles for SBOMs."""
    
    # in-toto statement type URI
    STATEMENT_TYPE = "https://in-toto.io/Statement/v1"
    
    # SLSA Provenance predicate type
    SLSA_PROVENANCE_TYPE = "https://slsa.dev/provenance/v1"
    
    def __init__(self):
        """Initialize the attestation generator."""
        pass
    
    def generate_sbom_attestation(
        self,
        sbom_path: str,
        predicate_type: str = SLSA_PROVENANCE_TYPE,
        builder_id: Optional[str] = None,
        build_invocation_id: Optional[str] = None,
        build_config: Optional[Dict[str, Any]] = None,
        materials: Optional[List[Dict[str, Any]]] = None
    ) -> Dict[str, Any]:
        """Generate an in-toto attestation for an SBOM.
        
        Args:
            sbom_path: Path to SBOM file
            predicate_type: Predicate type URI (default: SLSA Provenance v1)
            builder_id: Builder identity (e.g., GitHub Actions workflow)
            build_invocation_id: Build invocation ID (e.g., GitHub run ID)
            build_config: Build configuration parameters
            materials: Build materials (source commits, dependencies)
            
        Returns:
            in-toto attestation bundle (dict)
            
        Raises:
            FileNotFoundError: If SBOM file doesn't exist
            ValueError: If SBOM is invalid
        """
        if not os.path.exists(sbom_path):
            raise FileNotFoundError(f"SBOM file not found: {sbom_path}")
        
        # Calculate SBOM digest
        sbom_digest = self._calculate_sha256(sbom_path)
        sbom_name = Path(sbom_path).name
        
        # Build subject
        subject = [{
            "name": sbom_name,
            "digest": {
                "sha256": sbom_digest
            }
        }]
        
        # Build predicate
        predicate = self._build_predicate(
            builder_id=builder_id,
            build_invocation_id=build_invocation_id,
            build_config=build_config or {},
            materials=materials or []
        )
        
        # Assemble attestation
        attestation = {
            "_type": self.STATEMENT_TYPE,
            "subject": subject,
            "predicateType": predicate_type,
            "predicate": predicate
        }
        
        return attestation
    
    def _calculate_sha256(self, file_path: str) -> str:
        """Calculate SHA256 hash of a file.
        
        Args:
            file_path: Path to file
            
        Returns:
            SHA256 hash as hex string
        """
        sha256 = hashlib.sha256()
        
        with open(file_path, 'rb') as f:
            while True:
                chunk = f.read(8192)
                if not chunk:
                    break
                sha256.update(chunk)
        
        return sha256.hexdigest()
    
    def _build_predicate(
        self,
        builder_id: Optional[str],
        build_invocation_id: Optional[str],
        build_config: Dict[str, Any],
        materials: List[Dict[str, Any]]
    ) -> Dict[str, Any]:
        """Build SLSA provenance predicate.
        
        Args:
            builder_id: Builder identity
            build_invocation_id: Build invocation ID
            build_config: Build configuration
            materials: Build materials
            
        Returns:
            SLSA provenance predicate
        """
        # Get default values from environment
        if not builder_id:
            builder_id = os.environ.get(
                "GITHUB_REPOSITORY",
                os.environ.get("BUILDER_ID", "unknown-builder")
            )
        
        if not build_invocation_id:
            build_invocation_id = os.environ.get(
                "GITHUB_RUN_ID",
                os.environ.get("BUILD_INVOCATION_ID", "unknown")
            )
        
        # Build timestamp
        build_finished_on = datetime.now(timezone.utc).isoformat()
        
        # Construct predicate
        predicate = {
            "buildDefinition": {
                "buildType": "https://github.com/cboyd0319/BazBOM/bazel-sbom-build@v1",
                "externalParameters": {
                    "workflow": os.environ.get("GITHUB_WORKFLOW", "unknown"),
                    "repository": os.environ.get("GITHUB_REPOSITORY", "unknown"),
                    "ref": os.environ.get("GITHUB_REF", "unknown")
                },
                "internalParameters": build_config,
                "resolvedDependencies": self._format_materials(materials)
            },
            "runDetails": {
                "builder": {
                    "id": builder_id,
                    "version": {}
                },
                "metadata": {
                    "invocationId": build_invocation_id,
                    "startedOn": os.environ.get("BUILD_START_TIME", build_finished_on),
                    "finishedOn": build_finished_on
                },
                "byproducts": []
            }
        }
        
        return predicate
    
    def _format_materials(self, materials: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Format materials into SLSA resolved dependencies format.
        
        Args:
            materials: List of material dictionaries
            
        Returns:
            Formatted resolved dependencies
        """
        formatted = []
        
        for material in materials:
            if isinstance(material, dict):
                # Convert to SLSA format
                formatted_material = {
                    "uri": material.get("uri", ""),
                    "digest": material.get("digest", {})
                }
                
                # Add optional fields
                if "name" in material:
                    formatted_material["name"] = material["name"]
                
                formatted.append(formatted_material)
        
        return formatted
    
    def create_attestation_bundle(
        self,
        sbom_path: str,
        signature: str,
        certificate: str,
        rekor_entry_url: Optional[str] = None,
        output_path: Optional[str] = None
    ) -> Dict[str, Any]:
        """Create a complete attestation bundle with signature and certificate.
        
        Args:
            sbom_path: Path to SBOM file
            signature: Base64-encoded signature
            certificate: PEM-encoded signing certificate
            rekor_entry_url: Rekor transparency log entry URL (optional)
            output_path: Path to write bundle JSON (optional)
            
        Returns:
            Attestation bundle dictionary
            
        Raises:
            FileNotFoundError: If SBOM doesn't exist
            ValueError: If signature or certificate is empty
        """
        if not signature:
            raise ValueError("Signature cannot be empty")
        if not certificate:
            raise ValueError("Certificate cannot be empty")
        
        # Generate base attestation
        attestation = self.generate_sbom_attestation(sbom_path)
        
        # Add signature envelope
        bundle = {
            "attestation": attestation,
            "signature": {
                "keyid": "",  # Keyless signing
                "sig": signature
            },
            "signing_cert": certificate,
            "rekor_entry": rekor_entry_url or "",
            "bundle_version": "1.0",
            "created_at": datetime.now(timezone.utc).isoformat()
        }
        
        # Write to file if requested
        if output_path:
            with open(output_path, 'w') as f:
                json.dump(bundle, f, indent=2)
            print(f"Attestation bundle written to: {output_path}")
        
        return bundle
    
    def verify_attestation_structure(self, attestation: Dict[str, Any]) -> bool:
        """Verify that an attestation has valid structure.
        
        Args:
            attestation: Attestation dictionary
            
        Returns:
            True if structure is valid, False otherwise
        """
        try:
            # Check required fields
            if attestation.get("_type") != self.STATEMENT_TYPE:
                print("Invalid _type field", file=sys.stderr)
                return False
            
            if "subject" not in attestation:
                print("Missing subject field", file=sys.stderr)
                return False
            
            if "predicateType" not in attestation:
                print("Missing predicateType field", file=sys.stderr)
                return False
            
            if "predicate" not in attestation:
                print("Missing predicate field", file=sys.stderr)
                return False
            
            # Validate subject structure
            for subj in attestation["subject"]:
                if "name" not in subj or "digest" not in subj:
                    print("Invalid subject structure", file=sys.stderr)
                    return False
            
            return True
            
        except Exception as e:
            print(f"Validation error: {e}", file=sys.stderr)
            return False


def main() -> int:
    """CLI entry point for in-toto attestation generation."""
    parser = argparse.ArgumentParser(
        description="Generate in-toto attestations for SBOMs"
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Command to execute")
    
    # Generate command
    gen_parser = subparsers.add_parser("generate", help="Generate attestation")
    gen_parser.add_argument(
        "sbom_path",
        help="Path to SBOM file"
    )
    gen_parser.add_argument(
        "-o", "--output",
        required=True,
        help="Output path for attestation JSON"
    )
    gen_parser.add_argument(
        "--builder-id",
        help="Builder identity (default: from GITHUB_REPOSITORY)"
    )
    gen_parser.add_argument(
        "--build-id",
        help="Build invocation ID (default: from GITHUB_RUN_ID)"
    )
    gen_parser.add_argument(
        "--build-config",
        help="Build configuration as JSON"
    )
    gen_parser.add_argument(
        "--materials",
        help="Build materials as JSON array"
    )
    
    # Bundle command
    bundle_parser = subparsers.add_parser("bundle", help="Create attestation bundle")
    bundle_parser.add_argument(
        "sbom_path",
        help="Path to SBOM file"
    )
    bundle_parser.add_argument(
        "--signature",
        required=True,
        help="Base64-encoded signature"
    )
    bundle_parser.add_argument(
        "--certificate",
        required=True,
        help="Path to PEM certificate file"
    )
    bundle_parser.add_argument(
        "--rekor-entry",
        help="Rekor entry URL"
    )
    bundle_parser.add_argument(
        "-o", "--output",
        required=True,
        help="Output path for bundle JSON"
    )
    
    # Verify command
    verify_parser = subparsers.add_parser("verify", help="Verify attestation structure")
    verify_parser.add_argument(
        "attestation_path",
        help="Path to attestation JSON file"
    )
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return 1
    
    try:
        generator = InTotoAttestationGenerator()
        
        if args.command == "generate":
            # Parse optional JSON arguments
            build_config = json.loads(args.build_config) if args.build_config else {}
            materials = json.loads(args.materials) if args.materials else []
            
            attestation = generator.generate_sbom_attestation(
                args.sbom_path,
                builder_id=args.builder_id,
                build_invocation_id=args.build_id,
                build_config=build_config,
                materials=materials
            )
            
            with open(args.output, 'w') as f:
                json.dump(attestation, f, indent=2)
            
            print(f"✓ Attestation generated: {args.output}")
            return 0
            
        elif args.command == "bundle":
            # Read certificate file
            with open(args.certificate, 'r') as f:
                certificate = f.read()
            
            bundle = generator.create_attestation_bundle(
                args.sbom_path,
                args.signature,
                certificate,
                rekor_entry_url=args.rekor_entry,
                output_path=args.output
            )
            
            print(f"✓ Attestation bundle created: {args.output}")
            return 0
            
        elif args.command == "verify":
            with open(args.attestation_path, 'r') as f:
                attestation = json.load(f)
            
            valid = generator.verify_attestation_structure(attestation)
            
            if valid:
                print(f"✓ Attestation structure is valid")
                return 0
            else:
                print(f"✗ Attestation structure is invalid", file=sys.stderr)
                return 1
                
    except FileNotFoundError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    except json.JSONDecodeError as e:
        print(f"ERROR: Invalid JSON: {e}", file=sys.stderr)
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
