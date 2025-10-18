#!/usr/bin/env python3
"""Comprehensive unit tests for license_analyzer.py."""

import json
import pytest
from pathlib import Path
from unittest.mock import patch

from tools.supplychain.license_analyzer import (
    COPYLEFT_LICENSES,
    PERMISSIVE_LICENSES,
    LICENSE_CONFLICTS,
    normalize_license,
    categorize_license,
    detect_license_conflicts,
    analyze_dependencies,
    generate_report,
    main,
)


class TestNormalizeLicense:
    """Test normalize_license function."""

    def test_normalize_empty_string(self):
        """Test normalizing empty string returns NOASSERTION."""
        assert normalize_license("") == "NOASSERTION"

    def test_normalize_none(self):
        """Test normalizing None returns NOASSERTION."""
        assert normalize_license(None) == "NOASSERTION"

    def test_normalize_apache_variations(self):
        """Test normalizing Apache 2.0 license variations."""
        assert normalize_license("apache license 2.0") == "Apache-2.0"
        assert normalize_license("Apache License 2.0") == "Apache-2.0"
        assert normalize_license("APACHE 2") == "Apache-2.0"
        assert normalize_license("apache-2") == "Apache-2.0"

    def test_normalize_mit_variations(self):
        """Test normalizing MIT license variations."""
        assert normalize_license("mit") == "MIT"
        assert normalize_license("MIT") == "MIT"
        assert normalize_license("mit license") == "MIT"
        assert normalize_license("MIT License") == "MIT"

    def test_normalize_bsd_variations(self):
        """Test normalizing BSD license variations."""
        assert normalize_license("bsd") == "BSD-3-Clause"
        assert normalize_license("BSD") == "BSD-3-Clause"
        assert normalize_license("bsd license") == "BSD-3-Clause"
        assert normalize_license("BSD License") == "BSD-3-Clause"

    def test_normalize_with_whitespace(self):
        """Test normalizing license with leading/trailing whitespace."""
        assert normalize_license("  MIT  ") == "MIT"
        assert normalize_license("\tApache 2\n") == "Apache-2.0"

    def test_normalize_unknown_license(self):
        """Test normalizing unknown license returns as-is."""
        assert normalize_license("Custom-License-1.0") == "Custom-License-1.0"
        assert normalize_license("Proprietary") == "Proprietary"


class TestCategorizeLicense:
    """Test categorize_license function."""

    @pytest.mark.parametrize(
        "license_id",
        [
            "GPL-2.0",
            "GPL-2.0-only",
            "GPL-3.0",
            "AGPL-3.0",
            "LGPL-2.1",
            "LGPL-3.0",
        ],
        ids=["GPL-2.0", "GPL-2.0-only", "GPL-3.0", "AGPL-3.0", "LGPL-2.1", "LGPL-3.0"],
    )
    def test_categorize_copyleft_licenses(self, license_id):
        """Test categorizing copyleft licenses."""
        assert categorize_license(license_id) == "copyleft"

    @pytest.mark.parametrize(
        "license_id",
        [
            "MIT",
            "Apache-2.0",
            "Apache-1.1",
            "BSD-2-Clause",
            "BSD-3-Clause",
            "ISC",
            "0BSD",
        ],
        ids=["MIT", "Apache-2.0", "Apache-1.1", "BSD-2", "BSD-3", "ISC", "0BSD"],
    )
    def test_categorize_permissive_licenses(self, license_id):
        """Test categorizing permissive licenses."""
        assert categorize_license(license_id) == "permissive"

    @pytest.mark.parametrize(
        "license_id",
        ["NOASSERTION", "NONE", ""],
        ids=["NOASSERTION", "NONE", "empty"],
    )
    def test_categorize_unknown_licenses(self, license_id):
        """Test categorizing unknown licenses."""
        assert categorize_license(license_id) == "unknown"

    def test_categorize_other_license(self):
        """Test categorizing other licenses."""
        assert categorize_license("Custom-License") == "other"
        assert categorize_license("Proprietary") == "other"


