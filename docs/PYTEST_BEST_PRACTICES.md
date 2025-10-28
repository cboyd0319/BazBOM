# PyTest Best Practices for BazBOM

## Overview

This document outlines pytest best practices specifically tailored for the BazBOM test suite. Following these practices ensures tests are **fast, deterministic, maintainable, and provide high signal**.

## Core Principles

### 1. Use Pure Pytest Style (Not unittest)

**❌ DON'T use unittest.TestCase**:
```python
import unittest

class TestMyFeature(unittest.TestCase):
    def setUp(self):
        self.data = load_data()
    
    def tearDown(self):
        cleanup()
    
    def test_something(self):
        self.assertEqual(process(self.data), expected)
```

**✅ DO use pytest style**:
```python
import pytest

@pytest.fixture
def data():
    return load_data()

def test_something(data):
    assert process(data) == expected
```

**Why pytest is better**:
- No class boilerplate (30% less code)
- Better assertion messages
- Automatic fixture cleanup
- More composable fixtures
- Parametrization support
- Better error reporting

### 2. Leverage Fixtures for Setup

**Fixture Scopes** (choose based on cost and isolation needs):
```python
@pytest.fixture(scope="function")  # Default, created per test
def per_test_data():
    return {"value": 1}

@pytest.fixture(scope="session")  # Created once per test session
def expensive_data():
    # Expensive operation (database load, large file parsing)
    return load_large_dataset()

@pytest.fixture(scope="module")  # Created once per test module
def module_data():
    return configure_module_state()
```

**Use session scope for immutable, expensive fixtures**:
```python
@pytest.fixture(scope="session")
def sample_sbom_data():
    """Large SBOM fixture created once."""
    return {
        "spdxVersion": "SPDX-2.3",
        "packages": [...]  # 1000+ packages
    }

def test_sbom_parser(sample_sbom_data):
    # Make a copy if you need to modify
    modified = dict(sample_sbom_data)
    modified["packages"].append(new_package)
    assert parse_sbom(modified) is not None
```

### 3. Factory Fixtures for Reusability

**❌ DON'T repeat mock setup**:
```python
def test_api_success():
    mock = Mock()
    mock.status_code = 200
    mock.json.return_value = {"key": "value"}
    # ... test logic

def test_api_failure():
    mock = Mock()
    mock.status_code = 500
    mock.json.side_effect = ValueError()
    # ... test logic
```

**✅ DO use factory fixtures**:
```python
@pytest.fixture
def mock_http_response():
    """Factory for creating mock HTTP responses."""
    def _create(status=200, data=None, error=None):
        mock = Mock()
        mock.status_code = status
        if error:
            mock.json.side_effect = error
        else:
            mock.json.return_value = data
        return mock
    return _create

def test_api_success(mock_http_response):
    response = mock_http_response(200, {"key": "value"})
    assert response.status_code == 200

def test_api_failure(mock_http_response):
    response = mock_http_response(500, error=ValueError())
    with pytest.raises(ValueError):
        response.json()
```

### 4. Parametrize Instead of Duplicate

**❌ DON'T duplicate tests**:
```python
def test_parse_version_1():
    assert parse_version("1.0.0") == (1, 0, 0)

def test_parse_version_2():
    assert parse_version("2.1.3") == (2, 1, 3)

def test_parse_version_3():
    assert parse_version("10.20.30") == (10, 20, 30)
```

**✅ DO parametrize**:
```python
@pytest.mark.parametrize("version_str,expected", [
    ("1.0.0", (1, 0, 0)),
    ("2.1.3", (2, 1, 3)),
    ("10.20.30", (10, 20, 30)),
], ids=["v1", "v2", "v10"])
def test_parse_version(version_str, expected):
    assert parse_version(version_str) == expected
```

**Benefits**:
- Single test function instead of 3
- Easy to add more test cases
- Clear test IDs for debugging
- Less code to maintain

### 5. Use tmp_path for File Operations

**❌ DON'T use manual tempfile**:
```python
import tempfile
import shutil

def test_file_processing():
    temp_dir = tempfile.mkdtemp()
    try:
        file_path = os.path.join(temp_dir, "test.json")
        with open(file_path, 'w') as f:
            json.dump(data, f)
        # ... test logic
    finally:
        shutil.rmtree(temp_dir)
```

**✅ DO use tmp_path fixture**:
```python
def test_file_processing(tmp_path):
    file_path = tmp_path / "test.json"
    file_path.write_text(json.dumps(data))
    # ... test logic
    # Automatic cleanup by pytest
```

**Even better - use temp_json_file fixture**:
```python
def test_file_processing(temp_json_file, tmp_path):
    file_path = temp_json_file({"key": "value"}, "test.json")
    # File is created and will be auto-cleaned
    result = process_file(file_path)
    assert result is not None
```

### 6. AAA Pattern (Arrange-Act-Assert)

