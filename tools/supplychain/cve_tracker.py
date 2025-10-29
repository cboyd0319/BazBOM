#!/usr/bin/env python3
"""CVE reference tracking using RipGrep for fast codebase scanning.

Find all CVE references in code comments, docs, or VEX statements to
track known issues and cross-reference with SBOM findings.
"""

import json
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Dict, Set, Optional


def check_ripgrep_available() -> bool:
    """Check if RipGrep is installed and available."""
    try:
        subprocess.run(
            ['rg', '--version'],
            capture_output=True,
            check=True,
            timeout=5
        )
        return True
    except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired):
        return False


def find_cve_references(workspace_path: str) -> List[Dict[str, str]]:
    """
    Search codebase for CVE references.
    Useful for tracking known issues and VEX statements.
    
    Args:
        workspace_path: Path to workspace root
        
    Returns:
        List of dictionaries containing CVE reference metadata
        
    Raises:
        RuntimeError: If RipGrep is not available
        ValueError: If workspace_path is invalid
    """
    if not check_ripgrep_available():
        raise RuntimeError(
            "RipGrep (rg) is not installed. "
            "Install from: https://github.com/BurntSushi/ripgrep#installation"
        )
    
    workspace = Path(workspace_path)
    if not workspace.exists():
        raise ValueError(f"Workspace path does not exist: {workspace_path}")
    
    if not workspace.is_dir():
        raise ValueError(f"Workspace path is not a directory: {workspace_path}")
    
    try:
        result = subprocess.run([
            'rg',
            '--type', 'java',
            '--type', 'kotlin',
            '--type', 'markdown',
            '--type', 'yaml',
            '--line-number',
            '--no-heading',
            r'CVE-\d{4}-\d{4,}',
            '--only-matching',
            str(workspace)
        ], capture_output=True, text=True, timeout=60)
    except subprocess.TimeoutExpired:
        raise RuntimeError(f"RipGrep search timed out after 60 seconds in {workspace_path}")
    except Exception as e:
        raise RuntimeError(f"Failed to run RipGrep: {str(e)}")
    
    cves = []
    for line in result.stdout.strip().split('\n'):
        if line:
            # Format: file.java:42:CVE-2023-12345
            match = re.match(r'(.+):(\d+):(CVE-\d{4}-\d{4,})', line)
            if match:
                cves.append({
                    'file': match.group(1),
                    'line': int(match.group(2)),
                    'cve': match.group(3)
                })
    
    return cves


def cross_reference_with_sbom(cves: List[Dict], sbom_findings_path: str) -> Dict[str, List[str]]:
    """
    Check if CVEs mentioned in code match SBOM scan findings.
    
    Args:
        cves: List of CVE references found in code
        sbom_findings_path: Path to SBOM findings JSON file
        
    Returns:
        Dictionary with categorized CVEs
        
    Raises:
        ValueError: If sbom_findings_path is invalid
    """
    findings_path = Path(sbom_findings_path)
    if not findings_path.exists():
        raise ValueError(f"SBOM findings file not found: {sbom_findings_path}")
    
    try:
        with open(sbom_findings_path, 'r', encoding='utf-8') as f:
            findings = json.load(f)
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON in {sbom_findings_path}: {str(e)}")
    except IOError as e:
        raise ValueError(f"Failed to read {sbom_findings_path}: {str(e)}")
    
    # Extract CVEs from findings
    found_cves = set()
    if 'vulnerabilities' in findings:
        for vuln in findings['vulnerabilities']:
            if 'cve' in vuln:
                found_cves.add(vuln['cve'])
            elif 'id' in vuln and vuln['id'].startswith('CVE-'):
                found_cves.add(vuln['id'])
    
    # Extract CVEs from code
    code_cves = {c['cve'] for c in cves}
    
    # Categorize
    in_both = code_cves & found_cves
    documented_only = code_cves - found_cves
    sbom_only = found_cves - code_cves
    
    return {
        'in_both': sorted(list(in_both)),
        'documented_only': sorted(list(documented_only)),
        'sbom_only': sorted(list(sbom_only))
    }


