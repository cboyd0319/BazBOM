# Container Image Scanning

**Status:** Beta (Available in BazBOM v0.5.1+)

---

## Overview

BazBOM can scan container images (Docker/OCI) for Java dependencies, generating SBOMs and identifying vulnerabilities in containerized applications.

### Features

- **OCI Image Parsing** - Parse Docker/OCI container images
- **Layer-by-Layer Analysis** - Analyze each container layer independently
- **Java Artifact Detection** - Find JARs, WARs, EARs in containers
- **Maven Metadata Extraction** - Extract groupId:artifactId:version from JARs
- **Container SBOM Generation** - Create SBOMs for containerized apps
- **Vulnerability Scanning** - Scan container dependencies for CVEs

---

## Quick Start

### Scan a Container Image

```bash
# Export Docker image to tar file
docker save myapp:latest -o myapp.tar

# Scan the container
bazbom scan --containers=bazbom

# BazBOM will automatically find and scan *.tar files in the current directory
```

### Full Workflow Example

```bash
# 1. Build your container
docker build -t myapp:latest .

# 2. Export to tar
docker save myapp:latest -o myapp.tar

# 3. Scan with BazBOM
bazbom scan --containers=bazbom

# Output:
[bazbom] container scanning requested
[bazbom] using container scanning strategy: Bazbom
[bazbom] found 1 container images to scan
[bazbom] scanning container: myapp.tar
[bazbom] parsing container image metadata
[bazbom] extracting 15 layers
[bazbom] scanning layer 0: /tmp/bazbom-layers-12345/layer-0
[bazbom] found 2 artifacts in layer 0
[bazbom] scanning layer 1: /tmp/bazbom-layers-12345/layer-1
[bazbom] found 8 artifacts in layer 1
[bazbom] total artifacts found: 42
[bazbom] container scan complete
```

---

## Supported Features

###  Implemented

- OCI/Docker image format parsing
- Layer extraction and analysis
- JAR/WAR/EAR file detection
- Maven metadata extraction from `META-INF/maven/*/pom.properties`
- SHA-256 hash calculation for artifacts
- Container SBOM generation with PURLs
- Layer attribution (which layer contains which artifact)

###  Partial / Future

- Docker HTTP API integration (currently uses tar files)
- Real-time container pulls from registry
- Multi-architecture image support
- Container vulnerability database integration
- Dockerfile analysis
- Base image detection and recursive scanning

---

## Container Scanning Strategies

BazBOM supports multiple container scanning approaches:

```bash
# Auto (tries Syft, falls back to BazBOM)
bazbom scan --containers=auto

# BazBOM native scanning (current implementation)
bazbom scan --containers=bazbom

# Syft integration (future)
bazbom scan --containers=syft
```

### Strategy Comparison

| Strategy | Status | Speed | Accuracy | Multi-Language |
|----------|--------|-------|----------|----------------|
| `bazbom` |  Available | Medium | High (Java) | Java only |
| `syft` |  Planned | Fast | Medium | All languages |
| `auto` |  Available | Variable | High | Depends |

---

## How It Works

### Container Scanning Pipeline

```
1. Find container images (*.tar files in workspace)
   ↓
2. Parse OCI manifest and configuration
   ↓
3. Extract all layers to temporary directory
   ↓
4. Scan each layer for Java artifacts
   ↓
5. Extract Maven metadata from JARs
   ↓
6. Calculate SHA-256 hashes
   ↓
7. Generate container SBOM
   ↓
8. Scan dependencies for vulnerabilities
   ↓
9. Output findings and SBOM
```

### Layer Analysis

Each container layer is analyzed independently:

```json
{
  "layer": "sha256:abc123...",
  "size": 123456789,
  "created_by": "RUN apt-get install -y openjdk-11-jdk",
  "artifacts": [
    {
      "path": "/app/lib/spring-core-5.3.20.jar",
      "type": "jar",
      "maven_coords": "org.springframework:spring-core:5.3.20",
      "sha256": "def456..."
    }
  ]
}
```

---

## Usage Examples

### Example 1: Spring Boot Application

```bash
# Build Spring Boot container
docker build -t myapp:latest .

# Export and scan
docker save myapp:latest -o myapp.tar
bazbom scan --containers=bazbom

# Output:
[bazbom] container scan complete:
[bazbom]   image: myapp:latest
[bazbom]   layers: 8
[bazbom]   artifacts: 127
[bazbom]   with Maven metadata: 125
[bazbom]   vulnerabilities: 3 (1 CRITICAL, 2 HIGH)
```

### Example 2: Multi-Stage Build

