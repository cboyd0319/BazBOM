# Test Coverage Plan: Path to 100% Coverage

## Executive Summary

This document outlines the comprehensive plan to achieve 100% test coverage across all 44 core modules in the BazBOM supply chain system, following PyTest Architect standards.

### Current State (As of 2025-10-19)

- **Total core modules:** 44
- **Modules at 100%:** 4 (9%)
- **Modules at 99%:** 2 (5%)
- **Modules at 98%:** 2 (5%)
- **Modules at 90%+:** 2 (5%)
- **Modules below 90%:** 34 (77%)
- **Overall coverage:** ~63%
- **Target coverage:** 100% for all modules

### Progress Made

#### Completed Improvements
1. **contribution_tracker.py**: 24% → 99% (+75%)
   - Added 30+ comprehensive tests
   - Covers all public methods, CLI actions, error handling
   
2. **license_analyzer.py**: 98% → 99% (+1%)
   - Added edge case for dict with no known keys
   
3. **kev_enrichment.py**: 97% → 98% (+1%)
   - Added RuntimeError test for network failure
   
4. **supply_chain_risk.py**: 43% → 64% (+21%)
   - Added tests for unmaintained packages
   - Added tests for deprecated Maven artifacts
   - Added edge case tests

**Total improvements:** +98 percentage points, 70+ tests added

---

## Testing Standards (PyTest Architect)

All tests MUST follow these standards:

### 1. Test Structure
- **AAA Pattern**: Arrange → Act → Assert
- **Naming**: `test_<function>_<scenario>_<expected>()`
- **Docstrings**: Clear intent explanation for complex tests
- **One behavior per test**: No testing multiple unrelated behaviors

### 2. Coverage Requirements
- **Line coverage**: 100% for pure functions, 95%+ for I/O-heavy code
- **Branch coverage**: 90%+ for all modules
- **Mutation testing** (optional): 85%+ mutation kill rate for critical logic

### 3. Test Quality
- **Deterministic**: No time dependencies (use `freezegun`), no network (mock), seeded RNG
- **Isolated**: Each test stands alone, no shared state
- **Fast**: Unit tests < 100ms typical, < 500ms worst case
- **Readable**: Clear names, minimal fixtures, explicit assertions

### 4. Parametrization
- Use `@pytest.mark.parametrize` for input matrices
- Include `ids=` for readable test names
- Test edge cases: empty, None, zero, large, Unicode

### 5. Error Handling
- Test all exception paths
- Verify error messages
- Test error recovery

### 6. Mocking
- Mock external dependencies (network, filesystem, DBs)
- Patch at import site
- Use `pytest-mock` or `unittest.mock`

---

## Module-by-Module Test Plan

### Phase 1: Critical Low-Coverage Modules (<50%)

These modules have the highest risk and require immediate attention.

#### 1. incremental_analyzer.py (27% → 100%)

**Current gaps:**
- Lines 17-26, 76-99, 114-160, 237-238, 246-260, 274-298, 302-405

**Required tests:**
- `analyze_changed_targets()` with various git diff scenarios
- `get_affected_sboms()` with different dependency graphs
- `filter_dependencies()` edge cases
- CLI main() function with all action types
- Error handling for git operations
- Edge cases: empty diffs, large changesets, binary files

**Estimated tests needed:** 25-35

#### 2. compliance_report.py (33% → 100%)

**Current gaps:**
- Lines 58->63, 64, 197-199, 203-219, 223-235, 253-271, 287-330, 346-388, 406-448, 469-524, 533-644

**Required tests:**
- `generate_compliance_report()` for various SBOM formats
- License compliance checks
- Security compliance checks
- Export to multiple formats (JSON, CSV, PDF)
- CLI with all flags
- Error handling for malformed SBOMs

**Estimated tests needed:** 30-40

#### 3. bazbom_cli.py (35% → 100%)

**Current gaps:**
- Large CLI module with many subcommands

**Required tests:**
- Each CLI subcommand (analyze, scan, report, export, etc.)
- Argument validation
- Help text generation
- Interactive mode
- Batch mode
- Error recovery
- Integration tests for command chains

**Estimated tests needed:** 40-50

#### 4. interactive_fix.py (36% → 100%)

**Current gaps:**
- Interactive user interface components

**Required tests:**
- User input handling
- Fix suggestion generation
- Automated fix application
- User confirmation flows
- Rollback capabilities
- Error handling for failed fixes

**Estimated tests needed:** 25-35

#### 5. container_scanner.py (37% → 100%)

**Current gaps:**
- Container image scanning logic

