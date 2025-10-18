# Test Coverage Roadmap for BazBOM

## Current Status
- **Overall Coverage:** 65%
- **Total Tests:** 1131
- **Test Execution Time:** ~9 seconds
- **Target Coverage:** 90%

## Coverage by Module Category

### ✅ Excellent Coverage (90%+) - 12 Modules
These modules have achieved or exceeded the 90% coverage target:

1. metrics_aggregator.py - 100%
2. provenance_builder.py - 100%
3. validators/__init__.py - 100%
4. validators/validate_sarif.py - 100%
5. vex_processor.py - 98%
6. license_analyzer.py - 98%
7. vulnerability_enrichment.py - 97%
8. kev_enrichment.py - 97%
9. graph_generator.py - 94%
10. epss_enrichment.py - 94%
11. ghsa_enrichment.py - 91%
12. vulncheck_enrichment.py - 88%

### 🟡 Good Coverage (70-89%) - 9 Modules
Close to target, need minor enhancements:

1. upgrade_recommender.py - 86%
2. bazbom_cli.py - 82%
3. build_system.py - 82%
4. validators/validate_sbom.py - 79%
5. bazbom_config.py - 74%
6. ai_query_engine.py - 73%
7. changelog_generator.py - 73%
8. sbom_diff.py - 73%
9. write_sbom.py - 71%

### 🔴 Needs Work (<70%) - 22 Modules
Priority targets for improving coverage:

#### Critical Priority (< 40%)
1. **contribution_tracker.py - 24%** (187 total lines, 137 uncovered)
   - Missing: report generation, badge calculation, statistics aggregation
   - Test file: test_contribution_tracker.py (exists, needs enhancement)
   
2. **compliance_report.py - 33%** (207 total lines, 137 uncovered)
   - Missing: report rendering, template processing, output generation
   - Test file: test_compliance_report.py (enhanced)
   
3. **interactive_fix.py - 36%** (196 total lines, 120 uncovered)
   - Missing: interactive UI, user prompts, fix application
   - Test file: test_interactive_fix.py (enhanced)
   
4. **license_extractor.py - 38%** (169 total lines, 101 uncovered)
   - Missing: JAR inspection, POM parsing, license detection
   - Test file: test_license_extractor.py (exists)
   
5. **rekor_integration.py - 38%** (207 total lines, 124 uncovered)
   - Missing: Rekor API calls, entry verification, search functionality
   - Test file: needs creation
   
6. **verify_sbom.py - 40%** (150 total lines, 86 uncovered)
   - Missing: complete verification workflows, Rekor checks
   - Test file: test_verify_sbom.py (baseline created)

#### High Priority (40-49%)
7. incremental_analyzer.py - 41%
8. supply_chain_risk.py - 43%
9. scan_container.py - 45%
10. osv_contributor.py - 46%
11. validators/validate_provenance.py - 46%
12. osv_query.py - 48%
13. drift_detector.py - 49%

#### Medium Priority (50-69%)
14. intoto_attestation.py - 51%
15. csv_exporter.py - 52%
16. sbom_signing.py - 53%
17. extract_maven_deps.py - 56%
18. badge_generator.py - 60%
19. conflict_detector.py - 63%
20. policy_check.py - 63%
21. purl_generator.py - 64%
22. sarif_adapter.py - 65%

## Test Quality Standards

All tests in this repository follow these standards:

### Structure
- **AAA Pattern:** Arrange - Act - Assert
- **Naming:** `test_<unit>_<scenario>_<expected>()`
- **Organization:** Group related tests in classes
- **Isolation:** Each test is independent

### Coverage Targets
- **Line Coverage:** ≥ 90%
- **Branch Coverage:** ≥ 85%
- **New Code:** 100% coverage for pure functions

### Best Practices
✅ Use parametrization for input matrices
✅ Mock external dependencies (network, filesystem, subprocess)
✅ Test error paths and exceptions
✅ Test boundary conditions (empty, None, large, unicode)
✅ Freeze time for deterministic tests (freezegun)
✅ Seed random generators (random.seed(1337))
✅ Use fixtures from conftest.py
✅ Fast execution (< 100ms per test typical)

## Test Enhancement Patterns

