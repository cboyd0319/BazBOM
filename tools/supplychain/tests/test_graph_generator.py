#!/usr/bin/env python3
"""Comprehensive tests for graph_generator.py.

Tests cover dependency graph generation in JSON and GraphML formats,
including depth calculation, node/edge creation, and XML escaping.
"""

import json
from pathlib import Path
from unittest.mock import mock_open, patch

import pytest

from tools.supplychain.graph_generator import (
    calculate_depths,
    generate_graph_json,
    generate_graphml,
)


class TestCalculateDepths:
    """Test cases for calculate_depths function."""

    def test_calculate_depths_empty_packages(self):
        """Test depth calculation with empty package list."""
        depths = calculate_depths([])
        
        assert depths == {}

    def test_calculate_depths_single_direct_dependency(self):
        """Test depth calculation with one direct dependency."""
        packages = [
            {
                "group": "com.google.guava",
                "name": "guava",
                "is_direct": True,
                "dependencies": []
            }
        ]
        
        depths = calculate_depths(packages)
        
        assert depths["com.google.guava:guava"] == 1

    def test_calculate_depths_direct_and_transitive(self):
        """Test depth calculation with direct and transitive dependencies."""
        packages = [
            {
                "group": "com.example",
                "name": "app",
                "is_direct": True,
                "dependencies": ["com.google.guava:guava"]
            },
            {
                "group": "com.google.guava",
                "name": "guava",
                "is_direct": False,
                "dependencies": ["com.google.guava:failureaccess"]
            },
            {
                "group": "com.google.guava",
                "name": "failureaccess",
                "is_direct": False,
                "dependencies": []
            }
        ]
        
        depths = calculate_depths(packages)
        
        assert depths["com.example:app"] == 1
        assert depths["com.google.guava:guava"] == 2
        assert depths["com.google.guava:failureaccess"] == 3

    def test_calculate_depths_shortest_path(self):
        """Test that shortest path is chosen when multiple paths exist."""
        packages = [
            {
                "group": "com.example",
                "name": "direct1",
                "is_direct": True,
                "dependencies": ["shared:lib"]
            },
            {
                "group": "com.example",
                "name": "direct2",
                "is_direct": True,
                "dependencies": ["intermediate:lib"]
            },
            {
                "group": "intermediate",
                "name": "lib",
                "is_direct": False,
                "dependencies": ["shared:lib"]
            },
            {
                "group": "shared",
                "name": "lib",
                "is_direct": False,
                "dependencies": []
            }
        ]
        
        depths = calculate_depths(packages)
        
        # shared:lib can be reached via direct1 (depth 2) or direct2->intermediate (depth 3)
        # Should choose shortest path: depth 2
        assert depths["shared:lib"] == 2

    def test_calculate_depths_no_direct_dependencies(self):
        """Test depth calculation when no packages are marked as direct."""
        packages = [
            {
                "group": "com.example",
                "name": "lib",
                "is_direct": False,
                "dependencies": []
            }
        ]
        
        depths = calculate_depths(packages)
        
        # Non-direct packages without paths from direct deps don't get depths
        assert "com.example:lib" not in depths

    def test_calculate_depths_circular_dependencies(self):
        """Test handling of circular dependencies (visited tracking)."""
        packages = [
            {
                "group": "a",
                "name": "a",
                "is_direct": True,
                "dependencies": ["b:b"]
            },
            {
                "group": "b",
                "name": "b",
                "is_direct": False,
                "dependencies": ["c:c"]
            },
            {
                "group": "c",
                "name": "c",
                "is_direct": False,
                "dependencies": ["b:b"]  # Circular reference
            }
        ]
        
        depths = calculate_depths(packages)
        
        # Should handle circular reference without infinite loop
        assert depths["a:a"] == 1
        assert depths["b:b"] == 2
        assert depths["c:c"] == 3


