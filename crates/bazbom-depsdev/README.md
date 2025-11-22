# bazbom-depsdev

Rust client for the [deps.dev API](https://deps.dev), providing package metadata, dependency graphs, and vulnerability information across multiple ecosystems.

## Features

- 🌐 Async API client using `reqwest`
- PKG Support for Maven, npm, PyPI, Cargo, Go, NuGet, RubyGems
- SEARCH Package version metadata with licenses and advisories
- WEB  Resolved dependency graphs
- LINK GitHub repository discovery
- ⚡ Fast and reliable with automatic error handling
- TEST Well-tested with comprehensive test suite

## Usage

```rust
use bazbom_depsdev::{DepsDevClient, System};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = DepsDevClient::new();

    // Get version information
    let version_info = client.get_version(
        System::Maven,
        "org.apache.logging.log4j:log4j-core",
        "2.20.0"
    ).await?;

    println!("Published: {}", version_info.published_at);
    println!("Licenses: {:?}", version_info.licenses);
    println!("Advisories: {:?}", version_info.advisories);

    // Get dependency graph
    let deps = client.get_dependencies(
        System::Maven,
        "org.springframework.boot:spring-boot-starter-web",
        "3.2.0"
    ).await?;

    println!("Total dependencies: {}", deps.nodes.len());
    println!("Direct dependencies: {}", deps.direct_dependencies().len());

    // Find GitHub repository
    if let Some(repo_url) = client.find_github_repo(
        System::Maven,
        "org.apache.logging.log4j:log4j-core",
        "2.20.0"
    ).await? {
        println!("Repository: {}", repo_url);
    }

    Ok(())
}
```

## API Reference

### `DepsDevClient`

- `get_version()` - Get version metadata, licenses, and advisories
- `get_dependencies()` - Get resolved dependency graph
- `get_package()` - Get package info with all available versions
- `find_github_repo()` - Discover GitHub repository URL

See [API documentation](https://docs.deps.dev/api/v3/) for more details.

## License

MIT
