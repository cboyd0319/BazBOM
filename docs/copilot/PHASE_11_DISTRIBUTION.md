# Phase 11: Enterprise Distribution

**Status:** Planned
**Priority:** 🟡 P1 - Enterprise Adoption
**Timeline:** Months 9-12 (12 weeks)
**Team Size:** 1-2 developers
**Dependencies:** Phase 0-3 complete, Phase 8 (scale) recommended

---

## Executive Summary

**Goal:** Make BazBOM easy to deploy and operate at enterprise scale.

**Current State:** Homebrew (macOS/Linux), binary releases (GitHub)

**Target State:**
- **Windows support** - MSI installer, Chocolatey, winget
- **Kubernetes operator** - Deploy BazBOM as a service in K8s
- **Air-gapped deployments** - Zero internet requirement
- **Enterprise package managers** - SCCM, Jamf, Puppet, Ansible

**Success Metrics:**
- ✅ Windows users can install in <5 minutes
- ✅ Kubernetes deployment in <10 minutes
- ✅ Air-gapped bundle includes all dependencies
- ✅ 1000+ developers deployed in <1 day (enterprise)

**Strategic Rationale:** Remove deployment friction to accelerate enterprise adoption.

---

## 11.1 Windows Support

### Current Gap

**Phase 0-3:** macOS and Linux binaries only

**Windows Market:** 40% of developers (Stack Overflow Survey 2024)

### Implementation

#### 11.1.1 Windows Binary Compilation

**Cross-Compile from Linux/macOS:**
```bash
# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Build Windows binary
cargo build --release --target x86_64-pc-windows-gnu

# Output: target/x86_64-pc-windows-gnu/release/bazbom.exe
```

**Native Windows Build (GitHub Actions):**
```yaml
# .github/workflows/windows-build.yml
name: Windows Build

on: [push, pull_request]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: x86_64-pc-windows-msvc

      - name: Build
        run: cargo build --release

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: bazbom-windows-x64
          path: target/release/bazbom.exe
```

#### 11.1.2 MSI Installer

**Tool:** WiX Toolset (Windows Installer XML)

**Features:**
- Install to `C:\Program Files\BazBOM\`
- Add to PATH automatically
- Uninstaller
- Windows Defender exclusions (if needed)
- Code signing (Authenticode certificate)

**WiX Configuration:**
```xml
<!-- bazbom.wxs -->
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="BazBOM" Language="1033" Version="1.0.0"
           Manufacturer="BazBOM Project" UpgradeCode="PUT-GUID-HERE">

    <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />

    <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
    <MediaTemplate EmbedCab="yes" />

    <Feature Id="ProductFeature" Title="BazBOM" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>

    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFiles64Folder">
        <Directory Id="INSTALLFOLDER" Name="BazBOM">
          <Component Id="BazBomExe" Guid="PUT-GUID-HERE">
            <File Id="BazBomExeFile" Source="target\release\bazbom.exe" KeyPath="yes" />
            <Environment Id="PATH" Name="PATH" Value="[INSTALLFOLDER]" Permanent="no" Part="last" Action="set" System="yes" />
          </Component>
        </Directory>
      </Directory>
    </Directory>

    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <ComponentRef Id="BazBomExe" />
    </ComponentGroup>
  </Product>
</Wix>
```

**Build MSI:**
```bash
# Install WiX Toolset
choco install wixtoolset

# Build
candle bazbom.wxs
light -ext WixUIExtension bazbom.wixobj -out bazbom.msi
```

#### 11.1.3 Chocolatey Package

**Package Manager:** Chocolatey (Windows equivalent of Homebrew)

**Spec:**
```powershell
# bazbom.nuspec
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>bazbom</id>
    <version>1.0.0</version>
    <title>BazBOM</title>
    <authors>BazBOM Project</authors>
    <projectUrl>https://bazbom.io</projectUrl>
    <licenseUrl>https://github.com/cboyd0319/BazBOM/blob/main/LICENSE</licenseUrl>
    <requireLicenseAcceptance>false</requireLicenseAcceptance>
    <description>Enterprise-grade SBOM and SCA for JVM projects</description>
    <tags>security sbom sca java bazel maven gradle</tags>
  </metadata>
  <files>
    <file src="tools\**" target="tools" />
  </files>