class TestGenerateGraphJson:
    """Test cases for generate_graph_json function."""

    def test_generate_graph_json_empty_packages(self):
        """Test graph generation with empty package list."""
        graph = generate_graph_json([])
        
        assert graph["version"] == "1.0"
        assert len(graph["graph"]["nodes"]) == 1  # Just root node
        assert graph["graph"]["nodes"][0]["id"] == "root"
        assert len(graph["graph"]["edges"]) == 0
        assert graph["statistics"]["total_packages"] == 0
        assert graph["statistics"]["max_depth"] == 0

    def test_generate_graph_json_single_package(self):
        """Test graph generation with single package."""
        packages = [
            {
                "name": "guava",
                "version": "31.1-jre",
                "group": "com.google.guava",
                "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
                "is_direct": True,
                "type": "maven",
                "dependencies": []
            }
        ]
        
        graph = generate_graph_json(packages)
        
        assert graph["statistics"]["total_packages"] == 1
        assert graph["statistics"]["direct_dependencies"] == 1
        assert graph["statistics"]["transitive_dependencies"] == 0
        
        # Check nodes
        nodes = {n["id"]: n for n in graph["graph"]["nodes"]}
        assert "root" in nodes
        assert "pkg:maven/com.google.guava/guava@31.1-jre" in nodes
        
        guava_node = nodes["pkg:maven/com.google.guava/guava@31.1-jre"]
        assert guava_node["name"] == "guava"
        assert guava_node["version"] == "31.1-jre"
        assert guava_node["is_direct"] is True
        assert guava_node["depth"] == 1
        
        # Check edges - root to direct dependency
        assert len(graph["graph"]["edges"]) == 1
        assert graph["graph"]["edges"][0]["from"] == "root"
        assert graph["graph"]["edges"][0]["to"] == "pkg:maven/com.google.guava/guava@31.1-jre"

    def test_generate_graph_json_with_transitive_deps(self):
        """Test graph generation with transitive dependencies."""
        packages = [
            {
                "name": "app",
                "version": "1.0",
                "group": "com.example",
                "purl": "pkg:maven/com.example/app@1.0",
                "is_direct": True,
                "type": "maven",
                "dependencies": ["com.google.guava:guava"]
            },
            {
                "name": "guava",
                "version": "31.1-jre",
                "group": "com.google.guava",
                "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
                "is_direct": False,
                "type": "maven",
                "dependencies": []
            }
        ]
        
        graph = generate_graph_json(packages)
        
        assert graph["statistics"]["total_packages"] == 2
        assert graph["statistics"]["direct_dependencies"] == 1
        assert graph["statistics"]["transitive_dependencies"] == 1
        
        # Check transitive edge exists
        edges = graph["graph"]["edges"]
        transitive_edge = next(
            (e for e in edges if e["from"] == "pkg:maven/com.example/app@1.0"),
            None
        )
        assert transitive_edge is not None
        assert transitive_edge["to"] == "pkg:maven/com.google.guava/guava@31.1-jre"

    def test_generate_graph_json_with_sha256(self):
        """Test that SHA256 is included in nodes when present."""
        packages = [
            {
                "name": "lib",
                "version": "1.0",
                "group": "com.example",
                "purl": "pkg:maven/com.example/lib@1.0",
                "is_direct": True,
                "type": "maven",
                "sha256": "abc123def456",
                "dependencies": []
            }
        ]
        
        graph = generate_graph_json(packages)
        
        lib_node = next(n for n in graph["graph"]["nodes"] if n["id"] != "root")
        assert "sha256" in lib_node
        assert lib_node["sha256"] == "abc123def456"

    def test_generate_graph_json_without_purl(self):
        """Test graph generation when package lacks PURL."""
        packages = [
            {
                "name": "lib",
                "version": "1.0",
                "is_direct": True,
                "dependencies": []
            }
        ]
        
        graph = generate_graph_json(packages)
        
        # Should use name as fallback for ID
        lib_node = next(n for n in graph["graph"]["nodes"] if n["id"] != "root")
        assert lib_node["id"] == "lib"
        assert lib_node["name"] == "lib"

    def test_generate_graph_json_deduplicates_packages(self):
        """Test that duplicate packages (same PURL) are deduplicated."""
        packages = [
            {
                "name": "lib",
                "version": "1.0",
                "purl": "pkg:maven/com.example/lib@1.0",
                "is_direct": True,
                "dependencies": []
            },
            {
                "name": "lib",
                "version": "1.0",
                "purl": "pkg:maven/com.example/lib@1.0",
                "is_direct": True,
                "dependencies": []
            }
        ]
        
        graph = generate_graph_json(packages)
        
        # Should only have 2 nodes: root + 1 unique lib
        assert len(graph["graph"]["nodes"]) == 2

    def test_generate_graph_json_max_depth(self):
        """Test that max_depth is calculated correctly."""
        packages = [
            {
                "group": "a",
                "name": "a",
                "is_direct": True,
                "dependencies": ["b:b"]
            },
            {
                "group": "b",
                "name": "b",
                "is_direct": False,
                "dependencies": ["c:c"]
            },
            {
                "group": "c",
                "name": "c",
                "is_direct": False,
                "dependencies": []
            }
        ]
        
        graph = generate_graph_json(packages)
        
        assert graph["statistics"]["max_depth"] == 3