```dockerfile
# Dockerfile
FROM maven:3.8-openjdk-11 AS build
WORKDIR /app
COPY pom.xml .
COPY src ./src
RUN mvn clean package

FROM openjdk:11-jre-slim
COPY --from=build /app/target/myapp.jar /app/
CMD ["java", "-jar", "/app/myapp.jar"]
```

```bash
# Build and scan
docker build -t myapp:latest .
docker save myapp:latest -o myapp.tar
bazbom scan --containers=bazbom

# BazBOM detects:
# - Base image: openjdk:11-jre-slim
# - Application JAR: myapp.jar
# - All transitive dependencies packaged in the JAR
```

### Example 3: CI/CD Integration

```yaml
# .github/workflows/container-scan.yml
name: Container Scan

on:
  push:
    branches: [ main ]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build container
        run: docker build -t myapp:${{ github.sha }} .
      
      - name: Export container
        run: docker save myapp:${{ github.sha }} -o myapp.tar
      
      - name: Scan container with BazBOM
        run: bazbom scan --containers=bazbom
      
      - name: Upload SBOM
        uses: actions/upload-artifact@v4
        with:
          name: container-sbom
          path: .bazbom/sbom/
      
      - name: Upload findings
        uses: actions/upload-artifact@v4
        with:
          name: container-findings
          path: .bazbom/findings/
```

---

## Container SBOM Format

### Example Container SBOM

```json
{
  "image_name": "myapp:latest",
  "image_digest": "sha256:abc123def456...",
  "base_image": "openjdk:11-jre-slim",
  "packages": [
    {
      "name": "org.springframework:spring-core",
      "version": "5.3.20",
      "purl": "pkg:maven/org.springframework/spring-core@5.3.20",
      "location": "/app/lib/spring-core-5.3.20.jar",
      "layer": "layer-3",
      "sha256": "def456..."
    },
    {
      "name": "org.apache.logging.log4j:log4j-core",
      "version": "2.14.1",
      "purl": "pkg:maven/org.apache.logging.log4j/log4j-core@2.14.1",
      "location": "/app/lib/log4j-core-2.14.1.jar",
      "layer": "layer-3",
      "sha256": "ghi789..."
    }
  ],
  "total_layers": 8,
  "total_packages": 127,
  "scan_timestamp": "2024-11-04T19:00:00Z"
}
```

---

## Configuration

### Container Scanning Options

Configure container scanning in `bazbom.yml`:

```yaml
containers:
  enabled: true
  strategy: bazbom  # auto | bazbom | syft
  
  # Analysis options
  analysis:
    extract_maven_metadata: true
    calculate_hashes: true
    scan_all_layers: true
    
  # Performance
  performance:
    parallel_layer_scanning: true
    max_concurrent_layers: 4
    temp_dir: /tmp/bazbom-containers
    
  # Filtering
  filters:
    # Only scan specific artifact types
    artifact_types:
      - jar
      - war
      - ear
    
    # Skip layers matching patterns
    skip_layers:
      - "apt-get update"
      - "yum install"
```

---

## Docker API Integration (Future)

### Planned Features

Direct Docker daemon integration without tar export:

```bash
# Scan running container (future)
bazbom scan --container myapp:latest

# Scan from registry (future)
bazbom scan --container docker.io/library/openjdk:11

# Scan all running containers (future)
bazbom scan --containers=auto --all-running
```

### Implementation Plan

```rust
// Future Docker API client
use hyperlocal::UnixClientExt;

let client = DockerClient::new();
client.pull_image("myapp:latest")?;
client.export_image("myapp:latest", temp_file)?;

// Then scan as normal
let scanner = ContainerScanner::new(temp_file);
let result = scanner.scan()?;
```

---

## Maven Metadata Extraction

### How It Works

BazBOM extracts Maven coordinates from JAR files:

```
1. Open JAR as ZIP archive
   ↓
2. Look for META-INF/maven/*/*/pom.properties
   ↓
3. Parse properties file:
      groupId=org.springframework
      artifactId=spring-core
      version=5.3.20
   ↓
4. Create Maven coordinates: org.springframework:spring-core:5.3.20
   ↓
5. Generate PURL: pkg:maven/org.springframework/spring-core@5.3.20
```

### Example pom.properties

```properties
# pom.properties file inside JAR
version=5.3.20
groupId=org.springframework
artifactId=spring-core
```

---

## Layer Attribution

### Why It Matters

Knowing which layer contains which artifact helps:

1. **Optimize builds** - Reorder Dockerfile to minimize layer changes
2. **Security audits** - Track when vulnerabilities were introduced
3. **Compliance** - Document software provenance per layer
4. **Debugging** - Understand where dependencies come from

### Example Layer Attribution

