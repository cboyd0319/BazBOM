# BazBOM Kubernetes Operator

Automatically scan Kubernetes workloads for vulnerabilities and generate SBOMs.

## Overview

The BazBOM Operator watches for `BazBOMScan` custom resources and automatically:
- Scans Kubernetes deployments for vulnerabilities
- Generates SBOMs in SPDX or CycloneDX format
- Stores results in ConfigMaps
- Updates scan status with vulnerability counts
- Supports scheduled scans via cron expressions

## Features

- **Automated Scanning**: Define scan configurations as Kubernetes resources
- **Scheduled Scans**: Run periodic scans with cron schedules
- **Multiple Build Systems**: Maven, Gradle, Bazel, Ant, sbt, Buildr
- **Container Scanning**: Scan container images for JVM artifacts
- **ML Prioritization**: Use machine learning to prioritize vulnerabilities
- **LLM Fixes**: Get AI-powered remediation suggestions
- **SBOM Storage**: Store generated SBOMs in ConfigMaps
- **Status Tracking**: Real-time scan status and vulnerability counts

## Installation

### Prerequisites

- Kubernetes 1.28+
- kubectl configured to access your cluster

### Install CRD

```bash
kubectl apply -f k8s/crd.yaml
```

### Create Namespace

```bash
kubectl create namespace bazbom-system
```

### Deploy Operator

```bash
kubectl apply -f k8s/deployment.yaml
```

### Verify Installation

```bash
kubectl get pods -n bazbom-system
kubectl logs -n bazbom-system deployment/bazbom-operator
```

## Usage

### Create a Scan

```yaml
apiVersion: bazbom.io/v1
kind: BazBOMScan
metadata:
  name: my-app-scan
  namespace: default
spec:
  targetDeployment: my-app
  schedule: "0 0 * * *"  # Daily at midnight
  buildSystem: maven
  scanOptions:
    scanContainers: true
    reachabilityAnalysis: true
    mlPrioritize: true
  outputFormat: spdx
  storeSbom: true
```

Apply the scan:

```bash
kubectl apply -f my-scan.yaml
```

### Check Scan Status

```bash
# List all scans
kubectl get bazbomscans

# Get detailed status
kubectl get bazbomscans my-app-scan -o yaml

# Watch for changes
kubectl get bazbomscans -w
```

### View Scan Results

```bash
# Get SBOM from ConfigMap
kubectl get configmap my-app-scan-sbom -o yaml

# View vulnerabilities in status
kubectl get bazbomscans my-app-scan -o jsonpath='{.status.vulnerabilities}'
```

## Configuration

### Scan Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `targetDeployment` | string | required | Name of deployment to scan |
| `schedule` | string | null | Cron schedule (e.g., "0 0 * * *") |
| `buildSystem` | string | auto-detect | maven, gradle, bazel, ant, sbt, buildr |
| `outputFormat` | string | spdx | spdx, cyclonedx, json |
| `storeSbom` | boolean | true | Store SBOM in ConfigMap |
| `scanOptions.scanContainers` | boolean | false | Scan container images |
| `scanOptions.reachabilityAnalysis` | boolean | false | Include reachability analysis |
| `scanOptions.mlPrioritize` | boolean | false | ML-powered prioritization |
| `scanOptions.llmFixes` | boolean | false | LLM fix suggestions |

### Policy Configuration

Create a ConfigMap with your policy:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: bazbom-policy
  namespace: default
data:
  bazbom.yml: |
    policy:
      severity_threshold: HIGH
      kev_policy:
        action: block
      epss_threshold: 0.5
```

Reference it in your scan:

```yaml
spec:
  policyConfigMap: bazbom-policy
```

## Examples

### Maven Application

```bash
kubectl apply -f k8s/example-scan.yaml
```

### Gradle Application

```yaml
apiVersion: bazbom.io/v1
kind: BazBOMScan
metadata:
  name: gradle-app
spec:
  targetDeployment: gradle-app
  buildSystem: gradle
  outputFormat: cyclonedx
```

### Bazel Monorepo

```yaml
apiVersion: bazbom.io/v1
kind: BazBOMScan
metadata:
  name: bazel-monorepo
spec:
  targetDeployment: bazel-app
  buildSystem: bazel
  scanOptions:
    reachabilityAnalysis: true
```

## Development

### Build

```bash
cargo build --release
```

### Run Locally (requires kubeconfig)

```bash
cargo run
```

### Build Docker Image

```bash
docker build -t bazbom/bazbom-operator:latest .
```

### Test

```bash
cargo test
```

### Integration Testing

Requires a Kubernetes cluster (kind, minikube, or real cluster):

```bash
# Create kind cluster
kind create cluster

# Install CRD
kubectl apply -f k8s/crd.yaml

# Run operator locally
cargo run

# Apply test scan
kubectl apply -f k8s/example-scan.yaml

# Watch logs
kubectl logs -f -n bazbom-system deployment/bazbom-operator
```

## Architecture

```
┌──────────────────────────────────────────┐
│         Kubernetes Cluster               │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │  BazBOM Operator (Deployment)      │ │
│  │  - Watches BazBOMScan resources    │ │
│  │  - Creates scan Jobs               │ │
│  │  - Updates status                  │ │
│  └────────────────────────────────────┘ │
│                ↓                         │
│  ┌────────────────────────────────────┐ │
│  │  Scan Jobs (Batch/v1)              │ │
│  │  - Runs bazbom CLI                 │ │
│  │  - Generates SBOM                  │ │
│  │  - Scans for vulnerabilities       │ │
│  └────────────────────────────────────┘ │
│                ↓                         │
│  ┌────────────────────────────────────┐ │
│  │  ConfigMaps (Storage)              │ │
│  │  - Stores SBOMs                    │ │
│  │  - Stores scan results             │ │
│  └────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

## Troubleshooting

### Operator Not Starting

```bash
# Check operator logs
kubectl logs -n bazbom-system deployment/bazbom-operator

# Check RBAC permissions
kubectl auth can-i get bazbomscans --as=system:serviceaccount:bazbom-system:bazbom-operator

# Verify CRD is installed
kubectl get crd bazbomscans.bazbom.io
```

### Scan Job Failing

```bash
# Get job status
kubectl get jobs -l app=bazbom-scan

# Check job logs
kubectl logs job/bazbom-scan-my-app

# Describe job for events
kubectl describe job bazbom-scan-my-app
```

### No Vulnerabilities Found

- Verify build system is correctly detected
- Check that bazbom binary is in container image
- Ensure network access to advisory databases (or use offline mode)
- Review scan logs for errors

## Roadmap

- [ ] Automatic remediation (create PRs)
- [ ] Slack/email notifications
- [ ] Grafana dashboard integration
- [ ] Multi-cluster support
- [ ] Helm chart
- [ ] ArgoCD integration
- [ ] Policy-as-code enforcement
- [ ] Admission controller for blocking vulnerable deployments

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - See LICENSE file in repository root

## Support

- GitHub Issues: https://github.com/cboyd0319/BazBOM/issues
- Discussions: https://github.com/cboyd0319/BazBOM/discussions
- Label: `component: operator`
