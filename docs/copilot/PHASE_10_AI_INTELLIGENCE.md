# Phase 10: AI-Powered Intelligence

**Status:** Planned (Research)
**Priority:** 🟢 P2 - Innovation/Differentiation
**Timeline:** Months 8-12 (16 weeks)
**Team Size:** 1 ML engineer + 1 Rust developer
**Dependencies:** Phase 0-3 complete, Phase 7 (threat intelligence) recommended

---

## Executive Summary

**Goal:** Use AI/ML to provide insights humans can't derive manually.

**Current Gap:** BazBOM uses rule-based prioritization (CVSS, KEV, EPSS). Doesn't learn from your specific environment.

**Target Capabilities:**
1. **ML-Powered Exploit Prediction** - Custom model trained on your tech stack
2. **Anomaly Detection** - Learn normal dependency patterns, flag unusual changes
3. **LLM-Assisted Remediation** - Generate migration guides for breaking changes
4. **Intelligent Triage** - Auto-categorize vulnerabilities (false positive, real threat, etc.)

**Success Metrics:**
- ✅ 50% reduction in manual triage time
- ✅ 90%+ accuracy in exploit prediction (better than EPSS)
- ✅ 80%+ accuracy in anomaly detection
- ✅ LLM generates useful migration guides 70%+ of the time

**Strategic Rationale:** Differentiate from competitors through AI-native features.

---

## 10.1 ML-Powered Exploit Prediction

### Problem with EPSS

**EPSS (Exploit Prediction Scoring System):**
- Generic model trained on all CVEs
- Doesn't know your tech stack
- Treats all Java apps the same

**Example:** EPSS says CVE-X has 30% exploit probability. But:
- Your app is internal-only (no internet access) → Real risk: 5%
- Your app is public-facing e-commerce → Real risk: 80%

### Custom Exploit Prediction Model

**Approach:** Train model on your historical vulnerability data

**Features:**
- **CVE Features:** CVSS score, age, availability of exploit code, vulnerability type (RCE, XSS, etc.)
- **Environment Features:** Public/internal, authentication required, network exposure
- **Package Features:** Popularity, maintainer reputation, download count
- **Your History:** Which CVEs have you been exploited by? (if any)

**Model:** Gradient Boosting (XGBoost or LightGBM)

**Implementation:**
```rust
// crates/bazbom-ml/src/exploit_prediction.rs
use lightgbm::Booster;

pub struct ExploitPredictor {
    model: Booster,
}

impl ExploitPredictor {
    pub fn load_model(path: &Path) -> Result<Self> {
        let model = Booster::from_file(path.to_str().unwrap())?;
        Ok(Self { model })
    }

    pub fn predict_exploit_probability(&self, vuln: &Vulnerability, context: &AppContext) -> Result<f64> {
        let features = self.extract_features(vuln, context);
        let prediction = self.model.predict(vec![features])?;

        Ok(prediction[0])  // Probability between 0.0 and 1.0
    }

    fn extract_features(&self, vuln: &Vulnerability, context: &AppContext) -> Vec<f64> {
        vec![
            vuln.cvss_score.unwrap_or(0.0),
            vuln.age_days as f64,
            if vuln.has_exploit_code { 1.0 } else { 0.0 },
            vuln.epss.unwrap_or(0.0),
            if context.public_facing { 1.0 } else { 0.0 },
            if context.requires_auth { 0.0 } else { 1.0 },
            vuln.package_download_count as f64,
            // ... more features
        ]
    }
}
```

**Training:**
```python
# tools/ml/train_exploit_predictor.py
import pandas as pd
import lightgbm as lgb
from sklearn.model_selection import train_test_split

# Load historical CVE data
df = pd.read_csv('data/historical_cves.csv')

# Features
X = df[['cvss_score', 'age_days', 'has_exploit', 'epss', 'public_facing', ...]]

# Target: was this CVE actually exploited in the wild?
y = df['was_exploited']

# Train/test split
X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2)

# Train model
model = lgb.LGBMClassifier(
    n_estimators=100,
    learning_rate=0.05,
    max_depth=6
)
model.fit(X_train, y_train)

# Evaluate
accuracy = model.score(X_test, y_test)
print(f'Accuracy: {accuracy:.2%}')

# Save model
model.booster_.save_model('exploit_predictor.txt')
```

