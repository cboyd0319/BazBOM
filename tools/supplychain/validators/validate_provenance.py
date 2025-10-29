#!/usr/bin/env python3
"""
Validator for SLSA Provenance attestations.

Validates SLSA provenance files against the SLSA v1.0 schema and performs
additional semantic checks.
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional

try:
    import jsonschema
    from jsonschema import validate, ValidationError, SchemaError
except ImportError:
    print("Error: jsonschema module not found. Install with: pip install jsonschema", file=sys.stderr)
    sys.exit(1)


class ProvenanceValidator:
    """Validates SLSA provenance attestations."""
    
    def __init__(self, schema_path: Optional[str] = None):
        """
        Initialize the validator.
        
        Args:
            schema_path: Path to SLSA provenance JSON schema file.
                        If None, uses bundled schema.
        """
        if schema_path:
            self.schema_path = Path(schema_path)
        else:
            # Default to schema in same repo
            script_dir = Path(__file__).parent
            self.schema_path = script_dir.parent / "sbom_schemas" / "slsa-provenance-v1.0-schema.json"
        
        self.schema = self._load_schema()
        self.errors: List[str] = []
        self.warnings: List[str] = []
    
    def _load_schema(self) -> Dict[str, Any]:
        """
        Load the SLSA provenance JSON schema.
        
        Returns:
            Parsed JSON schema dictionary
            
        Raises:
            FileNotFoundError: If schema file doesn't exist
            ValueError: If schema is invalid JSON
        """
        if not self.schema_path.exists():
            raise FileNotFoundError(
                f"SLSA provenance schema not found: {self.schema_path}\n"
                f"Expected schema file at: {self.schema_path.absolute()}"
            )
        
        try:
            with open(self.schema_path, 'r', encoding='utf-8') as f:
                schema = json.load(f)
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON in schema file {self.schema_path}: {e}")
        
        # Validate that the schema itself is valid
        try:
            jsonschema.Draft7Validator.check_schema(schema)
        except SchemaError as e:
            raise ValueError(f"Invalid JSON Schema: {e}")
        
        return schema
    
    def validate_file(self, provenance_path: str) -> bool:
        """
        Validate a single provenance file.
        
        Args:
            provenance_path: Path to provenance JSON file
            
        Returns:
            True if validation passes, False otherwise
        """
        self.errors = []
        self.warnings = []
        
        path = Path(provenance_path)
        if not path.exists():
            self.errors.append(f"File not found: {provenance_path}")
            return False
        
        # Load provenance document
        try:
            with open(path, 'r', encoding='utf-8') as f:
                provenance = json.load(f)
        except json.JSONDecodeError as e:
            self.errors.append(f"Invalid JSON in {provenance_path}: {e}")
            return False
        except PermissionError:
            self.errors.append(f"Permission denied reading {provenance_path}")
            return False
        except Exception as e:
            self.errors.append(f"Failed to read {provenance_path}: {e}")
            return False
        
        # Validate against schema
        if not self._validate_schema(provenance, provenance_path):
            return False
        
        # Perform semantic validation
        self._validate_semantics(provenance, provenance_path)
        
        return len(self.errors) == 0
    
    def _validate_schema(self, provenance: Dict[str, Any], file_path: str) -> bool:
        """
        Validate provenance against JSON schema.
        
        Args:
            provenance: Parsed provenance document
            file_path: Path to file (for error messages)
            
        Returns:
            True if schema validation passes, False otherwise
        """
        try:
            validate(instance=provenance, schema=self.schema)
            return True
        except ValidationError as e:
            # Extract useful error information
            error_path = " → ".join(str(p) for p in e.path) if e.path else "root"
            self.errors.append(
                f"Schema validation failed in {file_path} at {error_path}: {e.message}"
            )
            return False
        except Exception as e:
            self.errors.append(f"Schema validation error in {file_path}: {e}")
            return False
    
    def _validate_semantics(self, provenance: Dict[str, Any], file_path: str) -> None:
        """
        Perform semantic validation beyond schema checks.
        
        Args:
            provenance: Parsed provenance document
            file_path: Path to file (for error messages)
        """
        # Check predicate type
        predicate_type = provenance.get('predicateType', '')
        if predicate_type and not predicate_type.startswith('https://slsa.dev/provenance/'):
            self.warnings.append(
                f"{file_path}: Unexpected predicate type: {predicate_type}"
            )
        
        # Check subject has at least one digest
        subjects = provenance.get('subject', [])
        for i, subject in enumerate(subjects):
            if not subject.get('digest'):
                self.errors.append(
                    f"{file_path}: Subject {i} missing digest"
                )
            elif not any(subject['digest'].values()):
                self.errors.append(
                    f"{file_path}: Subject {i} has empty digest"
                )
        
        # Check builder ID is a valid URI
        predicate = provenance.get('predicate', {})
        run_details = predicate.get('runDetails', {})
        builder = run_details.get('builder', {})
        builder_id = builder.get('id', '')
        
        if builder_id:
            if not (builder_id.startswith('https://') or builder_id.startswith('http://')):
                self.warnings.append(
                    f"{file_path}: Builder ID should be a URI: {builder_id}"
                )
        
        # Check timestamps if present
        metadata = run_details.get('metadata', {})
        started_on = metadata.get('startedOn')
        finished_on = metadata.get('finishedOn')
        
        if started_on and finished_on:
            # Basic check that finished is after started (if both are ISO timestamps)
            try:
                from datetime import datetime
                start = datetime.fromisoformat(started_on.replace('Z', '+00:00'))
                finish = datetime.fromisoformat(finished_on.replace('Z', '+00:00'))
                if finish < start:
                    self.errors.append(
                        f"{file_path}: finishedOn timestamp is before startedOn"
                    )
            except (ValueError, TypeError):
                self.warnings.append(
                    f"{file_path}: Could not parse timestamps for validation"
                )
    
    def get_errors(self) -> List[str]:
        """Get list of validation errors."""
        return self.errors
    
    def get_warnings(self) -> List[str]:
        """Get list of validation warnings."""
        return self.warnings


def main():
    """Main entry point for provenance validation."""
    parser = argparse.ArgumentParser(
        description='Validate SLSA provenance attestations against schema',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog='''
Examples:
  # Validate single provenance file
  %(prog)s workspace_sbom.provenance.json
  
  # Validate multiple files
  %(prog)s app.provenance.json lib.provenance.json
  
  # Validate with custom schema
  %(prog)s --schema custom-schema.json *.provenance.json
  
  # Treat warnings as errors
  %(prog)s --strict *.provenance.json
'''
    )
    
    parser.add_argument(
        'files',
        nargs='+',
        help='Provenance JSON files to validate'
    )
    parser.add_argument(
        '--schema',
        help='Path to SLSA provenance JSON schema (optional)'
    )
    parser.add_argument(
        '--strict',
        action='store_true',
        help='Treat warnings as errors'
    )
    parser.add_argument(
        '--quiet',
        action='store_true',
        help='Only output errors, not per-file success messages'
    )
    
    args = parser.parse_args()
    
    # Initialize validator
    try:
        validator = ProvenanceValidator(schema_path=args.schema)
    except (FileNotFoundError, ValueError) as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 2
    
    # Validate each file
    all_valid = True
    total_errors = 0
    total_warnings = 0
    
    for file_path in args.files:
        valid = validator.validate_file(file_path)
        
        errors = validator.get_errors()
        warnings = validator.get_warnings()
        
        total_errors += len(errors)
        total_warnings += len(warnings)
        
        if not valid:
            all_valid = False
            print(f" INVALID: {file_path}", file=sys.stderr)
            for error in errors:
                print(f"  ERROR: {error}", file=sys.stderr)
        elif warnings:
            if args.strict:
                all_valid = False
                print(f"[WARNING]  WARNINGS: {file_path}", file=sys.stderr)
            else:
                print(f"[WARNING]  VALID (with warnings): {file_path}")
            for warning in warnings:
                print(f"  WARNING: {warning}", file=sys.stderr if args.strict else sys.stdout)
        elif not args.quiet:
            print(f"[OK] VALID: {file_path}")
    
    # Print summary
    if len(args.files) > 1:
        print(f"\n{'='*60}")
        print(f"Validated {len(args.files)} file(s)")
        print(f"Valid: {len(args.files) - (total_errors if total_errors > 0 else 0)}")
        print(f"Errors: {total_errors}")
        print(f"Warnings: {total_warnings}")
        print(f"{'='*60}")
    
    # Exit with appropriate code
    if not all_valid:
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
