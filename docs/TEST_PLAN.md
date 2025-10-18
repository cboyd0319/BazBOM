# BazBOM Comprehensive Test Plan

## Executive Summary

This document outlines the comprehensive test strategy for achieving 90%+ code coverage across all Python modules in the BazBOM project. As of the current state, we have achieved **55.37% overall coverage** (up from 51%), with 906 passing tests.

## Coverage Goals

- **Target**: 90% line coverage, 85% branch coverage
- **Current**: 55.37% overall coverage
- **Progress**: +4.37% improvement from baseline

## Completed Modules (90%+ Coverage)

### ✅ Metrics Aggregator (100% Coverage)
- **File**: `tools/supplychain/metrics_aggregator.py`
- **Tests**: `test_metrics_aggregator.py` (32 tests)
- **Coverage**: 100% lines, 70% branches
- **Key test areas**:
  - JSON file loading with error handling
  - Vulnerability metrics aggregation
  - Dependency metrics calculation
  - License metrics aggregation
  - Report generation (JSON and text formats)
  - CLI interface with various argument combinations

### ✅ License Analyzer (98% Coverage)
- **File**: `tools/supplychain/license_analyzer.py`
- **Tests**: `test_license_analyzer.py` (60 tests)
- **Coverage**: 98% lines, 95% branches
- **Key test areas**:
  - License normalization (Apache, MIT, BSD variations)
  - License categorization (copyleft, permissive, unknown)
  - Conflict detection between incompatible licenses
  - Dependency analysis with various input formats
  - Report generation with flags
  - CLI interface with all argument combinations
  - Edge cases (Unicode, empty inputs, malformed data)

### ✅ VEX Processor (98% Coverage)
- **File**: `tools/supplychain/vex_processor.py`
- **Tests**: `test_vex_processor.py` + `test_vex_processor_extended.py` (42 tests)
- **Coverage**: 98% lines, 96% branches
- **Key test areas**:
  - VEX statement loading from directory
  - Parsing multiple VEX formats (simplified, CSAF)
  - Finding suppression with various status values
  - Package matching logic
  - Filtering findings
  - Statement validation
  - CLI with full processing and validation-only modes
  - Severity recalculation after suppression

### ✅ Provenance Builder (100% Coverage)
- **File**: `tools/supplychain/provenance_builder.py`
- **Tests**: Existing tests in `test_provenance_builder.py`
- **Coverage**: 100% lines, 100% branches
- **Maintained by**: Existing test suite

### ✅ Validate SARIF (100% Coverage)
- **File**: `tools/supplychain/validators/validate_sarif.py`
- **Tests**: Existing tests in `test_validate_sarif.py`
- **Coverage**: 100% lines, 100% branches
- **Maintained by**: Existing test suite

## High Priority Modules for Testing (0% Coverage)

### 🔴 Priority 1: CLI and User-Facing Tools

#### bazbom_cli.py (0% coverage, 147 statements)
**Complexity**: High - Entry point for CLI interface
**Test Strategy**:
- Mock build system detection
- Test all CLI commands and flags
- Validate error handling for missing dependencies
- Test configuration loading
- Mock file I/O operations

#### compliance_report.py (0% coverage, 207 statements)
**Complexity**: High - Complex report generation
**Test Strategy**:
- Mock Jinja2 template loading
- Test report generation with various data inputs
- Validate HTML, PDF, DOCX, XLSX output formats
- Test certificate generation
- Test executive summary generation

#### interactive_fix.py (0% coverage, 196 statements)
**Complexity**: High - Interactive user interface
**Test Strategy**:
- Mock user input/output
- Test fix recommendation logic
- Validate interactive prompts
- Test batch processing mode

### 🟡 Priority 2: Integration Modules

#### osv_contributor.py (0% coverage, 194 statements)
**Complexity**: Medium - OSV API integration
**Test Strategy**:
- Mock HTTP requests to OSV API
- Test contribution submission
- Validate data formatting
- Test authentication handling

#### scan_container.py (0% coverage, 134 statements)
**Complexity**: Medium - Container scanning
**Test Strategy**:
- Mock Docker/container operations
- Test image analysis
- Validate SBOM extraction from containers
- Test layer analysis

#### verify_sbom.py (0% coverage, 150 statements)
**Complexity**: Medium - SBOM verification
**Test Strategy**:
- Test signature verification
- Mock cryptographic operations
- Validate certificate chain checking
- Test various SBOM formats

#### contribution_tracker.py (0% coverage, 187 statements)
**Complexity**: Medium - Contribution tracking
**Test Strategy**:
- Mock Git operations
- Test contribution analysis
- Validate metrics calculation
- Test report generation

### 🟢 Priority 3: Validation Modules