class TestGenerateGraphML:
    """Test cases for generate_graphml function."""

    def test_generate_graphml_empty_packages(self):
        """Test GraphML generation with empty package list."""
        graphml = generate_graphml([])
        
        assert '<?xml version="1.0" encoding="UTF-8"?>' in graphml
        assert '<graphml' in graphml
        assert '<node id="root">' in graphml
        assert '</graphml>' in graphml

    def test_generate_graphml_single_package(self):
        """Test GraphML generation with single package."""
        packages = [
            {
                "name": "guava",
                "version": "31.1-jre",
                "group": "com.google.guava",
                "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
                "is_direct": True,
                "type": "maven",
                "dependencies": []
            }
        ]
        
        graphml = generate_graphml(packages)
        
        assert '<node id="pkg:maven/com.google.guava/guava@31.1-jre">' in graphml
        assert '<data key="name">guava</data>' in graphml
        assert '<data key="version">31.1-jre</data>' in graphml
        assert '<data key="is_direct">true</data>' in graphml
        assert '<data key="depth">1</data>' in graphml
        assert '<edge source="root" target="pkg:maven/com.google.guava/guava@31.1-jre">' in graphml

    def test_generate_graphml_xml_escaping(self):
        """Test that XML special characters are properly escaped."""
        packages = [
            {
                "name": "lib<script>",
                "version": "1.0&test",
                "purl": "pkg:maven/com.example/lib<script>@1.0&test",
                "is_direct": True,
                "dependencies": []
            }
        ]
        
        graphml = generate_graphml(packages)
        
        # Check that special characters are escaped
        assert "&lt;script&gt;" in graphml
        assert "&amp;test" in graphml
        assert "<script>" not in graphml  # Raw tag should not appear

    def test_generate_graphml_with_edges(self):
        """Test GraphML edge generation for transitive dependencies."""
        packages = [
            {
                "name": "app",
                "version": "1.0",
                "group": "com.example",
                "purl": "pkg:maven/com.example/app@1.0",
                "is_direct": True,
                "type": "maven",
                "dependencies": ["com.google.guava:guava"]
            },
            {
                "name": "guava",
                "version": "31.1-jre",
                "group": "com.google.guava",
                "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
                "is_direct": False,
                "type": "maven",
                "dependencies": []
            }
        ]
        
        graphml = generate_graphml(packages)
        
        # Check that edge from app to guava exists
        assert '<edge source="pkg:maven/com.example/app@1.0" target="pkg:maven/com.google.guava/guava@31.1-jre">' in graphml
        assert '<data key="edgetype">depends_on</data>' in graphml

    def test_generate_graphml_attribute_keys(self):
        """Test that all required attribute keys are defined."""
        graphml = generate_graphml([])
        
        required_keys = [
            '<key id="name"',
            '<key id="version"',
            '<key id="type"',
            '<key id="purl"',
            '<key id="depth"',
            '<key id="is_direct"',
            '<key id="edgetype"'
        ]
        
        for key in required_keys:
            assert key in graphml

    def test_generate_graphml_deduplication(self):
        """Test that GraphML deduplicates packages with same PURL."""
        packages = [
            {
                "name": "lib",
                "version": "1.0",
                "purl": "pkg:maven/com.example/lib@1.0",
                "is_direct": True,
                "dependencies": []
            },
            {
                "name": "lib",
                "version": "1.0",
                "purl": "pkg:maven/com.example/lib@1.0",
                "is_direct": True,
                "dependencies": []
            }
        ]
        
        graphml = generate_graphml(packages)
        
        # Count occurrences of the node ID
        count = graphml.count('<node id="pkg:maven/com.example/lib@1.0">')
        assert count == 1  # Should only appear once

    def test_generate_graphml_depth_calculation(self):
        """Test that depths are correctly calculated and included in GraphML."""
        packages = [
            {
                "group": "a",
                "name": "a",
                "purl": "pkg:maven/a/a@1.0",
                "is_direct": True,
                "dependencies": ["b:b"]
            },
            {
                "group": "b",
                "name": "b",
                "purl": "pkg:maven/b/b@1.0",
                "is_direct": False,
                "dependencies": []
            }
        ]
        
        graphml = generate_graphml(packages)
        
        # Check depths in output
        assert '<data key="depth">1</data>' in graphml  # Direct dependency
        assert '<data key="depth">2</data>' in graphml  # Transitive dependency