class TestDetectLicenseConflicts:
    """Test detect_license_conflicts function."""

    def test_detect_no_conflicts(self):
        """Test no conflicts when compatible licenses are used."""
        licenses = ["MIT", "Apache-2.0", "BSD-3-Clause"]
        conflicts = detect_license_conflicts(licenses)
        assert conflicts == []

    def test_detect_gpl2_apache2_conflict(self):
        """Test detecting GPL-2.0 and Apache-2.0 conflict."""
        licenses = ["GPL-2.0", "Apache-2.0"]
        conflicts = detect_license_conflicts(licenses)

        assert len(conflicts) == 1
        assert conflicts[0]["severity"] == "high"
        assert "GPL-2.0" in conflicts[0]["licenses"]
        assert "Apache-2.0" in conflicts[0]["licenses"]

    def test_detect_gpl3_apache11_conflict(self):
        """Test detecting GPL-3.0 and Apache-1.1 conflict."""
        licenses = ["GPL-3.0", "Apache-1.1"]
        conflicts = detect_license_conflicts(licenses)

        assert len(conflicts) == 1
        assert conflicts[0]["severity"] == "high"

    def test_detect_conflict_with_only_variant(self):
        """Test detecting conflict with GPL-2.0-only variant."""
        licenses = ["GPL-2.0-only", "Apache-2.0"]
        conflicts = detect_license_conflicts(licenses)

        assert len(conflicts) == 1

    def test_detect_no_conflict_with_single_license(self):
        """Test no conflict with single license."""
        licenses = ["GPL-2.0"]
        conflicts = detect_license_conflicts(licenses)
        assert conflicts == []

    def test_detect_empty_licenses(self):
        """Test empty licenses list returns no conflicts."""
        conflicts = detect_license_conflicts([])
        assert conflicts == []

    def test_detect_duplicate_licenses_no_conflict(self):
        """Test duplicate licenses don't cause conflicts."""
        licenses = ["MIT", "MIT", "Apache-2.0", "Apache-2.0"]
        conflicts = detect_license_conflicts(licenses)
        assert conflicts == []


class TestAnalyzeDependencies:
    """Test analyze_dependencies function."""

    def test_analyze_empty_dependencies(self):
        """Test analyzing empty dependencies list."""
        result = analyze_dependencies([])

        assert result["total_dependencies"] == 0
        assert result["unique_licenses"] == 0
        assert result["license_distribution"] == {}
        assert result["copyleft_licenses"] == 0
        assert result["permissive_licenses"] == 0
        assert result["unknown_licenses"] == 0
        assert result["copyleft_dependencies"] == []
        assert result["unknown_license_dependencies"] == []
        assert result["conflicts"] == []

    def test_analyze_dependencies_with_license_field(self):
        """Test analyzing dependencies with 'license' field."""
        dependencies = [
            {"name": "pkg1", "version": "1.0", "license": "MIT"},
            {"name": "pkg2", "version": "2.0", "license": "Apache-2.0"},
            {"name": "pkg3", "version": "3.0", "license": "GPL-2.0"},
        ]

        result = analyze_dependencies(dependencies)

        assert result["total_dependencies"] == 3
        assert result["unique_licenses"] == 3
        assert result["permissive_licenses"] == 2
        assert result["copyleft_licenses"] == 1
        assert len(result["copyleft_dependencies"]) == 1
        assert result["copyleft_dependencies"][0]["name"] == "pkg3"

    def test_analyze_dependencies_with_licenseDeclared_field(self):
        """Test analyzing dependencies with 'licenseDeclared' field."""
        dependencies = [
            {"name": "pkg1", "version": "1.0", "licenseDeclared": "MIT"},
        ]

        result = analyze_dependencies(dependencies)

        assert result["total_dependencies"] == 1
        assert result["unique_licenses"] == 1

    def test_analyze_dependencies_with_unknown_licenses(self):
        """Test analyzing dependencies with unknown licenses."""
        dependencies = [
            {"name": "pkg1", "version": "1.0", "license": "NOASSERTION"},
            {"name": "pkg2", "version": "2.0", "license": ""},
        ]

        result = analyze_dependencies(dependencies)

        assert result["unknown_licenses"] == 1  # Both normalize to NOASSERTION
        assert len(result["unknown_license_dependencies"]) == 2

    def test_analyze_dependencies_with_conflicts(self):
        """Test analyzing dependencies with conflicting licenses."""
        dependencies = [
            {"name": "pkg1", "version": "1.0", "license": "GPL-2.0"},
            {"name": "pkg2", "version": "2.0", "license": "Apache-2.0"},
        ]

        result = analyze_dependencies(dependencies)

        assert len(result["conflicts"]) == 1

    def test_analyze_dependencies_with_purl(self):
        """Test analyzing dependencies with PURL field."""
        dependencies = [
            {
                "name": "pkg1",
                "version": "1.0",
                "license": "GPL-3.0",
                "purl": "pkg:maven/com.example/pkg1@1.0",
            },
        ]

        result = analyze_dependencies(dependencies)

        assert result["copyleft_dependencies"][0]["purl"] == "pkg:maven/com.example/pkg1@1.0"

    def test_analyze_dependencies_license_distribution(self):
        """Test license distribution counting."""
        dependencies = [
            {"name": "pkg1", "version": "1.0", "license": "MIT"},
            {"name": "pkg2", "version": "2.0", "license": "MIT"},
            {"name": "pkg3", "version": "3.0", "license": "Apache-2.0"},
        ]

        result = analyze_dependencies(dependencies)

        assert result["license_distribution"]["MIT"] == 2
        assert result["license_distribution"]["Apache-2.0"] == 1

    def test_analyze_dependencies_normalizes_licenses(self):
        """Test that license normalization is applied."""
        dependencies = [
            {"name": "pkg1", "version": "1.0", "license": "mit license"},
            {"name": "pkg2", "version": "2.0", "license": "apache 2"},
        ]

        result = analyze_dependencies(dependencies)

        assert "MIT" in result["license_distribution"]
        assert "Apache-2.0" in result["license_distribution"]

    def test_analyze_dependencies_missing_name_version(self):
        """Test analyzing dependencies with missing name/version."""
        dependencies = [
            {"license": "MIT"},
        ]

        result = analyze_dependencies(dependencies)

        assert result["total_dependencies"] == 1
        # Should not crash, defaults to "unknown"

    def test_analyze_dependencies_multiple_copyleft(self):
        """Test analyzing multiple copyleft dependencies."""
        dependencies = [
            {"name": "pkg1", "version": "1.0", "license": "GPL-2.0"},
            {"name": "pkg2", "version": "2.0", "license": "GPL-3.0"},
            {"name": "pkg3", "version": "3.0", "license": "LGPL-2.1"},
        ]

        result = analyze_dependencies(dependencies)

        assert result["copyleft_licenses"] == 3
        assert len(result["copyleft_dependencies"]) == 3


