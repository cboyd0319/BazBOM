# CI/CD Integration

Integrate BazBOM into CI pipelines for automated SBOM generation and vulnerability scanning.

## GitHub Actions (Recommended)

### Minimal Workflow

```yaml
name: BazBOM Security Scan

on:
  push:
    branches: [main]
  pull_request:

jobs:
  sbom:
    runs-on: ubuntu-latest
    
    permissions:
      contents: read
      security-events: write
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Run BazBOM
        uses: cboyd0319/BazBOM@main
        with:
          fail-on-critical: true
          upload-sbom: true
          upload-sarif: true
```

**What it does:**
1. Auto-detects build system (Maven/Gradle/Bazel)
2. Generates SBOM
3. Scans for vulnerabilities
4. Uploads SARIF to GitHub Security tab
5. Fails build on critical CVEs

### With Caching

```yaml
name: BazBOM with Cache

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Cache BazBOM Database
        uses: actions/cache@v4
        with:
          path: ~/.bazbom/db
          key: bazbom-db-${{ runner.os }}-${{ hashFiles('.bazbom-db-version') }}
          restore-keys: bazbom-db-${{ runner.os }}-
      
      - name: Sync Advisory Database
        run: |
          bazbom db sync
          date > .bazbom-db-version
      
      - name: Scan Project
        run: bazbom scan . --format spdx --out-dir ./reports
      
      - name: Upload SBOM
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: ./reports/sbom.spdx.json
      
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: ./reports/sca_findings.sarif
```

**Why cache:** Avoid downloading CVE database on every run (saves 2-3 minutes).

### Bazel-Specific

```yaml
name: Bazel Monorepo Scan

on:
  pull_request:

jobs:
  incremental-scan:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Need history for git diff
      
      - uses: bazelbuild/setup-bazel@v6
        with:
          bazel-version: 7.x
      
      - name: Incremental Scan (Affected Targets)
        run: |
          bazbom scan . \
            --bazel-affected-by-files $(git diff --name-only origin/main...HEAD) \
            --out-dir ./reports
      
      - name: Upload Results
        uses: actions/upload-artifact@v4
        with:
          name: sbom-incremental
          path: ./reports/
```

**Performance:** 6x faster (8 min vs 45 min for full scan)

### Maven-Specific

```yaml
name: Maven Project Scan

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - uses: actions/setup-java@v4
        with:
          distribution: 'temurin'
          java-version: '17'
      
      - name: Build and Generate Dependency Graph
        run: mvn clean install bazbom:graph
      
      - name: Scan with BazBOM
        run: bazbom scan . --format spdx --out-dir ./reports
      
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: ./reports/sca_findings.sarif
```

### Gradle-Specific

```yaml
name: Gradle Project Scan

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - uses: actions/setup-java@v4
        with:
          distribution: 'temurin'
          java-version: '17'
      
      - name: Setup Gradle
        uses: gradle/gradle-build-action@v3
      
      - name: Generate Dependency Graph
        run: ./gradlew bazbomGraph
      
      - name: Scan with BazBOM
        run: bazbom scan . --format spdx --out-dir ./reports
      
      - name: Upload Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: ./reports/
```

## GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - security

sbom-scan:
  stage: security
  image: ubuntu:latest
  
before_script:
    - apt-get update && apt-get install -y git curl pkg-config libssl-dev
    - git clone https://github.com/cboyd0319/BazBOM.git /tmp/bazbom
    - (cd /tmp/bazbom && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source $HOME/.cargo/env && cargo build --release -p bazbom)
    - install -m 0755 /tmp/bazbom/target/release/bazbom /usr/local/bin/bazbom
  
  script:
    - bazbom db sync
    - bazbom scan . --format spdx --out-dir ./reports
  
  artifacts:
    paths:
      - reports/
    expire_in: 30 days
