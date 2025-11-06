# Phase 6: Visualization & Observability

**Status:** Planned
**Priority:**  P1 - High Impact
**Timeline:** Months 3-5 (8 weeks)
**Team Size:** 2 developers (1 Rust backend, 1 frontend)
**Dependencies:** Phase 0-3 complete

---

## Executive Summary

**Goal:** Make security insights accessible to non-technical stakeholders (CISOs, executives, compliance teams).

**Gap:** All competitors have web dashboards. BazBOM is CLI-only.

**Target:** Self-hosted web dashboard showing vulnerability trends, dependency graphs, compliance status.

**Success Metrics:**
-  CISO understands security posture in <5 minutes
-  Zero manual report generation
-  Dashboard loads in <2 seconds
-  Works offline (no external dependencies)

---

## 6.1 Web Dashboard (Rust + HTMX)

### Why HTMX over React/Vue?

- **Simplicity:** No complex build pipeline
- **Performance:** Server-rendered, fast load times
- **Lightweight:** <15KB JavaScript (vs. 300KB+ for React)
- **Rust-friendly:** Templates in Rust, minimal JS

### Architecture

```
┌─────────────────┐
│   Browser       │
│   (HTMX)        │
└────────┬────────┘
         │ HTTP
┌────────────────┐
│  Axum Server    │  (Rust)
│  - Routes       │
│  - Templates    │
│  - API          │
└────────┬────────┘
         │
┌────────────────┐
│ .bazbom/cache/  │  (Local files)
│ - sbom.json     │
│ - findings.json │
│ - graph.json    │
└─────────────────┘
```

### Routes

- `GET /` - Dashboard homepage
- `GET /vulnerabilities` - Vulnerability list
- `GET /dependencies` - Dependency graph
- `GET /sbom` - SBOM explorer
- `GET /policy` - Policy compliance status
- `GET /trends` - Charts (vulnerabilities over time)
- `GET /api/*` - JSON API for external integrations

### Technology Stack

- **Backend:** Axum (Rust web framework)
- **Templates:** Tera (Jinja2-like)
- **Frontend:** HTMX + Alpine.js + Tailwind CSS
- **Charts:** Chart.js
- **Database:** None (reads from `.bazbom/cache/`)

### Implementation

```rust
// crates/bazbom-dashboard/src/main.rs
use axum::{Router, routing::get};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(dashboard_handler))
        .route("/vulnerabilities", get(vulnerabilities_handler))
        .route("/dependencies", get(dependencies_handler))
        .nest_service("/static", ServeDir::new("static"));

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn dashboard_handler() -> Html<String> {
    let findings = load_findings().unwrap();
    let stats = calculate_stats(&findings);

    let template = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>BazBOM Dashboard</title>
        <script src="https://unpkg.com/htmx.org"></script>
        <script src="https://cdn.tailwindcss.com"></script>
    </head>
    <body class="bg-gray-100">
        <div class="container mx-auto p-8">
            <h1 class="text-4xl font-bold mb-8">BazBOM Security Dashboard</h1>

            <div class="grid grid-cols-4 gap-4 mb-8">
                <div class="bg-white p-6 rounded shadow">
                    <h3 class="text-gray-500 text-sm">Total Dependencies</h3>
                    <p class="text-3xl font-bold">{{ total_deps }}</p>
                </div>
                <div class="bg-red-100 p-6 rounded shadow">
                    <h3 class="text-gray-500 text-sm">Critical Vulnerabilities</h3>
                    <p class="text-3xl font-bold text-red-600">{{ critical_count }}</p>
                </div>
                <div class="bg-yellow-100 p-6 rounded shadow">
                    <h3 class="text-gray-500 text-sm">High Vulnerabilities</h3>
                    <p class="text-3xl font-bold text-yellow-600">{{ high_count }}</p>
                </div>
                <div class="bg-green-100 p-6 rounded shadow">
                    <h3 class="text-gray-500 text-sm">Clean Dependencies</h3>
                    <p class="text-3xl font-bold text-green-600">{{ clean_count }}</p>
                </div>
            </div>

            <div class="bg-white p-6 rounded shadow">
                <h2 class="text-2xl font-bold mb-4">Recent Vulnerabilities</h2>
                <div hx-get="/api/vulnerabilities" hx-trigger="load" hx-swap="innerHTML">
                    Loading...
                </div>
            </div>
        </div>
    </body>
    </html>
    "#;

    Html(template.replace("{{ total_deps }}", &stats.total_deps.to_string())
                .replace("{{ critical_count }}", &stats.critical_count.to_string())
                .replace("{{ high_count }}", &stats.high_count.to_string())
                .replace("{{ clean_count }}", &stats.clean_count.to_string()))
}
```