#### validators/validate_provenance.py (0% coverage, 142 statements)
**Complexity**: Low - Schema validation
**Test Strategy**:
- Test SLSA provenance schema validation
- Validate required fields checking
- Test error message generation
- Parametrize with valid/invalid inputs

## Partially Tested Modules Requiring Improvement

### 🟠 High Impact (< 50% coverage)

#### license_extractor.py (38% coverage)
**Missing Coverage**:
- JAR file inspection (lines 191-225)
- POM.xml parsing (lines 238-263)
- License file extraction (lines 276-302)
- Multiple license handling (lines 315-324)

**Test Strategy**:
- Create test JAR files with various license formats
- Mock ZIP file operations
- Test XML parsing edge cases
- Validate license text extraction

#### rekor_integration.py (38% coverage)
**Missing Coverage**:
- Rekor API interaction (lines 115-122, 180-199)
- Entry verification (lines 210-234)
- Certificate validation (lines 323-436)

**Test Strategy**:
- Mock Rekor API calls
- Test entry submission and retrieval
- Validate signature verification
- Test error handling for API failures

#### incremental_analyzer.py (41% coverage)
**Missing Coverage**:
- Git diff analysis (lines 129-143)
- Target identification (lines 157-181)
- Dependency graph traversal (lines 185-266)

**Test Strategy**:
- Mock Git operations
- Create test repository with commits
- Test change detection
- Validate affected target calculation

#### supply_chain_risk.py (43% coverage)
**Missing Coverage**:
- Maven deprecation checking (lines 86-112)
- Network requests (lines 127-147)
- Main function (lines 198-281)

**Test Strategy**:
- Mock Maven Central API
- Test typosquatting detection edge cases
- Validate risk scoring
- Test bulk package analysis

#### osv_query.py (48% coverage)
**Missing Coverage**:
- Batch vulnerability queries (lines 154-166)
- OSV API pagination (lines 207-209)
- Offline database mode (lines 238-380)

**Test Strategy**:
- Mock OSV API responses
- Test rate limiting
- Validate batch processing
- Test offline database queries

#### drift_detector.py (49% coverage)
**Missing Coverage**:
- SBOM comparison logic (lines 312-382)
- Change categorization (lines 391-486)

**Test Strategy**:
- Create test SBOM pairs
- Test version drift detection
- Validate change severity calculation
- Test report generation

### 🟡 Medium Impact (50-70% coverage)

#### intoto_attestation.py (51% coverage)
- Need tests for attestation bundle creation
- Material tracking
- Predicate building

#### csv_exporter.py (52% coverage)
- Need tests for CSV generation
- Excel format support
- Column customization

#### sbom_signing.py (53% coverage)
- Need tests for signing operations
- Key management
- Verification logic

#### extract_maven_deps.py (56% coverage)
- Need tests for POM parsing
- Transitive dependency resolution
- Version conflict handling

#### badge_generator.py (60% coverage)
- Need tests for SVG generation
- Badge styling
- Status calculation

#### conflict_detector.py (63% coverage)
- Need tests for version conflict detection
- Resolution strategies
- Report formatting

#### policy_check.py (63% coverage)
- Need tests for policy evaluation
- Threshold checking
- Violation reporting

#### purl_generator.py (64% coverage)
- Need tests for PURL generation edge cases
- Qualifier handling
- Invalid input handling

#### sarif_adapter.py (65% coverage)
- Need tests for SARIF generation
- Rule mapping
- Location formatting

## Test Infrastructure

### Test Organization
```
tools/supplychain/tests/
├── conftest.py                     # Shared fixtures
├── fixtures/                        # Test data files
│   ├── sample_sbom.json
│   ├── sample_vex.json
│   └── sample_sca_findings.json
├── test_*.py                        # Test modules
└── README.md                        # Test documentation
```

### Fixtures and Utilities

#### Shared Fixtures (conftest.py)
- `tmp_path_factory`: Temporary directory for file operations
- `sample_sbom`: Pre-built SBOM for testing
- `sample_dependencies`: Common dependency data
- `mock_http_responses`: Mock HTTP server responses
- `freeze_time`: Time freezing for deterministic tests

#### Test Data Requirements
- Sample SBOMs (minimal, complex, malformed)
- Sample VEX statements (various formats)
- Sample vulnerability data (OSV, NVD, GHSA)
- Sample Maven artifacts
- Sample container images (metadata only)

### Testing Best Practices

#### 1. AAA Pattern (Arrange-Act-Assert)
```python
def test_feature():
    # Arrange: Set up test data and conditions
    input_data = {"key": "value"}
    
    # Act: Execute the code under test
    result = function_under_test(input_data)
    
    # Assert: Verify the outcome
    assert result["expected"] == "value"
```