class TestGenerateReport:
    """Test generate_report function."""

    def test_generate_basic_report(self, tmp_path, capsys):
        """Test generating basic report without flags."""
        output_file = tmp_path / "report.json"
        analysis = {
            "total_dependencies": 5,
            "unique_licenses": 3,
            "copyleft_licenses": 1,
            "permissive_licenses": 2,
            "unknown_licenses": 0,
            "license_distribution": {"MIT": 2, "Apache-2.0": 2, "GPL-2.0": 1},
            "copyleft_dependencies": [],
            "unknown_license_dependencies": [],
            "conflicts": [],
        }

        generate_report(analysis, str(output_file), False, False)

        assert output_file.exists()
        with open(output_file) as f:
            report = json.load(f)

        assert report["version"] == "1.0"
        assert report["summary"]["total_dependencies"] == 5
        assert report["summary"]["unique_licenses"] == 3
        assert "copyleft_dependencies" not in report

        captured = capsys.readouterr()
        assert "License analysis report written" in captured.err

    def test_generate_report_with_copyleft_flag(self, tmp_path, capsys):
        """Test generating report with copyleft flag enabled."""
        output_file = tmp_path / "report.json"
        analysis = {
            "total_dependencies": 5,
            "unique_licenses": 3,
            "copyleft_licenses": 1,
            "permissive_licenses": 2,
            "unknown_licenses": 0,
            "license_distribution": {"MIT": 2, "Apache-2.0": 2, "GPL-2.0": 1},
            "copyleft_dependencies": [
                {"name": "pkg1", "version": "1.0", "license": "GPL-2.0", "purl": ""},
            ],
            "unknown_license_dependencies": [],
            "conflicts": [],
        }

        generate_report(analysis, str(output_file), False, True)

        with open(output_file) as f:
            report = json.load(f)

        assert "copyleft_dependencies" in report
        assert len(report["copyleft_dependencies"]) == 1

        captured = capsys.readouterr()
        assert "Found 1 copyleft dependencies" in captured.err

    def test_generate_report_with_unknown_licenses(self, tmp_path, capsys):
        """Test generating report with unknown licenses."""
        output_file = tmp_path / "report.json"
        analysis = {
            "total_dependencies": 2,
            "unique_licenses": 1,
            "copyleft_licenses": 0,
            "permissive_licenses": 0,
            "unknown_licenses": 1,
            "license_distribution": {"NOASSERTION": 2},
            "copyleft_dependencies": [],
            "unknown_license_dependencies": [
                {"name": "pkg1", "version": "1.0", "license": "NOASSERTION", "purl": ""},
                {"name": "pkg2", "version": "2.0", "license": "NOASSERTION", "purl": ""},
            ],
            "conflicts": [],
        }

        generate_report(analysis, str(output_file), False, False)

        with open(output_file) as f:
            report = json.load(f)

        assert "unknown_license_dependencies" in report
        assert len(report["unknown_license_dependencies"]) == 2

        captured = capsys.readouterr()
        assert "Found 2 dependencies with unknown licenses" in captured.err

    def test_generate_report_with_conflicts(self, tmp_path, capsys):
        """Test generating report with conflicts."""
        output_file = tmp_path / "report.json"
        analysis = {
            "total_dependencies": 2,
            "unique_licenses": 2,
            "copyleft_licenses": 1,
            "permissive_licenses": 1,
            "unknown_licenses": 0,
            "license_distribution": {"GPL-2.0": 1, "Apache-2.0": 1},
            "copyleft_dependencies": [],
            "unknown_license_dependencies": [],
            "conflicts": [
                {
                    "licenses": ["GPL-2.0", "Apache-2.0"],
                    "reason": "Incompatible",
                    "severity": "high",
                }
            ],
        }

        generate_report(analysis, str(output_file), True, False)

        with open(output_file) as f:
            report = json.load(f)

        assert "conflicts" in report
        assert len(report["conflicts"]) == 1

        captured = capsys.readouterr()
        assert "Found 1 license conflicts" in captured.err

    def test_generate_report_no_copyleft_without_flag(self, tmp_path):
        """Test report doesn't include copyleft deps without flag."""
        output_file = tmp_path / "report.json"
        analysis = {
            "total_dependencies": 1,
            "unique_licenses": 1,
            "copyleft_licenses": 1,
            "permissive_licenses": 0,
            "unknown_licenses": 0,
            "license_distribution": {"GPL-2.0": 1},
            "copyleft_dependencies": [
                {"name": "pkg1", "version": "1.0", "license": "GPL-2.0", "purl": ""},
            ],
            "unknown_license_dependencies": [],
            "conflicts": [],
        }

        generate_report(analysis, str(output_file), False, False)

        with open(output_file) as f:
            report = json.load(f)

        assert "copyleft_dependencies" not in report


