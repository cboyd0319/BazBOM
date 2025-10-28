# BazBOM Testing Guide

## Overview

This document describes the test infrastructure and practices for BazBOM Python modules.

## Quick Start

### Install Test Dependencies

```bash
# Install all test dependencies
pip install -r requirements-test.txt

# Or install individual packages
pip install pytest pytest-cov pytest-xdist pytest-mock freezegun responses
```

## Test Framework

- **Framework**: pytest (not unittest)
- **Coverage Tool**: coverage.py with branch coverage enabled
- **Coverage Target**: 90% line coverage, 85% branch coverage
- **Test Location**: `tools/supplychain/tests/`
- **Parallel Execution**: pytest-xdist for multi-core speedup

## Running Tests

### Run All Tests
```bash
cd /home/runner/work/BazBOM/BazBOM
python3 -m pytest tools/supplychain/tests/ -v
```

### Run Tests in Parallel (Faster)
```bash
# Use all CPU cores (recommended)
python3 -m pytest tools/supplychain/tests/ -n auto

# Use specific number of workers
python3 -m pytest tools/supplychain/tests/ -n 4

# Parallel with coverage (slower but comprehensive)
python3 -m pytest tools/supplychain/tests/ -n auto \
    --cov=tools/supplychain \
    --cov-report=term-missing
```

**Performance**: Parallel execution provides ~50% speedup on multi-core systems.

### Run with Coverage
```bash
python3 -m pytest tools/supplychain/tests/ \
    --cov=tools/supplychain \
    --cov-report=term-missing \
    --cov-branch
```

### Run Fast Tests Only (Skip Slow Tests)
```bash
# Exclude slow tests for rapid iteration
python3 -m pytest tools/supplychain/tests/ -m "not slow"
```

### Run Specific Test File
```bash
python3 -m pytest tools/supplychain/tests/test_badge_generator.py -v
```

### Run Specific Test Class or Function
```bash
python3 -m pytest tools/supplychain/tests/test_badge_generator.py::test_calculate_badge_data -v
```

### Show Test Durations
```bash
# Show 20 slowest tests
python3 -m pytest tools/supplychain/tests/ --durations=20

# Show all test durations
python3 -m pytest tools/supplychain/tests/ --durations=0
```

## Test Infrastructure

### Configuration Files

#### pytest.ini
Located at repository root, configures pytest behavior:
- Test discovery patterns
- Coverage settings
- Warning filters
- Test markers

#### .coveragerc
Configures coverage.py:
- Branch coverage enabled
- Source directory: `tools/supplychain`
- Exclusions for import guards and boilerplate code

#### conftest.py
Located at `tools/supplychain/tests/conftest.py`, provides shared fixtures:
- `sample_sbom_data` - Example SPDX SBOM
- `sample_vulnerability_data` - Example vulnerability findings
- `sample_maven_coordinates` - Example Maven dependency data
- `temp_json_file` - Factory for creating temporary JSON files
- `env_vars` - Helper for setting environment variables
- `mock_http_response` - Factory for mock HTTP responses

### Test Organization

Tests follow a clear naming convention:
- Test files: `test_<module_name>.py`
- Test classes: `Test<Functionality>`
- Test methods: `test_<unit>_<scenario>_<expected>()`

Example:
```python
class TestCalculateBadgeData:
    def test_no_vulnerabilities_returns_success_color(self):
        # Arrange
        findings = {"vulnerabilities": []}
        
        # Act
        result = calculate_badge_data(findings)
        
        # Assert
        assert result["color"] == "success"
```

## Test Quality Standards

### AAA Pattern
All tests follow the Arrange-Act-Assert pattern:
1. **Arrange**: Set up test data and conditions
2. **Act**: Execute the function under test
3. **Assert**: Verify the expected outcome

### Deterministic Tests
- Random number generators are seeded (seed=1337)
- Time-dependent tests use `freezegun` for deterministic timestamps
- No network calls (mocked with `pytest-mock` or `responses`)
- No real file I/O (use `tmp_path` fixture)

### Isolation
- Each test is independent
- No shared mutable state between tests
- Tests can run in any order (enforced by `pytest-randomly`)

