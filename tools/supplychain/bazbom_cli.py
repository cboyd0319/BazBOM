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


def run_bazel_aspect_scan(path: Path, include_test: bool = False, target: Optional[str] = None) -> List[dict]:
    """Run Bazel aspect to collect dependency information programmatically.
    
    This function invokes the Bazel aspect to traverse the build graph and
    collect dependency information, then parses the JSON output.
    
    Args:
        path: Project path (Bazel workspace root)
        include_test: Whether to include test dependencies (not yet implemented)
        target: Specific Bazel target to analyze (e.g., "//app:main")
                If None, analyzes workspace-level dependencies
        
    Returns:
        List of dependency dictionaries with Maven coordinates and provenance
        
    Raises:
        RuntimeError: If Bazel build fails or output cannot be parsed
    """
    import subprocess
    import tempfile
    
    # If a specific target is requested, use the aspect on that target
    if target:
        # Create a temporary sbom target for the specified target
        print(f"Running Bazel aspect on target: {target}")
        
        # Build using the aspect directly
        # The output will be in bazel-bin based on the target path
        try:
            result = subprocess.run(
                ["bazel", "build", 
                 f"--aspects=//tools/supplychain:aspects.bzl%sbom_aspect",
                 "--output_groups=sbom_info",
                 target],
                cwd=str(path),
                capture_output=True,
                text=True,
                timeout=300,
            )
            
            if result.returncode != 0:
                raise RuntimeError(f"Bazel aspect build failed: {result.stderr}")
            
            # Note: Aspect output is not directly accessible, we need to use the sbom rule
            # For now, fall back to workspace deps
            print("Note: Direct aspect output not yet supported, using workspace dependencies")
        
        except subprocess.TimeoutExpired:
            raise RuntimeError("Bazel build timed out after 5 minutes")
        except FileNotFoundError:
            raise RuntimeError("Bazel not found in PATH. Please install Bazel.")
    
    # Build the workspace_deps.json target
    # This uses extract_maven_deps.py which reads maven_install.json
    try:
        print("Running Bazel to extract dependencies...")
        result = subprocess.run(
            ["bazel", "build", "//:extract_deps"],
            cwd=str(path),
            capture_output=True,
            text=True,
            timeout=300,
        )
        
        if result.returncode != 0:
            raise RuntimeError(f"Bazel build failed: {result.stderr}")
        
        # Read the generated deps JSON
        deps_json_path = path / "bazel-bin" / "workspace_deps.json"
        if not deps_json_path.exists():
            raise RuntimeError(f"Expected output not found: {deps_json_path}")
        
        with open(deps_json_path, 'r') as f:
            data = json.load(f)
        
        # Extract packages from the JSON
        packages = data.get("packages", [])
        
        print(f"✓ Successfully extracted {len(packages)} dependencies from maven_install.json")
        
        return packages
        
    except subprocess.TimeoutExpired:
        raise RuntimeError("Bazel build timed out after 5 minutes")
    except FileNotFoundError:
        raise RuntimeError("Bazel not found in PATH. Please install Bazel.")
    except json.JSONDecodeError as e:
        raise RuntimeError(f"Failed to parse Bazel output JSON: {e}")


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
    
    # For Bazel, use programmatic aspect invocation
    if build_system.get_name() == "Bazel":
        try:
            dependencies = run_bazel_aspect_scan(path, include_test=args.include_test or config.get_include_test_deps())
            
            print(f"\nFound {len(dependencies)} dependencies from Bazel aspect")
            
            # Export dependencies to JSON
            deps_json = {
                "build_system": "Bazel",
                "project_path": str(path),
                "total_dependencies": len(dependencies),
                "dependencies": dependencies,
            }
            
            output_file = Path(args.output) if args.output else Path("bazel-dependencies.json")
            with open(output_file, 'w', encoding='utf-8') as f:
                json.dump(deps_json, f, indent=2)
            
            print(f"Dependencies exported to: {output_file}")
            
            # TODO: Generate SBOM from dependencies
            # TODO: Run vulnerability scan
            # TODO: Generate SARIF output
            
            return 0
            
        except RuntimeError as e:
            print(f"ERROR: {str(e)}", file=sys.stderr)
            print("\nFallback: For manual Bazel SBOM generation, use:")
            print(f"  cd {path}")
            print("  bazel build //:workspace_sbom")
            print("  bazel build //:sca_scan_osv")
            return 1
    
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
    
    # Handle fast-discovery mode with RipGrep
    if hasattr(args, 'fast_discovery') and args.fast_discovery:
        try:
            from dependency_scanner import check_ripgrep_available, find_bazel_maven_jars
            if not check_ripgrep_available():
                print("⚠️  RipGrep not found - fast discovery unavailable", file=sys.stderr)
                print("   Install: https://github.com/BurntSushi/ripgrep#installation", file=sys.stderr)
                if not hasattr(args, 'no_fast_discovery') or not args.no_fast_discovery:
                    print("   Falling back to standard analysis", file=sys.stderr)
            else:
                print("✅ RipGrep detected - enabling fast discovery mode")
                # Fast dependency discovery will be used by build system
        except ImportError:
            print("⚠️  dependency_scanner module not found", file=sys.stderr)
    
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