**Required tests:**
- Docker image analysis
- Layer extraction
- Dependency discovery in containers
- Vulnerability mapping
- Support for multiple base images
- Error handling for inaccessible registries

**Estimated tests needed:** 20-30

#### 6. verify_sbom.py (40% → 100%)

**Current gaps:**
- Lines 21-23, 105, 109-110, 122, 126-127, 151-166, 183-191, 207-210, 218-240, 245-331

**Required tests:**
- SBOM signature verification
- Hash verification
- Timestamp verification
- Certificate chain validation
- Revocation checking
- CLI with all verification modes

**Estimated tests needed:** 20-25

#### 7. rekor_integration.py (42% → 100%)

**Current gaps:**
- Transparency log integration

**Required tests:**
- Rekor entry submission
- Entry retrieval
- Verification
- Offline mode
- Error handling for API failures

**Estimated tests needed:** 20-25

#### 8-14. Additional Phase 1 Modules

Similar detailed plans needed for:
- license_scanner.py (45%)
- scan_container.py (45%)
- osv_contributor.py (46%)
- dependency_verifier.py (48%)
- osv_query.py (48%)
- drift_detector.py (49%)

**Phase 1 total:** ~250-400 tests needed

---

### Phase 2: Medium-Coverage Modules (50-75%)

#### Strategy
Focus on:
1. Uncovered branches in conditional logic
2. Error handling paths
3. Edge cases in data processing
4. CLI argument combinations

**Modules:**
- cve_tracker.py (51%)
- intoto_attestation.py (51%)
- csv_exporter.py (52%)
- sbom_signing.py (53%)
- dependency_scanner.py (54%)
- extract_maven_deps.py (56%)
- badge_generator.py (60%)
- conflict_detector.py (63%)
- policy_check.py (63%)
- sarif_adapter.py (65%)
- ai_query_engine.py (73%)
- changelog_generator.py (73%)
- sbom_diff.py (73%)
- bazbom_config.py (74%)

**Phase 2 total:** ~140-250 tests needed

---

### Phase 3: High-Coverage Modules (75-99%)

#### Strategy
Focus on:
1. Missing branches in edge cases
2. Exception paths
3. Rare code paths
4. Platform-specific code

**Modules:**
- build_system.py (82%)
- graph_generator.py (85%)
- upgrade_recommender.py (86%)
- ghsa_enrichment.py (88%)
- vulncheck_enrichment.py (88%)
- epss_enrichment.py (94%)
- vulnerability_enrichment.py (97%)

**Phase 3 total:** ~20-80 tests needed

---

## Test Patterns and Templates

### Pattern 1: Testing Network-Based Functions

```python
@patch('module.requests.get')
def test_fetch_data_success(self, mock_get):
    """Test successful data fetch."""
    # Arrange
    mock_response = Mock()
    mock_response.status_code = 200
    mock_response.json.return_value = {"key": "value"}
    mock_get.return_value = mock_response
    
    # Act
    result = fetch_data("test-id")
    
    # Assert
    assert result == {"key": "value"}
    mock_get.assert_called_once()

@patch('module.requests.get')
def test_fetch_data_network_error(self, mock_get):
    """Test fetch handles network errors gracefully."""
    import requests
    mock_get.side_effect = requests.RequestException("Network error")
    
    with pytest.raises(RuntimeError, match="Failed to fetch"):
        fetch_data("test-id")
```

### Pattern 2: Testing File I/O

```python
def test_load_from_file(self, tmp_path):
    """Test loading data from file."""
    # Arrange
    data_file = tmp_path / "data.json"
    data_file.write_text('{"key": "value"}')
    
    # Act
    result = load_data(str(data_file))
    
    # Assert
    assert result["key"] == "value"

def test_load_from_nonexistent_file(self, tmp_path):
    """Test loading from nonexistent file raises error."""
    data_file = tmp_path / "nonexistent.json"
    
    with pytest.raises(FileNotFoundError):
        load_data(str(data_file))
```

### Pattern 3: Testing CLI Functions

```python
def test_main_with_args(self, monkeypatch, capsys):
    """Test CLI main function."""
    monkeypatch.setattr(
        sys, 'argv',
        ['prog', 'action', '--flag', 'value']
    )
    
    result = main()
    
    assert result == 0
    captured = capsys.readouterr()
    assert "Expected output" in captured.out
```

### Pattern 4: Parametrized Edge Cases

```python
@pytest.mark.parametrize(
    "input_val, expected",
    [
        ("", "default"),           # empty
        (None, "default"),         # None
        ("value", "value"),        # normal
        ("a" * 1000, "trimmed"),  # large
        ("日本語", "日本語"),       # Unicode
    ],
    ids=["empty", "none", "normal", "large", "unicode"]
)
def test_process_value(input_val, expected):
    """Test value processing with various inputs."""
    result = process(input_val)
    assert result == expected
```

