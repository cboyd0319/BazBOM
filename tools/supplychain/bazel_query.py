#!/usr/bin/env python3
"""
Bazel query support for selective target scanning in monorepos.

This module enables:
1. Scanning specific targets using Bazel query syntax
2. Incremental scanning with rdeps() to find affected targets
3. Scalable analysis for large monorepos (5000+ targets)
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import List, Set, Optional


def run_bazel_query(query: str, workspace_path: Path, timeout: int = 60) -> List[str]:
    """
    Execute a Bazel query and return matching targets.
    
    Args:
        query: Bazel query expression (e.g., 'kind(java_binary, //...)')
        workspace_path: Path to Bazel workspace root
        timeout: Query timeout in seconds
        
    Returns:
        List of target labels
        
    Raises:
        subprocess.CalledProcessError: If query fails
        subprocess.TimeoutExpired: If query times out
    """
    try:
        result = subprocess.run(
            ["bazel", "query", query, "--output=label"],
            cwd=workspace_path,
            capture_output=True,
            text=True,
            check=True,
            timeout=timeout,
        )
        targets = [line.strip() for line in result.stdout.splitlines() if line.strip()]
        return targets
    except subprocess.CalledProcessError as e:
        print(f"Error: Bazel query failed: {e.stderr}", file=sys.stderr)
        raise
    except subprocess.TimeoutExpired:
        print(f"Error: Bazel query timed out after {timeout} seconds", file=sys.stderr)
        raise


def find_targets_by_kind(
    kind: str, 
    package: str = "//...", 
    workspace_path: Path = Path(".")
) -> List[str]:
    """
    Find all targets of a specific kind using Bazel query.
    
    Args:
        kind: Target kind (e.g., 'java_binary', 'java_library', 'java_test')
        package: Package pattern (default: '//...' for all packages)
        workspace_path: Path to Bazel workspace root
        
    Returns:
        List of matching target labels
        
    Example:
        >>> find_targets_by_kind('java_binary', '//src/java/...')
        ['//src/java:compare_resolvers', '//src/java:get_top_x_repos']
    """
    query = f"kind({kind}, {package})"
    return run_bazel_query(query, workspace_path)


def find_rdeps(
    universe: str,
    target_set: str,
    workspace_path: Path = Path("."),
    depth: Optional[int] = None,
) -> List[str]:
    """
    Find reverse dependencies (what depends on the target set).
    
    This is the key function for incremental scanning: given changed files,
    find all targets affected by those changes.
    
    Args:
        universe: Universe of targets to search (e.g., '//...')
        target_set: Target pattern or file set to find rdeps for
        workspace_path: Path to Bazel workspace root
        depth: Optional depth limit for rdeps search
        
    Returns:
        List of affected target labels
        
    Example:
        >>> # Find all targets affected by changes to a specific file
        >>> find_rdeps('//src/java/...', 'set(src/java/main/lib/top_x.java)')
        ['//src/java:get_top_x_repos']
    """
    if depth is not None:
        query = f"rdeps({universe}, {target_set}, {depth})"
    else:
        query = f"rdeps({universe}, {target_set})"
    
    return run_bazel_query(query, workspace_path, timeout=120)


def find_affected_by_files(
    files: List[str],
    universe: str = "//...",
    workspace_path: Path = Path("."),
) -> List[str]:
    """
    Find all targets affected by changes to specific files.
    
    This enables incremental scanning: scan only targets that are affected
    by changed files in a PR or commit.
    
    Args:
        files: List of changed file paths (relative to workspace root)
        universe: Universe of targets to search (default: '//...')
        workspace_path: Path to Bazel workspace root
        
    Returns:
        List of affected target labels
        
    Example:
        >>> # Find targets affected by changes in a PR
        >>> changed_files = ['src/java/main/lib/top_x.java', 'src/java/BUILD.bazel']
        >>> find_affected_by_files(changed_files, '//src/java/...')
        ['//src/java:get_top_x_repos', '//src/java:lib']
    """
    if not files:
        return []
    
    # Build file set for query
    # Escape special characters and quote file paths
    file_set = ", ".join(f'"{f}"' for f in files)
    target_set = f"set({file_set})"
    
    return find_rdeps(universe, target_set, workspace_path)


def scan_targets_incremental(
    changed_files: List[str],
    target_kind: Optional[str] = None,
    universe: str = "//...",
    workspace_path: Path = Path("."),
) -> List[str]:
    """
    Incremental scan: find targets to scan based on changed files.
    
    This is the primary function for PR-based incremental analysis:
    1. Find all targets affected by changed files (using rdeps)
    2. Optionally filter to specific target kinds (e.g., only java_binary)
    3. Return list of targets to scan
    
    Args:
        changed_files: List of changed file paths
        target_kind: Optional filter by target kind (e.g., 'java_binary')
        universe: Universe pattern for search
        workspace_path: Path to Bazel workspace root
        
    Returns:
        List of target labels to scan
        
    Example:
        >>> # Scan only java_binary targets affected by PR changes
        >>> files = ['src/java/lib/utils.java']
        >>> scan_targets_incremental(files, target_kind='java_binary', universe='//src/java/...')
        ['//src/java:my_app']
    """
    print(f"[bazel-query] finding targets affected by {len(changed_files)} files", file=sys.stderr)
    
    # Find all affected targets
    affected = find_affected_by_files(changed_files, universe, workspace_path)
    
    if not affected:
        print("[bazel-query] no affected targets found", file=sys.stderr)
        return []
    
    print(f"[bazel-query] found {len(affected)} affected targets", file=sys.stderr)
    
    # Filter by kind if specified
    if target_kind:
        # Create a set expression from affected targets
        target_set = " + ".join(f'"{t}"' for t in affected)
        query = f"kind({target_kind}, {target_set})"
        filtered = run_bazel_query(query, workspace_path)
        print(f"[bazel-query] filtered to {len(filtered)} {target_kind} targets", file=sys.stderr)
        return filtered
    
    return affected


def validate_query(query: str, workspace_path: Path = Path(".")) -> bool:
    """
    Validate a Bazel query without executing it.
    
    Args:
        query: Bazel query expression to validate
        workspace_path: Path to Bazel workspace root
        
    Returns:
        True if query is valid, False otherwise
    """
    try:
        # Try a dry-run with limited output
        subprocess.run(
            ["bazel", "query", query, "--output=label", "--keep_going"],
            cwd=workspace_path,
            capture_output=True,
            check=True,
            timeout=10,
        )
        return True
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return False


def main():
    """CLI for Bazel query operations."""
    parser = argparse.ArgumentParser(
        description="Bazel query support for selective target scanning"
    )
    parser.add_argument(
        "--workspace",
        type=Path,
        default=Path("."),
        help="Path to Bazel workspace root",
    )
    parser.add_argument(
        "--query",
        help="Bazel query expression to execute",
    )
    parser.add_argument(
        "--kind",
        help="Find targets of specific kind (e.g., java_binary)",
    )
    parser.add_argument(
        "--universe",
        default="//...",
        help="Universe pattern for search (default: //...)",
    )
    parser.add_argument(
        "--affected-by-files",
        nargs="+",
        help="Find targets affected by these files (incremental scan)",
    )
    parser.add_argument(
        "--filter-kind",
        help="Filter results to specific target kind",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Output JSON file for results",
    )
    parser.add_argument(
        "--format",
        choices=["targets", "json"],
        default="targets",
        help="Output format (default: targets)",
    )
    
    args = parser.parse_args()
    
    try:
        # Execute query based on arguments
        if args.query:
            targets = run_bazel_query(args.query, args.workspace)
        elif args.kind:
            targets = find_targets_by_kind(args.kind, args.universe, args.workspace)
        elif args.affected_by_files:
            targets = scan_targets_incremental(
                args.affected_by_files,
                target_kind=args.filter_kind,
                universe=args.universe,
                workspace_path=args.workspace,
            )
        else:
            print("Error: must specify --query, --kind, or --affected-by-files", file=sys.stderr)
            return 1
        
        # Output results
        if args.format == "json":
            result = {
                "workspace": str(args.workspace.resolve()),
                "query": args.query or f"kind({args.kind}, {args.universe})" if args.kind else "rdeps",
                "targets": targets,
                "count": len(targets),
            }
            output_str = json.dumps(result, indent=2)
        else:
            output_str = "\n".join(targets)
        
        if args.output:
            args.output.write_text(output_str)
            print(f"[bazel-query] wrote {len(targets)} targets to {args.output}", file=sys.stderr)
        else:
            print(output_str)
        
        print(f"[bazel-query] found {len(targets)} matching targets", file=sys.stderr)
        return 0
        
    except subprocess.CalledProcessError as e:
        print(f"Error: Bazel query failed: {e}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