class TestMainFunction:
    """Test cases for main() function."""

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_missing_input(self, mock_parse_args, capsys):
        """Test error when neither --sbom nor --deps is provided."""
        from tools.supplychain.graph_generator import main
        
        mock_parse_args.return_value = type('Args', (), {
            'sbom': None,
            'deps': None,
            'output_json': 'out.json',
            'output_graphml': None
        })()
        
        result = main()
        
        assert result == 1
        captured = capsys.readouterr()
        assert "Either --sbom or --deps must be specified" in captured.err

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_missing_output(self, mock_parse_args, capsys):
        """Test error when no output format is specified."""
        from tools.supplychain.graph_generator import main
        
        mock_parse_args.return_value = type('Args', (), {
            'sbom': 'sbom.json',
            'deps': None,
            'output_json': None,
            'output_graphml': None
        })()
        
        result = main()
        
        assert result == 1
        captured = capsys.readouterr()
        assert "At least one output format must be specified" in captured.err

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_deps_file_not_found(self, mock_parse_args, capsys):
        """Test error when deps file doesn't exist."""
        from tools.supplychain.graph_generator import main
        
        mock_parse_args.return_value = type('Args', (), {
            'sbom': None,
            'deps': '/nonexistent/deps.json',
            'output_json': 'out.json',
            'output_graphml': None
        })()
        
        result = main()
        
        assert result == 1
        captured = capsys.readouterr()
        assert "Deps file not found" in captured.err

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_deps_invalid_json(self, mock_parse_args, tmp_path, capsys):
        """Test error when deps file has invalid JSON."""
        from tools.supplychain.graph_generator import main
        
        deps_file = tmp_path / "deps.json"
        deps_file.write_text("invalid json{")
        
        mock_parse_args.return_value = type('Args', (), {
            'sbom': None,
            'deps': str(deps_file),
            'output_json': 'out.json',
            'output_graphml': None
        })()
        
        result = main()
        
        assert result == 1
        captured = capsys.readouterr()
        assert "Invalid JSON in deps file" in captured.err

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_sbom_file_not_found(self, mock_parse_args, capsys):
        """Test error when SBOM file doesn't exist."""
        from tools.supplychain.graph_generator import main
        
        mock_parse_args.return_value = type('Args', (), {
            'sbom': '/nonexistent/sbom.json',
            'deps': None,
            'output_json': 'out.json',
            'output_graphml': None
        })()
        
        result = main()
        
        assert result == 1
        captured = capsys.readouterr()
        assert "SBOM file not found" in captured.err

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_with_deps_success(self, mock_parse_args, tmp_path, capsys):
        """Test successful execution with deps file."""
        from tools.supplychain.graph_generator import main
        
        deps_file = tmp_path / "deps.json"
        deps_file.write_text(json.dumps({
            "packages": [
                {
                    "name": "lib",
                    "version": "1.0",
                    "purl": "pkg:maven/com.example/lib@1.0",
                    "is_direct": True,
                    "dependencies": []
                }
            ]
        }))
        
        output_json = tmp_path / "out.json"
        
        mock_parse_args.return_value = type('Args', (), {
            'sbom': None,
            'deps': str(deps_file),
            'output_json': str(output_json),
            'output_graphml': None
        })()
        
        result = main()
        
        assert result == 0
        assert output_json.exists()
        
        # Verify output
        with open(output_json) as f:
            graph = json.load(f)
        assert graph["statistics"]["total_packages"] == 1

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_with_sbom_success(self, mock_parse_args, tmp_path, capsys):
        """Test successful execution with SBOM file."""
        from tools.supplychain.graph_generator import main
        
        sbom_file = tmp_path / "sbom.json"
        sbom_file.write_text(json.dumps({
            "packages": [
                {
                    "SPDXID": "SPDXRef-Package-lib",
                    "name": "lib",
                    "versionInfo": "1.0",
                    "externalRefs": [
                        {
                            "referenceType": "purl",
                            "referenceLocator": "pkg:maven/com.example/lib@1.0"
                        }
                    ]
                }
            ]
        }))
        
        output_graphml = tmp_path / "out.graphml"
        
        mock_parse_args.return_value = type('Args', (), {
            'sbom': str(sbom_file),
            'deps': None,
            'output_json': None,
            'output_graphml': str(output_graphml)
        })()
        
        result = main()
        
        assert result == 0
        assert output_graphml.exists()
        assert '<graphml' in output_graphml.read_text()

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_sbom_skips_root_package(self, mock_parse_args, tmp_path):
        """Test that SBOM processing skips root package."""
        from tools.supplychain.graph_generator import main
        
        sbom_file = tmp_path / "sbom.json"
        sbom_file.write_text(json.dumps({
            "packages": [
                {
                    "SPDXID": "SPDXRef-Package-root",
                    "name": "root",
                    "versionInfo": "1.0"
                },
                {
                    "SPDXID": "SPDXRef-Package-lib",
                    "name": "lib",
                    "versionInfo": "1.0",
                    "externalRefs": []
                }
            ]
        }))
        
        output_json = tmp_path / "out.json"
        
        mock_parse_args.return_value = type('Args', (), {
            'sbom': str(sbom_file),
            'deps': None,
            'output_json': str(output_json),
            'output_graphml': None
        })()
        
        result = main()
        
        assert result == 0
        with open(output_json) as f:
            graph = json.load(f)
        # Should only have 1 package (lib), root is filtered
        assert graph["statistics"]["total_packages"] == 1