</package>
```

**Install Script:**
```powershell
# tools/chocolateyinstall.ps1
$ErrorActionPreference = 'Stop'

$packageName = 'bazbom'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url64 = 'https://github.com/cboyd0319/BazBOM/releases/download/v1.0.0/bazbom-x86_64-pc-windows-msvc.zip'

Install-ChocolateyZipPackage `
  -PackageName $packageName `
  -Url64bit $url64 `
  -UnzipLocation $toolsDir `
  -Checksum64 'PUT-SHA256-HERE' `
  -ChecksumType64 'sha256'
```

**User Installation:**
```powershell
choco install bazbom
```

#### 11.1.4 winget Package

**Windows Package Manager:** winget (built into Windows 11)

**Manifest:**
```yaml
# bazbom.yaml
PackageIdentifier: BazBOM.BazBOM
PackageVersion: 1.0.0
PackageName: BazBOM
Publisher: BazBOM Project
License: MIT
LicenseUrl: https://github.com/cboyd0319/BazBOM/blob/main/LICENSE
ShortDescription: Enterprise-grade SBOM and SCA for JVM projects
PackageUrl: https://bazbom.io
Installers:
  - Architecture: x64
    InstallerType: msi
    InstallerUrl: https://github.com/cboyd0319/BazBOM/releases/download/v1.0.0/bazbom.msi
    InstallerSha256: PUT-SHA256-HERE
ManifestType: singleton
ManifestVersion: 1.0.0
```

**Submit to winget repository:** https://github.com/microsoft/winget-pkgs

**User Installation:**
```powershell
winget install BazBOM.BazBOM
```

---

## 11.2 Kubernetes Operator

### Use Case

**Enterprise Scenario:**
- 100+ microservices in Kubernetes
- Each service needs SBOM + vulnerability scanning
- Centralized security dashboard
- Automated policy enforcement

### Architecture

```
┌──────────────────────────────────────────┐
│         Kubernetes Cluster               │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │  BazBOM Operator (Deployment)      │ │
│  │  - Watches for new Deployments    │ │
│  │  - Triggers scans                  │ │
│  │  - Stores results in ConfigMaps    │ │
│  └────────────────────────────────────┘ │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │  Scan Jobs (CronJob)               │ │
│  │  - Periodic rescans (daily/weekly) │ │
│  │  - Uses bazbom:latest image       │ │
│  └────────────────────────────────────┘ │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │  Dashboard (Optional)              │ │
│  │  - Web UI to view results          │ │
│  │  - Phase 6 dashboard in K8s        │ │
│  └────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

### Custom Resource Definition (CRD)

**Define BazBOMScan resource:**
```yaml
# crd/bazbomscan-crd.yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: bazbomscans.bazbom.io
spec:
  group: bazbom.io
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              properties:
                targetDeployment:
                  type: string
                  description: "Name of Deployment to scan"
                schedule:
                  type: string
                  description: "Cron schedule for scans (e.g., '0 0 * * *')"
                policyFile:
                  type: string
                  description: "ConfigMap containing bazbom.yml policy"
            status:
              type: object
              properties:
                lastScanTime:
                  type: string
                  format: date-time
                vulnerabilities:
                  type: object
                  properties:
                    critical:
                      type: integer
                    high:
                      type: integer
                    medium:
                      type: integer
                    low:
                      type: integer
                sbomUrl:
                  type: string
  scope: Namespaced
  names:
    plural: bazbomscans
    singular: bazbomscan
    kind: BazBOMScan
    shortNames:
      - bbs
