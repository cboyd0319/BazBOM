#!/usr/bin/env python3
"""Container image scanning using RipGrep for fast JAR and OS package discovery.

Provides faster alternative to traditional 'find' commands for large container images.
"""

import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import List, Dict, Optional


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


def extract_jars_from_image(image_layers_path: str) -> List[Dict[str, str]]:
    """
    Use ripgrep to find all JAR files in extracted container layers.
    Faster than 'find' for large images.
    
    Args:
        image_layers_path: Path to extracted container image layers
        
    Returns:
        List of dictionaries containing JAR file metadata
        
    Raises:
        RuntimeError: If RipGrep is not available
        ValueError: If image_layers_path is invalid
    """
    if not check_ripgrep_available():
        raise RuntimeError(
            "RipGrep (rg) is not installed. "
            "Install from: https://github.com/BurntSushi/ripgrep#installation"
        )
    
    layers_path = Path(image_layers_path)
    if not layers_path.exists():
        raise ValueError(f"Image layers path does not exist: {image_layers_path}")
    
    if not layers_path.is_dir():
        raise ValueError(f"Image layers path is not a directory: {image_layers_path}")
    
    try:
        result = subprocess.run([
            'rg',
            '--files',
            '--glob', '*.jar',
            str(layers_path)
        ], capture_output=True, text=True, timeout=60)
    except subprocess.TimeoutExpired:
        raise RuntimeError(f"RipGrep search timed out after 60 seconds in {image_layers_path}")
    except Exception as e:
        raise RuntimeError(f"Failed to run RipGrep: {str(e)}")
    
    jars = []
    for jar_path in result.stdout.strip().split('\n'):
        if jar_path:
            # Extract Maven coordinates from JAR filename
            filename = Path(jar_path).name
            # Pattern: artifact-version.jar
            if '-' in filename:
                parts = filename.replace('.jar', '').rsplit('-', 1)
                jars.append({
                    'artifact': parts[0],
                    'version': parts[1] if len(parts) > 1 else 'unknown',
                    'path': jar_path
                })
            else:
                # No version in filename
                jars.append({
                    'artifact': filename.replace('.jar', ''),
                    'version': 'unknown',
                    'path': jar_path
                })
    
    return jars


def find_os_packages(image_layers_path: str) -> Dict[str, str]:
    """
    Find OS package manifests in container layers.
    Supports: dpkg (Debian/Ubuntu), rpm (RHEL/CentOS), apk (Alpine)
    
    Args:
        image_layers_path: Path to extracted container image layers
        
    Returns:
        Dictionary mapping package manager type to manifest file path
        
    Raises:
        RuntimeError: If RipGrep is not available
        ValueError: If image_layers_path is invalid
    """
    if not check_ripgrep_available():
        raise RuntimeError(
            "RipGrep (rg) is not installed. "
            "Install from: https://github.com/BurntSushi/ripgrep#installation"
        )
    
    layers_path = Path(image_layers_path)
    if not layers_path.exists():
        raise ValueError(f"Image layers path does not exist: {image_layers_path}")
    
    if not layers_path.is_dir():
        raise ValueError(f"Image layers path is not a directory: {image_layers_path}")
    
    manifests = {}
    
    # Debian/Ubuntu packages
    try:
        dpkg_result = subprocess.run([
            'rg', '--files', '--glob', '*/var/lib/dpkg/status', str(layers_path)
        ], capture_output=True, text=True, timeout=30)
        
        if dpkg_result.stdout.strip():
            manifests['dpkg'] = dpkg_result.stdout.strip().split('\n')[0]
    except subprocess.TimeoutExpired:
        pass
    except Exception:
        pass
    
    # Alpine packages
    try:
        apk_result = subprocess.run([
            'rg', '--files', '--glob', '*/lib/apk/db/installed', str(layers_path)
        ], capture_output=True, text=True, timeout=30)
        
        if apk_result.stdout.strip():
            manifests['apk'] = apk_result.stdout.strip().split('\n')[0]
    except subprocess.TimeoutExpired:
        pass
    except Exception:
        pass
    
    # RHEL/CentOS packages
    try:
        rpm_result = subprocess.run([
            'rg', '--files', '--glob', '*/var/lib/rpm/Packages', str(layers_path)
        ], capture_output=True, text=True, timeout=30)
        
        if rpm_result.stdout.strip():
            manifests['rpm'] = rpm_result.stdout.strip().split('\n')[0]
    except subprocess.TimeoutExpired:
        pass
    except Exception:
        pass
    
    return manifests