class TestMainFunction:
    """Test main CLI function."""

    def test_main_with_dict_dependencies_key(self, tmp_path, monkeypatch):
        """Test main with dict input using 'dependencies' key."""
        input_file = tmp_path / "input.json"
        output_file = tmp_path / "output.json"

        input_data = {
            "dependencies": [
                {"name": "pkg1", "version": "1.0", "license": "MIT"},
            ]
        }
        input_file.write_text(json.dumps(input_data))

        monkeypatch.setattr(
            "sys.argv",
            [
                "license_analyzer.py",
                "--input",
                str(input_file),
                "--output",
                str(output_file),
            ],
        )

        main()

        assert output_file.exists()
        with open(output_file) as f:
            report = json.load(f)
        assert report["summary"]["total_dependencies"] == 1

    def test_main_with_dict_packages_key(self, tmp_path, monkeypatch):
        """Test main with dict input using 'packages' key."""
        input_file = tmp_path / "input.json"
        output_file = tmp_path / "output.json"

        input_data = {
            "packages": [
                {"name": "pkg1", "version": "1.0", "license": "Apache-2.0"},
            ]
        }
        input_file.write_text(json.dumps(input_data))

        monkeypatch.setattr(
            "sys.argv",
            [
                "license_analyzer.py",
                "--input",
                str(input_file),
                "--output",
                str(output_file),
            ],
        )

        main()

        assert output_file.exists()

    def test_main_with_list_input(self, tmp_path, monkeypatch):
        """Test main with list input."""
        input_file = tmp_path / "input.json"
        output_file = tmp_path / "output.json"

        input_data = [
            {"name": "pkg1", "version": "1.0", "license": "MIT"},
            {"name": "pkg2", "version": "2.0", "license": "Apache-2.0"},
        ]
        input_file.write_text(json.dumps(input_data))

        monkeypatch.setattr(
            "sys.argv",
            [
                "license_analyzer.py",
                "--input",
                str(input_file),
                "--output",
                str(output_file),
            ],
        )

        main()

        assert output_file.exists()
        with open(output_file) as f:
            report = json.load(f)
        assert report["summary"]["total_dependencies"] == 2

    def test_main_with_check_conflicts_flag(self, tmp_path, monkeypatch):
        """Test main with --check-conflicts flag."""
        input_file = tmp_path / "input.json"
        output_file = tmp_path / "output.json"

        input_data = [
            {"name": "pkg1", "version": "1.0", "license": "GPL-2.0"},
            {"name": "pkg2", "version": "2.0", "license": "Apache-2.0"},
        ]
        input_file.write_text(json.dumps(input_data))

        monkeypatch.setattr(
            "sys.argv",
            [
                "license_analyzer.py",
                "--input",
                str(input_file),
                "--output",
                str(output_file),
                "--check-conflicts",
            ],
        )

        main()

        assert output_file.exists()
        with open(output_file) as f:
            report = json.load(f)
        assert "conflicts" in report

    def test_main_with_flag_copyleft_flag(self, tmp_path, monkeypatch):
        """Test main with --flag-copyleft flag."""
        input_file = tmp_path / "input.json"
        output_file = tmp_path / "output.json"

        input_data = [
            {"name": "pkg1", "version": "1.0", "license": "GPL-3.0"},
        ]
        input_file.write_text(json.dumps(input_data))

        monkeypatch.setattr(
            "sys.argv",
            [
                "license_analyzer.py",
                "--input",
                str(input_file),
                "--output",
                str(output_file),
                "--flag-copyleft",
            ],
        )

        main()

        assert output_file.exists()
        with open(output_file) as f:
            report = json.load(f)
        assert "copyleft_dependencies" in report

    def test_main_with_nonexistent_input_file(self, tmp_path, monkeypatch, capsys):
        """Test main with nonexistent input file."""
        input_file = tmp_path / "nonexistent.json"
        output_file = tmp_path / "output.json"

        monkeypatch.setattr(
            "sys.argv",
            [
                "license_analyzer.py",
                "--input",
                str(input_file),
                "--output",
                str(output_file),
            ],
        )

        with pytest.raises(SystemExit) as exc_info:
            main()

        assert exc_info.value.code == 1
        captured = capsys.readouterr()
        assert "Error analyzing licenses" in captured.err

    def test_main_with_invalid_json(self, tmp_path, monkeypatch, capsys):
        """Test main with invalid JSON input."""
        input_file = tmp_path / "invalid.json"
        output_file = tmp_path / "output.json"

        input_file.write_text("{invalid json")

        monkeypatch.setattr(
            "sys.argv",
            [
                "license_analyzer.py",
                "--input",
                str(input_file),
                "--output",
                str(output_file),
            ],
        )

        with pytest.raises(SystemExit) as exc_info:
            main()

        assert exc_info.value.code == 1
        captured = capsys.readouterr()
        assert "Error analyzing licenses" in captured.err

    def test_main_without_required_args(self, monkeypatch):
        """Test main without required arguments."""
        monkeypatch.setattr(
            "sys.argv",
            ["license_analyzer.py"],
        )

        with pytest.raises(SystemExit) as exc_info:
            main()

        assert exc_info.value.code == 2  # argparse error