#### 2. Parametrization for Input Matrices
```python
@pytest.mark.parametrize(
    "input_value,expected",
    [
        ("MIT", "permissive"),
        ("GPL-2.0", "copyleft"),
        ("NOASSERTION", "unknown"),
    ],
    ids=["MIT", "GPL", "unknown"]
)
def test_categorize_license(input_value, expected):
    assert categorize_license(input_value) == expected
```

#### 3. Mocking External Dependencies
```python
@patch('module.requests.get')
def test_http_request(mock_get):
    mock_get.return_value.status_code = 200
    mock_get.return_value.json.return_value = {"data": "value"}
    
    result = fetch_data()
    assert result == {"data": "value"}
```

#### 4. Error Handling Tests
```python
def test_invalid_input_raises_error():
    with pytest.raises(ValueError, match="Invalid input"):
        process_data(None)
```

#### 5. Deterministic Tests
- Seed random number generators
- Freeze time for timestamp-dependent code
- Mock network calls
- Use fixed test data

## Coverage Measurement

### Running Tests with Coverage
```bash
# Run all tests with coverage
pytest --cov=tools/supplychain --cov-report=term-missing

# Run specific module tests
pytest tools/supplychain/tests/test_metrics_aggregator.py --cov=tools/supplychain/metrics_aggregator

# Generate HTML coverage report
pytest --cov=tools/supplychain --cov-report=html

# Generate JSON coverage report for CI
pytest --cov=tools/supplychain --cov-report=json
```

### Coverage Goals by Module Type
- **Core business logic**: 95%+ line coverage, 90%+ branch coverage
- **CLI/interface code**: 85%+ line coverage, 80%+ branch coverage
- **Integration code**: 80%+ line coverage, 75%+ branch coverage
- **Utility functions**: 100% line coverage, 95%+ branch coverage

## Test Execution Strategy

### Local Development
1. Run focused tests for changed files
2. Use `pytest -x` to fail fast
3. Use `pytest -v` for verbose output
4. Run coverage checks before committing

### CI/CD Pipeline
1. Run full test suite on every PR
2. Enforce minimum coverage thresholds
3. Generate coverage reports as artifacts
4. Fail builds if coverage drops below threshold

### Test Performance
- Keep unit tests under 100ms each
- Use pytest markers to separate slow tests
- Parallelize test execution with pytest-xdist
- Cache expensive fixtures

## Next Steps

### Phase 1: Complete High-Priority Modules (Week 1-2)
- [ ] bazbom_cli.py (0% → 90%)
- [ ] compliance_report.py (0% → 85%)
- [ ] interactive_fix.py (0% → 80%)

### Phase 2: Integration Modules (Week 3-4)
- [ ] osv_contributor.py (0% → 85%)
- [ ] scan_container.py (0% → 80%)
- [ ] verify_sbom.py (0% → 85%)
- [ ] contribution_tracker.py (0% → 80%)
- [ ] validators/validate_provenance.py (0% → 95%)

### Phase 3: Improve Partially Tested Modules (Week 5-6)
- [ ] license_extractor.py (38% → 90%)
- [ ] rekor_integration.py (38% → 85%)
- [ ] incremental_analyzer.py (41% → 85%)
- [ ] supply_chain_risk.py (43% → 85%)
- [ ] osv_query.py (48% → 85%)
- [ ] drift_detector.py (49% → 85%)

### Phase 4: Polish and Maintain (Ongoing)
- [ ] intoto_attestation.py (51% → 90%)
- [ ] csv_exporter.py (52% → 85%)
- [ ] sbom_signing.py (53% → 85%)
- [ ] extract_maven_deps.py (56% → 85%)
- [ ] badge_generator.py (60% → 85%)
- [ ] All modules 60-70% → 90%+

## Test Maintenance

### Regular Activities
- Review and update tests when code changes
- Add tests for new features
- Remove obsolete tests
- Refactor duplicated test code
- Update test documentation

### Code Review Checklist
- [ ] All new code has corresponding tests
- [ ] Tests follow AAA pattern
- [ ] Tests are deterministic (no flaky tests)
- [ ] Error cases are tested
- [ ] Edge cases are covered
- [ ] Tests have descriptive names
- [ ] Coverage doesn't decrease

## Conclusion

Achieving 90% coverage across all modules is an achievable goal with systematic execution. The test plan outlined here provides a clear roadmap with prioritization based on module complexity and impact. The infrastructure established (fixtures, utilities, CI integration) will support efficient test development and maintenance.

**Current Status**: 55.37% coverage, 906 passing tests
**Target Status**: 90%+ coverage, estimated 2000+ tests
**Estimated Effort**: 6-8 weeks with dedicated focus