---

## Execution Strategy

### Recommended Order

1. **Quick wins** (Phase 3): Complete high-coverage modules first
   - Builds momentum
   - Establishes patterns
   - ~1-2 days

2. **Medium modules** (Phase 2): Tackle 50-75% coverage modules
   - Use patterns from Phase 3
   - ~3-5 days

3. **Critical modules** (Phase 1): Complete low-coverage modules
   - Most time-consuming
   - Highest impact
   - ~5-10 days

### Parallel Work Streams

Can parallelize by:
- **Module type**: CLI vs library vs validators
- **Functionality**: Enrichment vs scanning vs reporting
- **Complexity**: Simple utils vs complex integrations

### Daily Goals

- **Day 1-2**: Complete Phase 3 (7 modules to 100%)
- **Day 3-7**: Complete Phase 2 (14 modules to 100%)
- **Day 8-15**: Complete Phase 1 (14 modules to 100%)

---

## Quality Gates

### Before Committing Tests

- [ ] All tests pass locally
- [ ] Coverage meets target (90%+ line, 85%+ branch)
- [ ] No flaky tests (run 10 times)
- [ ] Tests run in < 10 seconds (unit tests)
- [ ] No real network calls
- [ ] No hard-coded paths
- [ ] Proper error messages in assertions

### Before Marking Module Complete

- [ ] 100% line coverage
- [ ] 90%+ branch coverage
- [ ] All public functions tested
- [ ] All error paths tested
- [ ] All edge cases documented
- [ ] Mutation testing (optional) passed
- [ ] Code review completed

---

## Tools and Automation

### Required Tools

```bash
# Test execution
pytest>=7.0.0
pytest-cov>=4.0.0
pytest-randomly>=3.12.0
pytest-mock>=3.10.0

# Test helpers
freezegun>=1.2.0
responses>=0.22.0
pytest-httpx>=0.21.0

# Quality checking
coverage>=7.0.0
mutmut>=2.4.0  # Optional: mutation testing
pytest-benchmark>=4.0.0  # Optional: performance testing
```

### Coverage Commands

```bash
# Run all tests with coverage
pytest --cov=tools/supplychain --cov-report=html --cov-report=term-missing --cov-branch

# Run specific module tests
pytest tools/supplychain/tests/test_MODULE.py --cov=tools/supplychain/MODULE --cov-report=term-missing --cov-branch

# Find modules below target
pytest --cov=tools/supplychain --cov-report=json --cov-branch
python scripts/find_low_coverage.py coverage.json

# Mutation testing (optional)
mutmut run --paths-to-mutate=tools/supplychain/MODULE.py
mutmut results
```

### Automation Scripts

Create helper scripts:

1. **`scripts/coverage_report.py`**: Generate coverage matrix
2. **`scripts/test_generator.py`**: Scaffold tests from source
3. **`scripts/find_untested.py`**: List untested functions
4. **`scripts/validate_tests.py`**: Check test quality

---

## Success Metrics

### Target Metrics

- **Line coverage**: 100% for all modules
- **Branch coverage**: 90%+ for all modules
- **Test count**: ~1000+ tests total
- **Test execution time**: < 60 seconds for full suite
- **Mutation score**: 85%+ for critical modules
- **Flake rate**: 0%

### Current Progress

- **Modules at 100%**: 4/44 (9%)
- **Modules at 90%+**: 8/44 (18%)
- **Total tests**: ~1200
- **Overall coverage**: 63%

### Target State

- **Modules at 100%**: 44/44 (100%)
- **Total tests**: ~2000+
- **Overall coverage**: 100%

---

## Risk Mitigation

### Risks

1. **Time constraints**: Achieving 100% for 44 modules is extensive
2. **Flaky tests**: Network mocking, time dependencies
3. **False confidence**: High coverage ≠ good tests
4. **Maintenance burden**: More tests = more maintenance

### Mitigations

1. **Prioritize critical paths**: Focus on high-impact code first
2. **Test determinism**: Strict mocking, frozen time, seeded RNG
3. **Mutation testing**: Verify tests actually catch bugs
4. **Continuous review**: Regular test quality audits

---

## Conclusion

Achieving 100% test coverage across 44 modules is an ambitious but achievable goal. The phased approach outlined here provides a clear path forward, with:

- **~410-730 tests** remaining to be written
- **~10-15 days** of focused development
- **Clear patterns and templates** for efficient test creation
- **Quality gates** to ensure test effectiveness