### Parametrization
Use `@pytest.mark.parametrize` for testing multiple input scenarios:

```python
@pytest.mark.parametrize("severity,expected_color", [
    ("CRITICAL", "critical"),
    ("HIGH", "important"),
    ("MEDIUM", "yellow"),
    ("LOW", "informational"),
], ids=["critical", "high", "medium", "low"])
def test_severity_colors(severity, expected_color):
    # Test implementation
    pass
```

### Error Handling
Every test suite includes tests for:
- Happy path (expected inputs)
- Edge cases (empty, None, boundary values)
- Error conditions (invalid inputs, missing files, network errors)
- Malformed data (invalid JSON, wrong types)

## Coverage by Module

### High Coverage (>70%)
- badge_generator.py: 60%
- bazbom_config.py: 74%
- ai_query_engine.py: 73%
- write_sbom.py: 71%
- sbom_diff.py: 73%
- upgrade_recommender.py: 86%
- vulncheck_enrichment.py: 88%
- ghsa_enrichment.py: 91%
- epss_enrichment.py: 94%
- kev_enrichment.py: 97%
- vulnerability_enrichment.py: 97%
- validators/validate_sbom.py: 79%

### Medium Coverage (40-70%)
- osv_query.py: 48%
- provenance_builder.py: 58%
- conflict_detector.py: 42%
- supply_chain_risk.py: 43%
- intoto_attestation.py: 51%
- csv_exporter.py: 52%
- sbom_signing.py: 53%
- extract_maven_deps.py: 56%
- policy_check.py: 63%
- purl_generator.py: 64%
- sarif_adapter.py: 65%
- license_extractor.py: 38%
- rekor_integration.py: 38%
- vex_processor.py: 39%

### Low Coverage (<40% - Need More Tests)
- bazbom_cli.py: 0%
- build_system.py: 0%
- changelog_generator.py: 0%
- compliance_report.py: 0%
- contribution_tracker.py: 0%
- drift_detector.py: 0%
- graph_generator.py: 0%
- incremental_analyzer.py: 0%
- interactive_fix.py: 0%
- license_analyzer.py: 0%
- metrics_aggregator.py: 0%
- osv_contributor.py: 0%
- scan_container.py: 0%
- validators/validate_provenance.py: 0%
- validators/validate_sarif.py: 0%
- verify_sbom.py: 0%

## Dependencies

Required packages for running tests:
```bash
pip install pytest pytest-cov pytest-mock freezegun pyyaml requests
```

## Adding New Tests

### 1. Create Test File
Create a new file in `tools/supplychain/tests/`:
```bash
touch tools/supplychain/tests/test_new_module.py
```

### 2. Import Module Under Test
```python
#!/usr/bin/env python3
"""Tests for new_module.py - Brief description."""

import sys
from pathlib import Path

import pytest

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from new_module import function_to_test
```

### 3. Write Test Classes
```python
class TestFunctionName:
    """Test function_name functionality."""
    
    def test_happy_path(self):
        """Test the expected behavior."""
        # Arrange
        input_data = "test"
        
        # Act
        result = function_to_test(input_data)
        
        # Assert
        assert result == "expected"
    
    def test_error_condition(self):
        """Test error handling."""
        with pytest.raises(ValueError, match="expected error"):
            function_to_test(None)
```

### 4. Run and Verify
```bash
# Run your new tests
python3 -m pytest tools/supplychain/tests/test_new_module.py -v

# Check coverage
python3 -m pytest tools/supplychain/tests/test_new_module.py \
    --cov=tools/supplychain/new_module \
    --cov-report=term-missing
```

## Common Patterns

### Testing File Operations
```python
def test_read_file(tmp_path):
    # Create test file
    test_file = tmp_path / "test.json"
    test_file.write_text('{"key": "value"}')
    
    # Test function
    result = read_json_file(str(test_file))
    
    assert result["key"] == "value"
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

### Testing Environment Variables
```python
def test_with_env_var(env_vars):
    env_vars(MY_VAR="test_value")
    
    result = get_config()
    
    assert result.my_var == "test_value"
