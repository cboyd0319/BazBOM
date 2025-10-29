#!/usr/bin/env python3
"""License compliance scanning using RipGrep for fast source file analysis.

Scans 10,000+ Java/Kotlin/Scala source files for license headers and
generates compliance reports.
"""

import csv
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Set, Optional


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


def scan_license_headers(workspace_path: str) -> Dict[str, List[str]]:
    """
    Use ripgrep to find license headers in source files.
    
    Args:
        workspace_path: Path to workspace root
        
    Returns:
        Dictionary mapping license_type to list of file paths
        
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
    
    licenses = defaultdict(list)
    
    # Common license patterns
    patterns = {
        'Apache-2.0': r'Licensed under the Apache License, Version 2\.0',
        'MIT': r'Permission is hereby granted, free of charge',
        'GPL-3.0': r'GNU General Public License.*version 3',
        'GPL-2.0': r'GNU General Public License.*version 2',
        'BSD-3-Clause': r'Redistribution and use in source and binary forms',
        'BSD-2-Clause': r'Redistribution and use in source and binary forms.*2-clause',
        'Proprietary': r'Copyright.*All rights reserved',
        'LGPL-3.0': r'GNU Lesser General Public License.*version 3',
        'LGPL-2.1': r'GNU Lesser General Public License.*version 2\.1',
    }
    
    for license_name, pattern in patterns.items():
        try:
            result = subprocess.run([
                'rg',
                '--type', 'java',
                '--type', 'kotlin',
                '--type', 'scala',
                '--files-with-matches',
                '--ignore-case',
                '--max-count', '1',
                pattern,
                str(workspace)
            ], capture_output=True, text=True, timeout=30)
            
            if result.stdout:
                file_list = [f for f in result.stdout.strip().split('\n') if f]
                licenses[license_name].extend(file_list)
        except subprocess.TimeoutExpired:
            raise RuntimeError(
                f"RipGrep search timed out for license pattern: {license_name}"
            )
        except Exception as e:
            raise RuntimeError(
                f"Failed to search for {license_name} license: {str(e)}"
            )
    
    return dict(licenses)


def find_unlicensed_files(workspace_path: str) -> List[str]:
    """
    Find source files WITHOUT any license header.
    
    Args:
        workspace_path: Path to workspace root
        
    Returns:
        List of file paths without license headers
        
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
        # Find all source files
        all_files_result = subprocess.run([
            'rg', '--files',
            '--type', 'java',
            '--type', 'kotlin',
            '--type', 'scala',
            str(workspace)
        ], capture_output=True, text=True, timeout=30)
        
        all_files = set(all_files_result.stdout.strip().split('\n')) if all_files_result.stdout else set()
        
        # Find files with ANY license pattern
        licensed_files_result = subprocess.run([
            'rg',
            '--type', 'java',
            '--type', 'kotlin',
            '--type', 'scala',
            '--files-with-matches',
            '--ignore-case',
            r'(Copyright|License|SPDX-License-Identifier)',
            str(workspace)
        ], capture_output=True, text=True, timeout=30)
        
        licensed_files = set(licensed_files_result.stdout.strip().split('\n')) if licensed_files_result.stdout else set()
        
        # Return difference
        unlicensed = all_files - licensed_files
        return sorted([f for f in unlicensed if f])
    
    except subprocess.TimeoutExpired:
        raise RuntimeError("RipGrep search timed out while finding unlicensed files")
    except Exception as e:
        raise RuntimeError(f"Failed to find unlicensed files: {str(e)}")


def generate_license_report(workspace_path: str, output_path: str) -> None:
    """
    Generate CSV report of license compliance.
    
    Args:
        workspace_path: Path to workspace root
        output_path: Path to output CSV file
        
    Raises:
        RuntimeError: If RipGrep is not available or scanning fails
        ValueError: If workspace_path is invalid
    """
    licenses = scan_license_headers(workspace_path)
    unlicensed = find_unlicensed_files(workspace_path)
    
    # Define compliant licenses (permissive)
    compliant_licenses = {'Apache-2.0', 'MIT', 'BSD-3-Clause', 'BSD-2-Clause'}
    
    try:
        with open(output_path, 'w', newline='', encoding='utf-8') as f:
            writer = csv.writer(f)
            writer.writerow(['File', 'License', 'Compliance Status'])
            
            for license_type, files in licenses.items():
                for file_path in files:
                    status = 'COMPLIANT' if license_type in compliant_licenses else 'REVIEW_REQUIRED'
                    writer.writerow([file_path, license_type, status])
            
            for file_path in unlicensed:
                writer.writerow([file_path, 'MISSING', 'NON_COMPLIANT'])
        
    except IOError as e:
        raise RuntimeError(f"Failed to write license report to {output_path}: {str(e)}")


