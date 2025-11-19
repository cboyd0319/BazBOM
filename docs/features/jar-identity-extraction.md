# JAR Identity Extraction

BazBOM can identify unknown JAR files by extracting Maven coordinates from multiple sources. This enables vulnerability analysis and upgrade intelligence for JARs discovered in containers, lib directories, or fat JARs without accompanying pom.xml files.

## Overview

When a JAR file is discovered without context (no pom.xml, no lockfile), BazBOM attempts to identify it using three strategies:

1. **pom.properties** - Most reliable source, embedded by Maven during build
2. **MANIFEST.MF** - Fallback using Implementation-* or Bundle-* attributes
3. **Checksum lookup** - Query Maven Central by SHA-256 hash

## API Usage

### Identify a Single JAR

```rust
use bazbom::shading::{identify_jar, JarIdentity};
use std::path::Path;

let identity = identify_jar(Path::new("unknown.jar"), None)?;

if let Some(id) = identity {
    println!("Found: {}:{}:{}", id.group_id, id.artifact_id, id.version);
    println!("PURL: {}", id.purl());
}
```

### Extract and Identify Nested JARs

For fat JARs (uber JARs, Spring Boot applications):

```rust
use bazbom::shading::extract_and_identify_jars;
use std::path::Path;

let results = extract_and_identify_jars(
    Path::new("app.jar"),
    Path::new("/tmp/extracted"),
    None, // or Some(&http_agent) for checksum lookups
)?;

for jar in &results {
    println!("JAR: {}", jar.archive_name);
    if let Some(ref id) = jar.identity {
        println!("  -> {}:{}:{}", id.group_id, id.artifact_id, id.version);
    } else {
        println!("  -> Could not identify");
    }
}
```

### Scan a Directory for JARs

For lib directories or extracted container layers:

```rust
use bazbom::shading::scan_and_identify_jars;
use std::path::Path;

let results = scan_and_identify_jars(
    Path::new("/app/lib"),
    Some(&http_agent), // Enable Maven Central lookups
)?;

for jar in results {
    if let Some(id) = jar.identity {
        println!("{}: {}", jar.archive_name, id.purl());
    }
}
```

## Identification Strategies

### 1. pom.properties (Primary)

Maven embeds a `pom.properties` file at `META-INF/maven/{groupId}/{artifactId}/pom.properties`:

```properties
groupId=org.apache.commons
artifactId=commons-lang3
version=3.12.0
```

This is the most reliable source as it's automatically generated during the build.

### 2. MANIFEST.MF (Fallback)

If no pom.properties exists, BazBOM parses MANIFEST.MF attributes:

**Standard JAR attributes:**
- `Implementation-Title` -> artifactId
- `Implementation-Version` -> version
- `Implementation-Vendor-Id` -> groupId

**OSGi bundle attributes:**
- `Bundle-SymbolicName` -> groupId.artifactId
- `Bundle-Version` -> version

### 3. Maven Central Checksum (Remote)

When an HTTP agent is provided, BazBOM can query Maven Central's search API by SHA-256 checksum:

```
https://search.maven.org/solrsearch/select?q=1:{sha256}&rows=1&wt=json
```

This identifies JARs that don't contain embedded metadata.

## Return Types

### JarIdentity

```rust
pub struct JarIdentity {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub source: JarIdentitySource,
    pub checksum: Option<String>,
}

pub enum JarIdentitySource {
    PomProperties,
    Manifest,
    ChecksumLookup,
    Fingerprint,
}
```

### IdentifiedJar

Returned by bulk operations:

```rust
pub struct IdentifiedJar {
    pub path: PathBuf,
    pub archive_name: String,
    pub identity: Option<JarIdentity>,
}
```

## Helper Methods

### Convert to PURL

```rust
let purl = identity.purl();
// "pkg:maven/org.apache.commons/commons-lang3@3.12.0"
```

### Convert to GAV

```rust
let gav = identity.gav();
// "org.apache.commons:commons-lang3:3.12.0"
```

## Performance Considerations

- **Local extraction**: ~1ms per JAR for pom.properties/MANIFEST.MF
- **Checksum computation**: ~10ms per MB of JAR
- **Maven Central lookup**: Network latency (~100-500ms per request)

For bulk operations, consider:
- Disabling checksum lookups (`agent: None`) for speed
- Using `identify_jars()` with rayon for parallelization
- Caching results for repeated scans

## Integration Points

### Container Scanning

When analyzing container images, use `scan_and_identify_jars` on extracted filesystem layers to identify Java dependencies in `/app/lib`, `/opt/lib`, etc.

### Fat JAR Analysis

Use `extract_and_identify_jars` to analyze Spring Boot executable JARs or Maven shade plugin outputs.

### Shaded JAR Detection

Combined with the existing shading detection in `shading.rs`, you can:
1. Identify original artifact coordinates
2. Detect class relocations
3. Map shaded classes back to source artifacts

## Limitations

- **Stripped JARs**: Some build processes remove pom.properties
- **Repackaged JARs**: Modified JARs won't match checksums
- **Private artifacts**: Maven Central only contains public artifacts
- **Shaded dependencies**: Relocated classes may not be identified

## Future Enhancements

- Class fingerprinting for shaded dependencies (partially implemented)
- JCenter/JFrog Artifactory lookup support
- Caching layer for repeated lookups
- Batch Maven Central queries