**Structure every test clearly**:
```python
def test_sbom_generation(sample_packages):
    # Arrange - set up test data
    packages = sample_packages
    sbom_name = "test-sbom"
    
    # Act - execute the function under test
    result = generate_spdx_document(packages, sbom_name)
    
    # Assert - verify the outcome
    assert result["spdxVersion"] == "SPDX-2.3"
    assert result["name"] == sbom_name
    assert len(result["packages"]) >= len(packages)
```

### 7. Test Edge Cases and Error Paths

**Test the happy path AND edge cases**:
```python
# Happy path
def test_parse_purl_valid():
    purl = "pkg:maven/com.google.guava/guava@31.1-jre"
    assert parse_purl(purl)["name"] == "guava"

# Edge cases
@pytest.mark.parametrize("invalid_purl", [
    "",                    # Empty string
    "not-a-purl",         # Invalid format
    "pkg:npm/lodash",     # Wrong ecosystem
    None,                 # None value
], ids=["empty", "invalid", "wrong_ecosystem", "none"])
def test_parse_purl_edge_cases(invalid_purl):
    result = parse_purl(invalid_purl)
    assert result == {} or result is None

# Error paths
def test_parse_purl_raises_on_invalid_type():
    with pytest.raises(TypeError):
        parse_purl(123)  # Should raise TypeError
```

### 8. Deterministic Tests

**Always seed randomness**:
```python
# In conftest.py (autouse fixture)
@pytest.fixture(autouse=True, scope="function")
def _seed_rng():
    random.seed(1337)
    try:
        import numpy as np
        np.random.seed(1337)
    except ImportError:
        pass
```

**Use time control for time-dependent tests**:
```python
# Use freezegun fixture from conftest.py
def test_timestamp_generation(freeze_time):
    with freeze_time("2025-01-01 00:00:00"):
        timestamp = generate_timestamp()
        assert timestamp == "2025-01-01T00:00:00Z"
```

**Mock external dependencies**:
```python
def test_api_call(mocker, mock_http_response):
    # Mock the actual HTTP call
    mock_resp = mock_http_response(200, {"data": "value"})
    mocker.patch('module.requests.get', return_value=mock_resp)
    
    # Test code that calls requests.get
    result = fetch_data()
    assert result["data"] == "value"
```

### 9. Use Markers for Test Organization

**Mark slow tests**:
```python
@pytest.mark.slow
@pytest.mark.performance
def test_process_large_dataset():
    # Test that processes 10,000 items
    result = process(large_dataset)
    assert len(result) == 10000
```

**Run different test categories**:
```bash
# Skip slow tests during development
pytest -m "not slow"

# Run only integration tests
pytest -m integration

# Run only performance tests
pytest -m performance
```

### 10. Clear Test Names

**Test names should describe behavior**:
```python
# ❌ Poor names
def test_1():
def test_func():
def test_parse():

# ✅ Good names - describe scenario and expected outcome
def test_parse_purl_with_version_returns_package_dict():
def test_parse_purl_empty_string_returns_empty_dict():
def test_parse_purl_invalid_format_raises_value_error():
```

### 11. Keep Tests Small and Focused

**One behavior per test**:
```python
# ❌ Testing multiple behaviors
def test_sbom_document():
    doc = generate_spdx_document(packages, "test")
    assert doc["spdxVersion"] == "SPDX-2.3"  # Version
    assert doc["name"] == "test"              # Name
    assert len(doc["packages"]) > 0           # Packages
    assert doc["creationInfo"]["created"]     # Creation info
    # Too much in one test!

# ✅ One behavior per test
def test_spdx_document_has_correct_version():
    doc = generate_spdx_document([], "test")
    assert doc["spdxVersion"] == "SPDX-2.3"

def test_spdx_document_includes_all_packages():
    packages = [{"name": "pkg1"}, {"name": "pkg2"}]
    doc = generate_spdx_document(packages, "test")
    assert len(doc["packages"]) >= len(packages)

def test_spdx_document_has_creation_timestamp():
    doc = generate_spdx_document([], "test")
    assert "created" in doc["creationInfo"]
```

### 12. Performance Optimization

**Measure before optimizing**:
```bash
# Find slowest tests
pytest --durations=20

# Profile test execution
pytest --durations=0 | sort -n
```

**Use parallel execution**:
```bash
# Install pytest-xdist
pip install pytest-xdist

# Run tests in parallel
pytest -n auto  # Use all CPU cores
pytest -n 4     # Use 4 workers
```

**Optimize fixture scope**:
```python
# ❌ Expensive fixture created for every test
@pytest.fixture
def large_dataset():
    return load_1000_records()  # Slow!

# ✅ Expensive fixture created once
@pytest.fixture(scope="session")
def large_dataset():
    return load_1000_records()  # Only once!

# Tests copy if they need to modify
def test_with_modification(large_dataset):
    data = list(large_dataset)  # Make a copy
    data.append(new_record)
    assert process(data) is not None
```

