# BazBOM: Next-Level Features Implementation Specification

> **Purpose:** Transform BazBOM from excellent SBOM tool to the **industry-leading supply chain security platform** for Bazel ecosystems.

---

## Table of Contents

1. [Vulnerability Data Enrichment (Foundation)](#0-vulnerability-data-enrichment-foundation)
2. [SBOM Attestation & Transparency Logs](#1-sbom-attestation--transparency-logs)
3. [Automated SBOM Compliance Reports](#2-automated-sbom-compliance-reports)
4. [Policy-as-Code Framework](#3-policy-as-code-framework-with-industry-templates)
5. [Continuous SBOM Diffing & Drift Detection](#4-continuous-sbom-diffing--drift-detection)
6. [Supply Chain Attack Detection](#5-supply-chain-attack-detection-advanced)
7. [Open-Source Vulnerability Database Contributions](#6-open-source-vulnerability-database-contributions)
8. [Community-Driven Benchmark Suite](#7-community-driven-benchmark-suite)
9. [AI Chat Interface for SBOM Queries](#8-ai-chat-interface-for-sbom-queries)
10. [AI-Powered Dependency Upgrade Recommendations](#9-ai-powered-dependency-upgrade-recommendations)

---

## 0. Vulnerability Data Enrichment (Foundation)

**Priority:** FIRST (Foundation for all other features)
**Effort:** 2-3 weeks
**Impact:** HIGH - Enriches all downstream features

### The Problem

Current vulnerability data from OSV/NVD lacks critical context:
- **No prioritization** - All "High" CVEs treated equally, but some are exploited in the wild
- **No exploitability data** - Can't distinguish theoretical vs. practical vulnerabilities
- **No business context** - Missing whether vulnerability affects our specific usage

### The Solution: Multi-Source Vulnerability Enrichment

Integrate **4 critical data sources** beyond OSV/NVD:

#### Source 1: CISA KEV (Known Exploited Vulnerabilities)
**What:** US Cybersecurity & Infrastructure Security Agency's catalog of CVEs actively exploited
**Why:** If a CVE is in KEV, it's a CRITICAL priority (real-world attacks happening NOW)
**API:** https://www.cisa.gov/known-exploited-vulnerabilities-catalog
**Data Format:** JSON catalog updated daily

**Example Entry:**
```json
{
  "cveID": "CVE-2021-44228",
  "vendorProject": "Apache",
  "product": "Log4j",
  "vulnerabilityName": "Log4Shell",
  "dateAdded": "2021-12-10",
  "shortDescription": "Remote code execution via JNDI lookup",
  "requiredAction": "Apply updates per vendor instructions",
  "dueDate": "2021-12-24"
}
```

**Integration Plan:**
```python
# tools/supplychain/kev_enrichment.py

import requests
from datetime import datetime, timedelta
from typing import Dict, Optional

class KEVEnricher:
    """Enrich vulnerabilities with CISA KEV data."""

    KEV_CATALOG_URL = "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json"
    CACHE_TTL_HOURS = 24

    def __init__(self, cache_dir: str = ".bazel-cache/kev"):
        self.cache_dir = cache_dir
        self._kev_catalog = None

    def fetch_kev_catalog(self) -> Dict:
        """Download latest KEV catalog with caching."""
        cache_file = f"{self.cache_dir}/kev_catalog.json"

        # Check cache freshness
        if os.path.exists(cache_file):
            cache_age = datetime.now() - datetime.fromtimestamp(os.path.getmtime(cache_file))
            if cache_age < timedelta(hours=self.CACHE_TTL_HOURS):
                with open(cache_file) as f:
                    return json.load(f)

        # Fetch fresh data
        response = requests.get(self.KEV_CATALOG_URL, timeout=30)
        response.raise_for_status()
        data = response.json()

        # Cache it
        os.makedirs(self.cache_dir, exist_ok=True)
        with open(cache_file, 'w') as f:
            json.dump(data, f)

        return data

    def is_known_exploited(self, cve_id: str) -> Optional[Dict]:
        """Check if CVE is in CISA KEV catalog."""
        if not self._kev_catalog:
            self._kev_catalog = self.fetch_kev_catalog()

        for vuln in self._kev_catalog.get("vulnerabilities", []):
            if vuln.get("cveID") == cve_id:
                return {
                    "in_kev": True,
                    "date_added": vuln.get("dateAdded"),
                    "due_date": vuln.get("dueDate"),
                    "required_action": vuln.get("requiredAction"),
                    "notes": vuln.get("notes", ""),
                    "vulnerability_name": vuln.get("vulnerabilityName")
                }

        return {"in_kev": False}

    def enrich_finding(self, finding: Dict) -> Dict:
        """Add KEV context to vulnerability finding."""
        cve_id = finding.get("cve") or finding.get("id")
        if not cve_id or not cve_id.startswith("CVE-"):
            return finding

        kev_data = self.is_known_exploited(cve_id)
        finding["kev"] = kev_data

        # Boost severity if in KEV
        if kev_data["in_kev"]:
            finding["effective_severity"] = "CRITICAL"
            finding["priority"] = "IMMEDIATE"
            finding["kev_context"] = f"⚠️ ACTIVELY EXPLOITED: {kev_data['vulnerability_name']}"

        return finding
```

**Usage:**
```python
# In osv_query.py
from kev_enrichment import KEVEnricher

enricher = KEVEnricher()
findings = osv_scan_results()

for finding in findings:
    finding = enricher.enrich_finding(finding)

    if finding["kev"]["in_kev"]:
        print(f"🚨 CRITICAL: {finding['cve']} is KNOWN EXPLOITED IN THE WILD!")
```

---

#### Source 2: EPSS (Exploit Prediction Scoring System)
**What:** FIRST.org's ML-based prediction of likelihood a CVE will be exploited
**Why:** Prioritize CVEs most likely to be exploited (0.1% vs 90% probability matters!)
**API:** https://api.first.org/data/v1/epss
**Data Format:** CSV/JSON scores updated daily

**Example Entry:**
```json
{
  "cve": "CVE-2021-44228",
  "epss": "0.97538",  // 97.5% probability of exploitation
  "percentile": "0.99999"  // Top 0.001% most dangerous
}
```

**Integration Plan:**
```python
# tools/supplychain/epss_enrichment.py

import requests
import csv
from io import StringIO
from typing import Dict, List

class EPSSEnricher:
    """Enrich vulnerabilities with EPSS scores."""

    EPSS_API_URL = "https://api.first.org/data/v1/epss"
    BATCH_SIZE = 100  # API supports batch queries
    CACHE_TTL_HOURS = 24

    def __init__(self, cache_dir: str = ".bazel-cache/epss"):
        self.cache_dir = cache_dir

    def fetch_epss_scores(self, cve_list: List[str]) -> Dict[str, Dict]:
        """Fetch EPSS scores for multiple CVEs (batched)."""
        scores = {}

        # Batch CVEs to reduce API calls
        for i in range(0, len(cve_list), self.BATCH_SIZE):
            batch = cve_list[i:i + self.BATCH_SIZE]

            # Query API
            params = {"cve": ",".join(batch)}
            response = requests.get(self.EPSS_API_URL, params=params, timeout=30)
            response.raise_for_status()

            data = response.json()
            for entry in data.get("data", []):
                cve = entry.get("cve")
                scores[cve] = {
                    "epss_score": float(entry.get("epss", 0)),
                    "epss_percentile": float(entry.get("percentile", 0)),
                    "date": entry.get("date")
                }

        return scores

    def get_priority_level(self, epss_score: float) -> str:
        """Map EPSS score to priority level."""
        if epss_score >= 0.75:
            return "CRITICAL"  # Top 25% most likely
        elif epss_score >= 0.50:
            return "HIGH"
        elif epss_score >= 0.25:
            return "MEDIUM"
        else:
            return "LOW"

    def enrich_findings(self, findings: List[Dict]) -> List[Dict]:
        """Add EPSS scores to all findings."""
        cve_list = [f.get("cve") for f in findings if f.get("cve")]

        if not cve_list:
            return findings

        epss_scores = self.fetch_epss_scores(cve_list)

        for finding in findings:
            cve = finding.get("cve")
            if cve and cve in epss_scores:
                epss_data = epss_scores[cve]
                finding["epss"] = epss_data
                finding["exploitation_probability"] = f"{epss_data['epss_score'] * 100:.1f}%"
                finding["epss_priority"] = self.get_priority_level(epss_data["epss_score"])

        return findings
```

**Output Enhancement:**
```json
{
  "cve": "CVE-2021-44228",
  "severity": "CRITICAL",
  "cvss_score": 10.0,
  "epss": {
    "epss_score": 0.97538,
    "epss_percentile": 0.99999,
    "date": "2025-01-17"
  },
  "exploitation_probability": "97.5%",
  "epss_priority": "CRITICAL",
  "kev": {
    "in_kev": true,
    "vulnerability_name": "Log4Shell"
  }
}
```

---

#### Source 3: VulnCheck KEV (Extended Exploit Intelligence)
**What:** Commercial-grade exploit intelligence (free tier available)
**Why:** More detailed exploit data than CISA KEV (exploit maturity, attack vectors)
**API:** https://docs.vulncheck.com/
**Data:** Exploit availability, weaponization status, attack complexity

**Integration Plan:**
```python
# tools/supplychain/vulncheck_enrichment.py

class VulnCheckEnricher:
    """Enrich with VulnCheck exploit intelligence."""

    API_URL = "https://api.vulncheck.com/v3/index/vulncheck-kev"

    def __init__(self, api_key: Optional[str] = None):
        # API key optional for public KEV, required for advanced features
        self.api_key = api_key or os.getenv("VULNCHECK_API_KEY")

    def get_exploit_status(self, cve_id: str) -> Dict:
        """Get detailed exploit status."""
        headers = {"Authorization": f"Bearer {self.api_key}"} if self.api_key else {}

        response = requests.get(
            f"{self.API_URL}",
            params={"cve": cve_id},
            headers=headers,
            timeout=30
        )

        if response.status_code == 200:
            data = response.json()
            return {
                "exploit_available": data.get("exploit_available", False),
                "exploit_maturity": data.get("exploit_maturity", "unknown"),
                "attack_vector": data.get("attack_vector", "unknown"),
                "weaponized": data.get("weaponized", False)
            }

        return {"exploit_available": False}
```

---

#### Source 4: GitHub Security Advisories (Enhanced)
**What:** GitHub's curated security advisories with ecosystem-specific context
**Why:** Language/framework-specific remediation guidance
**API:** https://docs.github.com/en/graphql/reference/objects#securityadvisory
**Data:** Affected versions, patched versions, workarounds

**Integration Plan:**
```python
# tools/supplychain/ghsa_enrichment.py

class GHSAEnricher:
    """Enrich with GitHub Security Advisory data."""

    GRAPHQL_URL = "https://api.github.com/graphql"

    def __init__(self, github_token: Optional[str] = None):
        self.token = github_token or os.getenv("GITHUB_TOKEN")

    def query_advisory(self, cve_id: str) -> Dict:
        """Fetch GHSA data via GraphQL."""
        query = """
        query($cve: String!) {
          securityAdvisories(first: 1, identifier: {type: CVE, value: $cve}) {
            nodes {
              summary
              description
              severity
              vulnerabilities(first: 10) {
                nodes {
                  package { name ecosystem }
                  vulnerableVersionRange
                  firstPatchedVersion { identifier }
                }
              }
              references { url }
            }
          }
        }
        """

        headers = {"Authorization": f"bearer {self.token}"}
        response = requests.post(
            self.GRAPHQL_URL,
            json={"query": query, "variables": {"cve": cve_id}},
            headers=headers,
            timeout=30
        )

        if response.status_code == 200:
            data = response.json()
            advisories = data.get("data", {}).get("securityAdvisories", {}).get("nodes", [])
            if advisories:
                return advisories[0]

        return {}
```

---

### Combined Enrichment Pipeline

```python
# tools/supplychain/vulnerability_enrichment.py

from typing import List, Dict
from kev_enrichment import KEVEnricher
from epss_enrichment import EPSSEnricher
from vulncheck_enrichment import VulnCheckEnricher
from ghsa_enrichment import GHSAEnricher

class VulnerabilityEnricher:
    """Master enrichment pipeline combining all sources."""

    def __init__(self):
        self.kev = KEVEnricher()
        self.epss = EPSSEnricher()
        self.vulncheck = VulnCheckEnricher()
        self.ghsa = GHSAEnricher()

    def enrich_all(self, findings: List[Dict]) -> List[Dict]:
        """Enrich findings with all available data sources."""

        # Step 1: Add EPSS scores (batch operation)
        findings = self.epss.enrich_findings(findings)

        # Step 2: Add KEV status (batch from cache)
        for finding in findings:
            finding = self.kev.enrich_finding(finding)

        # Step 3: Add exploit intelligence (optional, requires API key)
        for finding in findings:
            cve = finding.get("cve")
            if cve:
                exploit_data = self.vulncheck.get_exploit_status(cve)
                finding["exploit"] = exploit_data

        # Step 4: Add GHSA remediation guidance
        for finding in findings:
            cve = finding.get("cve")
            if cve:
                ghsa_data = self.ghsa.query_advisory(cve)
                finding["ghsa"] = ghsa_data

        # Step 5: Calculate composite risk score
        for finding in findings:
            finding["risk_score"] = self._calculate_risk_score(finding)
            finding["priority"] = self._calculate_priority(finding)

        # Step 6: Sort by risk score (highest first)
        findings.sort(key=lambda x: x.get("risk_score", 0), reverse=True)

        return findings

    def _calculate_risk_score(self, finding: Dict) -> float:
        """Calculate composite risk score (0-100)."""
        score = 0.0

        # Base CVSS score (0-40 points)
        cvss = finding.get("cvss_score", 0)
        score += (cvss / 10.0) * 40

        # EPSS score (0-30 points)
        epss = finding.get("epss", {}).get("epss_score", 0)
        score += epss * 30

        # KEV status (0-20 points)
        if finding.get("kev", {}).get("in_kev"):
            score += 20

        # Exploit availability (0-10 points)
        if finding.get("exploit", {}).get("weaponized"):
            score += 10
        elif finding.get("exploit", {}).get("exploit_available"):
            score += 5

        return round(score, 2)

    def _calculate_priority(self, finding: Dict) -> str:
        """Calculate actionable priority level."""
        risk_score = finding.get("risk_score", 0)
        in_kev = finding.get("kev", {}).get("in_kev", False)

        # KEV = immediate priority
        if in_kev:
            return "P0-IMMEDIATE"

        # High risk score
        if risk_score >= 80:
            return "P1-CRITICAL"
        elif risk_score >= 60:
            return "P2-HIGH"
        elif risk_score >= 40:
            return "P3-MEDIUM"
        else:
            return "P4-LOW"
```

---

### Enhanced Output Format

```json
{
  "cve": "CVE-2021-44228",
  "package": {
    "purl": "pkg:maven/org.apache.logging.log4j/log4j-core@2.14.1",
    "name": "log4j-core",
    "version": "2.14.1"
  },

  "base_metrics": {
    "severity": "CRITICAL",
    "cvss_score": 10.0,
    "cvss_vector": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:C/C:H/I:H/A:H"
  },

  "kev": {
    "in_kev": true,
    "date_added": "2021-12-10",
    "due_date": "2021-12-24",
    "vulnerability_name": "Log4Shell",
    "required_action": "Apply updates per vendor instructions"
  },

  "epss": {
    "epss_score": 0.97538,
    "epss_percentile": 0.99999,
    "exploitation_probability": "97.5%",
    "epss_priority": "CRITICAL"
  },

  "exploit": {
    "exploit_available": true,
    "exploit_maturity": "functional",
    "weaponized": true,
    "attack_vector": "network"
  },

  "ghsa": {
    "summary": "Remote code execution via JNDI lookup",
    "first_patched_version": "2.15.0",
    "vulnerable_version_range": ">=2.0.0, <2.15.0",
    "references": [
      "https://github.com/advisories/GHSA-jfh8-c2jp-5v3q",
      "https://nvd.nist.gov/vuln/detail/CVE-2021-44228"
    ]
  },

  "risk_score": 97.54,
  "priority": "P0-IMMEDIATE",
  "priority_explanation": "Known exploited (KEV) + 97.5% exploitation probability (EPSS) + weaponized exploit available",

  "affected_targets": ["//services/api", "//services/auth"],

  "remediation": {
    "fixed_versions": ["2.15.0", "2.16.0", "2.17.0"],
    "recommended_action": "Upgrade to 2.17.0 immediately",
    "estimated_effort": "2-4 hours",
    "workaround": "Set log4j2.formatMsgNoLookups=true"
  }
}
```

---

### Integration into BazBOM

**File:** `tools/supplychain/osv_query.py`

```python
# Add to osv_query.py

from vulnerability_enrichment import VulnerabilityEnricher

def scan_dependencies(sbom_path: str) -> Dict:
    """Enhanced vulnerability scanning with enrichment."""

    # Step 1: Basic OSV/NVD scan (existing code)
    findings = osv_scan(sbom_path)

    # Step 2: ENRICH with KEV, EPSS, exploit data
    enricher = VulnerabilityEnricher()
    findings = enricher.enrich_all(findings)

    # Step 3: Generate prioritized output
    return {
        "scan_date": datetime.now().isoformat(),
        "total_findings": len(findings),
        "by_priority": {
            "P0_IMMEDIATE": [f for f in findings if f["priority"] == "P0-IMMEDIATE"],
            "P1_CRITICAL": [f for f in findings if f["priority"] == "P1-CRITICAL"],
            "P2_HIGH": [f for f in findings if f["priority"] == "P2-HIGH"],
            "P3_MEDIUM": [f for f in findings if f["priority"] == "P3-MEDIUM"],
            "P4_LOW": [f for f in findings if f["priority"] == "P4-LOW"]
        },
        "findings": findings
    }
```

---

### SARIF Enhancement

**File:** `tools/supplychain/sarif_adapter.py`

```python
def convert_to_sarif(findings: List[Dict]) -> Dict:
    """Enhanced SARIF with KEV/EPSS context."""

    results = []
    for finding in findings:
        result = {
            "ruleId": finding["cve"],
            "level": _map_priority_to_level(finding["priority"]),
            "message": {
                "text": _format_enriched_message(finding)
            },
            "locations": [...],
            "properties": {
                "cvss": finding["base_metrics"]["cvss_score"],
                "epss": finding.get("epss", {}).get("epss_score"),
                "in_kev": finding.get("kev", {}).get("in_kev", False),
                "risk_score": finding.get("risk_score"),
                "priority": finding.get("priority"),
                "exploitation_probability": finding.get("exploitation_probability")
            }
        }
        results.append(result)

    return {"version": "2.1.0", "runs": [{"results": results}]}

def _format_enriched_message(finding: Dict) -> str:
    """Format human-readable message with enrichment context."""
    msg = f"{finding['cve']}: {finding.get('summary', 'Vulnerability detected')}\n\n"

    # Add KEV warning
    if finding.get("kev", {}).get("in_kev"):
        msg += f"⚠️ KNOWN EXPLOITED IN THE WILD (CISA KEV)\n"
        msg += f"Due Date: {finding['kev']['due_date']}\n\n"

    # Add EPSS context
    epss = finding.get("epss", {})
    if epss:
        msg += f"Exploitation Probability: {finding.get('exploitation_probability')} "
        msg += f"(EPSS Percentile: {epss.get('epss_percentile', 0) * 100:.2f}%)\n\n"

    # Add exploit status
    exploit = finding.get("exploit", {})
    if exploit.get("weaponized"):
        msg += f"⚠️ WEAPONIZED EXPLOIT AVAILABLE\n\n"
    elif exploit.get("exploit_available"):
        msg += f"Public exploit available ({exploit.get('exploit_maturity')})\n\n"

    # Add remediation
    remediation = finding.get("remediation", {})
    if remediation:
        msg += f"Recommended Action: {remediation.get('recommended_action')}\n"

    return msg
```

---

### CLI Output Enhancement

```bash
$ bazel run //:sca_scan

🔍 Vulnerability Scan Complete
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📊 Summary:
  Total Findings: 23

  🚨 P0 - IMMEDIATE (KEV):     2  ← FIX NOW
  🔴 P1 - CRITICAL:            5  ← This week
  🟠 P2 - HIGH:                8  ← This sprint
  🟡 P3 - MEDIUM:              6  ← Next quarter
  🟢 P4 - LOW:                 2  ← Backlog

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🚨 P0 - IMMEDIATE ACTION REQUIRED

1. CVE-2021-44228 (Log4Shell)
   Package: org.apache.logging.log4j:log4j-core@2.14.1
   Risk Score: 97.5/100

   ⚠️ KNOWN EXPLOITED (CISA KEV - Due: 2021-12-24)
   ⚠️ WEAPONIZED EXPLOIT AVAILABLE
   📈 97.5% exploitation probability (EPSS)

   Affects: //services/api, //services/auth (2 targets)
   Fix: Upgrade to 2.17.0 (Est. 2-4 hours)

   $ bazel run //tools:upgrade -- org.apache.logging.log4j:log4j-core:2.17.0

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Full report: bazel-bin/sca_findings_enriched.json
SARIF output: bazel-bin/sca_findings.sarif
```

---

### Data Sources Configuration

**File:** `tools/supplychain/enrichment_config.yaml`

```yaml
enrichment:
  enabled: true

  sources:
    kev:
      enabled: true
      cache_ttl_hours: 24
      url: "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json"

    epss:
      enabled: true
      cache_ttl_hours: 24
      url: "https://api.first.org/data/v1/epss"
      batch_size: 100

    vulncheck:
      enabled: false  # Requires API key
      api_key_env: "VULNCHECK_API_KEY"

    ghsa:
      enabled: true
      token_env: "GITHUB_TOKEN"  # Optional, higher rate limits

  risk_scoring:
    weights:
      cvss: 0.40      # 40% weight
      epss: 0.30      # 30% weight
      kev: 0.20       # 20% weight
      exploit: 0.10   # 10% weight

  priority_thresholds:
    p0_immediate: 80  # Risk score >= 80 OR in KEV
    p1_critical: 60
    p2_high: 40
    p3_medium: 20
```

---

### Testing Strategy

**File:** `tools/supplychain/tests/test_enrichment.py`

```python
import pytest
from vulnerability_enrichment import VulnerabilityEnricher

def test_kev_enrichment():
    """Test KEV detection."""
    enricher = VulnerabilityEnricher()

    finding = {
        "cve": "CVE-2021-44228",
        "severity": "HIGH"
    }

    enriched = enricher.kev.enrich_finding(finding)

    assert enriched["kev"]["in_kev"] == True
    assert enriched["priority"] == "IMMEDIATE"
    assert "Log4Shell" in enriched["kev"]["vulnerability_name"]

def test_epss_scoring():
    """Test EPSS probability calculation."""
    enricher = VulnerabilityEnricher()

    findings = [
        {"cve": "CVE-2021-44228", "cvss_score": 10.0},
        {"cve": "CVE-2023-99999", "cvss_score": 9.0}  # Lower EPSS
    ]

    enriched = enricher.epss.enrich_findings(findings)

    # Log4Shell should have very high EPSS
    assert enriched[0]["epss"]["epss_score"] > 0.9

def test_risk_score_calculation():
    """Test composite risk scoring."""
    enricher = VulnerabilityEnricher()

    # High CVSS + High EPSS + KEV = max risk
    finding = {
        "cvss_score": 10.0,
        "epss": {"epss_score": 0.95},
        "kev": {"in_kev": True},
        "exploit": {"weaponized": True}
    }

    score = enricher._calculate_risk_score(finding)
    assert score > 95  # Should be near maximum
```

---

### Documentation

**File:** `docs/VULNERABILITY_ENRICHMENT.md`

```markdown
# Vulnerability Data Enrichment

BazBOM enriches vulnerability findings with multiple authoritative data sources
to provide actionable prioritization.

## Data Sources

### 1. CISA KEV (Known Exploited Vulnerabilities)
**Purpose:** Identify CVEs being actively exploited in the wild
**Update Frequency:** Daily
**Priority:** If in KEV → P0 (fix immediately)

### 2. EPSS (Exploit Prediction Scoring System)
**Purpose:** Predict exploitation probability (0-100%)
**Update Frequency:** Daily
**Model:** Machine learning trained on historical exploit data

### 3. GHSA (GitHub Security Advisories)
**Purpose:** Ecosystem-specific remediation guidance
**Coverage:** Maven, npm, PyPI, RubyGems, NuGet, Rust, Go

### 4. VulnCheck KEV (Optional)
**Purpose:** Advanced exploit intelligence
**Requires:** API key (free tier available)

## Risk Scoring

BazBOM calculates a composite risk score (0-100):

```
Risk Score = (CVSS × 0.40) + (EPSS × 0.30) + (KEV × 0.20) + (Exploit × 0.10)
```

**Priority Mapping:**
- **P0 (Immediate):** In CISA KEV or Risk Score ≥ 80
- **P1 (Critical):** Risk Score ≥ 60
- **P2 (High):** Risk Score ≥ 40
- **P3 (Medium):** Risk Score ≥ 20
- **P4 (Low):** Risk Score < 20

## Configuration

Enable/disable sources in `.bazelrc`:

```bash
# Enable all enrichment sources
build:sca --define=enrich_vulns=true

# Use offline mode (KEV only, no API calls)
build:sca --define=enrich_offline=true
```

## API Keys (Optional)

For enhanced features, set environment variables:

```bash
export VULNCHECK_API_KEY="your-key"  # Enhanced exploit data
export GITHUB_TOKEN="ghp_xxx"        # Higher GHSA rate limits
```

Free tier limits:
- EPSS: Unlimited (public API)
- KEV: Unlimited (public dataset)
- GHSA: 60 req/hour (unauthenticated), 5000/hour (authenticated)
- VulnCheck: 100 req/day (free tier)

## Example Output

See [enriched-finding-example.json](examples/enriched-finding.json)
```

---

### Success Metrics

**How we'll measure success:**

1. **Prioritization Accuracy:**
   - 95%+ of P0 findings are actionable (not false positives)
   - Security teams report 50%+ reduction in "alert fatigue"

2. **Remediation Speed:**
   - Mean time to remediate (MTTR) for critical CVEs reduced by 40%
   - KEV findings remediated within SLA (CISA due dates)

3. **Adoption:**
   - 80%+ of BazBOM users enable enrichment
   - GitHub Security tab shows enriched context

---

### Implementation Checklist

- [ ] Implement KEV enrichment (`kev_enrichment.py`)
- [ ] Implement EPSS enrichment (`epss_enrichment.py`)
- [ ] Implement GHSA enrichment (`ghsa_enrichment.py`)
- [ ] Implement VulnCheck integration (`vulncheck_enrichment.py`)
- [ ] Implement risk scoring algorithm (`vulnerability_enrichment.py`)
- [ ] Update SARIF adapter with enriched fields
- [ ] Update CLI output formatting
- [ ] Add configuration file support (`enrichment_config.yaml`)
- [ ] Write unit tests (95%+ coverage)
- [ ] Write integration tests
- [ ] Write documentation (`docs/VULNERABILITY_ENRICHMENT.md`)
- [ ] Add examples (`examples/enriched-finding.json`)
- [ ] Update CI/CD workflows to use enrichment
- [ ] Add telemetry (track enrichment hit rates)

---

## Next Steps

Once enrichment is implemented, it becomes the **foundation** for:
1. **Policy-as-Code** (e.g., "Block if KEV=true")
2. **SBOM Diffing** (track when KEV CVEs are introduced)
3. **Compliance Reports** (show P0 remediation rates)
4. **AI Recommendations** (prioritize upgrades by risk score)

---


## 1. SBOM Attestation & Transparency Logs

[Content continues from vulnerability enrichment section above...]

**Priority:** 1
**Effort:** 3-4 weeks
**Impact:** CRITICAL - Industry differentiator for enterprise trust

### The Problem

**Trust deficit in supply chain:**
- SBOMs can be tampered with after generation
- No way to prove SBOM was generated during the actual build
- Downstream consumers can't verify SBOM authenticity
- Compliance frameworks (SLSA, NIST) require cryptographic guarantees