---

## 6.2 Dependency Graph Visualization

### Library: Cytoscape.js

**Why:** Interactive, supports large graphs (10K+ nodes), performant

**Features:**
- Pan/zoom
- Search for specific dependencies
- Highlight vulnerable components (red nodes)
- Click node to see details

### Implementation

```javascript
// static/js/dep-graph.js
fetch('/api/graph')
    .then(r => r.json())
    .then(data => {
        cytoscape({
            container: document.getElementById('cy'),
            elements: data.elements,
            style: [
                {
                    selector: 'node',
                    style: {
                        'label': 'data(name)',
                        'background-color': 'data(color)',
                        'width': 30,
                        'height': 30
                    }
                },
                {
                    selector: 'edge',
                    style: {
                        'width': 2,
                        'line-color': '#ccc',
                        'target-arrow-color': '#ccc',
                        'target-arrow-shape': 'triangle'
                    }
                },
                {
                    selector: '.vulnerable',
                    style: {
                        'background-color': '#ef4444'  // Red for vulnerable
                    }
                }
            ],
            layout: {
                name: 'cose',
                animate: false
            }
        });
    });
```

---

## 6.3 Executive Reports (PDF)

### Use Case: Monthly security report for CISO

**Format:** PDF with charts, summary, recommendations

**Tool:** typst (modern LaTeX alternative in Rust)

```rust
// crates/bazbom/src/reports/pdf.rs
use typst::compile;

pub fn generate_executive_report(findings: &Findings) -> Result<Vec<u8>> {
    let template = r#"
    #set page(paper: "us-letter")
    #set text(font: "Liberation Sans")

    = BazBOM Security Report
    _Generated: #datetime.today()_

    == Executive Summary

    - Total Dependencies: #findings.total_deps
    - Critical Vulnerabilities: #findings.critical_count
    - Remediation Rate: #findings.remediation_rate%

    == Top Risks

    #for vuln in findings.top_risks {
        - *#vuln.id* (#vuln.severity): #vuln.package
          CVSS: #vuln.cvss, EPSS: #vuln.epss
    }

    == Recommendations

    1. Fix CRITICAL vulnerabilities within 7 days
    2. Plan upgrades for HIGH severity issues
    3. Review policy exceptions quarterly
    "#;

    let pdf_bytes = compile(template)?;
    Ok(pdf_bytes)
}
```

---

## 6.4 Slack/Teams Integration

### Notifications

- New CRITICAL vulnerability detected
- Policy violation in PR
- Weekly summary report

**Implementation:**

```rust
// crates/bazbom/src/notifications/slack.rs
use reqwest::Client;

pub async fn send_slack_notification(webhook_url: &str, message: &str) -> Result<()> {
    let client = Client::new();
    let payload = json!({
        "text": message,
        "blocks": [{
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": message
            }
        }]
    });

    client.post(webhook_url)
        .json(&payload)
        .send()
        .await?;

    Ok(())
}
```

**Usage:**

```bash
# Configure webhook
bazbom config set slack_webhook https://hooks.slack.com/services/XXX

# Send notifications on scan
bazbom scan --notify-slack
```

---

## Success Criteria

- [ ] Web dashboard runs locally on `http://localhost:8080`
- [ ] Dashboard loads in <2 seconds with 10K dependencies
- [ ] Dependency graph renders 5K nodes without lag
- [ ] PDF reports generate in <5 seconds
- [ ] Slack notifications deliver in <1 second
- [ ] Works offline (no CDN dependencies, optional local copies)
- [ ] Responsive design (works on mobile/tablet)

---

**Resource:** 2 developers × 8 weeks = $32K-48K

**Last Updated:** 2025-10-30