def check_copyleft_licenses(workspace_path: str) -> Dict[str, List[str]]:
    """
    Find files with copyleft licenses (GPL, LGPL).
    
    Args:
        workspace_path: Path to workspace root
        
    Returns:
        Dictionary mapping copyleft license types to file paths
        
    Raises:
        RuntimeError: If RipGrep is not available
        ValueError: If workspace_path is invalid
    """
    all_licenses = scan_license_headers(workspace_path)
    
    copyleft_licenses = {
        'GPL-3.0', 'GPL-2.0', 'LGPL-3.0', 'LGPL-2.1'
    }
    
    copyleft_files = {}
    for license_type, files in all_licenses.items():
        if license_type in copyleft_licenses:
            copyleft_files[license_type] = files
    
    return copyleft_files


def main():
    """CLI entry point for license scanner."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='License compliance scanning using RipGrep'
    )
    parser.add_argument(
        'workspace',
        help='Path to workspace root'
    )
    parser.add_argument(
        '--output',
        help='Output CSV file for license report'
    )
    parser.add_argument(
        '--format',
        choices=['csv', 'json'],
        default='csv',
        help='Output format (default: csv)'
    )
    parser.add_argument(
        '--check-copyleft',
        action='store_true',
        help='Check for copyleft licenses only'
    )
    parser.add_argument(
        '--find-unlicensed',
        action='store_true',
        help='Find files without license headers'
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
    
    try:
        if args.check_copyleft:
            # Check for copyleft licenses
            copyleft = check_copyleft_licenses(args.workspace)
            
            if copyleft:
                print(f"Found {sum(len(files) for files in copyleft.values())} files with copyleft licenses:", file=sys.stderr)
                for license_type, files in copyleft.items():
                    print(f"  {license_type}: {len(files)} files", file=sys.stderr)
                
                result = {
                    'copyleft_found': True,
                    'licenses': copyleft,
                    'total_files': sum(len(files) for files in copyleft.values())
                }
                
                if args.output:
                    with open(args.output, 'w', encoding='utf-8') as f:
                        json.dump(result, f, indent=2)
                else:
                    print(json.dumps(result, indent=2))
                
                return 1  # Exit with error if copyleft found
            else:
                print("No copyleft licenses found", file=sys.stderr)
                return 0
        
        elif args.find_unlicensed:
            # Find unlicensed files
            unlicensed = find_unlicensed_files(args.workspace)
            
            print(f"Found {len(unlicensed)} files without license headers", file=sys.stderr)
            
            result = {
                'unlicensed_count': len(unlicensed),
                'unlicensed_files': unlicensed
            }
            
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(result, f, indent=2)
            else:
                print(json.dumps(result, indent=2))
            
            return 0
        
        else:
            # Full license report
            licenses = scan_license_headers(args.workspace)
            unlicensed = find_unlicensed_files(args.workspace)
            
            total_files = sum(len(files) for files in licenses.values()) + len(unlicensed)
            print(f"Scanned {total_files} source files", file=sys.stderr)
            print(f"  Licensed: {sum(len(files) for files in licenses.values())}", file=sys.stderr)
            print(f"  Unlicensed: {len(unlicensed)}", file=sys.stderr)
            
            if args.output:
                if args.format == 'csv':
                    generate_license_report(args.workspace, args.output)
                    print(f"License report written to {args.output}", file=sys.stderr)
                else:  # json
                    result = {
                        'total_files': total_files,
                        'licenses': licenses,
                        'unlicensed': unlicensed
                    }
                    with open(args.output, 'w', encoding='utf-8') as f:
                        json.dump(result, f, indent=2)
            else:
                result = {
                    'total_files': total_files,
                    'licenses': {k: len(v) for k, v in licenses.items()},
                    'unlicensed_count': len(unlicensed)
                }
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
