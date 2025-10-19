#!/usr/bin/env python3
"""License extractor for inspecting JAR files and POM files to extract license information."""

import argparse
import json
import sys
import zipfile
from defusedxml import ElementTree as ET
from typing import Dict, List, Any, Optional, Set
from pathlib import Path
import re


# Common license SPDX identifiers and patterns
LICENSE_PATTERNS = {
    'Apache-2.0': [
        r'apache\s+license.*version\s+2\.0',
        r'apache\s+2\.0',
        r'aslv?2',
    ],
    'MIT': [
        r'mit\s+license',
        r'mit license',
    ],
    'BSD-3-Clause': [
        r'bsd\s+3-clause',
        r'3-clause\s+bsd',
    ],
    'BSD-2-Clause': [
        r'bsd\s+2-clause',
        r'2-clause\s+bsd',
    ],
    'GPL-2.0': [
        r'gnu\s+general\s+public\s+license.*version\s+2',
        r'gplv?2',
    ],
    'GPL-3.0': [
        r'gnu\s+general\s+public\s+license.*version\s+3',
        r'gplv?3',
    ],
    'LGPL-2.1': [
        r'gnu\s+lesser\s+general\s+public\s+license.*version\s+2\.1',
        r'lgplv?2\.1',
    ],
    'LGPL-3.0': [
        r'gnu\s+lesser\s+general\s+public\s+license.*version\s+3',
        r'lgplv?3',
    ],
    'EPL-1.0': [
        r'eclipse\s+public\s+license.*version\s+1\.0',
        r'eplv?1',
    ],
    'EPL-2.0': [
        r'eclipse\s+public\s+license.*version\s+2\.0',
        r'eplv?2',
    ],
    'MPL-2.0': [
        r'mozilla\s+public\s+license.*version\s+2\.0',
        r'mplv?2',
    ],
}


def normalize_license_text(text: str) -> str:
    """Normalize license text for pattern matching."""
    return re.sub(r'\s+', ' ', text.lower().strip())


def detect_license_from_text(text: str) -> Optional[str]:
    """
    Detect SPDX license identifier from license text.
    
    Args:
        text: License text content
        
    Returns:
        SPDX license identifier or None
    """
    normalized = normalize_license_text(text)
    
    for spdx_id, patterns in LICENSE_PATTERNS.items():
        for pattern in patterns:
            if re.search(pattern, normalized):
                return spdx_id
    
    return None


def extract_from_jar_manifest(jar_path: str) -> Dict[str, Any]:
    """
    Extract license information from JAR manifest file.
    
    Args:
        jar_path: Path to JAR file
        
    Returns:
        Dictionary with manifest information
    """
    info = {
        'licenses': [],
        'vendor': None,
        'version': None,
        'name': None
    }
    
    try:
        with zipfile.ZipFile(jar_path, 'r') as jar:
            if 'META-INF/MANIFEST.MF' in jar.namelist():
                manifest = jar.read('META-INF/MANIFEST.MF').decode('utf-8', errors='ignore')
                
                # Parse manifest
                for line in manifest.split('\n'):
                    line = line.strip()
                    if ':' not in line:
                        continue
                    
                    key, value = line.split(':', 1)
                    key = key.strip()
                    value = value.strip()
                    
                    if key in ['Bundle-License', 'License']:
                        info['licenses'].append(value)
                    elif key in ['Implementation-Vendor', 'Bundle-Vendor']:
                        info['vendor'] = value
                    elif key in ['Implementation-Version', 'Bundle-Version']:
                        info['version'] = value
                    elif key in ['Implementation-Title', 'Bundle-Name']:
                        info['name'] = value
    except Exception as e:
        print(f"Warning: Error reading manifest from {jar_path}: {e}", file=sys.stderr)
    
    return info


def extract_from_license_files(jar_path: str) -> List[str]:
    """
    Extract license information from embedded license files in JAR.
    
    Args:
        jar_path: Path to JAR file
        
    Returns:
        List of detected SPDX license identifiers
    """
    licenses = []
    license_file_patterns = [
        'META-INF/LICENSE',
        'META-INF/LICENSE.txt',
        'META-INF/LICENSE.md',
        'META-INF/NOTICE',
        'LICENSE',
        'LICENSE.txt',
    ]
    
    try:
        with zipfile.ZipFile(jar_path, 'r') as jar:
            for entry in jar.namelist():
                entry_lower = entry.lower()
                
                # Check if this is a license file
                is_license_file = False
                for pattern in license_file_patterns:
                    if entry.endswith(pattern) or entry_lower.endswith(pattern.lower()):
                        is_license_file = True
                        break
                
                if is_license_file:
                    try:
                        content = jar.read(entry).decode('utf-8', errors='ignore')
                        detected = detect_license_from_text(content)
                        if detected and detected not in licenses:
                            licenses.append(detected)
                    except Exception as e:
                        print(f"Warning: Error reading {entry} from {jar_path}: {e}", file=sys.stderr)
    except Exception as e:
        print(f"Warning: Error reading JAR {jar_path}: {e}", file=sys.stderr)
    
    return licenses


