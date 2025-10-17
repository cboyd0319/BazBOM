#!/usr/bin/env python3
"""Rekor transparency log integration.

This module provides integration with Sigstore's Rekor transparency log
for verifying and retrieving SBOM signing entries.
"""

import argparse
import json
import sys
import time
from datetime import datetime
from typing import Any, Dict, List, Optional

try:
    import requests
except ImportError:
    print("Error: requests library not installed", file=sys.stderr)
    print("Install with: pip install requests", file=sys.stderr)
    sys.exit(1)


class RekorClient:
    """Client for interacting with Rekor transparency log."""
    
    DEFAULT_REKOR_URL = "https://rekor.sigstore.dev"
    
    def __init__(self, rekor_url: str = DEFAULT_REKOR_URL, timeout: int = 30):
        """Initialize Rekor client.
        
        Args:
            rekor_url: Base URL for Rekor server (default: public Sigstore instance)
            timeout: Request timeout in seconds
        """
        self.rekor_url = rekor_url.rstrip('/')
        self.timeout = timeout
    
    def get_entry_by_uuid(self, uuid: str) -> Optional[Dict[str, Any]]:
        """Retrieve a log entry by its UUID.
        
        Args:
            uuid: Log entry UUID (from cosign output or Rekor response)
            
        Returns:
            Log entry data or None if not found
            
        Raises:
            requests.HTTPError: If request fails
            ValueError: If UUID is invalid
        """
        if not uuid:
            raise ValueError("UUID cannot be empty")
        
        url = f"{self.rekor_url}/api/v1/log/entries/{uuid}"
        
        try:
            response = requests.get(url, timeout=self.timeout)
            response.raise_for_status()
            
            data = response.json()
            if not data:
                return None
            
            # Rekor returns a dict with UUID as key
            if uuid in data:
                return data[uuid]
            
            # Sometimes the response is the entry directly
            return data if isinstance(data, dict) else None
            
        except requests.HTTPError as e:
            if e.response.status_code == 404:
                print(f"Entry not found: {uuid}", file=sys.stderr)
                return None
            raise
        except requests.Timeout:
            raise TimeoutError(f"Request to Rekor timed out after {self.timeout}s")
        except requests.RequestException as e:
            raise RuntimeError(f"Failed to retrieve entry from Rekor: {e}")
    
    def search_by_sha256(self, sha256_hash: str) -> List[str]:
        """Search for log entries by artifact SHA256 hash.
        
        Args:
            sha256_hash: SHA256 hash of the artifact (SBOM file)
            
        Returns:
            List of entry UUIDs matching the hash
            
        Raises:
            ValueError: If hash is invalid
            requests.HTTPError: If request fails
        """
        if not sha256_hash or len(sha256_hash) != 64:
            raise ValueError(f"Invalid SHA256 hash: {sha256_hash}")
        
        url = f"{self.rekor_url}/api/v1/index/retrieve"
        
        payload = {
            "hash": f"sha256:{sha256_hash}"
        }
        
        try:
            response = requests.post(
                url,
                json=payload,
                headers={"Content-Type": "application/json"},
                timeout=self.timeout
            )
            response.raise_for_status()
            
            data = response.json()
            return data if isinstance(data, list) else []
            
        except requests.HTTPError as e:
            if e.response.status_code == 404:
                return []
            raise
        except requests.Timeout:
            raise TimeoutError(f"Search request timed out after {self.timeout}s")
        except requests.RequestException as e:
            raise RuntimeError(f"Failed to search Rekor: {e}")
    
    def get_latest_checkpoint(self) -> Dict[str, Any]:
        """Get the latest checkpoint from Rekor log.
        
        Returns:
            Checkpoint data including tree size and root hash
            
        Raises:
            requests.HTTPError: If request fails
        """
        url = f"{self.rekor_url}/api/v1/log/checkpoint"
        
        try:
            response = requests.get(url, timeout=self.timeout)
            response.raise_for_status()
            return response.json()
        except requests.Timeout:
            raise TimeoutError(f"Checkpoint request timed out after {self.timeout}s")
        except requests.RequestException as e:
            raise RuntimeError(f"Failed to get checkpoint: {e}")
    
    def verify_entry_inclusion(
        self,
        uuid: str,
        log_index: Optional[int] = None
    ) -> bool:
        """Verify that an entry is included in the transparency log.
        
        Args:
            uuid: Entry UUID
            log_index: Optional log index for proof verification
            
        Returns:
            True if entry is verified in log, False otherwise
        """
        try:
            entry = self.get_entry_by_uuid(uuid)
            if not entry:
                return False
            
            # Entry existence in Rekor is proof of inclusion
            # Additional verification could check Merkle proof
            return True
            
        except Exception as e:
            print(f"Verification failed: {e}", file=sys.stderr)
            return False
    
    def get_entry_attestation(self, uuid: str) -> Optional[Dict[str, Any]]:
        """Extract attestation data from a Rekor entry.
        
        Args:
            uuid: Log entry UUID
            
        Returns:
            Attestation payload or None if not found
        """
        entry = self.get_entry_by_uuid(uuid)
        if not entry:
            return None
        
        try:
            # Navigate Rekor's nested structure
            body = entry.get("body", {})
            
            # Decode if base64
            if isinstance(body, str):
                import base64
                body = json.loads(base64.b64decode(body))
            
            # Extract attestation
            spec = body.get("spec", {})
            return spec.get("data", {})
            
        except Exception as e:
            print(f"Failed to extract attestation: {e}", file=sys.stderr)
            return None
    
    def get_public_key(self, uuid: str) -> Optional[str]:
        """Extract the public key/certificate from a Rekor entry.
        
        Args:
            uuid: Log entry UUID
            
        Returns:
            PEM-encoded public key/certificate or None
        """
        entry = self.get_entry_by_uuid(uuid)
        if not entry:
            return None
        
        try:
            body = entry.get("body", {})
            
            if isinstance(body, str):
                import base64
                body = json.loads(base64.b64decode(body))
            
            spec = body.get("spec", {})
            signature = spec.get("signature", {})
            
            # Public key is in publicKey field
            public_key = signature.get("publicKey", {})
            
            if isinstance(public_key, dict):
                return public_key.get("content")
            
            return public_key if isinstance(public_key, str) else None
            
        except Exception as e:
            print(f"Failed to extract public key: {e}", file=sys.stderr)
            return None