def find_vex_statements(workspace_path: str) -> List[Dict[str, str]]:
    """
    Find VEX statement files and extract CVE references.
    
    Args:
        workspace_path: Path to workspace root
        
    Returns:
        List of VEX statements with CVE references
        
    Raises:
        RuntimeError: If RipGrep is not available
        ValueError: If workspace_path is invalid
    """
    if not check_ripgrep_available():
        raise RuntimeError(
            "RipGrep (rg) is not installed. "
            "Install from: https://github.com/BurntSushi/ripgrep#installation"
        )
    
    workspace = Path(workspace_path)
    if not workspace.exists():
        raise ValueError(f"Workspace path does not exist: {workspace_path}")
    
    if not workspace.is_dir():
        raise ValueError(f"Workspace path is not a directory: {workspace_path}")
    
    vex_statements = []
    
    # Find VEX statement files
    try:
        vex_files_result = subprocess.run([
            'rg', '--files',
            '--glob', '*/vex/*.json',
            '--glob', '*/vex/*.yaml',
            '--glob', '*/vex/*.yml',
            str(workspace)
        ], capture_output=True, text=True, timeout=30)
    except subprocess.TimeoutExpired:
        raise RuntimeError("RipGrep search timed out while finding VEX files")
    except Exception as e:
        raise RuntimeError(f"Failed to find VEX files: {str(e)}")
    
    vex_files = [f for f in vex_files_result.stdout.strip().split('\n') if f]
    
    # Extract CVEs from each VEX file
    for vex_file in vex_files:
        try:
            cve_result = subprocess.run([
                'rg',
                r'CVE-\d{4}-\d{4,}',
                '--only-matching',
                vex_file
            ], capture_output=True, text=True, timeout=5)
            
            cves = set(cve_result.stdout.strip().split('\n')) if cve_result.stdout else set()
            cves = {c for c in cves if c}
            
            if cves:
                vex_statements.append({
                    'file': vex_file,
                    'cves': sorted(list(cves))
                })
        except subprocess.TimeoutExpired:
            continue
        except Exception:
            continue
    
    return vex_statements


def main():
    """CLI entry point for CVE tracker."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='CVE reference tracking using RipGrep'
    )
    parser.add_argument(
        'workspace',
        nargs='?',
        help='Path to workspace root'
    )
    parser.add_argument(
        '--output',
        help='Output JSON file for CVE references'
    )
    parser.add_argument(
        '--sbom-findings',
        help='Path to SBOM findings JSON for cross-reference'
    )
    parser.add_argument(
        '--find-vex',
        action='store_true',
        help='Find VEX statements with CVE references'
    )
    parser.add_argument(
        '--check',
        action='store_true',
        help='Check if RipGrep is available and exit'
    )
    
    args = parser.parse_args()
    
    # Check mode
    if args.check:
        if check_ripgrep_available():
            print("[OK] RipGrep detected - enabling fast mode", file=sys.stderr)
            return 0
        else:
            print("[WARNING]  RipGrep not found - fast scanning disabled", file=sys.stderr)
            print("   Install: https://github.com/BurntSushi/ripgrep#installation", file=sys.stderr)
            return 1
    
    if not args.workspace:
        print("ERROR: workspace path is required", file=sys.stderr)
        return 1
    
    try:
        if args.find_vex:
            # Find VEX statements
            vex_statements = find_vex_statements(args.workspace)
            
            print(f"Found {len(vex_statements)} VEX statement files", file=sys.stderr)
            
            result = {
                'vex_file_count': len(vex_statements),
                'vex_statements': vex_statements
            }
            
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(result, f, indent=2)
            else:
                print(json.dumps(result, indent=2))
        
        else:
            # Find CVE references
            cves = find_cve_references(args.workspace)
            
            print(f"Found {len(cves)} CVE references in code", file=sys.stderr)
            
            result = {
                'cve_reference_count': len(cves),
                'unique_cves': len(set(c['cve'] for c in cves)),
                'cve_references': cves
            }
            
            # Cross-reference if SBOM findings provided
            if args.sbom_findings:
                cross_ref = cross_reference_with_sbom(cves, args.sbom_findings)
                result['cross_reference'] = cross_ref
                
                print(f"\nCross-reference with SBOM:", file=sys.stderr)
                print(f"  In both code and SBOM: {len(cross_ref['in_both'])}", file=sys.stderr)
                print(f"  Documented only: {len(cross_ref['documented_only'])}", file=sys.stderr)
                print(f"  SBOM only: {len(cross_ref['sbom_only'])}", file=sys.stderr)
            
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(result, f, indent=2)
            else:
                print(json.dumps(result, indent=2))
        
        return 0
        
    except (RuntimeError, ValueError) as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nInterrupted by user", file=sys.stderr)
        return 130


if __name__ == '__main__':
    sys.exit(main())
