#!/usr/bin/env python3
"""Tests for bazbom_config.py - Configuration file support."""

import sys
from pathlib import Path

import pytest
import yaml

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from bazbom_config import BazBOMConfig, create_default_config


class TestBazBOMConfigInit:
    """Test BazBOMConfig initialization."""
    
    def test_init_with_defaults(self):
        """Test initialization with default configuration."""
        config = BazBOMConfig()
        
        assert config.config is not None
        assert config.get("build_system") == "auto"
        assert config.get("include_test_deps") is False
        assert config.get("output_formats") == ["spdx"]
    
    def test_init_with_empty_dict(self):
        """Test initialization with empty config dict uses defaults."""
        config = BazBOMConfig({})
        
        assert config.get("build_system") == "auto"
        assert config.get("severity_threshold") == "MEDIUM"
    
    def test_init_with_custom_config(self):
        """Test initialization with custom configuration."""
        custom = {"build_system": "maven", "include_test_deps": True}
        config = BazBOMConfig(custom)
        
        assert config.get("build_system") == "maven"
        assert config.get("include_test_deps") is True
        # Defaults should still be present for non-overridden keys
        assert config.get("output_formats") == ["spdx"]
    
    def test_init_merges_nested_dicts(self):
        """Test that nested dictionaries are merged, not replaced."""
        custom = {
            "policy": {
                "max_critical": 5
            }
        }
        config = BazBOMConfig(custom)
        
        # Custom value should be set
        assert config.get("policy.max_critical") == 5
        # Default values should still exist
        assert config.get("policy.block_critical") is True
        assert config.get("policy.max_high") == 10


class TestBazBOMConfigFromFile:
    """Test loading configuration from file."""
    
    def test_from_file_success(self, tmp_path):
        """Test loading valid configuration file."""
        config_data = {
            "build_system": "gradle",
            "severity_threshold": "HIGH",
        }
        config_file = tmp_path / "bazbom.yml"
        with open(config_file, "w") as f:
            yaml.dump(config_data, f)
        
        config = BazBOMConfig.from_file(config_file)
        
        assert config.get("build_system") == "gradle"
        assert config.get("severity_threshold") == "HIGH"
    
    def test_from_file_not_found(self):
        """Test loading non-existent file raises FileNotFoundError."""
        non_existent = Path("/nonexistent/bazbom.yml")
        
        with pytest.raises(FileNotFoundError, match="Configuration file not found"):
            BazBOMConfig.from_file(non_existent)
    
    def test_from_file_invalid_yaml(self, tmp_path):
        """Test loading invalid YAML raises ValueError."""
        config_file = tmp_path / "bazbom.yml"
        with open(config_file, "w") as f:
            f.write("invalid: yaml: content:\n  bad indentation")
        
        with pytest.raises(ValueError, match="Invalid YAML"):
            BazBOMConfig.from_file(config_file)
    
    def test_from_file_not_a_dict(self, tmp_path):
        """Test loading file with non-dict content raises ValueError."""
        config_file = tmp_path / "bazbom.yml"
        with open(config_file, "w") as f:
            f.write("- list\n- of\n- items\n")
        
        with pytest.raises(ValueError, match="Invalid configuration file"):
            BazBOMConfig.from_file(config_file)
    
    def test_from_file_empty(self, tmp_path):
        """Test loading empty file uses defaults."""
        config_file = tmp_path / "bazbom.yml"
        with open(config_file, "w") as f:
            f.write("{}")  # Empty dict in YAML
        
        config = BazBOMConfig.from_file(config_file)
        
        # Should use all defaults
        assert config.get("build_system") == "auto"