def license_report_command(args) -> int:
    """Generate license compliance report using RipGrep.
    
    Args:
        args: Parsed command-line arguments
        
    Returns:
        Exit code
    """
    try:
        from license_scanner import (
            check_ripgrep_available,
            scan_license_headers,
            find_unlicensed_files,
            check_copyleft_licenses,
            generate_license_report
        )
    except ImportError:
        print("ERROR: license_scanner module not found", file=sys.stderr)
        return 1
    
    if not check_ripgrep_available():
        print("ERROR: RipGrep (rg) is not installed", file=sys.stderr)
        print("Install from: https://github.com/BurntSushi/ripgrep#installation", file=sys.stderr)
        return 1
    
    workspace = Path(args.workspace or '.')
    
    print(f"Scanning licenses in: {workspace}")
    
    try:
        if args.check_copyleft:
            # Check for copyleft licenses only
            copyleft = check_copyleft_licenses(str(workspace))
            
            if copyleft:
                print(f"\n⚠️  Found {sum(len(files) for files in copyleft.values())} files with copyleft licenses:")
                for license_type, files in copyleft.items():
                    print(f"\n  {license_type}: {len(files)} files")
                    if args.verbose:
                        for file_path in files[:10]:  # Show first 10
                            print(f"    - {file_path}")
                        if len(files) > 10:
                            print(f"    ... and {len(files) - 10} more")
                return 1  # Exit with error if copyleft found
            else:
                print("\n✅ No copyleft licenses found")
                return 0
        
        elif args.find_unlicensed:
            # Find unlicensed files only
            unlicensed = find_unlicensed_files(str(workspace))
            
            print(f"\nFound {len(unlicensed)} files without license headers")
            
            if unlicensed and args.verbose:
                print("\nUnlicensed files:")
                for file_path in unlicensed[:20]:  # Show first 20
                    print(f"  - {file_path}")
                if len(unlicensed) > 20:
                    print(f"  ... and {len(unlicensed) - 20} more")
            
            if args.output:
                result = {
                    'unlicensed_count': len(unlicensed),
                    'unlicensed_files': unlicensed
                }
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(result, f, indent=2)
                print(f"\nReport saved to: {args.output}")
            
            return 0
        
        else:
            # Full license report
            if args.output:
                output_format = args.format or 'csv'
                if output_format == 'csv':
                    generate_license_report(str(workspace), args.output)
                    print(f"\n✅ License report saved to: {args.output}")
                else:  # json
                    licenses = scan_license_headers(str(workspace))
                    unlicensed = find_unlicensed_files(str(workspace))
                    
                    result = {
                        'total_files': sum(len(files) for files in licenses.values()) + len(unlicensed),
                        'licenses': licenses,
                        'unlicensed': unlicensed
                    }
                    with open(args.output, 'w', encoding='utf-8') as f:
                        json.dump(result, f, indent=2)
                    print(f"\n✅ License report saved to: {args.output}")
            else:
                licenses = scan_license_headers(str(workspace))
                unlicensed = find_unlicensed_files(str(workspace))
                
                total_files = sum(len(files) for files in licenses.values()) + len(unlicensed)
                print(f"\nScanned {total_files} source files")
                print(f"  Licensed: {sum(len(files) for files in licenses.values())}")
                print(f"  Unlicensed: {len(unlicensed)}")
                
                if licenses:
                    print("\nLicense distribution:")
                    for license_type, files in sorted(licenses.items(), key=lambda x: len(x[1]), reverse=True):
                        print(f"  {license_type}: {len(files)} files")
            
            return 0
            
    except (RuntimeError, ValueError) as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nInterrupted by user", file=sys.stderr)
        return 130