### 13. Avoid Test Interdependencies

**❌ Tests should NEVER depend on each other**:
```python
# BAD - test order matters
class TestWorkflow:
    def test_1_create_user(self):
        self.user = create_user()
    
    def test_2_update_user(self):
        update_user(self.user)  # Depends on test_1!
```

**✅ Each test is independent**:
```python
@pytest.fixture
def user():
    return create_user()

def test_create_user():
    user = create_user()
    assert user is not None

def test_update_user(user):
    updated = update_user(user)
    assert updated is not None
```

### 14. Assertion Best Practices

**Use clear assertions**:
```python
# ❌ Poor assertions
assert result
assert not result
assert len(items) == 3

# ✅ Clear assertions with messages
assert result is not None, "Result should not be None"
assert result == expected, f"Expected {expected}, got {result}"
assert len(items) == 3, f"Expected 3 items, got {len(items)}"
```

**Use pytest.raises for exceptions**:
```python
# Test that function raises ValueError
def test_invalid_input_raises_error():
    with pytest.raises(ValueError, match="Invalid input"):
        process_data(invalid_input)
```

### 15. Documentation and Comments

**Docstrings for complex tests**:
```python
def test_conflict_detection_with_transitive_deps():
    """Test that conflict detector finds version conflicts in transitive dependencies.
    
    Given:
        - Package A depends on Library X v1.0
        - Package B depends on Library X v2.0
        - Both A and B are in the dependency tree
    
    When:
        - Running conflict detection
    
    Then:
        - Should detect Library X version conflict
        - Should suggest resolution strategy
    """
    # Test implementation...
```

## Performance Benchmarks

### Target Metrics
- Unit tests: < 100ms per test (typical)
- Integration tests: < 500ms per test
- Total suite: < 2.0s (with parallelization)
- Coverage: ≥ 90% line, ≥ 85% branch

### Current Performance
| Metric | Current | Target |
|--------|---------|--------|
| Total Runtime | 2.78s | <2.0s |
| Fast Tests | 2.78s | <2.0s |
| Slow Tests | 5 tests | <10 tests |
| Parallel Execution | No | Yes |

## Common Anti-Patterns to Avoid

### 1. ❌ Over-Mocking
```python
# Don't mock everything - test real interactions
def test_with_too_many_mocks(mocker):
    mocker.patch('module.func1')
    mocker.patch('module.func2')
    mocker.patch('module.func3')
    # Test becomes meaningless
```

### 2. ❌ Flaky Tests
```python
# DON'T use sleep or real time
def test_timeout():
    start = time.time()
    result = slow_operation()
    assert time.time() - start < 1.0  # Flaky!

# DO mock time or use time control
def test_timeout(freeze_time):
    with freeze_time("2025-01-01 00:00:00"):
        result = operation_with_timeout()
        assert result is not None
```

### 3. ❌ Hidden Dependencies
```python
# DON'T access global state
GLOBAL_CONFIG = {}

def test_uses_global():
    GLOBAL_CONFIG['key'] = 'value'  # Pollutes global state
    result = process()
    assert result

# DO use fixtures and parameters
def test_with_config(monkeypatch):
    monkeypatch.setattr('module.CONFIG', {'key': 'value'})
    result = process()
    assert result
```

### 4. ❌ Testing Implementation Details
```python
# DON'T test internal methods
def test_internal_cache():
    obj = MyClass()
    assert obj._internal_cache == {}  # Implementation detail

# DO test public behavior
def test_caching_improves_performance():
    obj = MyClass()
    first = obj.get_data()
    second = obj.get_data()
    # Assert behavior, not implementation
    assert first == second
```

## Checklist for New Tests

Before submitting tests, verify:

- [ ] Tests use pytest style (not unittest.TestCase)
- [ ] Fixtures are used appropriately (correct scope)
- [ ] Parametrization used instead of duplicate tests
- [ ] Tests use tmp_path for file operations
- [ ] AAA pattern followed (Arrange-Act-Assert)
- [ ] Edge cases and error paths tested
- [ ] Tests are deterministic (seeded RNG, mocked time)
- [ ] External dependencies mocked
- [ ] Tests are independent (no shared state)
- [ ] Test names are descriptive
- [ ] One behavior per test
- [ ] Tests run in < 100ms (or marked as @pytest.mark.slow)
- [ ] Assertions have clear messages
- [ ] Docstrings for complex test scenarios

## References

- [pytest documentation](https://docs.pytest.org/)
- [pytest fixtures](https://docs.pytest.org/en/stable/fixture.html)
- [pytest parametrize](https://docs.pytest.org/en/stable/parametrize.html)
- [pytest-xdist for parallel execution](https://pytest-xdist.readthedocs.io/)
- [BazBOM TEST_PLAN.md](./TEST_PLAN.md)
- [BazBOM Testing Guide](testing/TESTING_GUIDE.md)