```

### Operator Implementation

**Language:** Rust (using kube-rs crate)

```rust
// crates/bazbom-operator/src/main.rs
use k8s_openapi::api::apps::v1::Deployment;
use kube::{Api, Client, runtime::{Controller, watcher}};

struct BazBomOperator {
    client: Client,
}

impl BazBomOperator {
    async fn run(&self) -> Result<()> {
        let api: Api<BazBOMScan> = Api::all(self.client.clone());

        Controller::new(api, watcher::Config::default())
            .run(reconcile, error_policy, Arc::new(self.client.clone()))
            .for_each(|_| futures::future::ready(()))
            .await;

        Ok(())
    }
}

async fn reconcile(scan: Arc<BazBOMScan>, ctx: Arc<Client>) -> Result<Action> {
    println!("Reconciling BazBOMScan: {}", scan.metadata.name.as_ref().unwrap());

    // Get target Deployment
    let deployments: Api<Deployment> = Api::namespaced(ctx.clone(), &scan.metadata.namespace.as_ref().unwrap());
    let deployment = deployments.get(&scan.spec.target_deployment).await?;

    // Trigger scan Job
    create_scan_job(&scan, &deployment, &ctx).await?;

    // Update status
    update_scan_status(&scan, &ctx).await?;

    Ok(Action::requeue(Duration::from_secs(300)))  // Recheck every 5 minutes
}