def scan_container_command(args) -> int:
    """Scan container image for dependencies using RipGrep.
    
    Args:
        args: Parsed command-line arguments
        
    Returns:
        Exit code
    """
    try:
        from container_scanner import (
            check_ripgrep_available,
            scan_container_image,
            extract_jars_from_image,
            find_os_packages
        )
    except ImportError:
        print("ERROR: container_scanner module not found", file=sys.stderr)
        return 1
    
    if not check_ripgrep_available():
        print("ERROR: RipGrep (rg) is not installed", file=sys.stderr)
        print("Install from: https://github.com/BurntSushi/ripgrep#installation", file=sys.stderr)
        return 1
    
    if args.layers_path:
        # Scan already-extracted layers
        print(f"Scanning extracted layers in: {args.layers_path}")
        try:
            jars = extract_jars_from_image(args.layers_path)
            os_packages = find_os_packages(args.layers_path)
            
            print(f"\n✅ Scan complete:")
            print(f"  JAR files found: {len(jars)}")
            print(f"  OS package systems: {', '.join(os_packages.keys()) or 'none'}")
            
            if args.output:
                sbom = {
                    'layers_path': args.layers_path,
                    'jvm_dependencies': jars,
                    'jvm_dependency_count': len(jars),
                    'os_packages': os_packages,
                    'scanner': 'BazBOM Container Scanner (RipGrep-accelerated)'
                }
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(sbom, f, indent=2)
                print(f"\nSBOM saved to: {args.output}")
            
            return 0
            
        except (RuntimeError, ValueError) as e:
            print(f"ERROR: {str(e)}", file=sys.stderr)
            return 1
    
    elif args.image:
        # Scan container image
        print(f"Scanning container image: {args.image}")
        try:
            sbom = scan_container_image(args.image, args.output)
            
            print(f"\n✅ Scan complete:")
            print(f"  JAR files found: {sbom.get('jvm_dependency_count', 0)}")
            print(f"  OS package systems: {', '.join(sbom.get('os_packages', {}).keys()) or 'none'}")
            
            return 0
            
        except (RuntimeError, ValueError) as e:
            print(f"ERROR: {str(e)}", file=sys.stderr)
            return 1
    
    else:
        print("ERROR: Either --image or --layers-path must be provided", file=sys.stderr)
        return 1


def verify_command(args) -> int:
    """Verify dependency usage using RipGrep.
    
    Args:
        args: Parsed command-line arguments
        
    Returns:
        Exit code
    """
    try:
        from dependency_verifier import (
            check_ripgrep_available,
            find_unused_dependencies,
            find_undeclared_dependencies,
            generate_usage_report
        )
    except ImportError:
        print("ERROR: dependency_verifier module not found", file=sys.stderr)
        return 1
    
    if not check_ripgrep_available():
        print("ERROR: RipGrep (rg) is not installed", file=sys.stderr)
        print("Install from: https://github.com/BurntSushi/ripgrep#installation", file=sys.stderr)
        return 1
    
    workspace = Path(args.workspace or '.')
    maven_install_json = args.maven_install_json or 'maven_install.json'
    
    print(f"Verifying dependencies in: {workspace}")
    print(f"Using lockfile: {maven_install_json}")
    
    try:
        if args.check_unused:
            # Check for unused dependencies
            unused = find_unused_dependencies(str(workspace), maven_install_json)
            
            print(f"\nFound {len(unused)} unused dependencies")
            
            if unused:
                print("\n⚠️  Unused dependencies (consider removing):")
                for dep in unused[:20]:  # Show first 20
                    print(f"  - {dep}")
                if len(unused) > 20:
                    print(f"  ... and {len(unused) - 20} more")
                
                if args.output:
                    result = {
                        'unused_count': len(unused),
                        'unused_dependencies': unused
                    }
                    with open(args.output, 'w', encoding='utf-8') as f:
                        json.dump(result, f, indent=2)
                    print(f"\nReport saved to: {args.output}")
                
                return 1  # Exit with error if unused deps found
            else:
                print("\n✅ No unused dependencies found")
                return 0
        
        elif args.check_undeclared:
            # Check for undeclared dependencies
            undeclared = find_undeclared_dependencies(str(workspace), maven_install_json)
            
            print(f"\nFound {len(undeclared)} undeclared dependencies")
            
            if undeclared:
                print("\n⚠️  Undeclared dependencies (missing from maven_install.json):")
                for dep in undeclared[:20]:
                    print(f"  - {dep}")
                if len(undeclared) > 20:
                    print(f"  ... and {len(undeclared) - 20} more")
                
                if args.output:
                    result = {
                        'undeclared_count': len(undeclared),
                        'undeclared_dependencies': undeclared
                    }
                    with open(args.output, 'w', encoding='utf-8') as f:
                        json.dump(result, f, indent=2)
                    print(f"\nReport saved to: {args.output}")
                
                return 1  # Exit with error if undeclared deps found
            else:
                print("\n✅ No undeclared dependencies found")
                return 0
        
        else:
            # Full usage report
            report = generate_usage_report(str(workspace), maven_install_json)
            
            print(f"\nDependency Usage Report:")
            print(f"  Declared dependencies: {report['declared_count']}")
            print(f"  Referenced dependencies: {report['referenced_count']}")
            print(f"  Used dependencies: {report['used_count']}")
            print(f"  Unused dependencies: {report['unused_count']}")
            print(f"  Undeclared dependencies: {report['undeclared_count']}")
            print(f"  Dependency usage rate: {report['usage_rate']}%")
            
            if report['unused_count'] > 0:
                print(f"\n⚠️  {report['unused_count']} unused dependencies found")
            
            if report['undeclared_count'] > 0:
                print(f"⚠️  {report['undeclared_count']} undeclared dependencies found")
            
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(report, f, indent=2)
                print(f"\nReport saved to: {args.output}")
            
            return 0
            
    except (RuntimeError, ValueError) as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nInterrupted by user", file=sys.stderr)
        return 130


