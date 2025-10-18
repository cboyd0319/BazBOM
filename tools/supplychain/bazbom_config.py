#!/usr/bin/env python3
"""Configuration file support for BazBOM.

This module provides support for bazbom.yml configuration files,
enabling users to customize BazBOM behavior without command-line flags.
"""

import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML not installed. Install with: pip install pyyaml", file=sys.stderr)
    sys.exit(1)


class BazBOMConfig:
    """BazBOM configuration loaded from bazbom.yml."""
    
    DEFAULT_CONFIG = {
        "build_system": "auto",
        "include_test_deps": False,
        "output_formats": ["spdx"],
        "severity_threshold": "MEDIUM",
        "policy": {
            "block_critical": True,
            "fail_on_policy_violation": True,
            "max_critical": 0,
            "max_high": 10,
        },
        "vulnerability_sources": {
            "osv": {"enabled": True},
            "nvd": {"enabled": False},
        },
        "output": {
            "sbom_path": "sbom.spdx.json",
            "findings_path": "sca_findings.json",
            "sarif_path": "sca_findings.sarif",
        },
    }
    
    def __init__(self, config_dict: Optional[Dict[str, Any]] = None):
        """Initialize configuration.
        
        Args:
            config_dict: Configuration dictionary (defaults to DEFAULT_CONFIG)
        """
        self.config = self._merge_with_defaults(config_dict or {})
    
    def _merge_with_defaults(self, config: Dict[str, Any]) -> Dict[str, Any]:
        """Merge user config with defaults.
        
        Args:
            config: User configuration
            
        Returns:
            Merged configuration
        """
        merged = self.DEFAULT_CONFIG.copy()
        
        # Deep merge nested dictionaries
        for key, value in config.items():
            if key in merged and isinstance(merged[key], dict) and isinstance(value, dict):
                merged[key] = {**merged[key], **value}
            else:
                merged[key] = value
        
        return merged
    
    @classmethod
    def from_file(cls, path: Path) -> 'BazBOMConfig':
        """Load configuration from YAML file.
        
        Args:
            path: Path to bazbom.yml
            
        Returns:
            BazBOMConfig instance
            
        Raises:
            FileNotFoundError: If config file doesn't exist
            ValueError: If config file is invalid
        """
        if not path.exists():
            raise FileNotFoundError(f"Configuration file not found: {path}")
        
        try:
            with open(path, 'r', encoding='utf-8') as f:
                config_dict = yaml.safe_load(f)
            
            if not isinstance(config_dict, dict):
                raise ValueError(f"Invalid configuration file: {path}")
            
            return cls(config_dict)
            
        except yaml.YAMLError as e:
            raise ValueError(f"Invalid YAML in configuration file: {e}")
    
    @classmethod
    def find_and_load(cls, start_dir: Optional[Path] = None) -> 'BazBOMConfig':
        """Find and load bazbom.yml from current or parent directories.
        
        Args:
            start_dir: Directory to start search from (defaults to current dir)
            
        Returns:
            BazBOMConfig instance (uses defaults if no config found)
        """
        if start_dir is None:
            start_dir = Path.cwd()
        
        # Search for bazbom.yml in current and parent directories
        current = start_dir
        while True:
            config_path = current / "bazbom.yml"
            if config_path.exists():
                try:
                    return cls.from_file(config_path)
                except (FileNotFoundError, ValueError) as e:
                    print(f"WARNING: Error loading config from {config_path}: {e}",
                          file=sys.stderr)
                    break
            
            # Move to parent directory
            parent = current.parent
            if parent == current:
                # Reached root
                break
            current = parent
        
        # No config found, use defaults
        return cls()
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get configuration value.
        
        Args:
            key: Configuration key (supports dot notation for nested keys)
            default: Default value if key not found
            
        Returns:
            Configuration value
        """
        keys = key.split('.')
        value = self.config
        
        for k in keys:
            if isinstance(value, dict) and k in value:
                value = value[k]
            else:
                return default
        
        return value
    
    def get_build_system(self) -> str:
        """Get build system setting."""
        return self.get('build_system', 'auto')
    
    def get_include_test_deps(self) -> bool:
        """Get include_test_deps setting."""
        return self.get('include_test_deps', False)
    
    def get_output_formats(self) -> List[str]:
        """Get output format list."""
        return self.get('output_formats', ['spdx'])
    
    def get_severity_threshold(self) -> str:
        """Get severity threshold."""
        return self.get('severity_threshold', 'MEDIUM')
    
    def should_block_critical(self) -> bool:
        """Check if critical vulnerabilities should block builds."""
        return self.get('policy.block_critical', True)
    
    def should_fail_on_policy_violation(self) -> bool:
        """Check if policy violations should fail builds."""
        return self.get('policy.fail_on_policy_violation', True)
    
    def get_max_critical_vulns(self) -> int:
        """Get maximum allowed critical vulnerabilities."""
        return self.get('policy.max_critical', 0)
    
    def get_max_high_vulns(self) -> int:
        """Get maximum allowed high vulnerabilities."""
        return self.get('policy.max_high', 10)
    
    def to_dict(self) -> Dict[str, Any]:
        """Export configuration as dictionary.
        
        Returns:
            Configuration dictionary
        """
        return self.config.copy()
    
    def save(self, path: Path) -> None:
        """Save configuration to YAML file.
        
        Args:
            path: Path to save configuration
            
        Raises:
            IOError: If unable to write file
        """
        try:
            with open(path, 'w', encoding='utf-8') as f:
                yaml.dump(self.config, f, default_flow_style=False, sort_keys=False)
        except IOError as e:
            raise IOError(f"Failed to save configuration to {path}: {str(e)}")


def create_default_config(path: Path) -> None:
    """Create a default bazbom.yml configuration file.
    
    Args:
        path: Path where to create the config file
        
    Raises:
        FileExistsError: If file already exists
        IOError: If unable to write file
    """
    if path.exists():
        raise FileExistsError(f"Configuration file already exists: {path}")
    
    config = BazBOMConfig()
    config.save(path)
    print(f"Created default configuration: {path}")


def main():
    """Main entry point for config utility."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='BazBOM configuration utility',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Create default configuration file
  %(prog)s --init
  
  # Show current configuration
  %(prog)s --show
  
  # Validate configuration file
  %(prog)s --validate bazbom.yml
        """
    )
    
    parser.add_argument(
        '--init',
        action='store_true',
        help='Create default bazbom.yml in current directory'
    )
    
    parser.add_argument(
        '--show',
        action='store_true',
        help='Show current configuration (from bazbom.yml or defaults)'
    )
    
    parser.add_argument(
        '--validate',
        type=str,
        metavar='FILE',
        help='Validate configuration file'
    )
    
    args = parser.parse_args()
    
    if args.init:
        try:
            create_default_config(Path("bazbom.yml"))
        except FileExistsError as e:
            print(f"ERROR: {str(e)}", file=sys.stderr)
            sys.exit(1)
        except IOError as e:
            print(f"ERROR: {str(e)}", file=sys.stderr)
            sys.exit(2)
    
    elif args.show:
        config = BazBOMConfig.find_and_load()
        print("Current BazBOM configuration:")
        print(yaml.dump(config.to_dict(), default_flow_style=False, sort_keys=False))
    
    elif args.validate:
        try:
            config = BazBOMConfig.from_file(Path(args.validate))
            print(f"✓ Configuration file is valid: {args.validate}")
            print(f"  Build system: {config.get_build_system()}")
            print(f"  Output formats: {', '.join(config.get_output_formats())}")
            print(f"  Severity threshold: {config.get_severity_threshold()}")
        except (FileNotFoundError, ValueError) as e:
            print(f"ERROR: {str(e)}", file=sys.stderr)
            sys.exit(1)
    
    else:
        parser.print_help()


if __name__ == '__main__':
    main()
