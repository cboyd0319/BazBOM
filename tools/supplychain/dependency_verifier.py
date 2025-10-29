#!/usr/bin/env python3
"""Dependency verification using RipGrep to detect unused dependencies.

Verifies that dependencies in maven_install.json are actually referenced
in code, helping identify bloat and reduce attack surface.
"""

import json
import subprocess
import sys
from pathlib import Path
from typing import Set, List, Dict, Optional


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


def get_declared_dependencies(maven_install_json: str) -> Set[str]:
    """
    Parse maven_install.json for all declared dependencies.
    
    Args:
        maven_install_json: Path to maven_install.json file
        
    Returns:
        Set of dependency labels in Bazel format
        
    Raises:
        ValueError: If file doesn't exist or is invalid
    """
    json_path = Path(maven_install_json)
    if not json_path.exists():
        raise ValueError(f"maven_install.json not found: {maven_install_json}")
    
    try:
        with open(maven_install_json, 'r', encoding='utf-8') as f:
            data = json.load(f)
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON in {maven_install_json}: {str(e)}")
    except IOError as e:
        raise ValueError(f"Failed to read {maven_install_json}: {str(e)}")
    
    deps = set()
    
    # Handle dependency_tree structure
    if 'dependency_tree' in data and 'dependencies' in data['dependency_tree']:
        for artifact in data['dependency_tree']['dependencies']:
            coord = artifact.get('coord', '')
            if coord:
                # Convert Maven coordinates to Bazel label format
                # e.g., "com.google.guava:guava:31.1-jre" -> "com_google_guava_guava"
                parts = coord.split(':')
                if len(parts) >= 2:
                    group = parts[0].replace('.', '_').replace('-', '_')
                    artifact_id = parts[1].replace('.', '_').replace('-', '_')
                    label = f"{group}_{artifact_id}"
                    deps.add(label)
    
    # Also check artifacts at top level (alternative format)
    if 'artifacts' in data:
        for coord, _ in data['artifacts'].items():
            parts = coord.split(':')
            if len(parts) >= 2:
                group = parts[0].replace('.', '_').replace('-', '_')
                artifact_id = parts[1].replace('.', '_').replace('-', '_')
                label = f"{group}_{artifact_id}"
                deps.add(label)
    
    return deps