def parse_pom_licenses(pom_path: str) -> List[Dict[str, str]]:
    """
    Parse licenses from POM file.
    
    Args:
        pom_path: Path to POM file
        
    Returns:
        List of license dictionaries with name and url
    """
    licenses = []
    
    try:
        tree = ET.parse(pom_path)
        root = tree.getroot()
        
        # Handle XML namespace
        ns = {'maven': 'http://maven.apache.org/POM/4.0.0'}
        if root.tag.startswith('{'):
            ns_match = re.match(r'\{([^}]+)\}', root.tag)
            if ns_match:
                ns['maven'] = ns_match.group(1)
        
        # Find licenses
        license_elements = root.findall('.//maven:licenses/maven:license', ns)
        if not license_elements:
            # Try without namespace
            license_elements = root.findall('.//licenses/license')
        
        for lic in license_elements:
            name_elem = lic.find('maven:name', ns)
            if name_elem is None:
                name_elem = lic.find('name')
            
            url_elem = lic.find('maven:url', ns)
            if url_elem is None:
                url_elem = lic.find('url')
            
            license_info = {}
            if name_elem is not None and name_elem.text:
                license_info['name'] = name_elem.text.strip()
            if url_elem is not None and url_elem.text:
                license_info['url'] = url_elem.text.strip()
            
            if license_info:
                licenses.append(license_info)
    except Exception as e:
        print(f"Warning: Error parsing POM {pom_path}: {e}", file=sys.stderr)
    
    return licenses


def normalize_license_name_to_spdx(license_name: str) -> Optional[str]:
    """
    Convert common license names to SPDX identifiers.
    
    Args:
        license_name: License name from POM or manifest
        
    Returns:
        SPDX identifier or None
    """
    normalized = normalize_license_text(license_name)
    
    # Direct matches
    if 'apache' in normalized and '2.0' in normalized:
        return 'Apache-2.0'
    if 'mit' in normalized:
        return 'MIT'
    if 'bsd' in normalized:
        if '3-clause' in normalized or '3 clause' in normalized:
            return 'BSD-3-Clause'
        elif '2-clause' in normalized or '2 clause' in normalized:
            return 'BSD-2-Clause'
    if 'gpl' in normalized:
        if '3' in normalized:
            return 'GPL-3.0' if 'lesser' not in normalized else 'LGPL-3.0'
        elif '2' in normalized:
            return 'GPL-2.0' if 'lesser' not in normalized else 'LGPL-2.1'
    if 'eclipse' in normalized and 'public' in normalized:
        if '2.0' in normalized:
            return 'EPL-2.0'
        else:
            return 'EPL-1.0'
    if 'mozilla' in normalized and 'public' in normalized:
        return 'MPL-2.0'
    
    return None


def extract_jar_licenses(jar_path: str) -> Dict[str, Any]:
    """
    Extract all license information from a JAR file.
    
    Args:
        jar_path: Path to JAR file
        
    Returns:
        Dictionary with license information
    """
    result = {
        'jar_path': jar_path,
        'spdx_licenses': set(),
        'raw_licenses': [],
        'metadata': {}
    }
    
    # Extract from manifest
    manifest_info = extract_from_jar_manifest(jar_path)
    result['metadata'] = manifest_info
    
    for lic in manifest_info.get('licenses', []):
        result['raw_licenses'].append({'source': 'manifest', 'value': lic})
        spdx = normalize_license_name_to_spdx(lic)
        if spdx:
            result['spdx_licenses'].add(spdx)
    
    # Extract from embedded license files
    file_licenses = extract_from_license_files(jar_path)
    for spdx in file_licenses:
        result['spdx_licenses'].add(spdx)
        result['raw_licenses'].append({'source': 'license_file', 'value': spdx})
    
    # Convert set to list for JSON serialization
    result['spdx_licenses'] = list(result['spdx_licenses'])
    
    return result


def extract_pom_licenses_enhanced(pom_path: str) -> List[str]:
    """
    Extract SPDX license identifiers from POM file.
    
    Args:
        pom_path: Path to POM file
        
    Returns:
        List of SPDX license identifiers
    """
    spdx_licenses = []
    
    pom_licenses = parse_pom_licenses(pom_path)
    for lic in pom_licenses:
        name = lic.get('name', '')
        spdx = normalize_license_name_to_spdx(name)
        if spdx and spdx not in spdx_licenses:
            spdx_licenses.append(spdx)
    
    return spdx_licenses


def main():
    parser = argparse.ArgumentParser(description='License extractor for JAR and POM files')
    parser.add_argument('--jar', help='Path to JAR file')
    parser.add_argument('--pom', help='Path to POM file')
    parser.add_argument('--jar-list', help='File containing list of JAR paths (one per line)')
    parser.add_argument('--output', required=True, help='Output JSON file')
    
    args = parser.parse_args()
    
    results = []
    
    # Process single JAR
    if args.jar:
        print(f"Extracting licenses from {args.jar}", file=sys.stderr)
        jar_info = extract_jar_licenses(args.jar)
        results.append(jar_info)
    
    # Process JAR list
    if args.jar_list:
        with open(args.jar_list, 'r') as f:
            jar_paths = [line.strip() for line in f if line.strip()]
        
        print(f"Extracting licenses from {len(jar_paths)} JARs", file=sys.stderr)
        for jar_path in jar_paths:
            if Path(jar_path).exists():
                jar_info = extract_jar_licenses(jar_path)
                results.append(jar_info)
            else:
                print(f"Warning: JAR not found: {jar_path}", file=sys.stderr)
    
    # Process POM
    if args.pom:
        print(f"Extracting licenses from {args.pom}", file=sys.stderr)
        pom_licenses = extract_pom_licenses_enhanced(args.pom)
        results.append({
            'pom_path': args.pom,
            'spdx_licenses': pom_licenses,
            'raw_licenses': parse_pom_licenses(args.pom)
        })
    
    # Write results
    output = {
        'extracted_at': Path(args.output).name,
        'total_files': len(results),
        'results': results
    }
    
    with open(args.output, 'w') as f:
        json.dump(output, f, indent=2)
    
    print(f"License extraction complete. Results written to {args.output}", file=sys.stderr)
    print(f"Processed {len(results)} files", file=sys.stderr)
    
    return 0


if __name__ == '__main__':
    sys.exit(main())
