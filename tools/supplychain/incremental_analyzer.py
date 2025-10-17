#!/usr/bin/env python3
"""Incremental analyzer for detecting changed targets in Bazel based on git diff."""

import argparse
import json
import subprocess
import sys
from typing import List, Set, Dict, Any
from pathlib import Path


def run_git_command(args: List[str], cwd: str = None) -> str:
    """Run a git command and return output."""
    try:
        result = subprocess.run(
            ['git'] + args,
            capture_output=True,
            text=True,
            check=True,
            cwd=cwd
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        print(f"Error running git command: {e}", file=sys.stderr)
        print(f"stderr: {e.stderr}", file=sys.stderr)
        return ""


def get_changed_files(base_ref: str = "HEAD~1", cwd: str = None) -> List[str]:
    """
    Get list of changed files between current HEAD and base ref.
    
    Args:
        base_ref: Git reference to compare against (default: HEAD~1)
        cwd: Working directory
        
    Returns:
        List of changed file paths
    """
    output = run_git_command(['diff', '--name-only', base_ref, 'HEAD'], cwd=cwd)
    if not output:
        return []
    return [line.strip() for line in output.split('\n') if line.strip()]


def get_bazel_targets_from_files(files: List[str], workspace_root: str) -> Set[str]:
    """
    Convert file paths to Bazel target patterns.
    
    Args:
        files: List of file paths
        workspace_root: Path to workspace root
        
    Returns:
        Set of Bazel target patterns
    """
    targets = set()
    
    for file in files:
        file_path = Path(file)
        
        # Skip non-source files
        if file_path.suffix in ['.md', '.txt', '.json', '.yaml', '.yml', '.gitignore']:
            continue
        
        # Find the package directory (directory with BUILD or BUILD.bazel)
        current_dir = file_path.parent
        
        # Handle root directory files
        if not str(current_dir) or str(current_dir) == '.':
            targets.add('//:*')
            continue
        
        # Add pattern for all targets in the package
        target_pattern = f"//{current_dir}:*"
        targets.add(target_pattern)
        
        # Also add the directory pattern for recursive queries
        dir_pattern = f"//{current_dir}/..."
        targets.add(dir_pattern)
    
    return targets


def query_affected_targets(changed_targets: Set[str], workspace_root: str) -> List[str]:
    """
    Use bazel query to find all targets affected by changes.
    
    Args:
        changed_targets: Set of changed target patterns
        workspace_root: Path to workspace root
        
    Returns:
        List of fully qualified target labels
    """
    if not changed_targets:
        return []
    
    all_targets = []
    
    for target_pattern in changed_targets:
        try:
            # Query for reverse dependencies (what depends on this)
            query = f"rdeps(//..., {target_pattern})"
            result = subprocess.run(
                ['bazel', 'query', query, '--output=label'],
                capture_output=True,
                text=True,
                cwd=workspace_root,
                timeout=30
            )
            
            if result.returncode == 0:
                targets = [line.strip() for line in result.stdout.split('\n') if line.strip()]
                all_targets.extend(targets)
            else:
                print(f"Warning: bazel query failed for {target_pattern}: {result.stderr}", file=sys.stderr)
        except subprocess.TimeoutExpired:
            print(f"Warning: bazel query timed out for {target_pattern}", file=sys.stderr)
        except Exception as e:
            print(f"Warning: Error querying {target_pattern}: {e}", file=sys.stderr)
    
    # Remove duplicates and sort
    return sorted(set(all_targets))


def get_all_sbom_targets(workspace_root: str) -> List[str]:
    """Get all SBOM generation targets in the workspace."""
    try:
        result = subprocess.run(
            ['bazel', 'query', 'filter(".*_sbom$", //...)', '--output=label'],
            capture_output=True,
            text=True,
            cwd=workspace_root,
            timeout=30
        )
        
        if result.returncode == 0:
            return [line.strip() for line in result.stdout.split('\n') if line.strip()]
    except Exception as e:
        print(f"Warning: Error querying SBOM targets: {e}", file=sys.stderr)
    
    return []


def filter_sbom_targets(affected_targets: List[str], workspace_root: str) -> List[str]:
    """
    Filter affected targets to only include SBOM generation targets.
    
    Args:
        affected_targets: List of affected target labels
        workspace_root: Path to workspace root
        
    Returns:
        List of SBOM target labels to build
    """
    sbom_targets = []
    
    for target in affected_targets:
        # Check if this is already an SBOM target
        if target.endswith('_sbom'):
            sbom_targets.append(target)
        else:
            # Check if there's a corresponding SBOM target
            sbom_target = target + '_sbom'
            # We could query if this exists, but for simplicity, just add the pattern
            # The build will fail gracefully if it doesn't exist
            try:
                result = subprocess.run(
                    ['bazel', 'query', sbom_target],
                    capture_output=True,
                    text=True,
                    cwd=workspace_root,
                    timeout=5
                )
                if result.returncode == 0:
                    sbom_targets.append(sbom_target)
            except:
                pass
    
    return sbom_targets


def main():
    parser = argparse.ArgumentParser(description='Incremental analyzer for Bazel targets')
    parser.add_argument('--workspace', default='.', help='Path to workspace root')
    parser.add_argument('--base-ref', default='HEAD~1', help='Git reference to compare against')
    parser.add_argument('--output', help='Output JSON file for results')
    parser.add_argument('--output-format', choices=['json', 'targets', 'bazel-query'], 
                        default='targets', help='Output format')
    parser.add_argument('--full-analysis', action='store_true', 
                        help='Force full analysis (return all targets)')
    
    args = parser.parse_args()
    
    workspace_root = Path(args.workspace).resolve()
    
    if args.full_analysis:
        print("Full analysis mode: returning all targets", file=sys.stderr)
        sbom_targets = get_all_sbom_targets(str(workspace_root))
        affected_targets = sbom_targets
        changed_files = []
        changed_patterns = set()
    else:
        # Get changed files
        print(f"Detecting changes since {args.base_ref}...", file=sys.stderr)
        changed_files = get_changed_files(args.base_ref, str(workspace_root))
        
        if not changed_files:
            print("No changed files detected", file=sys.stderr)
            affected_targets = []
            changed_patterns = set()
        else:
            print(f"Found {len(changed_files)} changed files", file=sys.stderr)
            
            # Convert to Bazel target patterns
            changed_patterns = get_bazel_targets_from_files(changed_files, str(workspace_root))
            print(f"Generated {len(changed_patterns)} target patterns", file=sys.stderr)
            
            # Query for affected targets
            print("Querying for affected targets...", file=sys.stderr)
            affected_targets = query_affected_targets(changed_patterns, str(workspace_root))
            
            if not affected_targets:
                print("No affected targets found, falling back to all SBOM targets", file=sys.stderr)
                affected_targets = get_all_sbom_targets(str(workspace_root))
            
            # Filter to SBOM targets
            affected_targets = filter_sbom_targets(affected_targets, str(workspace_root))
    
    print(f"Analysis complete: {len(affected_targets)} targets to build", file=sys.stderr)
    
    # Prepare result
    result = {
        "analysis_type": "full" if args.full_analysis else "incremental",
        "base_ref": args.base_ref,
        "changed_files": changed_files,
        "changed_patterns": list(changed_patterns),
        "affected_targets": affected_targets,
        "target_count": len(affected_targets)
    }
    
    # Output based on format
    if args.output_format == 'json':
        output_str = json.dumps(result, indent=2)
    elif args.output_format == 'targets':
        output_str = ' '.join(affected_targets) if affected_targets else '//...'
    elif args.output_format == 'bazel-query':
        # Output in a format suitable for bazel query
        if affected_targets:
            output_str = ' + '.join(f'"{t}"' for t in affected_targets)
        else:
            output_str = '//...'
    
    # Write to file or stdout
    if args.output:
        with open(args.output, 'w') as f:
            if args.output_format == 'json':
                json.dump(result, f, indent=2)
            else:
                f.write(output_str)
        print(f"Results written to {args.output}", file=sys.stderr)
    else:
        print(output_str)
    
    return 0


if __name__ == '__main__':
    sys.exit(main())