def get_referenced_dependencies(workspace_path: str) -> Set[str]:
    """
    Use ripgrep to find all @maven// references in BUILD files.
    
    Args:
        workspace_path: Path to workspace root
        
    Returns:
        Set of dependency labels referenced in BUILD files
        
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
            '--glob', 'BUILD.bazel',
            '--glob', 'BUILD',
            '--no-filename',
            '--no-heading',
            r'@maven//:[a-zA-Z0-9_.-]+',
            '--only-matching',
            str(workspace)
        ], capture_output=True, text=True, timeout=60)
    except subprocess.TimeoutExpired:
        raise RuntimeError(f"RipGrep search timed out after 60 seconds in {workspace_path}")
    except Exception as e:
        raise RuntimeError(f"Failed to run RipGrep: {str(e)}")
    
    references = set()
    for match in result.stdout.strip().split('\n'):
        if match:
            # Extract label name: @maven//:com_google_guava_guava
            label = match.replace('@maven//:', '')
            references.add(label)
    
    return references


def find_unused_dependencies(workspace_path: str, maven_install_json: str) -> List[str]:
    """
    Find dependencies declared but never referenced in BUILD files.
    
    Args:
        workspace_path: Path to workspace root
        maven_install_json: Path to maven_install.json file
        
    Returns:
        Sorted list of unused dependency labels
        
    Raises:
        RuntimeError: If RipGrep is not available
        ValueError: If paths are invalid
    """
    declared = get_declared_dependencies(maven_install_json)
    referenced = get_referenced_dependencies(workspace_path)
    
    unused = declared - referenced
    return sorted(list(unused))


def find_undeclared_dependencies(workspace_path: str, maven_install_json: str) -> List[str]:
    """
    Find dependencies referenced but not declared in maven_install.json.
    
    Args:
        workspace_path: Path to workspace root
        maven_install_json: Path to maven_install.json file
        
    Returns:
        Sorted list of undeclared dependency labels
        
    Raises:
        RuntimeError: If RipGrep is not available
        ValueError: If paths are invalid
    """
    declared = get_declared_dependencies(maven_install_json)
    referenced = get_referenced_dependencies(workspace_path)
    
    undeclared = referenced - declared
    return sorted(list(undeclared))


def generate_usage_report(workspace_path: str, maven_install_json: str) -> Dict:
    """
    Generate comprehensive report of dependency usage.
    
    Args:
        workspace_path: Path to workspace root
        maven_install_json: Path to maven_install.json file
        
    Returns:
        Dictionary containing usage statistics
        
    Raises:
        RuntimeError: If RipGrep is not available
        ValueError: If paths are invalid
    """
    declared = get_declared_dependencies(maven_install_json)
    referenced = get_referenced_dependencies(workspace_path)
    unused = declared - referenced
    undeclared = referenced - declared
    
    usage_rate = (100 * len(referenced & declared) / len(declared)) if declared else 0
    
    return {
        'declared_count': len(declared),
        'referenced_count': len(referenced),
        'used_count': len(referenced & declared),
        'unused_count': len(unused),
        'undeclared_count': len(undeclared),
        'usage_rate': round(usage_rate, 1),
        'unused_dependencies': sorted(list(unused)),
        'undeclared_dependencies': sorted(list(undeclared))
    }


def main():
    """CLI entry point for dependency verifier."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Dependency verification using RipGrep'
    )
    parser.add_argument(
        'workspace',
        nargs='?',
        help='Path to workspace root'
    )
    parser.add_argument(
        '--maven-install-json',
        default='maven_install.json',
        help='Path to maven_install.json (default: maven_install.json)'
    )
    parser.add_argument(
        '--output',
        help='Output JSON file for report'
    )
    parser.add_argument(
        '--check-unused',
        action='store_true',
        help='Check for unused dependencies only'
    )
    parser.add_argument(
        '--check-undeclared',
        action='store_true',
        help='Check for undeclared dependencies only'
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
        if args.check_unused:
            # Check for unused dependencies
            unused = find_unused_dependencies(args.workspace, args.maven_install_json)
            
            print(f"Found {len(unused)} unused dependencies", file=sys.stderr)
            
            result = {
                'unused_count': len(unused),
                'unused_dependencies': unused
            }
            
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(result, f, indent=2)
            else:
                print(json.dumps(result, indent=2))
            
            if unused:
                print("\nUnused dependencies (consider removing):", file=sys.stderr)
                for dep in unused:
                    print(f"  - {dep}", file=sys.stderr)
                return 1  # Exit with error if unused deps found
            
            return 0
        
        elif args.check_undeclared:
            # Check for undeclared dependencies
            undeclared = find_undeclared_dependencies(args.workspace, args.maven_install_json)
            
            print(f"Found {len(undeclared)} undeclared dependencies", file=sys.stderr)
            
            result = {
                'undeclared_count': len(undeclared),
                'undeclared_dependencies': undeclared
            }
            
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(result, f, indent=2)
            else:
                print(json.dumps(result, indent=2))
            
            if undeclared:
                print("\nUndeclared dependencies (missing from maven_install.json):", file=sys.stderr)
                for dep in undeclared:
                    print(f"  - {dep}", file=sys.stderr)
                return 1  # Exit with error if undeclared deps found
            
            return 0
        
        else:
            # Full usage report
            report = generate_usage_report(args.workspace, args.maven_install_json)
            
            print(f"Declared dependencies: {report['declared_count']}", file=sys.stderr)
            print(f"Referenced dependencies: {report['referenced_count']}", file=sys.stderr)
            print(f"Used dependencies: {report['used_count']}", file=sys.stderr)
            print(f"Unused dependencies: {report['unused_count']}", file=sys.stderr)
            print(f"Undeclared dependencies: {report['undeclared_count']}", file=sys.stderr)
            print(f"Dependency usage rate: {report['usage_rate']}%", file=sys.stderr)
            
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(report, f, indent=2)
                print(f"\nReport written to {args.output}", file=sys.stderr)
            else:
                print(json.dumps(report, indent=2))
            
            return 0
        
    except (RuntimeError, ValueError) as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nInterrupted by user", file=sys.stderr)
        return 130


if __name__ == '__main__':
    sys.exit(main())
