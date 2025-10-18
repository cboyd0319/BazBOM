#!/usr/bin/env python3
"""BazBOM CLI - Unified command-line interface for all build systems.

This is the main entry point for the standalone BazBOM CLI that works
with Maven, Gradle, Bazel, and other build systems.
"""

import argparse
import json
import sys
import time
from pathlib import Path
from typing import List, Optional, Set

# Import BazBOM modules
try:
    from build_system import detect_build_system, Dependency
    from bazbom_config import BazBOMConfig
except ImportError:
    # Handle cases where modules are in different locations
    import os
    sys.path.insert(0, os.path.dirname(__file__))
    from build_system import detect_build_system, Dependency
    from bazbom_config import BazBOMConfig


__version__ = "1.0.0"


def perform_scan(path: Path, config: BazBOMConfig, args) -> int:
    """Perform a single scan of the project.
    
    Args:
        path: Project path to scan
        config: BazBOM configuration
        args: Command-line arguments
        
    Returns:
        Exit code (0 for success)
    """
    # Detect build system
    build_system = detect_build_system(path)
    
    if not build_system:
        print("ERROR: Could not detect build system", file=sys.stderr)
        print("Supported: Maven (pom.xml), Gradle (build.gradle), Bazel (WORKSPACE)",
              file=sys.stderr)
        return 1
    
    print(f"Detected build system: {build_system.get_name()}")
    
    # For Bazel, delegate to existing tooling
    if build_system.get_name() == "Bazel":
        print("\nFor Bazel projects, use:")
        print(f"  cd {path}")
        print("  bazel build //:sbom_all")
        print("  bazel build //:sca_scan_osv")
        return 0
    
    # Resolve dependencies
    try:
        include_test = args.include_test or config.get_include_test_deps()
        dependencies = build_system.resolve_dependencies(
            path,
            include_test_deps=include_test
        )
        
        print(f"\nFound {len(dependencies)} dependencies")
        
        # Export dependencies to JSON
        deps_json = {
            "build_system": build_system.get_name(),
            "project_path": str(path),
            "total_dependencies": len(dependencies),
            "dependencies": [dep.to_dict() for dep in dependencies],
        }
        
        output_file = Path(args.output) if args.output else Path("dependencies.json")
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(deps_json, f, indent=2)
        
        print(f"Dependencies exported to: {output_file}")
        
        # TODO: Generate SBOM from dependencies
        # TODO: Run vulnerability scan
        # TODO: Generate SARIF output
        
        return 0
        
    except RuntimeError as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        return 1


def get_build_files_mtimes(path: Path, build_system_name: str) -> dict:
    """Get modification times of build files for change detection.
    
    Args:
        path: Project directory path
        build_system_name: Name of detected build system
        
    Returns:
        Dictionary mapping file paths to modification times
    """
    mtimes = {}
    
    # Define files to watch based on build system
    watch_files = []
    if build_system_name == "Maven":
        watch_files = ["pom.xml", "**/*.pom", "**/pom.xml"]
    elif build_system_name == "Gradle":
        watch_files = ["build.gradle", "build.gradle.kts", "settings.gradle",
                      "settings.gradle.kts", "**/*.gradle", "**/*.gradle.kts"]
    elif build_system_name == "Bazel":
        watch_files = ["WORKSPACE", "MODULE.bazel", "BUILD", "BUILD.bazel",
                      "**/*.bzl", "**/BUILD", "**/BUILD.bazel"]
    
    # Collect modification times
    for pattern in watch_files:
        if "**" in pattern:
            # Recursive glob
            for file_path in path.rglob(pattern.replace("**/", "")):
                if file_path.is_file():
                    try:
                        mtimes[str(file_path)] = file_path.stat().st_mtime
                    except OSError:
                        pass
        else:
            # Direct file
            file_path = path / pattern
            if file_path.is_file():
                try:
                    mtimes[str(file_path)] = file_path.stat().st_mtime
                except OSError:
                    pass
    
    return mtimes


