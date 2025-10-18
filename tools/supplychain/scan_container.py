#!/usr/bin/env python3
"""Container Image SBOM Generator for BazBOM.

This module provides functionality to generate SBOMs for container images,
including application dependencies, OS packages, and base image layers.
Supports Docker images and OCI-compatible containers.
"""

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Dict, List, Optional, Tuple


class ContainerInspector:
    """Inspect container images and extract SBOM information."""
    
    def __init__(self, image: str):
        """Initialize container inspector.
        
        Args:
            image: Container image name/tag or registry URL
        """
        self.image = image
        self.temp_dir = None
    
    def inspect_image(self) -> Dict:
        """Inspect container image metadata.
        
        Returns:
            Dictionary with image metadata
            
        Raises:
            RuntimeError: If docker/podman is not available or inspection fails
        """
        # Try docker first, then podman
        for runtime in ['docker', 'podman']:
            if self._is_runtime_available(runtime):
                return self._inspect_with_runtime(runtime)
        
        raise RuntimeError(
            "Neither docker nor podman is available. "
            "Please install one to scan container images."
        )
    
    def _is_runtime_available(self, runtime: str) -> bool:
        """Check if container runtime is available.
        
        Args:
            runtime: Container runtime name (docker, podman)
            
        Returns:
            True if runtime is available
        """
        try:
            subprocess.run(
                [runtime, '--version'],
                capture_output=True,
                check=True,
                timeout=5
            )
            return True
        except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired):
            return False
    
    def _inspect_with_runtime(self, runtime: str) -> Dict:
        """Inspect image using specific container runtime.
        
        Args:
            runtime: Container runtime to use
            
        Returns:
            Image inspection data
            
        Raises:
            RuntimeError: If inspection fails
        """
        try:
            result = subprocess.run(
                [runtime, 'inspect', self.image],
                capture_output=True,
                check=True,
                text=True,
                timeout=30
            )
            
            inspection = json.loads(result.stdout)
            if not inspection or len(inspection) == 0:
                raise RuntimeError(f"No inspection data returned for image: {self.image}")
            
            return inspection[0]
            
        except subprocess.CalledProcessError as e:
            raise RuntimeError(
                f"Failed to inspect image {self.image}: {e.stderr}"
            )
        except json.JSONDecodeError as e:
            raise RuntimeError(
                f"Failed to parse inspection output for {self.image}: {str(e)}"
            )
        except subprocess.TimeoutExpired:
            raise RuntimeError(
                f"Timeout while inspecting image {self.image}"
            )
    
    def extract_layers(self, runtime: str = 'docker') -> List[str]:
        """Extract image layers for analysis.
        
        Args:
            runtime: Container runtime to use
            
        Returns:
            List of layer IDs
            
        Raises:
            RuntimeError: If layer extraction fails
        """
        inspection = self._inspect_with_runtime(runtime)
        
        layers = []
        if 'RootFS' in inspection and 'Layers' in inspection['RootFS']:
            layers = inspection['RootFS']['Layers']
        
        return layers
    
    def scan_os_packages(self, runtime: str = 'docker') -> List[Dict]:
        """Scan OS packages installed in the image.
        
        Args:
            runtime: Container runtime to use
            
        Returns:
            List of OS package information
            
        Raises:
            RuntimeError: If package scanning fails
        """
        packages = []
        
        # Try to detect package manager and list packages
        package_managers = [
            ('dpkg', ['dpkg-query', '-W', '-f', '${Package}\t${Version}\n']),
            ('rpm', ['rpm', '-qa', '--queryformat', '%{NAME}\t%{VERSION}\n']),
            ('apk', ['apk', 'info', '-v']),
        ]
        
        for pm_name, pm_cmd in package_managers:
            try:
                result = subprocess.run(
                    [runtime, 'run', '--rm', self.image] + pm_cmd,
                    capture_output=True,
                    text=True,
                    timeout=60
                )
                
                if result.returncode == 0:
                    # Parse package list
                    for line in result.stdout.strip().split('\n'):
                        if not line:
                            continue
                        
                        parts = line.split('\t') if '\t' in line else line.split()
                        if len(parts) >= 2:
                            packages.append({
                                'name': parts[0],
                                'version': parts[1],
                                'package_manager': pm_name,
                            })
                    break
                    
            except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
                # Try next package manager
                continue
        
        return packages
    
    def scan_jar_files(self, runtime: str = 'docker') -> List[str]:
        """Find JAR files in the container image.
        
        Args:
            runtime: Container runtime to use
            
        Returns:
            List of JAR file paths
            
        Raises:
            RuntimeError: If JAR scanning fails
        """
        try:
            result = subprocess.run(
                [runtime, 'run', '--rm', self.image, 
                 'find', '/', '-name', '*.jar', '-type', 'f'],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            if result.returncode == 0:
                return [line.strip() for line in result.stdout.split('\n') if line.strip()]
            
            return []
            
        except subprocess.TimeoutExpired:
            raise RuntimeError(f"Timeout while scanning JAR files in {self.image}")


def generate_container_sbom(
    image: str,
    output_path: Optional[Path] = None,
    include_os_packages: bool = True,
    format: str = 'spdx'
) -> Dict:
    """Generate SBOM for a container image.
    
    Args:
        image: Container image name/tag
        output_path: Output file path (optional)
        include_os_packages: Include OS packages in SBOM
        format: Output format (spdx or cyclonedx)
        
    Returns:
        SBOM dictionary
        
    Raises:
        RuntimeError: If SBOM generation fails
    """
    inspector = ContainerInspector(image)
    
    print(f"Inspecting container image: {image}")
    
    # Get image metadata
    try:
        metadata = inspector.inspect_image()
    except RuntimeError as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        sys.exit(1)
    
    print(f"Image ID: {metadata.get('Id', 'unknown')[:12]}")
    
    # Initialize SBOM
    sbom = {
        'image': image,
        'image_id': metadata.get('Id', ''),
        'created': metadata.get('Created', ''),
        'architecture': metadata.get('Architecture', 'unknown'),
        'os': metadata.get('Os', 'unknown'),
        'components': [],
    }
    
    # Scan OS packages
    if include_os_packages:
        print("Scanning OS packages...")
        try:
            packages = inspector.scan_os_packages()
            print(f"Found {len(packages)} OS packages")
            
            for pkg in packages:
                sbom['components'].append({
                    'type': 'os-package',
                    'name': pkg['name'],
                    'version': pkg['version'],
                    'package_manager': pkg['package_manager'],
                    'purl': f"pkg:generic/{pkg['name']}@{pkg['version']}",
                })
        except RuntimeError as e:
            print(f"WARNING: Could not scan OS packages: {str(e)}", file=sys.stderr)
    
    # Scan JAR files (application dependencies)
    print("Scanning JAR files...")
    try:
        jar_files = inspector.scan_jar_files()
        print(f"Found {len(jar_files)} JAR files")
        
        for jar_path in jar_files:
            # Extract artifact info from JAR filename
            jar_name = Path(jar_path).name
            
            sbom['components'].append({
                'type': 'application',
                'name': jar_name,
                'path': jar_path,
                'purl': f"pkg:maven/{jar_name}",  # Simplified, ideally parse manifest
            })
    except RuntimeError as e:
        print(f"WARNING: Could not scan JAR files: {str(e)}", file=sys.stderr)
    
    # Convert to SPDX or CycloneDX format
    if format == 'spdx':
        sbom = convert_to_spdx(sbom)
    elif format == 'cyclonedx':
        sbom = convert_to_cyclonedx(sbom)
    
    # Write output
    if output_path:
        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(sbom, f, indent=2)
        print(f"\nSBOM written to: {output_path}")
    
    return sbom


def convert_to_spdx(raw_sbom: Dict) -> Dict:
    """Convert raw SBOM to SPDX 2.3 format.
    
    Args:
        raw_sbom: Raw SBOM data
        
    Returns:
        SPDX-formatted SBOM
    """
    packages = []
    relationships = []
    
    # Add document package
    doc_id = f"SPDXRef-DOCUMENT"
    image_id = f"SPDXRef-Image-{raw_sbom['image'].replace(':', '-').replace('/', '-')}"
    
    # Add image as top-level package
    packages.append({
        'SPDXID': image_id,
        'name': raw_sbom['image'],
        'versionInfo': raw_sbom.get('image_id', '')[:12],
        'downloadLocation': 'NOASSERTION',
        'filesAnalyzed': False,
        'licenseConcluded': 'NOASSERTION',
        'licenseDeclared': 'NOASSERTION',
    })
    
    # Add components
    for idx, component in enumerate(raw_sbom['components']):
        comp_id = f"SPDXRef-Component-{idx}"
        
        packages.append({
            'SPDXID': comp_id,
            'name': component['name'],
            'versionInfo': component.get('version', ''),
            'downloadLocation': 'NOASSERTION',
            'filesAnalyzed': False,
            'licenseConcluded': 'NOASSERTION',
            'licenseDeclared': 'NOASSERTION',
            'externalRefs': [
                {
                    'referenceCategory': 'PACKAGE-MANAGER',
                    'referenceType': 'purl',
                    'referenceLocator': component.get('purl', ''),
                }
            ] if component.get('purl') else [],
        })
        
        # Relationship: image CONTAINS component
        relationships.append({
            'spdxElementId': image_id,
            'relatedSpdxElement': comp_id,
            'relationshipType': 'CONTAINS',
        })
    
    return {
        'spdxVersion': 'SPDX-2.3',
        'dataLicense': 'CC0-1.0',
        'SPDXID': doc_id,
        'name': f"SBOM-{raw_sbom['image']}",
        'documentNamespace': f"https://bazbom.dev/sbom/{raw_sbom['image_id']}",
        'creationInfo': {
            'created': raw_sbom.get('created', ''),
            'creators': ['Tool: BazBOM-Container-Scanner'],
        },
        'packages': packages,
        'relationships': relationships,
    }


def convert_to_cyclonedx(raw_sbom: Dict) -> Dict:
    """Convert raw SBOM to CycloneDX 1.5 format.
    
    Args:
        raw_sbom: Raw SBOM data
        
    Returns:
        CycloneDX-formatted SBOM
    """
    components = []
    
    for component in raw_sbom['components']:
        comp_type = 'library' if component['type'] == 'application' else 'operating-system'
        
        components.append({
            'type': comp_type,
            'name': component['name'],
            'version': component.get('version', ''),
            'purl': component.get('purl', ''),
        })
    
    return {
        'bomFormat': 'CycloneDX',
        'specVersion': '1.5',
        'version': 1,
        'metadata': {
            'timestamp': raw_sbom.get('created', ''),
            'tools': [
                {
                    'name': 'BazBOM',
                    'version': '1.0.0',
                }
            ],
            'component': {
                'type': 'container',
                'name': raw_sbom['image'],
                'version': raw_sbom.get('image_id', '')[:12],
            }
        },
        'components': components,
    }


def main():
    """Main entry point for container SBOM scanner."""
    parser = argparse.ArgumentParser(
        prog='scan_container',
        description='Generate SBOM for container images',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Scan local image
  scan_container myapp:latest
  
  # Scan image from registry
  scan_container registry.io/myapp:v1.2.3
  
  # Include OS packages
  scan_container myapp:latest --include-os-packages
  
  # Output to file
  scan_container myapp:latest --output myapp-sbom.json
  
  # CycloneDX format
  scan_container myapp:latest --format cyclonedx
        """
    )
    
    parser.add_argument(
        'image',
        help='Container image name/tag or registry URL'
    )
    
    parser.add_argument(
        '--output',
        '-o',
        type=Path,
        help='Output file path (default: stdout)'
    )
    
    parser.add_argument(
        '--include-os-packages',
        action='store_true',
        default=True,
        help='Include OS packages in SBOM (default: true)'
    )
    
    parser.add_argument(
        '--format',
        choices=['spdx', 'cyclonedx'],
        default='spdx',
        help='Output format (default: spdx)'
    )
    
    args = parser.parse_args()
    
    try:
        sbom = generate_container_sbom(
            image=args.image,
            output_path=args.output,
            include_os_packages=args.include_os_packages,
            format=args.format
        )
        
        # Print to stdout if no output file
        if not args.output:
            print(json.dumps(sbom, indent=2))
        
        return 0
        
    except Exception as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