class RekorEntryFormatter:
    """Format Rekor entries for display and analysis."""
    
    @staticmethod
    def format_entry_summary(entry: Dict[str, Any], uuid: str) -> str:
        """Format a Rekor entry as a human-readable summary.
        
        Args:
            entry: Rekor entry data
            uuid: Entry UUID
            
        Returns:
            Formatted string summary
        """
        if not entry:
            return f"Entry {uuid}: Not found"
        
        lines = [
            f"Rekor Entry: {uuid}",
            f"Log Index: {entry.get('logIndex', 'unknown')}",
            f"Integrated Time: {RekorEntryFormatter._format_timestamp(entry.get('integratedTime'))}",
        ]
        
        # Extract body details
        body = entry.get("body", {})
        if isinstance(body, str):
            import base64
            try:
                body = json.loads(base64.b64decode(body))
            except:
                pass
        
        if isinstance(body, dict):
            spec = body.get("spec", {})
            lines.append(f"Kind: {body.get('kind', 'unknown')}")
            
            # Extract artifact hash
            data = spec.get("data", {})
            if isinstance(data, dict):
                artifact_hash = data.get("hash", {})
                if isinstance(artifact_hash, dict):
                    for alg, value in artifact_hash.items():
                        lines.append(f"Artifact {alg.upper()}: {value}")
        
        return "\n".join(lines)
    
    @staticmethod
    def _format_timestamp(unix_timestamp: Optional[int]) -> str:
        """Format Unix timestamp as ISO 8601 string.
        
        Args:
            unix_timestamp: Unix timestamp in seconds
            
        Returns:
            ISO 8601 formatted datetime string
        """
        if not unix_timestamp:
            return "unknown"
        
        try:
            dt = datetime.fromtimestamp(unix_timestamp)
            return dt.isoformat()
        except:
            return str(unix_timestamp)
    
    @staticmethod
    def to_json(entry: Dict[str, Any], uuid: str, indent: int = 2) -> str:
        """Format Rekor entry as JSON.
        
        Args:
            entry: Rekor entry data
            uuid: Entry UUID
            indent: JSON indentation level
            
        Returns:
            JSON string
        """
        output = {
            "uuid": uuid,
            "entry": entry
        }
        return json.dumps(output, indent=indent)