**CLI Integration:**
```bash
# Use custom exploit prediction
bazbom scan --ml-exploit-prediction

# Output:
# Vulnerabilities sorted by custom exploit prediction:
#
# 1. CVE-2024-xxxx in spring-web (Predicted exploit probability: 87%)
#    CVSS: 9.8, EPSS: 0.3 → Custom model: 0.87 (HIGH RISK for your app)
#    Reason: Public-facing, no auth, popular target
#
# 2. CVE-2023-yyyy in commons-io (Predicted exploit probability: 12%)
#    CVSS: 7.5, EPSS: 0.5 → Custom model: 0.12 (LOW RISK for your app)
#    Reason: Internal-only, auth required, low impact
```

---

## 10.2 Anomaly Detection

### Goal

**Detect unusual dependency changes that might indicate compromise:**
- New dependency from unknown maintainer
- Sudden increase in transitive dependencies
- Dependency from unusual geographic location (if metadata available)

### Approach

**Unsupervised Learning:** Isolation Forest or Autoencoder

**Features:**
- Number of transitive dependencies
- Average package age
- Maintainer reputation score
- Geographic origin (if available)
- Download velocity (downloads/day)

**Implementation:**
```rust
// crates/bazbom-ml/src/anomaly_detection.rs
use smartcore::ensemble::isolation_forest::IsolationForest;

pub struct AnomalyDetector {
    model: IsolationForest<f64>,
}

impl AnomalyDetector {
    pub fn train(historical_dependencies: Vec<DependencyFeatures>) -> Result<Self> {
        let features: Vec<Vec<f64>> = historical_dependencies
            .iter()
            .map(|d| d.to_feature_vector())
            .collect();

        let model = IsolationForest::fit(&features, Default::default())?;

        Ok(Self { model })
    }

    pub fn detect_anomalies(&self, dependencies: &[Dependency]) -> Result<Vec<Anomaly>> {
        let features: Vec<Vec<f64>> = dependencies
            .iter()
            .map(|d| DependencyFeatures::from_dependency(d).to_feature_vector())
            .collect();

        let predictions = self.model.predict(&features)?;

        let anomalies: Vec<_> = dependencies
            .iter()
            .zip(predictions.iter())
            .filter(|(_, &score)| score < -0.5)  // Anomaly threshold
            .map(|(dep, &score)| Anomaly {
                dependency: dep.clone(),
                anomaly_score: score,
                explanation: self.explain_anomaly(dep),
            })
            .collect();

        Ok(anomalies)
    }

    fn explain_anomaly(&self, dep: &Dependency) -> String {
        // Heuristic explanations
        if dep.transitive_count > 100 {
            format!("Unusually high number of transitive dependencies: {}", dep.transitive_count)
        } else if dep.maintainer_reputation < 0.3 {
            format!("Low maintainer reputation score: {:.2}", dep.maintainer_reputation)
        } else {
            "Unusual pattern detected".to_string()
        }
    }
}
```

**CLI Output:**
```bash
bazbom scan --detect-anomalies

# Output:
# ⚠️ Anomalies Detected:
#
# 1. com.example:suspicious-lib:1.0.0
#    Anomaly Score: -0.87 (HIGH)
#    Reason: Unusually high number of transitive dependencies: 247
#    Recommendation: Review this dependency carefully
#
# 2. org.unknown:new-package:0.1.0
#    Anomaly Score: -0.65 (MEDIUM)
#    Reason: New maintainer with low reputation
#    Recommendation: Wait for community adoption
```

---

## 10.3 LLM-Assisted Remediation

### Goal

**Problem:** Breaking changes in upgrades are scary

**Solution:** LLM generates migration guides