class TestGraphGeneratorIntegration:
    """Integration tests for graph generation."""

    def test_json_and_graphml_consistency(self):
        """Test that JSON and GraphML generate consistent graphs."""
        packages = [
            {
                "name": "app",
                "version": "1.0",
                "group": "com.example",
                "purl": "pkg:maven/com.example/app@1.0",
                "is_direct": True,
                "type": "maven",
                "dependencies": ["com.google.guava:guava"]
            },
            {
                "name": "guava",
                "version": "31.1-jre",
                "group": "com.google.guava",
                "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
                "is_direct": False,
                "type": "maven",
                "dependencies": []
            }
        ]
        
        json_graph = generate_graph_json(packages)
        graphml = generate_graphml(packages)
        
        # Both should have same number of nodes (excluding root in count)
        json_node_count = json_graph["statistics"]["total_packages"]
        graphml_node_count = graphml.count('<node id="pkg:maven/')
        
        assert json_node_count == graphml_node_count == 2
        
        # Both should have edges
        json_edge_count = len(json_graph["graph"]["edges"])
        graphml_edge_count = graphml.count('<edge source=')
        
        assert json_edge_count == graphml_edge_count

    def test_large_dependency_tree(self):
        """Test handling of larger dependency tree."""
        # Create a tree with 10 direct deps, each with 3 transitive deps
        packages = []
        
        for i in range(10):
            packages.append({
                "name": f"direct{i}",
                "version": "1.0",
                "group": "com.example",
                "purl": f"pkg:maven/com.example/direct{i}@1.0",
                "is_direct": True,
                "dependencies": [f"com.example:trans{i}-{j}" for j in range(3)]
            })
            
            for j in range(3):
                packages.append({
                    "name": f"trans{i}-{j}",
                    "version": "1.0",
                    "group": "com.example",
                    "purl": f"pkg:maven/com.example/trans{i}-{j}@1.0",
                    "is_direct": False,
                    "dependencies": []
                })
        
        graph = generate_graph_json(packages)
        
        assert graph["statistics"]["total_packages"] == 40
        assert graph["statistics"]["direct_dependencies"] == 10
        assert graph["statistics"]["transitive_dependencies"] == 30

    def test_missing_optional_fields(self):
        """Test handling of packages with minimal fields."""
        packages = [
            {
                "name": "minimal",
                "is_direct": True,
                "dependencies": []
            }
        ]
        
        json_graph = generate_graph_json(packages)
        graphml = generate_graphml(packages)
        
        # Should handle missing version, group, etc.
        assert json_graph is not None
        assert graphml is not None
        
        # Check fallback values
        node = json_graph["graph"]["nodes"][1]  # Skip root
        assert node["name"] == "minimal"
        assert node["version"] == "unknown"
        assert node["type"] == "unknown"

    def test_dependency_not_in_coord_map(self):
        """Test handling when dependency coordinate is not found in package map."""
        packages = [
            {
                "name": "app",
                "group": "com.example",
                "is_direct": True,
                "dependencies": ["com.unknown:missing:1.0"]  # This dep doesn't exist
            }
        ]
        
        json_graph = generate_graph_json(packages)
        graphml = generate_graphml(packages)
        
        # Should not crash, just skip the missing dependency
        assert json_graph is not None
        assert graphml is not None
        # There will be one edge from root to app (direct dependency)
        # But no edge from app to the missing dependency
        edges = json_graph["graph"]["edges"]
        root_to_app_edges = [e for e in edges if e["from"] == "root"]
        assert len(root_to_app_edges) == 1  # Only root -> app
        
        # No edge from app to missing dependency
        app_edges = [e for e in edges if e["from"] == "app"]
        assert len(app_edges) == 0


