# BazBOM Test Suite

This directory contains comprehensive pytest tests for BazBOM Python modules.

## Quick Start

### Run All Tests
```bash
cd /home/runner/work/BazBOM/BazBOM
python3 -m pytest tools/supplychain/tests/ -v
```

### Run with Coverage
```bash
python3 -m pytest tools/supplychain/tests/ \
    --cov=tools/supplychain \
    --cov-report=term-missing \
    --cov-branch
```

### Run Specific Module Tests
```bash
python3 -m pytest tools/supplychain/tests/test_badge_generator.py -v
```

## Test Structure

### Infrastructure Files
- **conftest.py** - Shared fixtures and test utilities
- **README.md** - This file

### Test Modules (6 new, 15 existing)

#### New Test Modules ✨
1. **test_badge_generator.py** (34 tests)
   - Security badge generation from vulnerability findings
   - Shields.io JSON, Markdown, and HTML badge generation
   - Severity level handling and license copyleft detection

2. **test_bazbom_config.py** (63 tests)
   - YAML configuration file loading and saving
   - Default configuration merging
   - Directory tree searching for configs
   - Environment variable integration

3. **test_license_extractor.py** (48 tests)
   - License text normalization and detection
   - Pattern matching for 10+ SPDX licenses
   - JAR manifest and embedded license file extraction
   - Error handling for corrupt JARs

4. **test_osv_query.py** (27 tests)
   - Package extraction from SPDX SBOMs
   - OSV API queries (single and batch)
   - CVSS score extraction and severity mapping
   - Network error handling

5. **test_provenance_builder.py** (18 tests)
   - SLSA provenance v1.0 attestation generation
   - Environment variable handling
   - Timestamp formatting and JSON serialization

6. **test_validate_sbom.py** (33 tests)
   - SPDX 2.3 specification validation
   - Required field checking
   - Package and relationship validation

#### Existing Test Modules
- test_ai_query_engine.py
- test_conflict_detector.py
- test_csv_exporter.py
- test_enrichment.py
- test_enrichment_integration.py
- test_extract_maven_deps.py
- test_policy_check.py
- test_purl_generator.py
- test_sarif_adapter.py
- test_sbom_diff.py
- test_signing.py
- test_supply_chain_risk.py
- test_upgrade_recommender.py
- test_vex_processor.py
- test_write_sbom.py

## Test Quality Standards

All tests in this suite follow these standards:

### Framework
- ✅ Pure pytest (no unittest.TestCase)
- ✅ AAA pattern (Arrange, Act, Assert)
- ✅ Clear, descriptive test names

### Determinism
- ✅ Seeded random number generators
- ✅ Frozen time for timestamp tests
- ✅ No real network calls (mocked)
- ✅ No real filesystem I/O (tmp_path)

### Performance
- ✅ Fast execution (<100ms per test)
- ✅ Parallel execution ready
- ✅ No unnecessary sleeps

### Coverage
- ✅ Happy path tested
- ✅ Error conditions tested
- ✅ Edge cases tested
- ✅ Boundary conditions tested

## Fixtures (conftest.py)

### Data Fixtures
- `sample_sbom_data` - Example SPDX SBOM with packages
- `sample_vulnerability_data` - Example OSV vulnerability findings
- `sample_maven_coordinates` - Example Maven dependency data

### Utility Fixtures
- `tmp_dir` / `tmp_path` - Temporary directory for test files
- `env_vars` - Helper for setting environment variables
- `mock_http_response` - Factory for creating mock HTTP responses
- `temp_json_file` - Factory for creating temporary JSON files

### Auto-used Fixtures
- `_seed_rng` - Seeds random number generators (random, numpy)

## Common Patterns

### Testing Functions with File I/O
```python
def test_read_config(temp_json_file):
    config_data = {"key": "value"}
    config_file = temp_json_file(config_data, "config.json")
    
    result = load_config(str(config_file))
    
    assert result["key"] == "value"
```

### Testing Error Conditions
```python
def test_invalid_input_raises_error(self):
    with pytest.raises(ValueError, match="expected error message"):
        function_under_test(invalid_input)
```

### Parametrized Tests
```python
@pytest.mark.parametrize("input,expected", [
    ("foo", "FOO"),
    ("bar", "BAR"),
], ids=["foo", "bar"])
def test_uppercase(input, expected):
    assert uppercase(input) == expected
```

### Mocking HTTP Requests
```python
@patch('module.requests.get')
def test_api_call(mock_get):
    mock_response = Mock()
    mock_response.json.return_value = {"data": "test"}
    mock_get.return_value = mock_response
    
    result = fetch_data()
    
    assert result == {"data": "test"}
```

## Coverage Status

**Overall Coverage:** 38.18%
**Target Coverage:** 90%

### High Coverage (>85%)
- epss_enrichment.py: 94%
- ghsa_enrichment.py: 91%
- vulncheck_enrichment.py: 88%
- upgrade_recommender.py: 86%
- kev_enrichment.py: 97%
- vulnerability_enrichment.py: 97%

### Good Coverage (70-85%)
- validators/validate_sbom.py: 79% ✨ NEW
- bazbom_config.py: 74% ✨ NEW
- ai_query_engine.py: 73%
- sbom_diff.py: 73%
- write_sbom.py: 71%

### Moderate Coverage (50-70%)
- badge_generator.py: 60% ✨ NEW
- provenance_builder.py: 58% ✨ NEW
- purl_generator.py: 64%
- sarif_adapter.py: 65%
- policy_check.py: 63%

### Needs Improvement (<50%)
- 22 modules at 0% coverage
- See TESTING.md for complete list

## Adding New Tests

1. **Create test file:**
   ```bash
   touch tools/supplychain/tests/test_your_module.py
   ```

2. **Import module under test:**
   ```python
   import sys
   from pathlib import Path
   
   sys.path.insert(0, str(Path(__file__).parent.parent))
   from your_module import function_to_test
   ```

3. **Write test class:**
   ```python
   class TestYourFunction:
       def test_happy_path(self):
           result = function_to_test("input")
           assert result == "expected"
   ```

4. **Run and verify:**
   ```bash
   pytest tools/supplychain/tests/test_your_module.py -v
   ```

## Resources

- **TESTING.md** - Comprehensive testing guide (repository root)
- **TEST_SUMMARY.md** - Project summary and metrics (repository root)
- **pytest.ini** - Test configuration (repository root)
- [pytest documentation](https://docs.pytest.org/)
- [Coverage.py documentation](https://coverage.readthedocs.io/)

## Statistics

- **Total Tests:** 561
- **New Tests Added:** 223
- **Test Code Lines:** 2,211 (new tests only)
- **Pass Rate:** 100%
- **Execution Time:** ~6 seconds
- **Deterministic:** Yes
- **Flaky Tests:** 0

---

For detailed information, see **TESTING.md** in the repository root.
