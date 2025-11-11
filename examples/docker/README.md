# BazBOM Docker Examples

This directory contains Docker and Docker Compose examples for running BazBOM in containerized environments.

## Quick Start

### 1. Basic Scan with Docker Compose

```bash
# Scan a project and generate SBOM
docker-compose up bazbom

# Results will be in ./output/
ls -lh output/
```

### 2. Full Stack with Dependency-Track

```bash
# Start BazBOM + Dependency-Track + PostgreSQL
docker-compose --profile full-stack up -d

# Access Dependency-Track UI at http://localhost:8081
# Default credentials: admin / admin

# Upload generated SBOM to Dependency-Track
curl -X "POST" "http://localhost:8081/api/v1/bom" \
  -H "Content-Type: multipart/form-data" \
  -H "X-Api-Key: YOUR_API_KEY" \
  -F "project=YOUR_PROJECT_UUID" \
  -F "bom=@output/sbom.spdx.json"
```

### 3. Comparison with OWASP Dependency-Check

```bash
# Run BazBOM and Dependency-Check side-by-side
docker-compose up bazbom
docker-compose --profile comparison up dependency-check

# Compare results
diff output/sca_findings.json output/dependency-check/dependency-check-report.json
```

## Configuration

### Scanning Your Own Project

Edit `docker-compose.yml` to mount your project:

```yaml
services:
  bazbom:
    volumes:
      # Replace with your project path
      - /path/to/your/project:/workspace:ro
```

### Custom Output Directory

```yaml
services:
  bazbom:
    volumes:
      # Custom output location
      - /path/to/output:/output
```

## Docker Profiles

- **Default**: BazBOM scanner only
- **full-stack**: BazBOM + Dependency-Track + PostgreSQL
- **comparison**: Add OWASP Dependency-Check for comparison

## Examples

### Maven Project

```bash
# Create sample Maven project
mkdir -p sample-project
cd sample-project
cat > pom.xml <<EOF
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>com.example</groupId>
  <artifactId>demo</artifactId>
  <version>1.0.0</version>
  <dependencies>
    <dependency>
      <groupId>org.springframework.boot</groupId>
      <artifactId>spring-boot-starter-web</artifactId>
      <version>2.7.0</version>
    </dependency>
  </dependencies>
</project>
EOF

# Scan with BazBOM
cd ..
docker-compose up bazbom
```

### Gradle Project

```bash
# Create sample Gradle project
mkdir -p sample-project
cd sample-project
cat > build.gradle <<EOF
plugins {
    id 'java'
}

repositories {
    mavenCentral()
}

dependencies {
    implementation 'com.google.guava:guava:31.1-jre'
    testImplementation 'junit:junit:4.13.2'
}
EOF

# Scan with BazBOM
cd ..
docker-compose up bazbom
```

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/docker-scan.yml
name: BazBOM Docker Scan

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run BazBOM in Docker
        run: |
          docker-compose -f examples/docker/docker-compose.yml up bazbom

      - name: Upload SBOM
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: examples/docker/output/sbom.spdx.json

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: examples/docker/output/sca_findings.sarif
```

### GitLab CI

```yaml
# .gitlab-ci.yml
bazbom-scan:
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker-compose -f examples/docker/docker-compose.yml up bazbom
  artifacts:
    paths:
      - examples/docker/output/
    reports:
      sast: examples/docker/output/sca_findings.sarif
```

## Dockerfile (Alternative)

If you prefer using a Dockerfile directly:

```dockerfile
# Dockerfile
FROM rust:1.91-bookworm as builder

WORKDIR /build
COPY . .
RUN cargo build --release -p bazbom

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    openjdk-11-jre-headless && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/bazbom /usr/local/bin/

WORKDIR /workspace
ENTRYPOINT ["bazbom"]
CMD ["--help"]
```

Build and run:

```bash
# Build image
docker build -t bazbom:latest .

# Scan a project
docker run --rm -v $(pwd):/workspace bazbom:latest scan .
```

## Troubleshooting

### Permission Issues

```bash
# Fix output directory permissions
sudo chown -R $(whoami):$(whoami) output/
```

### Network Issues (Air-Gapped)

```bash
# Pre-download advisory databases
bazbom db sync

# Copy to Docker volume
docker cp ~/.bazbom/db bazbom-scanner:/root/.bazbom/

# Run offline scan
docker-compose run bazbom scan . --offline-mode
```

### Large Projects

```bash
# Increase Docker memory limit
docker-compose run --memory=4g bazbom scan .
```

## Production Deployment

For production use, consider:

1. **Use pinned image versions** instead of `latest`
2. **Store advisory databases in a persistent volume**
3. **Use secrets management** for API keys (Dependency-Track, GitHub)
4. **Enable authentication** on Dependency-Track
5. **Set up regular database syncs** (`bazbom db sync`)
6. **Monitor resource usage** and set appropriate limits

## Additional Resources

- [BazBOM Documentation](../../docs/README.md)
- [Docker Best Practices](../../docs/operations/docker-best-practices.md)
- [Dependency-Track Documentation](https://docs.dependencytrack.org/)