**Example:**
- Upgrade: Spring Framework 5.x → 6.x
- User asks: "What breaks in my code?"
- LLM answers: "Spring 6 removed `@RequestMapping` shorthand. Update to `@GetMapping`/`@PostMapping`."

### Implementation

**Privacy-Preserving Options:**
1. **Local LLM** (Llama 3, Mistral) - No data leaves machine
2. **Optional Cloud LLM** (OpenAI, Anthropic) - User opts in

**Approach:**
```rust
// crates/bazbom-ml/src/llm_assistant.rs
use llm::models::Llama;

pub struct LlmAssistant {
    model: Option<Llama>,  // None if user doesn't want LLM features
}

impl LlmAssistant {
    pub fn generate_migration_guide(&self, upgrade: &DependencyUpgrade) -> Result<String> {
        let prompt = format!(
            "You are a Java expert. Explain the breaking changes when upgrading {} from {} to {}. \
             Be specific about code changes needed. Format as a numbered list.",
            upgrade.package_name,
            upgrade.from_version,
            upgrade.to_version
        );

        if let Some(model) = &self.model {
            // Local LLM
            let response = model.generate(&prompt, Default::default())?;
            Ok(response)
        } else {
            // Fallback: Search changelog
            Ok(self.fetch_changelog(upgrade)?)
        }
    }

    fn fetch_changelog(&self, upgrade: &DependencyUpgrade) -> Result<String> {
        // Fetch from GitHub releases, Maven Central, etc.
        Ok(format!("See changelog: https://github.com/{}/releases", upgrade.package_name))
    }
}
```

**CLI:**
```bash
# Generate migration guide
bazbom fix --upgrade spring-web:5.3.20:6.0.0 --explain

# Output (LLM-generated):
# Migration Guide: spring-web 5.3.20 → 6.0.0
#
# Breaking Changes:
#
# 1. Jakarta EE 9 Migration
#    - Replace: javax.servlet.* → jakarta.servlet.*
#    - Replace: javax.persistence.* → jakarta.persistence.*
#
# 2. Deprecated APIs Removed
#    - @RequestMapping shortcuts removed
#    - Use @GetMapping, @PostMapping, etc. instead
#
# 3. Minimum Java Version
#    - Spring 6 requires Java 17+
#    - Update your build: <maven.compiler.target>17</maven.compiler.target>
#
# 4. WebMvcConfigurer Changes
#    - Method signatures changed for CORS configuration
#    - See: https://docs.spring.io/spring-framework/docs/6.0.0/reference/html/web.html#mvc-cors
#
# Estimated effort: 4-8 hours for typical Spring Boot app
```

### Data Sources

**Training Data (if training custom model):**
- GitHub issue/PR descriptions (breaking change discussions)
- Stack Overflow Q&A (migration questions)
- Official migration guides (Spring, Apache, etc.)

**Inference (runtime):**
- Package changelogs
- GitHub release notes
- Community discussions

---

## 10.4 Intelligent Triage

### Goal

**Auto-categorize vulnerabilities:**
- **False Positive** - Doesn't apply to your code (e.g., Windows-only vuln in Linux app)
- **Real Threat** - Likely exploitable in your environment
- **Low Priority** - Real vuln but low impact (test-only dependency)

### Approach

**Classification Model:** Random Forest or Neural Network

**Training Data:**
- Your historical triage decisions (if available)
- Public CVE analysis (exploit-db, Metasploit modules)
- Reachability data (from Phase 3)

**Implementation:**
```rust
// crates/bazbom-ml/src/intelligent_triage.rs
pub struct IntelligentTriage {
    classifier: Classifier,
}

impl IntelligentTriage {
    pub fn categorize_vulnerability(&self, vuln: &Vulnerability, context: &AppContext) -> VulnCategory {
        let features = vec![
            vuln.cvss_score.unwrap_or(0.0),
            if vuln.reachable { 1.0 } else { 0.0 },
            if vuln.cisa_kev { 1.0 } else { 0.0 },
            if context.public_facing { 1.0 } else { 0.0 },
            vuln.epss.unwrap_or(0.0),
            // ... more features
        ];

        let prediction = self.classifier.predict(&features);

        match prediction {
            0 => VulnCategory::FalsePositive,
            1 => VulnCategory::RealThreat,
            2 => VulnCategory::LowPriority,
            _ => VulnCategory::Unknown,
        }
    }
}
```