class TestBazBOMConfigFindAndLoad:
    """Test finding and loading configuration from directory tree."""
    
    def test_find_and_load_in_current_dir(self, tmp_path, monkeypatch):
        """Test finding config in current directory."""
        config_data = {"build_system": "bazel"}
        config_file = tmp_path / "bazbom.yml"
        with open(config_file, "w") as f:
            yaml.dump(config_data, f)
        
        config = BazBOMConfig.find_and_load(tmp_path)
        
        assert config.get("build_system") == "bazel"
    
    def test_find_and_load_in_parent_dir(self, tmp_path):
        """Test finding config in parent directory."""
        # Create config in parent
        config_data = {"build_system": "maven"}
        config_file = tmp_path / "bazbom.yml"
        with open(config_file, "w") as f:
            yaml.dump(config_data, f)
        
        # Create subdirectory
        subdir = tmp_path / "subdir"
        subdir.mkdir()
        
        config = BazBOMConfig.find_and_load(subdir)
        
        assert config.get("build_system") == "maven"
    
    def test_find_and_load_no_config_uses_defaults(self, tmp_path):
        """Test that no config file found uses defaults."""
        config = BazBOMConfig.find_and_load(tmp_path)
        
        assert config.get("build_system") == "auto"
        assert config.get("severity_threshold") == "MEDIUM"
    
    def test_find_and_load_with_invalid_config_uses_defaults(self, tmp_path, capsys):
        """Test that invalid config file falls back to defaults with warning."""
        config_file = tmp_path / "bazbom.yml"
        with open(config_file, "w") as f:
            f.write("invalid yaml content {{{")
        
        config = BazBOMConfig.find_and_load(tmp_path)
        
        # Should use defaults
        assert config.get("build_system") == "auto"
        
        # Should print warning
        captured = capsys.readouterr()
        assert "WARNING" in captured.err


class TestBazBOMConfigGetters:
    """Test configuration getter methods."""
    
    def test_get_with_simple_key(self):
        """Test getting value with simple key."""
        config = BazBOMConfig({"build_system": "gradle"})
        
        assert config.get("build_system") == "gradle"
    
    def test_get_with_nested_key(self):
        """Test getting value with dot notation."""
        config = BazBOMConfig()
        
        assert config.get("policy.max_critical") == 0
        assert config.get("output.sbom_path") == "sbom.spdx.json"
    
    def test_get_with_default(self):
        """Test getting non-existent key returns default."""
        config = BazBOMConfig()
        
        assert config.get("nonexistent", "default_value") == "default_value"
    
    def test_get_with_nested_default(self):
        """Test getting non-existent nested key returns default."""
        config = BazBOMConfig()
        
        assert config.get("nonexistent.nested.key", 42) == 42
    
    def test_get_build_system(self):
        """Test get_build_system convenience method."""
        config = BazBOMConfig({"build_system": "maven"})
        
        assert config.get_build_system() == "maven"
    
    def test_get_include_test_deps(self):
        """Test get_include_test_deps convenience method."""
        config = BazBOMConfig({"include_test_deps": True})
        
        assert config.get_include_test_deps() is True
    
    def test_get_output_formats(self):
        """Test get_output_formats convenience method."""
        config = BazBOMConfig({"output_formats": ["spdx", "cyclonedx"]})
        
        assert config.get_output_formats() == ["spdx", "cyclonedx"]
    
    def test_get_severity_threshold(self):
        """Test get_severity_threshold convenience method."""
        config = BazBOMConfig({"severity_threshold": "HIGH"})
        
        assert config.get_severity_threshold() == "HIGH"
    
    def test_should_block_critical(self):
        """Test should_block_critical convenience method."""
        config = BazBOMConfig({"policy": {"block_critical": False}})
        
        assert config.should_block_critical() is False
    
    def test_should_fail_on_policy_violation(self):
        """Test should_fail_on_policy_violation convenience method."""
        config = BazBOMConfig({"policy": {"fail_on_policy_violation": False}})
        
        assert config.should_fail_on_policy_violation() is False
    
    def test_get_max_critical_vulns(self):
        """Test get_max_critical_vulns convenience method."""
        config = BazBOMConfig({"policy": {"max_critical": 5}})
        
        assert config.get_max_critical_vulns() == 5
    
    def test_get_max_high_vulns(self):
        """Test get_max_high_vulns convenience method."""
        config = BazBOMConfig({"policy": {"max_high": 20}})
        
        assert config.get_max_high_vulns() == 20