### Pattern 1: Basic Function Testing
```python
def test_function_happy_path(self):
    """Test function with valid inputs."""
    # Arrange
    input_data = {...}
    
    # Act
    result = function(input_data)
    
    # Assert
    assert result == expected
```

### Pattern 2: Parametrized Testing
```python
@pytest.mark.parametrize("input,expected", [
    ("valid", True),
    ("", False),
    (None, False),
], ids=["valid", "empty", "none"])
def test_validation_parametrized(self, input, expected):
    assert validate(input) == expected
```

### Pattern 3: Error Testing
```python
def test_function_raises_on_invalid_input(self):
    """Test function raises appropriate error."""
    with pytest.raises(ValueError, match="specific error message"):
        function(invalid_input)
```

### Pattern 4: Mock External Dependencies
```python
@patch('module.subprocess.run')
def test_function_with_external_call(self, mock_run):
    """Test function that calls external process."""
    mock_run.return_value = Mock(stdout="output", returncode=0)
    result = function()
    mock_run.assert_called_once()
```

## Roadmap to 90% Coverage

### Phase 1: Low-Hanging Fruit (Weeks 1-2)
Target: 65% → 75%
- Enhance existing test files for modules at 60-69%
- Add missing happy path tests
- Add parametrized tests for common inputs

### Phase 2: Medium Priority Modules (Weeks 3-4)
Target: 75% → 82%
- Focus on modules at 40-59%
- Add error path testing
- Add boundary condition tests

### Phase 3: Critical Modules (Weeks 5-6)
Target: 82% → 88%
- Complete testing for modules < 40%
- Add integration tests
- Add complex workflow tests

### Phase 4: Final Push (Weeks 7-8)
Target: 88% → 90%+
- Fill remaining gaps
- Add property-based tests
- Add mutation testing

## Immediate Next Steps

1. **Create missing test files:**
   - test_rekor_integration.py (0 tests currently)

2. **Enhance low-coverage modules (pick 1-2 per week):**
   - contribution_tracker.py: Add report generation tests
   - compliance_report.py: Add template rendering tests
   - interactive_fix.py: Add interactive workflow tests
   - license_extractor.py: Add JAR inspection tests

3. **Add integration tests:**
   - End-to-end SBOM generation workflow
   - Complete vulnerability scanning workflow
   - Full compliance report generation

4. **Add property-based tests:**
   - Use hypothesis for purl_generator.py
   - Use hypothesis for version comparison logic
   - Use hypothesis for parsing functions

## Test Infrastructure

### Fixtures Available (conftest.py)
- `tmp_dir` - Temporary directory for tests
- `sample_sbom_data` - Sample SPDX SBOM
- `sample_vulnerability_data` - Sample vulnerability data
- `sample_maven_coordinates` - Sample Maven coords
- `env_vars` - Helper to set environment variables
- `mock_http_response` - Factory for mock HTTP responses
- `temp_json_file` - Factory for temporary JSON files
- `_seed_rng` - Auto-seed RNG for deterministic tests

### Running Tests

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=tools/supplychain --cov-report=term-missing

# Run specific test file
pytest tools/supplychain/tests/test_module.py

# Run specific test
pytest tools/supplychain/tests/test_module.py::TestClass::test_method

# Run with verbose output
pytest -v

# Run fast (skip slow tests)
pytest -m "not slow"
```

## Resources

- **pytest Documentation:** https://docs.pytest.org/
- **Coverage.py Documentation:** https://coverage.readthedocs.io/
- **Freezegun (time mocking):** https://github.com/spulec/freezegun
- **pytest-mock:** https://pytest-mock.readthedocs.io/
- **Hypothesis (property testing):** https://hypothesis.readthedocs.io/

## Success Metrics

Track these metrics over time:
- [ ] Overall coverage: 90%+
- [ ] All modules: 70%+ minimum
- [ ] Critical modules: 90%+
- [ ] Test execution time: < 30 seconds
- [ ] All tests passing
- [ ] Zero test flakes
- [ ] Branch coverage: 85%+

## Contributors

When adding new tests:
1. Follow existing patterns in the test file
2. Use descriptive test names
3. Add docstrings to complex tests
4. Use parametrization for similar test cases
5. Mock external dependencies
6. Ensure fast execution
7. Run full test suite before committing
8. Update this roadmap if you complete a module

---

Last Updated: 2025-10-18
Current Coverage: 65%
Target Coverage: 90%
