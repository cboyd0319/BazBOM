# bazbom-upgrade-analyzer

Recursive transitive upgrade analysis with breaking change detection for JVM dependencies.

## Features

- 🔄 **Recursive Analysis**: Analyzes target package AND all dependencies it pulls in
- 💥 **Breaking Change Detection**: Parses GitHub release notes for breaking changes
- 🎯 **Risk Scoring**: LOW/MEDIUM/HIGH/CRITICAL based on multiple factors
- ⏱️  **Effort Estimation**: ML-based hour estimates for upgrades
- 📚 **Migration Guides**: Auto-discovers MIGRATION.md and UPGRADING.md
- 🌐 **Multi-Source Intelligence**: Combines deps.dev + GitHub + semver
- ⚡ **Smart Caching**: Avoids duplicate analysis

## Usage

```rust
use bazbom_upgrade_analyzer::UpgradeAnalyzer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut analyzer = UpgradeAnalyzer::new()?;

    let analysis = analyzer.analyze_upgrade(
        "org.apache.logging.log4j:log4j-core",
        "2.17.0",
        "2.20.0"
    ).await?;

    // Overall risk
    println!("Risk: {:?}", analysis.overall_risk);
    println!("Effort: {} hours", analysis.estimated_effort_hours);

    // Direct breaking changes
    println!("\nDirect breaking changes: {}", analysis.direct_breaking_changes.len());
    for change in &analysis.direct_breaking_changes {
        println!("  - {}", change.description);
    }

    // Transitive dependency upgrades
    println!("\nRequired upgrades: {}", analysis.required_upgrades.len());
    for upgrade in &analysis.required_upgrades {
        println!("  {} {} -> {}",
            upgrade.package,
            upgrade.from_version,
            upgrade.to_version
        );
        if !upgrade.breaking_changes.is_empty() {
            println!("    ⚠️  {} breaking changes", upgrade.breaking_changes.len());
        }
    }

    // Safety check
    if analysis.is_safe() {
        println!("\n✅ Safe to upgrade!");
    } else {
        println!("\n⚠️  Review required - {} breaking changes total",
            analysis.total_breaking_changes());
    }

    Ok(())
}
```

## How It Works

### Recursive Transitive Analysis

The key innovation is **recursive analysis**:

```
1. Analyze target package (e.g., log4j-core)
   ├─ Get version metadata from deps.dev
   ├─ Find GitHub repository
   ├─ Parse release notes for breaking changes
   └─ Calculate risk

2. Get dependency graphs for both versions
   ├─ Query deps.dev for from_version dependencies
   └─ Query deps.dev for to_version dependencies

3. Find changed dependencies
   ├─ Compare graphs
   └─ Identify version changes

4. RECURSIVELY analyze each changed dependency
   ├─ For log4j-api (required by log4j-core):
   │   ├─ Get version metadata
   │   ├─ Parse release notes
   │   ├─ Extract breaking changes ← KEY!
   │   └─ Calculate risk
   └─ Aggregate results

5. Calculate overall risk
   ├─ Combine direct + transitive risks
   └─ Estimate effort
```

This is why we catch breaking changes that other tools miss!

## Architecture

### Components

- **`analyzer.rs`**: Main recursive analysis engine
- **`github.rs`**: GitHub release notes parser
- **`semver.rs`**: Semantic version risk analyzer
- **`models.rs`**: Data structures (RiskLevel, BreakingChange, etc.)

### Dependencies

- `bazbom-depsdev` - deps.dev API client
- `octocrab` - GitHub API client
- `regex` - Pattern matching
- `semver` - Version parsing
- `futures` - Parallel async

## Breaking Change Detection

Searches for common patterns in GitHub releases:

```markdown
## Breaking Changes
- Method X removed
- API Y changed

⚠️  Configuration format changed
💥 Major rewrite of module Z
```

Also auto-discovers migration guides:
- `MIGRATION.md`
- `UPGRADING.md`
- `docs/migration/<version>.md`

## License

MIT