async fn create_scan_job(scan: &BazBOMScan, deployment: &Deployment, client: &Client) -> Result<()> {
    let jobs: Api<Job> = Api::namespaced(client.clone(), &scan.metadata.namespace.as_ref().unwrap());

    let job = Job {
        metadata: ObjectMeta {
            name: Some(format!("bazbom-scan-{}", scan.metadata.name.as_ref().unwrap())),
            ..Default::default()
        },
        spec: Some(JobSpec {
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "bazbom-scanner".to_string(),
                        image: Some("bazbom/bazbom:latest".to_string()),
                        command: Some(vec![
                            "bazbom".to_string(),
                            "scan".to_string(),
                            ".".to_string(),
                            "--format".to_string(),
                            "spdx".to_string(),
                        ]),
                        ..Default::default()
                    }],
                    restart_policy: Some("Never".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    jobs.create(&PostParams::default(), &job).await?;
    Ok(())
}
```

### Deployment

**Install CRD and Operator:**
```bash
# Install CRD
kubectl apply -f crd/bazbomscan-crd.yaml

# Deploy operator
kubectl apply -f k8s/operator-deployment.yaml

# Example scan resource
kubectl apply -f - <<EOF
apiVersion: bazbom.io/v1
kind: BazBOMScan
metadata:
  name: my-app-scan
  namespace: default
spec:
  targetDeployment: my-app
  schedule: "0 0 * * *"  # Daily at midnight
  policyFile: bazbom-policy
EOF
```

**View Results:**
```bash
# Check scan status
kubectl get bazbomscan my-app-scan -o yaml

# View vulnerabilities
kubectl get bazbomscan my-app-scan -o jsonpath='{.status.vulnerabilities}'

# Download SBOM
kubectl get configmap my-app-sbom -o jsonpath='{.data.sbom\.spdx\.json}' > sbom.json
```

---

## 11.3 Air-Gapped Deployments

### Use Case

**Industries:** Government, defense, finance, healthcare

**Requirement:** Zero internet access (compliance, security)

### Implementation

#### 11.3.1 Offline Bundle

**Contents:**
- BazBOM binary (all platforms)
- Advisory database (OSV, NVD, GHSA, KEV, EPSS)
- Documentation (offline HTML)
- License compliance database (SPDX licenses)

**Bundle Structure:**
```
bazbom-offline-bundle-v1.0.0/
├── bin/
│   ├── bazbom-linux-x64
│   ├── bazbom-linux-arm64
│   ├── bazbom-macos-x64
│   ├── bazbom-macos-arm64
│   ├── bazbom-windows-x64.exe
├── data/
│   ├── advisories/
│   │   ├── osv/            # OSV database (JSON files)
│   │   ├── nvd/            # NVD database (JSON files)
│   │   ├── ghsa/           # GHSA database (JSON files)
│   │   ├── kev/            # CISA KEV catalog
│   │   └── epss/           # EPSS scores
│   ├── licenses/
│   │   └── spdx.json       # SPDX license list
│   └── malicious-packages.json
├── docs/
│   └── offline-docs.html   # Full documentation (no external links)
├── install.sh              # Linux/macOS installer
├── install.ps1             # Windows installer
├── README.md
└── checksums.txt           # SHA256 for all files
```

**Generate Bundle:**
```bash
# tools/bundle/create-offline-bundle.sh
#!/bin/bash
set -e

VERSION="1.0.0"
BUNDLE_DIR="bazbom-offline-bundle-v${VERSION}"

mkdir -p "${BUNDLE_DIR}/bin"
mkdir -p "${BUNDLE_DIR}/data/advisories"
mkdir -p "${BUNDLE_DIR}/data/licenses"
mkdir -p "${BUNDLE_DIR}/docs"

# Copy binaries
cp target/release/bazbom-* "${BUNDLE_DIR}/bin/"

# Download advisory databases
echo "Downloading OSV database..."
rsync -az --delete gs://osv-vulnerabilities/ "${BUNDLE_DIR}/data/advisories/osv/"

echo "Downloading NVD database..."
curl -o "${BUNDLE_DIR}/data/advisories/nvd.json" https://services.nvd.nist.gov/rest/json/cves/2.0

echo "Downloading CISA KEV..."
curl -o "${BUNDLE_DIR}/data/advisories/kev/known_exploited_vulnerabilities.json" \
  https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json

echo "Downloading EPSS scores..."
curl -o "${BUNDLE_DIR}/data/advisories/epss/epss_scores.csv" \
  https://epss.cyentia.com/epss_scores-current.csv.gz
gunzip "${BUNDLE_DIR}/data/advisories/epss/epss_scores.csv.gz"

# Copy licenses
cp crates/bazbom-formats/data/spdx-licenses.json "${BUNDLE_DIR}/data/licenses/"

# Generate offline docs
mdbook build docs/  # Assumes mdbook for documentation
cp -r docs/book/ "${BUNDLE_DIR}/docs/offline-docs/"

# Generate checksums
cd "${BUNDLE_DIR}"
find . -type f -exec sha256sum {} \; > checksums.txt

# Create tarball
cd ..
tar -czf "${BUNDLE_DIR}.tar.gz" "${BUNDLE_DIR}"

echo "Offline bundle created: ${BUNDLE_DIR}.tar.gz"
echo "Size: $(du -h ${BUNDLE_DIR}.tar.gz | cut -f1)"
```

**Install from Bundle:**
```bash
# Extract
tar -xzf bazbom-offline-bundle-v1.0.0.tar.gz
cd bazbom-offline-bundle-v1.0.0

# Install
sudo ./install.sh --offline

# Verify installation
bazbom --version
bazbom db status

# Output:
# Advisory Database Status:
#   Location: /opt/bazbom/data/advisories
#   OSV: 1,234,567 vulnerabilities (last updated: 2025-10-30)
#   NVD: 234,567 CVEs (last updated: 2025-10-30)
#   KEV: 1,234 actively exploited (last updated: 2025-10-30)
#   EPSS: 234,567 scores (last updated: 2025-10-30)
#   Mode: OFFLINE
```

#### 11.3.2 Update Mechanism

**Problem:** How to update advisory database without internet?

**Solution:** Incremental update bundles

```bash
# Generate incremental update (weekly)
bazbom db create-update-bundle --since 2025-10-23 --output weekly-update.tar.gz

# On air-gapped machine: Apply update
bazbom db apply-update weekly-update.tar.gz

# Output:
# Applying update bundle...
# Added 234 new vulnerabilities
# Updated 56 existing vulnerabilities
# Database now up to date as of 2025-10-30
```

---

## 11.4 Enterprise Package Managers

### SCCM (Microsoft System Center Configuration Manager)

**Package Format:** MSI (already created in 11.1.2)

**Deployment:**
```powershell
# Create SCCM application
New-CMApplication -Name "BazBOM" -Publisher "BazBOM Project"

# Add deployment type
Add-CMDeploymentType -ApplicationName "BazBOM" `
  -MsiInstaller -ContentLocation "\\fileserver\packages\bazbom.msi" `
  -InstallationBehaviorType InstallForSystem

# Deploy to collection
Start-CMApplicationDeployment -ApplicationName "BazBOM" `
  -CollectionName "All Workstations" -DeployAction Install -DeployPurpose Required
```

### Jamf (macOS Enterprise Management)

**Package Format:** PKG

**Create PKG:**
```bash
# Build macOS package
pkgbuild --root /usr/local/bin \
  --identifier io.bazbom.bazbom \
  --version 1.0.0 \
  --install-location /usr/local/bin \
  bazbom.pkg
```

**Deploy via Jamf:**
```
1. Upload bazbom.pkg to Jamf Pro
2. Create policy: "Install BazBOM"
3. Scope: All computers or specific groups
4. Execute: At check-in
```

### Ansible Playbook

```yaml
# playbooks/install-bazbom.yml
---
- name: Install BazBOM on all hosts
  hosts: all
  become: yes
  tasks:
    - name: Download BazBOM binary (Linux)
      get_url:
        url: "https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-x86_64-unknown-linux-gnu.tar.gz"
        dest: /tmp/bazbom.tar.gz
        checksum: "sha256:PUT-CHECKSUM-HERE"
      when: ansible_os_family == "Debian" or ansible_os_family == "RedHat"

    - name: Extract BazBOM
      unarchive:
        src: /tmp/bazbom.tar.gz
        dest: /usr/local/bin
        remote_src: yes

    - name: Make executable
      file:
        path: /usr/local/bin/bazbom
        mode: '0755'

    - name: Verify installation
      command: bazbom --version
      register: version_output

    - name: Display version
      debug:
        msg: "{{ version_output.stdout }}"
```

**Deploy:**
```bash
ansible-playbook -i inventory.ini playbooks/install-bazbom.yml
```

---

## Success Criteria

### Phase 11 Completion Checklist

- [ ] Windows binary compiles and runs on Windows 10/11
- [ ] MSI installer installs correctly
- [ ] Chocolatey package published
- [ ] winget package submitted and approved
- [ ] Kubernetes operator deploys to cluster
- [ ] CRD for BazBOMScan works
- [ ] Operator triggers scans on new Deployments
- [ ] Offline bundle includes all data (<5GB)
- [ ] Offline bundle installs without errors
- [ ] Incremental updates work
- [ ] SCCM deployment tested
- [ ] Ansible playbook tested on 10+ hosts

### Deployment Speed

| Scenario | Target Time | Method |
|----------|-------------|--------|
| **Single Windows PC** | <5 minutes | MSI or Chocolatey |
| **100 Windows PCs** | <1 hour | SCCM deployment |
| **Kubernetes Cluster** | <10 minutes | Helm chart + operator |
| **Air-Gapped 1000 Servers** | <1 day | Offline bundle + Ansible |

---

## Resource Requirements

**Team:** 1-2 developers for 12 weeks
**Skills:** Windows packaging (WiX), Kubernetes (kube-rs), DevOps (Ansible, SCCM)
**Budget:** $24K-48K (contractors)

**Testing Infrastructure:**
- Windows 10/11 VMs
- Kubernetes test cluster (can use Kind/Minikube)
- Air-gapped test environment (VM with no network)

---

**Last Updated:** 2025-10-30
**Phase 11 Complete** - All phases documented!