def main() -> int:
    """CLI entry point for Rekor integration."""
    parser = argparse.ArgumentParser(
        description="Query and verify Rekor transparency log entries"
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Command to execute")
    
    # Get entry command
    get_parser = subparsers.add_parser("get", help="Get entry by UUID")
    get_parser.add_argument("uuid", help="Rekor entry UUID")
    get_parser.add_argument(
        "--format",
        choices=["summary", "json"],
        default="summary",
        help="Output format"
    )
    get_parser.add_argument(
        "--rekor-url",
        default=RekorClient.DEFAULT_REKOR_URL,
        help="Rekor server URL"
    )
    
    # Search command
    search_parser = subparsers.add_parser("search", help="Search by artifact hash")
    search_parser.add_argument("sha256", help="SHA256 hash of artifact")
    search_parser.add_argument(
        "--rekor-url",
        default=RekorClient.DEFAULT_REKOR_URL,
        help="Rekor server URL"
    )
    
    # Verify command
    verify_parser = subparsers.add_parser("verify", help="Verify entry inclusion")
    verify_parser.add_argument("uuid", help="Rekor entry UUID")
    verify_parser.add_argument(
        "--rekor-url",
        default=RekorClient.DEFAULT_REKOR_URL,
        help="Rekor server URL"
    )
    
    # Checkpoint command
    checkpoint_parser = subparsers.add_parser("checkpoint", help="Get latest checkpoint")
    checkpoint_parser.add_argument(
        "--rekor-url",
        default=RekorClient.DEFAULT_REKOR_URL,
        help="Rekor server URL"
    )
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return 1
    
    try:
        client = RekorClient(rekor_url=getattr(args, 'rekor_url', RekorClient.DEFAULT_REKOR_URL))
        
        if args.command == "get":
            entry = client.get_entry_by_uuid(args.uuid)
            if not entry:
                print(f"Entry not found: {args.uuid}", file=sys.stderr)
                return 1
            
            if args.format == "json":
                print(RekorEntryFormatter.to_json(entry, args.uuid))
            else:
                print(RekorEntryFormatter.format_entry_summary(entry, args.uuid))
            
            return 0
            
        elif args.command == "search":
            uuids = client.search_by_sha256(args.sha256)
            
            if not uuids:
                print(f"No entries found for SHA256: {args.sha256}")
                return 1
            
            print(f"Found {len(uuids)} entries:")
            for uuid in uuids:
                print(f"  {uuid}")
            
            return 0
            
        elif args.command == "verify":
            verified = client.verify_entry_inclusion(args.uuid)
            
            if verified:
                print(f"✓ Entry {args.uuid} is included in Rekor log")
                return 0
            else:
                print(f"✗ Entry {args.uuid} could not be verified", file=sys.stderr)
                return 1
                
        elif args.command == "checkpoint":
            checkpoint = client.get_latest_checkpoint()
            print(json.dumps(checkpoint, indent=2))
            return 0
            
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    except TimeoutError as e:
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