def scan_container_image(image_name: str, output_path: Optional[str] = None) -> Dict:
    """
    Complete container SBOM workflow with RipGrep acceleration.
    
    Args:
        image_name: Name or ID of container image to scan
        output_path: Optional path to save SBOM JSON
        
    Returns:
        Dictionary containing SBOM data
        
    Raises:
        RuntimeError: If scanning fails or required tools are unavailable
    """
    if not check_ripgrep_available():
        raise RuntimeError(
            "RipGrep (rg) is not installed. "
            "Install from: https://github.com/BurntSushi/ripgrep#installation"
        )
    
    # Check for container tools
    container_tool = None
    for tool in ['podman', 'docker']:
        try:
            subprocess.run([tool, '--version'], capture_output=True, check=True, timeout=5)
            container_tool = tool
            break
        except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired):
            continue
    
    if not container_tool:
        raise RuntimeError(
            "Neither podman nor docker is installed. "
            "Install one to scan container images."
        )
    
    with tempfile.TemporaryDirectory() as tmpdir:
        try:
            # Extract image layers
            print(f"Extracting {image_name} layers...", file=sys.stderr)
            tar_path = Path(tmpdir) / "image.tar"
            
            save_result = subprocess.run(
                [container_tool, 'save', image_name, '-o', str(tar_path)],
                capture_output=True,
                text=True,
                timeout=300
            )
            
            if save_result.returncode != 0:
                raise RuntimeError(
                    f"Failed to save container image: {save_result.stderr}"
                )
            
            # Extract tar
            print("Extracting tar archive...", file=sys.stderr)
            extract_result = subprocess.run(
                ['tar', '-xf', str(tar_path), '-C', tmpdir],
                capture_output=True,
                text=True,
                timeout=300
            )
            
            if extract_result.returncode != 0:
                raise RuntimeError(
                    f"Failed to extract tar archive: {extract_result.stderr}"
                )
            
            # Fast JAR discovery
            print("Scanning for JAR files...", file=sys.stderr)
            jars = extract_jars_from_image(tmpdir)
            
            # OS package discovery
            print("Scanning for OS packages...", file=sys.stderr)
            os_packages = find_os_packages(tmpdir)
            
            # Get current timestamp
            timestamp_result = subprocess.run(
                ['date', '-Iseconds'],
                capture_output=True,
                text=True,
                timeout=5
            )
            timestamp = timestamp_result.stdout.strip() if timestamp_result.returncode == 0 else ''
            
            # Generate SBOM
            sbom = {
                'image': image_name,
                'jvm_dependencies': jars,
                'jvm_dependency_count': len(jars),
                'os_packages': os_packages,
                'scanned_at': timestamp,
                'scanner': 'BazBOM Container Scanner (RipGrep-accelerated)'
            }
            
            # Save to file if requested
            if output_path:
                with open(output_path, 'w', encoding='utf-8') as f:
                    json.dump(sbom, f, indent=2)
                print(f"SBOM written to {output_path}", file=sys.stderr)
            
            return sbom
            
        except subprocess.TimeoutExpired:
            raise RuntimeError("Container extraction timed out")
        except Exception as e:
            raise RuntimeError(f"Failed to scan container: {str(e)}")


def main():
    """CLI entry point for container scanner."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Container image scanning using RipGrep'
    )
    parser.add_argument(
        'image',
        nargs='?',
        help='Container image name or ID to scan'
    )
    parser.add_argument(
        '--output',
        help='Output JSON file for SBOM'
    )
    parser.add_argument(
        '--layers-path',
        help='Path to already-extracted container layers (skip extraction)'
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
    
    if not args.image and not args.layers_path:
        print("ERROR: Either --image or --layers-path must be provided", file=sys.stderr)
        return 1
    
    try:
        if args.layers_path:
            # Scan already-extracted layers
            print(f"Scanning extracted layers in {args.layers_path}...", file=sys.stderr)
            jars = extract_jars_from_image(args.layers_path)
            os_packages = find_os_packages(args.layers_path)
            
            sbom = {
                'layers_path': args.layers_path,
                'jvm_dependencies': jars,
                'jvm_dependency_count': len(jars),
                'os_packages': os_packages,
                'scanner': 'BazBOM Container Scanner (RipGrep-accelerated)'
            }
            
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    json.dump(sbom, f, indent=2)
                print(f"SBOM written to {args.output}", file=sys.stderr)
            else:
                print(json.dumps(sbom, indent=2))
        else:
            # Scan container image
            sbom = scan_container_image(args.image, args.output)
            
            if not args.output:
                print(json.dumps(sbom, indent=2))
        
        print(f"\nScan complete:", file=sys.stderr)
        print(f"  JAR files found: {sbom.get('jvm_dependency_count', 0)}", file=sys.stderr)
        print(f"  OS package systems: {', '.join(sbom.get('os_packages', {}).keys()) or 'none'}", file=sys.stderr)
        
        return 0
        
    except (RuntimeError, ValueError) as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nInterrupted by user", file=sys.stderr)
        return 130


if __name__ == '__main__':
    sys.exit(main())