The investment in comprehensive test coverage will pay dividends in:
- Confidence in refactoring
- Faster debugging
- Better documentation
- Reduced production bugs
- Easier onboarding for new developers

---

## Appendix: Module Coverage Matrix

| Module | Current | Target | Gap | Priority | Est. Tests |
|--------|---------|--------|-----|----------|-----------|
| metrics_aggregator.py | 100% | 100% | 0% | ✅ Complete | 0 |
| provenance_builder.py | 100% | 100% | 0% | ✅ Complete | 0 |
| purl_generator.py | 100% | 100% | 0% | ✅ Complete | 0 |
| write_sbom.py | 100% | 100% | 0% | ✅ Complete | 0 |
| contribution_tracker.py | 99% | 100% | 1% | 🟢 Near complete | 1-2 |
| license_analyzer.py | 99% | 100% | 1% | 🟢 Near complete | 1-2 |
| vex_processor.py | 99% | 100% | 1% | 🟢 Near complete | 1-2 |
| kev_enrichment.py | 98% | 100% | 2% | 🟢 Near complete | 2-3 |
| license_extractor.py | 98% | 100% | 2% | 🟢 Near complete | 2-3 |
| vulnerability_enrichment.py | 97% | 100% | 3% | 🟢 High | 3-5 |
| epss_enrichment.py | 94% | 100% | 6% | 🟢 High | 5-8 |
| ghsa_enrichment.py | 88% | 100% | 12% | 🟡 Medium | 10-15 |
| vulncheck_enrichment.py | 88% | 100% | 12% | 🟡 Medium | 10-15 |
| upgrade_recommender.py | 86% | 100% | 14% | 🟡 Medium | 12-18 |
| graph_generator.py | 85% | 100% | 15% | 🟡 Medium | 12-20 |
| build_system.py | 82% | 100% | 18% | 🟡 Medium | 15-25 |
| bazbom_config.py | 74% | 100% | 26% | 🟠 Low | 20-30 |
| sbom_diff.py | 73% | 100% | 27% | 🟠 Low | 20-30 |
| changelog_generator.py | 73% | 100% | 27% | 🟠 Low | 20-30 |
| ai_query_engine.py | 73% | 100% | 27% | 🟠 Low | 20-30 |
| sarif_adapter.py | 65% | 100% | 35% | 🟠 Low | 25-35 |
| supply_chain_risk.py | 64% | 100% | 36% | 🟠 Low | 25-35 |
| policy_check.py | 63% | 100% | 37% | 🔴 Critical | 25-35 |
| conflict_detector.py | 63% | 100% | 37% | 🔴 Critical | 25-35 |
| badge_generator.py | 60% | 100% | 40% | 🔴 Critical | 30-40 |
| extract_maven_deps.py | 56% | 100% | 44% | 🔴 Critical | 30-40 |
| dependency_scanner.py | 54% | 100% | 46% | 🔴 Critical | 30-40 |
| sbom_signing.py | 53% | 100% | 47% | 🔴 Critical | 30-45 |
| csv_exporter.py | 52% | 100% | 48% | 🔴 Critical | 30-45 |
| intoto_attestation.py | 51% | 100% | 49% | 🔴 Critical | 30-45 |
| cve_tracker.py | 51% | 100% | 49% | 🔴 Critical | 30-45 |
| drift_detector.py | 49% | 100% | 51% | 🔴 Critical | 35-50 |
| osv_query.py | 48% | 100% | 52% | 🔴 Critical | 35-50 |
| dependency_verifier.py | 48% | 100% | 52% | 🔴 Critical | 35-50 |
| osv_contributor.py | 46% | 100% | 54% | 🔴 Critical | 35-50 |
| scan_container.py | 45% | 100% | 55% | 🔴 Critical | 35-50 |
| license_scanner.py | 45% | 100% | 55% | 🔴 Critical | 35-50 |
| rekor_integration.py | 42% | 100% | 58% | 🔴 Critical | 40-55 |
| verify_sbom.py | 40% | 100% | 60% | 🔴 Critical | 40-55 |
| container_scanner.py | 37% | 100% | 63% | 🔴 Critical | 40-60 |
| interactive_fix.py | 36% | 100% | 64% | 🔴 Critical | 40-60 |
| bazbom_cli.py | 35% | 100% | 65% | 🔴 Critical | 45-65 |
| compliance_report.py | 33% | 100% | 67% | 🔴 Critical | 45-65 |
| incremental_analyzer.py | 27% | 100% | 73% | 🔴 Critical | 50-70 |

**Total estimated tests needed:** ~830-1320 tests

