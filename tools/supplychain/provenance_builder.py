#!/usr/bin/env python3
"""Generate SLSA provenance attestations.

This script generates SLSA provenance v1.0 attestations for build artifacts.
"""

import argparse
import json
import sys
import os
from datetime import datetime
from typing import Any, Dict, List


def generate_slsa_provenance(
    artifact_name: str,
    artifact_digest: str = None,
    commit_sha: str = None,
    build_id: str = None,
    builder: str = None,
) -> Dict[str, Any]:
    """Generate a SLSA provenance v1.0 document.
    
    Args:
        artifact_name: Name of the artifact
        artifact_digest: SHA256 digest of the artifact
        commit_sha: Git commit SHA
        build_id: CI/CD build ID
        builder: Builder identity
        
    Returns:
        SLSA provenance document as a dictionary
    """
    timestamp = datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    
    # Default values
    if not commit_sha:
        commit_sha = os.environ.get("GITHUB_SHA", "unknown")
    if not build_id:
        build_id = os.environ.get("GITHUB_RUN_ID", "unknown")
    if not builder:
        builder = os.environ.get("GITHUB_REPOSITORY", "unknown-builder")
    
    provenance = {
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [
            {
                "name": artifact_name,
                "digest": {
                    "sha256": artifact_digest or "unknown"
                }
            }
        ],
        "predicateType": "https://slsa.dev/provenance/v1",
        "predicate": {
            "buildDefinition": {
                "buildType": "https://github.com/bazel-contrib/bazbom@v1",
                "externalParameters": {
                    "repository": builder,
                    "ref": commit_sha,
                },
                "internalParameters": {
                    "buildId": build_id,
                },
                "resolvedDependencies": []
            },
            "runDetails": {
                "builder": {
                    "id": f"https://github.com/{builder}/actions",
                    "version": {}
                },
                "metadata": {
                    "invocationId": build_id,
                    "startedOn": timestamp,
                    "finishedOn": timestamp,
                },
                "byproducts": []
            }
        }
    }
    
    return provenance


def main():
    parser = argparse.ArgumentParser(
        description="Generate SLSA provenance attestation"
    )
    parser.add_argument(
        "--artifact",
        required=True,
        help="Name of the artifact"
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output provenance JSON file"
    )
    parser.add_argument(
        "--digest",
        help="SHA256 digest of the artifact"
    )
    parser.add_argument(
        "--commit",
        help="Git commit SHA"
    )
    parser.add_argument(
        "--build-id",
        help="CI/CD build ID"
    )
    parser.add_argument(
        "--builder",
        help="Builder identity"
    )
    
    args = parser.parse_args()
    
    # Generate provenance
    provenance = generate_slsa_provenance(
        artifact_name=args.artifact,
        artifact_digest=args.digest,
        commit_sha=args.commit,
        build_id=args.build_id,
        builder=args.builder,
    )
    
    # Write output
    try:
        with open(args.output, "w") as f:
            json.dump(provenance, f, indent=2)
        print(f"Provenance written to {args.output}")
    except IOError as e:
        print(f"Error writing output file: {e}", file=sys.stderr)
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