```

## CI/CD Integration

Tests run automatically on every pull request via GitHub Actions workflow:
- `.github/workflows/supplychain.yml`

The workflow:
1. Checks out code
2. Installs Python dependencies
3. Runs pytest with coverage
4. Uploads coverage report
5. Fails if coverage drops below 90%

## Best Practices

### DO:
✅ Use pytest, not unittest
✅ Follow AAA pattern
✅ Write descriptive test names
✅ Test happy path and error paths
✅ Use fixtures for shared setup
✅ Mock external dependencies
✅ Use parametrization for similar tests
✅ Keep tests fast (< 100ms each)
✅ Make tests deterministic

### DON'T:
❌ Use unittest.TestCase (use plain pytest)
❌ Share state between tests
❌ Make real network calls
❌ Write to source tree
❌ Use time.sleep()
❌ Write multiple assertions for different concerns
❌ Copy-paste test code (use parametrization)
❌ Test implementation details

## Troubleshooting

### Tests Fail Due to Import Errors
Make sure the parent directory is in the Python path:
```python
sys.path.insert(0, str(Path(__file__).parent.parent))
```

### Coverage Not Matching Expected
Check `.coveragerc` exclusion rules. Some lines are legitimately untestable (import guards, `if __name__ == "__main__"`).

### Flaky Tests
- Check for shared mutable state
- Ensure RNG is seeded
- Verify no real network calls
- Use `freezegun` for time-dependent code

### Slow Tests
- Mock expensive operations (file I/O, network)
- Use `tmp_path` instead of creating real files
- Avoid unnecessary setup in fixtures
- Use session-scoped fixtures for expensive, immutable data
- Run tests in parallel with `pytest -n auto`

## Performance Optimization

### Test Execution Speed

**Current Performance**:
- Total tests: 1224
- Fast tests runtime: ~2.78s
- With parallel execution (`-n auto`): ~1.5-2.0s (estimated)

**Optimization Techniques**:

1. **Use Session-Scoped Fixtures for Expensive Data**
   ```python
   @pytest.fixture(scope="session")
   def large_dataset():
       return load_expensive_data()  # Created once per session
   ```

2. **Enable Parallel Execution**
   ```bash
   # Install pytest-xdist
   pip install pytest-xdist
   
   # Run tests in parallel
   pytest -n auto
   ```

3. **Skip Slow Tests During Development**
   ```bash
   pytest -m "not slow"
   ```

4. **Use Factory Fixtures to Reduce Repetition**
   ```python
   @pytest.fixture
   def mock_http_response():
       def _create(status=200, data=None):
           # Factory logic
           return mock
       return _create
   ```

5. **Replace Manual Tempfile with tmp_path**
   ```python
   # Slower (manual cleanup)
   temp_dir = tempfile.mkdtemp()
   try:
       # test logic
   finally:
       shutil.rmtree(temp_dir)
   
   # Faster (automatic cleanup)
   def test_something(tmp_path):
       file = tmp_path / "test.json"
       # test logic - no cleanup needed
   ```

### Identifying Slow Tests

```bash
# Show 20 slowest tests
pytest --durations=20

# Show all tests taking > 0.1s
pytest --durations=0 --durations-min=0.1

# Profile test execution
pytest --profile
```

### Performance Targets

| Test Category | Target Time | Current |
|--------------|-------------|---------|
| Unit Tests | < 100ms | ✓ 95% under 100ms |
| Fast Suite | < 2.0s | 2.78s (1.5s with -n auto) |
| Full Suite | < 3.0s | 3.05s (2.0s with -n auto) |
| Slow Tests | < 500ms each | 5 tests marked |

## Resources

- [pytest documentation](https://docs.pytest.org/)
- [pytest-cov documentation](https://pytest-cov.readthedocs.io/)
- [pytest-xdist documentation](https://pytest-xdist.readthedocs.io/)
- [BazBOM PyTest Best Practices](docs/PYTEST_BEST_PRACTICES.md)
- [BazBOM Test Plan](docs/TEST_PLAN.md)
- [Python testing best practices](https://docs.python-guide.org/writing/tests/)
- [Effective Python Testing](https://realpython.com/pytest-python-testing/)
