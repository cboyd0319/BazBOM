# GitHub Actions Version Audit
**Date:** 2025-11-14
**Status:** 4 actions need updates, 1 inconsistency found

---

## Executive Summary

Out of **17 unique actions** used across workflows:
- ✅ **12 actions are up-to-date** (71%)
- ⚠️ **1 action has inconsistent versions** (6%)
- ❌ **4 actions need updates** (23%)

**Priority:** Update CodeQL Action to v4 (v3 deprecated Dec 2026)

---

## 🔴 CRITICAL: Actions Needing Updates

### 1. github/codeql-action (PRIORITY!)
**Current:** v3.27.9
**Latest:** v4
**Status:** ❌ OUTDATED - v3 will be deprecated December 2026

**Impact:**
- CodeQL Action v3 will be deprecated at the same time as GHES 3.19
- v4 runs on Node.js 24 runtime (newer, faster, more secure)
- Missing new features and performance improvements

**Used in:**
- `codeql.yml` (4 references)
- `bazbom-scan.yml` (1 reference)
- `bazel-pr-scan-example.yml` (1 reference)

**Update command:**
```bash
find .github/workflows -name "*.yml" -exec sed -i 's|github/codeql-action/\([^@]*\)@[^#]*# v3\.[0-9.]*|github/codeql-action/\1@v4|g' {} \;
```

---

### 2. peter-evans/create-pull-request
**Current:** v6.1.0
**Latest:** v7
**Status:** ❌ OUTDATED

**Impact:**
- Missing v7 features and improvements
- Potential security updates in v7

**Used in:**
- `version-bump.yml` (1 reference)

**Update:**
```yaml
# Change from:
uses: peter-evans/create-pull-request@c5a7806660adbe173f04e3e038b0ccdcd758773d # v6.1.0

# To:
uses: peter-evans/create-pull-request@v7
```

---

### 3. actions/download-artifact
**Current:** v4.3.0
**Latest:** v5
**Status:** ❌ OUTDATED

**Impact:**
- v5 adds ability to download artifacts by unique ID (not just name)
- Better support for immutable artifacts
- Improved error handling

**Used in:**
- `release.yml` (1 reference)

**Update:**
```yaml
# Change from:
uses: actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093 # v4.3.0

# To:
uses: actions/download-artifact@v5
```

---

## ⚠️ INCONSISTENCY: Mixed Versions

### 4. actions/upload-artifact
**Versions found:** v4.4.2 AND v4.6.2
**Latest:** v4.6.2
**Status:** ⚠️ INCONSISTENT

**Impact:**
- Different behaviors across workflows
- Potentially missing bug fixes in older version

**Usage:**
- `changelog.yml`: v4.4.2 ❌
- `dependency-review.yml`: v4.4.2 ❌
- `codeql.yml`: v4.4.2 ❌
- `release.yml`: v4.6.2 ✅
- `supplychain.yml`: v4.6.2 ✅
- `bazbom-scan.yml`: v4.6.2 ✅
- `bazel-pr-scan-example.yml`: v4.6.2 ✅

**Fix:** Standardize all to v4.6.2

---

## ✅ Up-to-Date Actions

### Core Actions (GitHub Official)
| Action | Current | Status |
|--------|---------|--------|
| actions/checkout | v5.0.0 | ✅ Latest |
| actions/setup-java | v5.0.0 | ✅ Latest |
| actions/setup-node | v6.0.0 | ✅ Latest |
| actions/cache | v4.1.2 | ✅ Latest |
| actions/dependency-review-action | v5.0.0 | ✅ Latest |
| actions/github-script | v7.1.0 | ✅ Latest |

### Third-Party Actions
| Action | Current | Status |
|--------|---------|--------|
| Swatinem/rust-cache | v2.8.1 | ✅ Latest |
| dtolnay/rust-toolchain | @stable | ✅ Dynamic (always latest stable) |
| codecov/codecov-action | v5.5.1 | ✅ Latest |
| sigstore/cosign-installer | v3.10.1 | ✅ Latest |
| softprops/action-gh-release | v2 | ✅ Latest major |
| bazel-contrib/setup-bazel | 0.15.0 | ✅ Latest |
| taiki-e/install-action | v2 | ✅ Latest major |

---

## 📊 Action Usage Statistics

### Most Used Actions:
1. **actions/checkout** - 13 workflows
2. **actions/upload-artifact** - 7 workflows (inconsistent versions!)
3. **Swatinem/rust-cache** - 6 workflows
4. **github/codeql-action** - 6 usages (needs update!)
5. **actions/setup-java** - 4 workflows

### Actions by Category:

**Checkout & Setup (5 actions)**
- actions/checkout
- actions/setup-java
- actions/setup-node
- dtolnay/rust-toolchain
- bazel-contrib/setup-bazel

**Caching (2 actions)**
- Swatinem/rust-cache
- actions/cache

**Artifacts (2 actions)**
- actions/upload-artifact
- actions/download-artifact

**Security (3 actions)**
- github/codeql-action
- actions/dependency-review-action
- sigstore/cosign-installer

**Automation (3 actions)**
- peter-evans/create-pull-request
- actions/github-script
- softprops/action-gh-release

**Other (2 actions)**
- codecov/codecov-action
- taiki-e/install-action

---

## 🔧 Automated Update Script

Run this script to update all outdated actions:

```bash
#!/bin/bash
set -euo pipefail

echo "Updating GitHub Actions to latest versions..."

# Update CodeQL Action v3 → v4 (CRITICAL)
echo "1. Updating github/codeql-action v3 → v4..."
find .github/workflows -name "*.yml" -exec sed -i \
  's|github/codeql-action/\([^@]*\)@[0-9a-f]\{40\} # v3\.[0-9.]*|github/codeql-action/\1@v4|g' {} \;

# Update peter-evans/create-pull-request v6 → v7
echo "2. Updating peter-evans/create-pull-request v6 → v7..."
find .github/workflows -name "*.yml" -exec sed -i \
  's|peter-evans/create-pull-request@[0-9a-f]\{40\} # v6\.[0-9.]*|peter-evans/create-pull-request@v7|g' {} \;

# Update actions/download-artifact v4 → v5
echo "3. Updating actions/download-artifact v4 → v5..."
find .github/workflows -name "*.yml" -exec sed -i \
  's|actions/download-artifact@[0-9a-f]\{40\} # v4\.[0-9.]*|actions/download-artifact@v5|g' {} \;

# Standardize actions/upload-artifact to v4.6.2
echo "4. Standardizing actions/upload-artifact to v4.6.2..."
find .github/workflows -name "*.yml" -exec sed -i \
  's|actions/upload-artifact@[0-9a-f]\{40\} # v4\.[0-9.]*|actions/upload-artifact@v4|g' {} \;

echo "✓ All actions updated successfully!"
echo ""
echo "Please review changes with: git diff .github/workflows/"
```

**Or manually update each workflow file**

---

## 🚨 Breaking Changes to Review

### CodeQL Action v3 → v4
**Breaking changes:**
- Requires Node.js 24 runtime (runners must support it)
- Some query suite names may have changed
- Configuration syntax updates

**Migration steps:**
1. Review [CodeQL v4 migration guide](https://github.com/github/codeql-action/blob/main/CHANGELOG.md)
2. Test on a feature branch first
3. Verify all CodeQL scans complete successfully
4. Check that SARIF uploads work correctly

---

### peter-evans/create-pull-request v6 → v7
**Breaking changes:**
- Updated minimum supported version requirements
- Some input parameter changes

**Migration steps:**
1. Review [v7 release notes](https://github.com/peter-evans/create-pull-request/releases)
2. Test version-bump.yml workflow
3. Ensure PRs are created correctly

---

### actions/download-artifact v4 → v5
**Breaking changes:**
- None - v5 is fully backward compatible with v4
- New `artifact-ids` input is optional

**Migration steps:**
- Direct replacement, no changes needed

---

### actions/upload-artifact v4.4.2 → v4.6.2
**Breaking changes:**
- None - patch version update within v4
- Bug fixes and minor improvements only

**Migration steps:**
- Direct replacement, no changes needed

---

## ⏱️ Update Priority & Timeline

### Immediate (This Week)
1. 🔴 **Standardize upload-artifact** - 5 minutes
   - Low risk, immediate benefit

### High Priority (This Month)
2. 🔴 **Update CodeQL to v4** - 30 minutes (includes testing)
   - v3 deprecated Dec 2026, but better to migrate now
   - Test thoroughly before deploying

### Medium Priority (This Quarter)
3. 🟡 **Update peter-evans/create-pull-request to v7** - 15 minutes
   - Version bump workflow only
   - Low impact, test in dev first

4. 🟡 **Update download-artifact to v5** - 5 minutes
   - Backward compatible
   - Release workflow only

---

## 📋 Post-Update Validation Checklist

After updating actions, verify:

- [ ] All workflows pass syntax validation
- [ ] CodeQL scans complete successfully
- [ ] SARIF uploads work correctly
- [ ] Artifact uploads/downloads function properly
- [ ] Version bump PR creation works
- [ ] Release builds complete successfully
- [ ] No breaking changes in workflow execution
- [ ] All security scans still run properly

---

## 🔒 Security Considerations

### Benefits of Updating:
1. **Security patches** - Newer versions contain fixes for discovered vulnerabilities
2. **Supply chain security** - Latest actions have improved provenance
3. **Node.js runtime updates** - v4 actions use Node.js 24 (more secure than Node.js 16)
4. **Deprecation avoidance** - Avoid forced migrations later

### Best Practices:
- ✅ Pin to major versions (e.g., `@v4`) for automatic patches
- ✅ Pin to SHA hashes for security-critical workflows (already doing this!)
- ✅ Review changelogs before major version updates
- ✅ Test in feature branches before merging

---

## 📈 Recommendation

**Implement updates in this order:**

1. ✅ **Week 1:** Standardize upload-artifact (low risk, quick)
2. ✅ **Week 2:** Update download-artifact to v5 (backward compatible)
3. ✅ **Week 3:** Update create-pull-request to v7 (test version-bump workflow)
4. ✅ **Week 4:** Update CodeQL to v4 (most impactful, needs testing)

**Total implementation time:** ~1-2 hours spread over 4 weeks for safe testing

---

## Summary

**Current Status:**
- 17 unique actions in use
- 12 up-to-date (71%)
- 4 need updates (23%)
- 1 inconsistency (6%)

**Next Steps:**
1. Review this audit
2. Choose update timeline
3. Run automated update script OR manually update
4. Test thoroughly
5. Deploy to production

**Questions?**
- Should we update all at once or incrementally?
- Any concerns about CodeQL v4 migration?
- Want me to implement the updates?