class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_normalize_license_with_newlines(self):
        """Test normalizing license with newlines."""
        result = normalize_license("\n\nmit\n\n")
        assert result == "MIT"

    def test_analyze_dependencies_with_empty_dict(self):
        """Test analyzing dependencies with empty dict."""
        dependencies = [{}]

        result = analyze_dependencies(dependencies)

        assert result["total_dependencies"] == 1
        assert result["unknown_licenses"] == 1

    def test_analyze_dependencies_with_unicode(self):
        """Test analyzing dependencies with Unicode characters."""
        dependencies = [
            {"name": "café-☕", "version": "1.0", "license": "MIT"},
        ]

        result = analyze_dependencies(dependencies)

        assert result["total_dependencies"] == 1

    def test_detect_conflicts_with_many_licenses(self):
        """Test conflict detection with many licenses."""
        licenses = ["MIT", "Apache-2.0", "BSD-3-Clause", "ISC"] * 10
        conflicts = detect_license_conflicts(licenses)

        # No conflicts even with many duplicates
        assert conflicts == []

    def test_categorize_license_case_sensitive(self):
        """Test that categorize_license is case-sensitive."""
        # SPDX IDs are case-sensitive
        assert categorize_license("mit") == "other"  # lowercase not in set
        assert categorize_license("MIT") == "permissive"  # uppercase is correct

    def test_analyze_dependencies_very_long_license_name(self):
        """Test handling very long license names."""
        dependencies = [
            {"name": "pkg1", "version": "1.0", "license": "A" * 1000},
        ]

        result = analyze_dependencies(dependencies)

        assert result["total_dependencies"] == 1
        assert "A" * 1000 in result["license_distribution"]