def scan_command(args) -> int:
    """Execute scan command.
    
    Args:
        args: Parsed command-line arguments
        
    Returns:
        Exit code (0 for success)
    """
    path = Path(args.path)
    
    # Load configuration
    config = BazBOMConfig.find_and_load(path)
    
    print(f"Scanning project: {path}")
    
    # If watch mode, continuously monitor for changes
    if hasattr(args, 'watch') and args.watch:
        print("\n🔍 Watch mode enabled - monitoring for file changes...")
        print("Press Ctrl+C to stop\n")
        
        # Detect build system once
        build_system = detect_build_system(path)
        if not build_system:
            print("ERROR: Could not detect build system", file=sys.stderr)
            return 1
        
        build_system_name = build_system.get_name()
        
        # Perform initial scan
        print("=" * 60)
        print(f"Initial scan at {time.strftime('%Y-%m-%d %H:%M:%S')}")
        print("=" * 60)
        result = perform_scan(path, config, args)
        
        # Track file modification times
        prev_mtimes = get_build_files_mtimes(path, build_system_name)
        
        try:
            while True:
                time.sleep(2)  # Check every 2 seconds
                
                # Check for file changes
                curr_mtimes = get_build_files_mtimes(path, build_system_name)
                
                # Detect changes
                changed_files = []
                for file_path, mtime in curr_mtimes.items():
                    if file_path not in prev_mtimes or prev_mtimes[file_path] != mtime:
                        changed_files.append(file_path)
                
                # Also check for deleted files
                for file_path in prev_mtimes:
                    if file_path not in curr_mtimes:
                        changed_files.append(file_path + " (deleted)")
                
                if changed_files:
                    print("\n" + "=" * 60)
                    print(f"Change detected at {time.strftime('%Y-%m-%d %H:%M:%S')}")
                    print("=" * 60)
                    print("Changed files:")
                    for changed_file in changed_files:
                        print(f"  - {changed_file}")
                    print("\nRe-scanning...\n")
                    
                    # Perform scan
                    perform_scan(path, config, args)
                    
                    # Update tracked mtimes
                    prev_mtimes = curr_mtimes
        
        except KeyboardInterrupt:
            print("\n\n⏹  Watch mode stopped")
            return 0
    
    # Normal single-scan mode
    return perform_scan(path, config, args)


def init_command(args) -> int:
    """Execute init command to create default configuration.
    
    Args:
        args: Parsed command-line arguments
        
    Returns:
        Exit code
    """
    config_path = Path(args.path) / "bazbom.yml"
    
    if config_path.exists() and not args.force:
        print(f"ERROR: Configuration file already exists: {config_path}",
              file=sys.stderr)
        print("Use --force to overwrite", file=sys.stderr)
        return 1
    
    try:
        config = BazBOMConfig()
        config.save(config_path)
        print(f"Created configuration file: {config_path}")
        print("\nEdit bazbom.yml to customize settings:")
        print("  - build_system: auto|maven|gradle|bazel")
        print("  - include_test_deps: true|false")
        print("  - output_formats: [spdx, cyclonedx]")
        print("  - severity_threshold: CRITICAL|HIGH|MEDIUM|LOW")
        return 0
    except IOError as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        return 1


def version_command(args) -> int:
    """Print version information.
    
    Args:
        args: Parsed command-line arguments
        
    Returns:
        Exit code
    """
    print(f"BazBOM version {__version__}")
    print("Bazel-native SBOM and SCA for JVM projects")
    return 0


def main():
    """Main entry point for BazBOM CLI."""
    parser = argparse.ArgumentParser(
        prog='bazbom',
        description='BazBOM - Software Bill of Materials and Security Analysis',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Scan current directory
  bazbom scan .
  
  # Scan specific project
  bazbom scan /path/to/project
  
  # Include test dependencies
  bazbom scan . --include-test
  
  # Initialize configuration
  bazbom init
  
  # Show version
  bazbom version
        """
    )
    
    parser.add_argument(
        '--version',
        action='version',
        version=f'%(prog)s {__version__}'
    )
    
    subparsers = parser.add_subparsers(dest='command', help='Command to execute')
    
    # Scan command
    scan_parser = subparsers.add_parser(
        'scan',
        help='Scan project for dependencies and vulnerabilities'
    )
    scan_parser.add_argument(
        'path',
        nargs='?',
        default='.',
        help='Project directory to scan (default: current directory)'
    )
    scan_parser.add_argument(
        '--include-test',
        action='store_true',
        help='Include test dependencies'
    )
    scan_parser.add_argument(
        '--output',
        '-o',
        type=str,
        help='Output file path (default: dependencies.json)'
    )
    scan_parser.add_argument(
        '--format',
        choices=['json', 'spdx', 'cyclonedx', 'csv'],
        default='json',
        help='Output format (default: json)'
    )
    scan_parser.add_argument(
        '--watch',
        action='store_true',
        help='Watch for file changes and re-scan automatically'
    )
    
    # Init command
    init_parser = subparsers.add_parser(
        'init',
        help='Initialize BazBOM configuration file'
    )
    init_parser.add_argument(
        'path',
        nargs='?',
        default='.',
        help='Directory to create bazbom.yml in (default: current directory)'
    )
    init_parser.add_argument(
        '--force',
        action='store_true',
        help='Overwrite existing configuration file'
    )
    
    # Version command
    version_parser = subparsers.add_parser(
        'version',
        help='Show version information'
    )
    
    args = parser.parse_args()
    
    # Execute command
    if args.command == 'scan':
        sys.exit(scan_command(args))
    elif args.command == 'init':
        sys.exit(init_command(args))
    elif args.command == 'version':
        sys.exit(version_command(args))
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == '__main__':
    main()
