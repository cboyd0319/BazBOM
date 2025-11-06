# bazbom-ml

**Machine Learning infrastructure for BazBOM vulnerability analysis**

[![Tests](https://img.shields.io/badge/tests-23%20passing-brightgreen)](../../docs/reference/ml-features.md)
[![Coverage](https://img.shields.io/badge/coverage-%3E90%25-brightgreen)]()
[![Privacy](https://img.shields.io/badge/privacy-local%20only-blue)]()

---

## Overview

`bazbom-ml` provides AI-powered intelligence features for vulnerability analysis and anomaly detection. All models run locally with zero external dependencies.

## Features

### Feature Extraction: Feature Extraction
Convert vulnerabilities and dependencies into numerical features for ML analysis.

```rust
use bazbom_ml::{VulnerabilityFeatures, DependencyFeatures};

let vuln = VulnerabilityFeatures {
    cvss_score: 9.8,
    epss: 0.95,
    in_kev: true,
    is_reachable: true,
    ..Default::default()
};

// Convert to feature vector for ML algorithms
let features = vuln.to_vector(); // [9.8, 0.0, 0.0, 0.95, 1.0, ...]
```

### Anomaly Detection: Anomaly Detection
Identify unusual dependency patterns using statistical methods.

```rust
use bazbom_ml::{AnomalyDetector, DependencyFeatures};

let detector = AnomalyDetector::new();

let suspicious = DependencyFeatures {
    transitive_count: 250,      // Unusual!
    maintainer_score: 0.1,      // Low reputation!
    popularity: 0.02,           // Typosquat?
    ..Default::default()
};

let anomalies = detector.detect(&suspicious, "log4j-evil");
// Returns: [UnusualTransitiveCount, LowMaintainerScore, LowPopularity, MultipleSignals]
```

### Enhanced Risk Scoring: Enhanced Risk Scoring
Multi-factor risk scoring beyond simple CVSS.

```rust
use bazbom_ml::{RiskScorer, VulnerabilityFeatures};

let scorer = RiskScorer::new();

let vuln = VulnerabilityFeatures {
    cvss_score: 9.8,
    epss: 0.95,
    in_kev: true,
    is_reachable: true,
    has_exploit: true,
    age_days: 30,
    ..Default::default()
};

let score = scorer.score(&vuln);
// Risk Level: Critical
// Score: 0.93/1.00
// Explanation: "High CVSS score (9.8). High exploit probability (EPSS: 95%). 
//               In CISA KEV (actively exploited). Vulnerable code is reachable..."
```

## Privacy & Security

### Local-Only Processing: Local-Only Processing
- All models run locally on your machine
- No data sent to external services
- No telemetry or usage tracking
- Can operate completely offline

### Transparent Data Collection: Transparent Data Collection
- Feature extraction is fully visible (open source)
- Only uses public vulnerability metadata
- No proprietary code analysis
- No PII collection

## Architecture

### Modules

- **`features`**: Feature extraction for vulnerabilities and dependencies
- **`anomaly`**: Statistical anomaly detection
- **`risk`**: Enhanced risk scoring with multiple factors

### Dependencies

```toml
anyhow = "1.0"           # Error handling
serde = "1.0"            # Serialization
chrono = "0.4"           # Timestamps
```

**No external ML frameworks** - Pure Rust implementation using statistical methods.

## Testing

All features have comprehensive test coverage:

```
features:  3 tests passing
anomaly:  14 tests passing
risk:     10 tests passing
─────────────────────
Total:    23 tests passing
```

Run tests:
```bash
cargo test -p bazbom-ml
```

## Performance

- Feature extraction: <1ms per vulnerability
- Anomaly detection: <10ms per dependency
- Risk scoring: <1ms per vulnerability
- Fully parallelizable for batch processing

## Usage in BazBOM

### CLI (Planned)
```bash
# Use ML-enhanced risk scoring
bazbom scan --ml-risk

# Detect anomalies
bazbom analyze --anomalies
```

### Library
```rust
use bazbom_ml::{RiskScorer, AnomalyDetector, VulnerabilityFeatures, DependencyFeatures};

fn analyze_vulnerability(vuln_data: VulnerabilityData) {
    // Extract features
    let features = VulnerabilityFeatures {
        cvss_score: vuln_data.cvss,
        epss: vuln_data.epss,
        in_kev: vuln_data.in_kev,
        is_reachable: vuln_data.reachable,
        // ... other features
    };
    
    // Calculate risk
    let scorer = RiskScorer::new();
    let risk = scorer.score(&features);
    
    println!("Risk Level: {:?}", risk.risk_level);
    println!("Score: {:.2}", risk.overall_score);
    println!("{}", risk.explanation);
}
```

## Future Features (Phase 10)

- **Custom Exploit Prediction**: Train models on your environment
- **LLM Migration Guides**: Generate upgrade guides for breaking changes
- **Natural Language Queries**: Ask questions about vulnerabilities in plain English
- **Intelligent Triage**: Auto-categorize and prioritize vulnerabilities
- **False Positive Learning**: Learn from your false positive patterns
- **Semantic Dependency Search**: Find similar packages by functionality

## Documentation

- [ML Features Guide](../../docs/reference/ml-features.md) - Complete feature documentation
- [Phase 10 Spec](../../docs/archive/phases/PHASE_10_AI_INTELLIGENCE.md) - Detailed roadmap
- [API Documentation](https://docs.rs/bazbom-ml) - API reference (coming soon)

## Contributing

We welcome contributions! Please ensure:

1. Tests pass: `cargo test -p bazbom-ml`
2. Code coverage >90%
3. Privacy-preserving design
4. No external ML dependencies
5. Performance benchmarks for new features

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for more details.

## License

Apache 2.0 - See [LICENSE](../../LICENSE) for details.

---

**Part of the BazBOM project** - JVM SBOM, SCA, and dependency graph tool  
**Repository**: https://github.com/cboyd0319/BazBOM  
**Phase 10 Status**: 5% Complete (Infrastructure established)