**CLI:**
```bash
bazbom scan --intelligent-triage

# Output:
# Vulnerability Triage:
#
# 🔴 Real Threats (2) - Fix immediately
#   - CVE-2024-xxxx in spring-web (Confidence: 95%)
#   - CVE-2023-yyyy in log4j-core (Confidence: 89%)
#
# 🟡 Low Priority (5) - Plan fixes
#   - CVE-2022-zzzz in junit (Test-only, Confidence: 92%)
#   - ...
#
# ✅ False Positives (3) - Ignore
#   - CVE-2021-aaaa in netty (Windows-only, you're on Linux, Confidence: 87%)
#   - ...
```

---

## Privacy & Ethics

### Principles

1. **Local-First:** All ML models run locally (no data sent to cloud by default)
2. **Opt-In Cloud:** User explicitly enables OpenAI/Anthropic integration
3. **Transparent:** Document what data is used for training
4. **No PII:** Never train on source code containing secrets/credentials

### Implementation

**Config:**
```yaml
# bazbom.yml
ml:
  enabled: true
  exploit_prediction: true
  anomaly_detection: true
  llm_assistant:
    provider: "local"  # Options: "local", "openai", "anthropic", "disabled"
    model: "llama3-8b"
    api_key: null  # Only if using cloud provider
```

**Privacy Notice:**
```
BazBOM ML Features:

✅ Local Models (Default):
   - Exploit prediction (XGBoost model, 2MB)
   - Anomaly detection (Isolation Forest, 1MB)
   - LLM assistant (Llama 3 8B, 8GB RAM required)
   - Your data never leaves your machine

☁️ Cloud LLM (Optional):
   - If you enable OpenAI/Anthropic integration:
   - Package names, versions, CVE IDs sent to API
   - No source code, no secrets, no PII
   - See privacy policy: https://bazbom.io/privacy

Configure: bazbom config set ml.llm_assistant.provider local|openai|disabled
```

---

## Success Criteria

### Phase 10 Completion Checklist

- [ ] Custom exploit prediction model trained and evaluated (>90% accuracy)
- [ ] Anomaly detection detects unusual dependencies (>80% accuracy)
- [ ] LLM assistant generates useful migration guides (>70% satisfaction)
- [ ] Intelligent triage categorizes vulnerabilities correctly (>85% accuracy)
- [ ] Local LLM integration (Llama 3 or Mistral)
- [ ] Optional cloud LLM integration (OpenAI, Anthropic)
- [ ] Privacy-preserving architecture (local-first)
- [ ] Comprehensive documentation on ML features
- [ ] User can disable all ML features

### Competitive Benchmark

**After Phase 10:**

| Feature | Competitors | BazBOM |
|---------|------------|--------|
| **Rule-Based Prioritization** | ✅ All | ✅ Yes |
| **ML Exploit Prediction** | ⚠️ Some (proprietary) | ✅ Custom model |
| **Anomaly Detection** | ❌ Rare | ✅ Yes |
| **LLM Assistant** | ❌ None (as of 2024) | ✅ Local + Cloud |
| **Intelligent Triage** | ⚠️ Some | ✅ Yes |
| **Privacy** | ❌ Cloud-first | ✅ Local-first |

**Differentiation:** Only open source SCA with AI-native features and local-first privacy.

---

## Resource Requirements

**Team:** 1 ML engineer + 1 Rust developer for 16 weeks
**Skills:** Machine learning (XGBoost, PyTorch), Rust, LLM integration
**Budget:** $64K-96K (contractors)

**Infrastructure:**
- GPU for model training (cloud credits or local)
- LLM hosting (local: 16GB+ RAM, cloud: API keys)
- Training data storage (historical CVEs, changelogs)

---

**Last Updated:** 2025-10-30
**Next:** Phase 11 (Distribution)