```
Layer 0 (Base Image):
  - openjdk-11-jre
  
Layer 1 (System Packages):
  - ca-certificates
  - curl
  
Layer 2 (Application Dependencies):
  - spring-core-5.3.20.jar
  - spring-web-5.3.20.jar
  - log4j-core-2.14.1.jar
  
Layer 3 (Application JAR):
  - myapp-1.0.0.jar
```

---

## Troubleshooting

### No Container Images Found

```bash
# Check that tar files exist
ls -lh *.tar

# Specify directory explicitly
bazbom scan --containers=bazbom --workspace /path/to/images/
```

### Maven Metadata Not Found

Some JARs don't include pom.properties:

```bash
# Expected behavior - artifact detected but no Maven metadata
[bazbom] found artifact: /app/lib/custom.jar (no Maven metadata)

# BazBOM still detects and hashes the file
# You can manually add coordinates to SBOM if needed
```

### Memory Issues with Large Images

```bash
# Limit memory usage
BAZBOM_MAX_MEMORY=4G bazbom scan --containers=bazbom

# Process layers sequentially instead of parallel
bazbom scan --containers=bazbom --sequential-layers
```

### Temporary Files Not Cleaned Up

```bash
# BazBOM creates temp files in /tmp/bazbom-layers-*
# These are usually cleaned up automatically

# Manual cleanup if needed:
rm -rf /tmp/bazbom-layers-*
```

---

## Best Practices

### 1. Scan Before Push

Always scan containers before pushing to registry:

```bash
#!/bin/bash
# build-and-scan.sh

# Build container
docker build -t myapp:latest .

# Scan for vulnerabilities
docker save myapp:latest -o myapp.tar
bazbom scan --containers=bazbom --fail-on-vuln high

# Only push if scan passes
if [ $? -eq 0 ]; then
  docker push myapp:latest
else
  echo " Container scan failed - not pushing"
  exit 1
fi
```

### 2. Optimize Layer Order

Put frequently changing files in later layers:

```dockerfile
# Good: Dependencies change less often than code
FROM openjdk:11
COPY pom.xml .
RUN mvn dependency:go-offline
COPY src ./src
RUN mvn package

# Bad: Rebuilds everything on code change
FROM openjdk:11
COPY . .
RUN mvn package
```

### 3. Use Multi-Stage Builds

Keep production images small:

```dockerfile
# Build stage - includes build tools
FROM maven:3.8-openjdk-11 AS build
COPY . .
RUN mvn clean package

# Runtime stage - only JRE and app
FROM openjdk:11-jre-slim
COPY --from=build target/app.jar /app/
CMD ["java", "-jar", "/app/app.jar"]
```

### 4. Regular Scanning

Scan containers regularly, not just at build time:

```bash
# Weekly container audit
for image in $(docker images --format "{{.Repository}}:{{.Tag}}"); do
  echo "Scanning $image"
  docker save "$image" -o "$(echo $image | tr '/:' '-').tar"
  bazbom scan --containers=bazbom
done
```

---

## Performance

### Typical Scan Times

| Container Size | Layers | Artifacts | Scan Time |
|---------------|--------|-----------|-----------|
| Small (50 MB) | 5 | 10 | <30s |
| Medium (500 MB) | 10 | 50 | <2min |
| Large (2 GB) | 20 | 200 | <8min |

### Optimization Tips

```bash
# Use parallel layer scanning (default)
bazbom scan --containers=bazbom

# Sequential for low memory
bazbom scan --containers=bazbom --sequential-layers

# Skip hash calculation if not needed
bazbom scan --containers=bazbom --no-hashes
```

---

## Related Documentation

- [OCI Image Format](https://github.com/opencontainers/image-spec) - OCI image specification
- [Docker Save Format](https://docs.docker.com/engine/reference/commandline/save/) - Docker tar format
- [Security Guide](../security/README.md) - Security best practices

---

## Contributing

Help improve container scanning:

1. **Test with Your Images** - Try BazBOM with your container images
2. **Report Issues** - Open issues for bugs or missing features
3. **Docker API Integration** - Help implement Unix socket HTTP client
4. **Multi-Language Support** - Extend beyond Java to Node.js, Python, etc.

See [CONTRIBUTING.md](../CONTRIBUTING.md) for more information.

---

**Last Updated:** 2024-11-04  
**Version:** 0.5.1  
**Status:** Beta

**Current Limitations:**
- Requires manual `docker save` to tar file
- Java-only artifact detection
- No real-time registry pulls
- No multi-architecture support

**Roadmap:**
-  Basic OCI parsing
-  Layer extraction
-  Maven metadata extraction
-  Docker HTTP API integration
-  Multi-language support
-  Registry integration
-  Multi-arch support