```

## Jenkins

```groovy
pipeline {
    agent any
    
    stages {
        stage('Install BazBOM') {
            steps {
                sh '''
                  set -e
                  if ! command -v cargo >/dev/null; then
                    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                  fi
                  source "$HOME/.cargo/env"
                  rm -rf bazbom-src
                  git clone https://github.com/cboyd0319/BazBOM.git bazbom-src
                  cd bazbom-src
                  cargo build --release -p bazbom
                  sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom
                '''
            }
        }
        
        stage('Sync Database') {
            steps {
                sh 'bazbom db sync'
            }
        }
        
        stage('Scan') {
            steps {
                sh 'bazbom scan . --format spdx --out-dir ./reports'
            }
        }
        
        stage('Archive') {
            steps {
                archiveArtifacts artifacts: 'reports/**/*', fingerprint: true
            }
        }
    }
}
```

## CircleCI

```yaml
# .circleci/config.yml
version: 2.1

jobs:
  sbom-scan:
    docker:
      - image: cimg/rust:1.77
   
    steps:
      - checkout
      
      - run:
          name: Install BazBOM
          command: |
            sudo apt-get update && sudo apt-get install -y pkg-config libssl-dev git
            git clone --depth 1 https://github.com/cboyd0319/BazBOM.git /tmp/bazbom
            cd /tmp/bazbom
            cargo build --release -p bazbom
            sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom
      
      - restore_cache:
          keys:
            - bazbom-db-v1-{{ checksum ".bazbom-db-version" }}
            - bazbom-db-v1-
      
      - run:
          name: Sync Database
          command: |
            bazbom db sync
            date > .bazbom-db-version
      
      - save_cache:
          key: bazbom-db-v1-{{ checksum ".bazbom-db-version" }}
          paths:
            - ~/.bazbom/db
      
      - run:
          name: Scan
          command: bazbom scan . --format spdx --out-dir ./reports
      
      - store_artifacts:
          path: ./reports

workflows:
  version: 2
  scan:
    jobs:
      - sbom-scan
```

## Caching Strategies

### Advisory Database

**What to cache:** `~/.bazbom/db/` (OSV/NVD/GHSA/KEV/EPSS)

**Update frequency:** Daily (24 hours)

**Size:** ~200 MB compressed

**GitHub Actions:**
```yaml
- uses: actions/cache@v4
  with:
    path: ~/.bazbom/db
    key: bazbom-db-${{ runner.os }}-${{ hashFiles('.bazbom-db-version') }}
```

**GitLab CI:**
```yaml
cache:
  key: bazbom-db-$CI_COMMIT_REF_SLUG
  paths:
    - ~/.bazbom/db
```

### Build Artifacts

**What to cache:** Maven `.m2`, Gradle `.gradle`, Bazel `~/.cache/bazel`

**Why:** Faster dependency resolution

**GitHub Actions:**
```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.m2/repository
      ~/.gradle/caches
      ~/.cache/bazel
    key: deps-${{ runner.os }}-${{ hashFiles('**/pom.xml', '**/*.gradle*', 'MODULE.bazel') }}
```

## Policy Enforcement

### Fail on Critical CVEs

```yaml
- name: Policy Check
  run: bazbom policy check --fail-on critical
```

**Exit codes:**
- `0` - Pass
- `1` - Policy violations found

### Custom Policy File

```yaml
- name: Check PCI-DSS Compliance
  run: bazbom policy check --policy-file policies/pci-dss.yml
```

**Policy file example:**

```yaml
# policies/pci-dss.yml
policy:
  name: PCI-DSS
  severity_threshold: high
  block_licenses:
    - GPL-2.0
    - GPL-3.0
  require_sbom: true
  require_provenance: true
```

## Artifact Upload

### GitHub Security Tab

```yaml
- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: ./reports/sca_findings.sarif
```

**Result:** Vulnerabilities appear in GitHub Security tab

### SBOM Attestation

```yaml
- name: Generate Attestation
  uses: actions/attest-sbom@v1
  with:
    subject-path: ./reports/sbom.spdx.json
    sbom-path: ./reports/sbom.spdx.json