def find_cves_command(args) -> int:
    """Find CVE references in codebase using RipGrep.
    
    Args:
        args: Parsed command-line arguments
        
    Returns:
        Exit code
    """
    try:
        from cve_tracker import (
            check_ripgrep_available,
            find_cve_references,
            cross_reference_with_sbom,
            find_vex_statements
        )
    except ImportError:
        print("ERROR: cve_tracker module not found", file=sys.stderr)
        return 1
    
    if not check_ripgrep_available():
        print("ERROR: RipGrep (rg) is not installed", file=sys.stderr)
        print("Install from: https://github.com/BurntSushi/ripgrep#installation", file=sys.stderr)
        return 1
    
    workspace = Path(args.workspace or '.')
    
    print(f"Searching for CVE references in: {workspace}")
    
    try:
        if args.find_vex:
            # Find VEX statements
            vex_statements = find_vex_statements(str(workspace))
            
            print(f"\nFound {len(vex_statements)} VEX statement files")
            
            if vex_statements:
                total_cves = sum(len(vex['cves']) for vex in vex_statements)
                print(f"Total CVEs in VEX statements: {total_cves}")
                
                if args.verbose:
                    for vex in vex_statements:
                        print(f"\n  {vex['file']}")
                        for cve in vex['cves'][:5]:
                            print(f"    - {cve}")
                        if len(vex['cves']) > 5:
                            print(f"    ... and {len(vex['cves']) - 5} more")
            
            if args.output:
                result = {
                    'vex_file_count': len(vex_statements),
                    'vex_statements': vex_statements
                }
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(result, f, indent=2)
                print(f"\nReport saved to: {args.output}")
            
            return 0
        
        else:
            # Find CVE references in code
            cves = find_cve_references(str(workspace))
            
            unique_cves = len(set(c['cve'] for c in cves))
            print(f"\nFound {len(cves)} CVE references ({unique_cves} unique CVEs)")
            
            # Cross-reference if SBOM findings provided
            if args.sbom_findings:
                cross_ref = cross_reference_with_sbom(cves, args.sbom_findings)
                
                print(f"\nCross-reference with SBOM findings:")
                print(f"  In both code and SBOM: {len(cross_ref['in_both'])}")
                print(f"  Documented only (code): {len(cross_ref['documented_only'])}")
                print(f"  SBOM only (not in code): {len(cross_ref['sbom_only'])}")
                
                if args.verbose:
                    if cross_ref['documented_only']:
                        print(f"\nDocumented but not in SBOM (may be mitigated):")
                        for cve in cross_ref['documented_only'][:10]:
                            print(f"  - {cve}")
                    
                    if cross_ref['sbom_only']:
                        print(f"\nIn SBOM but not documented:")
                        for cve in cross_ref['sbom_only'][:10]:
                            print(f"  - {cve}")
            
            if args.output:
                result = {
                    'cve_reference_count': len(cves),
                    'unique_cves': unique_cves,
                    'cve_references': cves
                }
                
                if args.sbom_findings:
                    result['cross_reference'] = cross_ref
                
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(result, f, indent=2)
                print(f"\nReport saved to: {args.output}")
            
            return 0
            
    except (RuntimeError, ValueError) as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nInterrupted by user", file=sys.stderr)
        return 130


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
  
  # Scan with fast discovery (RipGrep)
  bazbom scan . --fast-discovery
  
  # Include test dependencies
  bazbom scan . --include-test
  
  # Generate license report
  bazbom license-report --output licenses.csv
  
  # Check for copyleft licenses
  bazbom license-report --check-copyleft
  
  # Scan container image
  bazbom scan-container myapp:latest --output container-sbom.json
  
  # Verify dependency usage
  bazbom verify --check-unused
  
  # Find CVE references
  bazbom find-cves --output cves.json
  
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
    scan_parser.add_argument(
        '--fast-discovery',
        action='store_true',
        help='Use RipGrep for 100x faster dependency discovery'
    )
    scan_parser.add_argument(
        '--no-fast-discovery',
        action='store_true',
        help='Disable RipGrep acceleration (use standard methods)'
    )
    
    # License report command
    license_parser = subparsers.add_parser(
        'license-report',
        help='Generate license compliance report (requires RipGrep)'
    )
    license_parser.add_argument(
        'workspace',
        nargs='?',
        default='.',
        help='Workspace root path (default: current directory)'
    )
    license_parser.add_argument(
        '--output',
        '-o',
        help='Output file path (CSV or JSON)'
    )
    license_parser.add_argument(
        '--format',
        choices=['csv', 'json'],
        default='csv',
        help='Output format (default: csv)'
    )
    license_parser.add_argument(
        '--check-copyleft',
        action='store_true',
        help='Check for copyleft licenses (GPL, LGPL) only'
    )
    license_parser.add_argument(
        '--find-unlicensed',
        action='store_true',
        help='Find files without license headers'
    )
    license_parser.add_argument(
        '--verbose',
        '-v',
        action='store_true',
        help='Show detailed file listings'
    )
    
    # Container scan command
    container_parser = subparsers.add_parser(
        'scan-container',
        help='Scan container image for dependencies (requires RipGrep)'
    )
    container_parser.add_argument(
        'image',
        nargs='?',
        help='Container image name or ID to scan'
    )
    container_parser.add_argument(
        '--output',
        '-o',
        help='Output JSON file for SBOM'
    )
    container_parser.add_argument(
        '--layers-path',
        help='Path to already-extracted container layers (skip extraction)'
    )
    
    # Verify command
    verify_parser = subparsers.add_parser(
        'verify',
        help='Verify dependency usage (requires RipGrep)'
    )
    verify_parser.add_argument(
        'workspace',
        nargs='?',
        default='.',
        help='Workspace root path (default: current directory)'
    )
    verify_parser.add_argument(
        '--maven-install-json',
        default='maven_install.json',
        help='Path to maven_install.json (default: maven_install.json)'
    )
    verify_parser.add_argument(
        '--output',
        '-o',
        help='Output JSON file for report'
    )
    verify_parser.add_argument(
        '--check-unused',
        action='store_true',
        help='Check for unused dependencies only'
    )
    verify_parser.add_argument(
        '--check-undeclared',
        action='store_true',
        help='Check for undeclared dependencies only'
    )
    
    # Find CVEs command
    cves_parser = subparsers.add_parser(
        'find-cves',
        help='Find CVE references in codebase (requires RipGrep)'
    )
    cves_parser.add_argument(
        'workspace',
        nargs='?',
        default='.',
        help='Workspace root path (default: current directory)'
    )
    cves_parser.add_argument(
        '--output',
        '-o',
        help='Output JSON file for CVE references'
    )
    cves_parser.add_argument(
        '--sbom-findings',
        help='Path to SBOM findings JSON for cross-reference'
    )
    cves_parser.add_argument(
        '--find-vex',
        action='store_true',
        help='Find VEX statements with CVE references'
    )
    cves_parser.add_argument(
        '--verbose',
        '-v',
        action='store_true',
        help='Show detailed CVE listings'
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
    elif args.command == 'license-report':
        sys.exit(license_report_command(args))
    elif args.command == 'scan-container':
        sys.exit(scan_container_command(args))
    elif args.command == 'verify':
        sys.exit(verify_command(args))
    elif args.command == 'find-cves':
        sys.exit(find_cves_command(args))
    elif args.command == 'init':
        sys.exit(init_command(args))
    elif args.command == 'version':
        sys.exit(version_command(args))
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == '__main__':
    main()