class TestMainFunctionEdgeCases:
    """Additional edge case tests for main function."""

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_sbom_invalid_json(self, mock_parse_args, tmp_path, capsys):
        """Test main with invalid JSON in SBOM file."""
        # Create invalid SBOM file
        sbom_file = tmp_path / "invalid.spdx.json"
        sbom_file.write_text("{ not valid json }")
        
        mock_parse_args.return_value = type('Args', (), {
            'deps': None,
            'sbom': str(sbom_file),
            'output_json': str(tmp_path / "graph.json"),
            'output_graphml': None
        })()
        
        from tools.supplychain.graph_generator import main
        result = main()
        
        assert result == 1
        captured = capsys.readouterr()
        assert "Invalid JSON" in captured.err

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_sbom_with_npm_purl(self, mock_parse_args, tmp_path):
        """Test main with SBOM containing npm packages."""
        sbom_data = {
            "spdxVersion": "SPDX-2.3",
            "packages": [
                {
                    "SPDXID": "SPDXRef-Package-root",
                    "name": "root"
                },
                {
                    "SPDXID": "SPDXRef-Package-react",
                    "name": "react",
                    "versionInfo": "18.0.0",
                    "externalRefs": [
                        {
                            "referenceType": "purl",
                            "referenceLocator": "pkg:npm/react@18.0.0"
                        }
                    ]
                }
            ]
        }
        
        sbom_file = tmp_path / "sbom.spdx.json"
        sbom_file.write_text(json.dumps(sbom_data))
        output_file = tmp_path / "graph.json"
        
        mock_parse_args.return_value = type('Args', (), {
            'deps': None,
            'sbom': str(sbom_file),
            'output_json': str(output_file),
            'output_graphml': None
        })()
        
        from tools.supplychain.graph_generator import main
        result = main()
        
        assert result == 0
        # Verify output was created
        assert output_file.exists()
        graph = json.loads(output_file.read_text())
        # Find the npm package node
        nodes = [n for n in graph["graph"]["nodes"] if n["name"] == "react"]
        assert len(nodes) == 1
        assert nodes[0]["type"] == "npm"

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_sbom_with_pypi_purl(self, mock_parse_args, tmp_path):
        """Test main with SBOM containing PyPI packages."""
        sbom_data = {
            "spdxVersion": "SPDX-2.3",
            "packages": [
                {
                    "SPDXID": "SPDXRef-Package-root",
                    "name": "root"
                },
                {
                    "SPDXID": "SPDXRef-Package-requests",
                    "name": "requests",
                    "versionInfo": "2.28.0",
                    "externalRefs": [
                        {
                            "referenceType": "purl",
                            "referenceLocator": "pkg:pypi/requests@2.28.0"
                        }
                    ]
                }
            ]
        }
        
        sbom_file = tmp_path / "sbom.spdx.json"
        sbom_file.write_text(json.dumps(sbom_data))
        output_file = tmp_path / "graph.json"
        
        mock_parse_args.return_value = type('Args', (), {
            'deps': None,
            'sbom': str(sbom_file),
            'output_json': str(output_file),
            'output_graphml': None
        })()
        
        from tools.supplychain.graph_generator import main
        result = main()
        
        assert result == 0
        graph = json.loads(output_file.read_text())
        nodes = [n for n in graph["graph"]["nodes"] if n["name"] == "requests"]
        assert len(nodes) == 1
        assert nodes[0]["type"] == "pypi"

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_sbom_with_malformed_maven_purl(self, mock_parse_args, tmp_path):
        """Test main with SBOM containing malformed Maven PURL (< 3 parts)."""
        sbom_data = {
            "spdxVersion": "SPDX-2.3",
            "packages": [
                {
                    "SPDXID": "SPDXRef-Package-root",
                    "name": "root"
                },
                {
                    "SPDXID": "SPDXRef-Package-lib",
                    "name": "lib",
                    "versionInfo": "1.0",
                    "externalRefs": [
                        {
                            "referenceType": "purl",
                            "referenceLocator": "pkg:maven/lib"  # Malformed: only 2 parts
                        }
                    ]
                }
            ]
        }
        
        sbom_file = tmp_path / "sbom.spdx.json"
        sbom_file.write_text(json.dumps(sbom_data))
        output_file = tmp_path / "graph.json"
        
        mock_parse_args.return_value = type('Args', (), {
            'deps': None,
            'sbom': str(sbom_file),
            'output_json': str(output_file),
            'output_graphml': None
        })()
        
        from tools.supplychain.graph_generator import main
        result = main()
        
        assert result == 0
        # Should not crash, just skip extracting group from malformed PURL
        graph = json.loads(output_file.read_text())
        nodes = [n for n in graph["graph"]["nodes"] if n["name"] == "lib"]
        assert len(nodes) == 1

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_sbom_without_external_refs(self, mock_parse_args, tmp_path):
        """Test main with SBOM package without external references."""
        sbom_data = {
            "spdxVersion": "SPDX-2.3",
            "packages": [
                {
                    "SPDXID": "SPDXRef-Package-root",
                    "name": "root"
                },
                {
                    "SPDXID": "SPDXRef-Package-lib",
                    "name": "lib",
                    "versionInfo": "1.0"
                    # No externalRefs field
                }
            ]
        }
        
        sbom_file = tmp_path / "sbom.spdx.json"
        sbom_file.write_text(json.dumps(sbom_data))
        output_file = tmp_path / "graph.json"
        
        mock_parse_args.return_value = type('Args', (), {
            'deps': None,
            'sbom': str(sbom_file),
            'output_json': str(output_file),
            'output_graphml': None
        })()
        
        from tools.supplychain.graph_generator import main
        result = main()
        
        assert result == 0
        # Should not crash when package has no externalRefs
        graph = json.loads(output_file.read_text())
        assert graph is not None

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_sbom_with_non_purl_refs(self, mock_parse_args, tmp_path):
        """Test main with SBOM package with externalRefs but no purl type."""
        sbom_data = {
            "spdxVersion": "SPDX-2.3",
            "packages": [
                {
                    "SPDXID": "SPDXRef-Package-root",
                    "name": "root"
                },
                {
                    "SPDXID": "SPDXRef-Package-lib",
                    "name": "lib",
                    "versionInfo": "1.0",
                    "externalRefs": [
                        {
                            "referenceType": "cpe23",  # Not purl
                            "referenceLocator": "cpe:2.3:a:vendor:product:1.0"
                        }
                    ]
                }
            ]
        }
        
        sbom_file = tmp_path / "sbom.spdx.json"
        sbom_file.write_text(json.dumps(sbom_data))
        output_file = tmp_path / "graph.json"
        
        mock_parse_args.return_value = type('Args', (), {
            'deps': None,
            'sbom': str(sbom_file),
            'output_json': str(output_file),
            'output_graphml': None
        })()
        
        from tools.supplychain.graph_generator import main
        result = main()
        
        assert result == 0
        # Should not crash when externalRefs has no purl type
        graph = json.loads(output_file.read_text())
        assert graph is not None

    @patch('tools.supplychain.graph_generator.argparse.ArgumentParser.parse_args')
    def test_main_sbom_with_unknown_purl_type(self, mock_parse_args, tmp_path):
        """Test main with SBOM containing unknown PURL type."""
        sbom_data = {
            "spdxVersion": "SPDX-2.3",
            "packages": [
                {
                    "SPDXID": "SPDXRef-Package-root",
                    "name": "root"
                },
                {
                    "SPDXID": "SPDXRef-Package-lib",
                    "name": "lib",
                    "versionInfo": "1.0",
                    "externalRefs": [
                        {
                            "referenceType": "purl",
                            "referenceLocator": "pkg:cargo/tokio@1.0.0"  # Unknown type (not maven/npm/pypi)
                        }
                    ]
                }
            ]
        }
        
        sbom_file = tmp_path / "sbom.spdx.json"
        sbom_file.write_text(json.dumps(sbom_data))
        output_file = tmp_path / "graph.json"
        
        mock_parse_args.return_value = type('Args', (), {
            'deps': None,
            'sbom': str(sbom_file),
            'output_json': str(output_file),
            'output_graphml': None
        })()
        
        from tools.supplychain.graph_generator import main
        result = main()
        
        assert result == 0
        # Should not crash with unknown PURL type
        graph = json.loads(output_file.read_text())
        nodes = [n for n in graph["graph"]["nodes"] if n["name"] == "lib"]
        assert len(nodes) == 1