```

**Why:** SLSA Level 3 provenance

### Dependency Track Integration

```yaml
- name: Upload to Dependency-Track
  env:
    DTRACK_URL: ${{ secrets.DTRACK_URL }}
    DTRACK_API_KEY: ${{ secrets.DTRACK_API_KEY }}
  run: |
    curl -X POST "$DTRACK_URL/api/v1/bom" \
      -H "X-Api-Key: $DTRACK_API_KEY" \
      -F "project=myapp" \
      -F "bom=@./reports/sbom.cyclonedx.json"
```

## Failure Modes

### Database Sync Fails

**Symptom:** `bazbom db sync` times out

**Cause:** Network issues, rate limiting

**Fix:** Use cached database:

```yaml
- name: Sync Database (with fallback)
  run: |
    bazbom db sync || echo "Using cached database"
```

### Build System Not Detected

**Symptom:** "No build system detected"

**Cause:** Missing build files or unsupported structure

**Fix:** Explicit build system flag:

```yaml
- run: bazbom scan . --build-system maven
```

### SARIF Upload Fails

**Symptom:** "security-events: write permission required"

**Cause:** Missing permissions in workflow

**Fix:**

```yaml
permissions:
  contents: read
  security-events: write
```

## Debugging CI Failures

### Enable Verbose Logging

```yaml
- name: Scan with Debug Logs
  run: bazbom scan . --log-level debug
```

### Dry Run Mode

```yaml
- name: Test Scan (No Upload)
  run: bazbom scan . --dry-run
```

### Inspect Outputs

```yaml
- name: Show SBOM
  run: cat ./reports/sbom.spdx.json | jq '.packages | length'

- name: Show Findings
  run: cat ./reports/sca_findings.json | jq '.vulnerabilities'
```

## Performance Tips

1. **Use caching** - Cache advisory database and build artifacts
2. **Incremental scans** - For Bazel, use `--bazel-affected-by-files`
3. **Parallel jobs** - Run SBOM generation and tests in parallel
4. **Fast mode** - Use `--fast` flag for pre-commit hooks
5. **Remote cache** - For Bazel, configure remote build cache

**Benchmark:**
- Full scan: 8-15 minutes (5000+ targets)
- Incremental: 2-5 minutes (50-100 affected targets)
- Fast mode: <10 seconds (no reachability)

## Example: Complete GitHub Actions Workflow

```yaml
name: Complete Security Pipeline

on:
  push:
    branches: [main]
  pull_request:

jobs:
  security-scan:
    runs-on: ubuntu-latest
    
    permissions:
      contents: read
      security-events: write
      id-token: write  # For SLSA attestation
    
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Setup Build System
        uses: actions/setup-java@v4
        with:
          distribution: 'temurin'
          java-version: '17'
      
      - name: Cache Dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.m2/repository
            ~/.gradle/caches
            ~/.bazbom/db
          key: ${{ runner.os }}-deps-${{ hashFiles('**/pom.xml', '**/*.gradle*') }}
      
      - uses: dtolnay/rust-toolchain@stable

      - name: Install BazBOM (build from source)
        run: |
          git clone --depth 1 https://github.com/cboyd0319/BazBOM.git /tmp/bazbom
          cd /tmp/bazbom
          cargo build --release -p bazbom
          sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom
          bazbom --version
      
      - name: Sync Advisory Database
        run: bazbom db sync
      
      - name: Build Project
        run: |
          if [ -f pom.xml ]; then
            mvn clean install -DskipTests
          elif [ -f build.gradle ]; then
            ./gradlew build -x test
          fi
      
      - name: Generate SBOM
        run: bazbom scan . --format spdx --out-dir ./reports
      
      - name: Policy Check
        run: bazbom policy check --fail-on critical
      
      - name: Upload SBOM
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: ./reports/sbom.spdx.json
      
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: ./reports/sca_findings.sarif
      
      - name: Attest SBOM
        uses: actions/attest-sbom@v1
        with:
          subject-path: ./reports/sbom.spdx.json
          sbom-path: ./reports/sbom.spdx.json
```

## Next Steps

- [Usage guide](USAGE.md) - CLI commands
- [Policy integration](user-guide/policy-integration.md) - Custom policies
- [Troubleshooting](user-guide/troubleshooting.md) - Debug CI issues
- [GitHub Action docs](../action.yml) - Full configuration reference