class TestBazBOMConfigSave:
    """Test saving configuration to file."""
    
    def test_save_success(self, tmp_path):
        """Test saving configuration to file."""
        config = BazBOMConfig({"build_system": "maven"})
        output_file = tmp_path / "output.yml"
        
        config.save(output_file)
        
        assert output_file.exists()
        with open(output_file, "r") as f:
            saved_data = yaml.safe_load(f)
        assert saved_data["build_system"] == "maven"
    
    def test_save_to_readonly_location_raises_error(self, tmp_path):
        """Test saving to read-only location raises IOError."""
        config = BazBOMConfig()
        readonly_dir = tmp_path / "readonly"
        readonly_dir.mkdir()
        # Make directory read-only
        readonly_dir.chmod(0o444)
        output_file = readonly_dir / "bazbom.yml"
        
        try:
            with pytest.raises(IOError, match="Failed to save configuration"):
                config.save(output_file)
        finally:
            # Restore permissions for cleanup
            readonly_dir.chmod(0o755)


class TestBazBOMConfigToDict:
    """Test exporting configuration as dictionary."""
    
    def test_to_dict(self):
        """Test exporting configuration as dictionary."""
        config = BazBOMConfig({"build_system": "gradle"})
        
        result = config.to_dict()
        
        assert isinstance(result, dict)
        assert result["build_system"] == "gradle"
    
    def test_to_dict_returns_copy(self):
        """Test that to_dict returns a copy, not reference."""
        config = BazBOMConfig()
        
        dict1 = config.to_dict()
        dict2 = config.to_dict()
        
        # Modifying dict1 should not affect dict2
        dict1["build_system"] = "modified"
        assert dict2["build_system"] == "auto"


class TestCreateDefaultConfig:
    """Test creating default configuration file."""
    
    def test_create_default_config(self, tmp_path, capsys):
        """Test creating default config file."""
        config_file = tmp_path / "bazbom.yml"
        
        create_default_config(config_file)
        
        assert config_file.exists()
        captured = capsys.readouterr()
        assert "Created default configuration" in captured.out
    
    def test_create_default_config_file_exists_raises_error(self, tmp_path):
        """Test creating config file that already exists raises error."""
        config_file = tmp_path / "bazbom.yml"
        config_file.touch()  # Create empty file
        
        with pytest.raises(FileExistsError, match="already exists"):
            create_default_config(config_file)
    
    def test_created_config_is_valid(self, tmp_path):
        """Test that created default config is valid and loadable."""
        config_file = tmp_path / "bazbom.yml"
        
        create_default_config(config_file)
        
        # Should be able to load it back
        config = BazBOMConfig.from_file(config_file)
        assert config.get("build_system") == "auto"


class TestMergeWithDefaults:
    """Test configuration merging logic."""
    
    def test_merge_empty_config(self):
        """Test merging empty config preserves all defaults."""
        config = BazBOMConfig({})
        
        assert config.get("build_system") == "auto"
        assert config.get("policy.max_critical") == 0
    
    def test_merge_overrides_top_level(self):
        """Test merging overrides top-level keys."""
        config = BazBOMConfig({"build_system": "gradle"})
        
        assert config.get("build_system") == "gradle"
    
    def test_merge_nested_dict_partial_override(self):
        """Test merging nested dict partially overrides."""
        config = BazBOMConfig({
            "policy": {
                "max_critical": 5
            }
        })
        
        # Overridden value
        assert config.get("policy.max_critical") == 5
        # Default values still present
        assert config.get("policy.block_critical") is True
        assert config.get("policy.max_high") == 10
    
    def test_merge_list_values_replace(self):
        """Test that list values are replaced, not merged."""
        config = BazBOMConfig({
            "output_formats": ["cyclonedx"]
        })
        
        # Should completely replace default list
        assert config.get("output_formats") == ["cyclonedx"]
    
    def test_merge_new_keys_added(self):
        """Test that new keys not in defaults are added."""
        config = BazBOMConfig({
            "custom_key": "custom_value"
        })
        
        assert config.get("custom_key") == "custom_value"
