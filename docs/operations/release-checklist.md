# Release Checklist

This checklist ensures consistent, high-quality releases for BazBOM.

## Pre-Release

- [ ] All planned features/fixes are complete and merged
- [ ] All tests pass: `cargo test --workspace`
- [ ] No security vulnerabilities: Review Dependabot alerts
- [ ] Documentation is up to date
- [ ] CHANGELOG.md has all changes documented under `[Unreleased]`
- [ ] Performance benchmarks meet expectations (if applicable)
- [ ] Breaking changes are clearly documented

## Version Bump

### Option 1: Automated (GitHub Actions)

1. [ ] Go to **Actions** → **Version Bump Automation**
2. [ ] Click **Run workflow**
3. [ ] Enter new version (e.g., `0.2.1` or `0.3.0-beta`)
4. [ ] Review and merge the created PR
5. [ ] After merge, create and push the tag:
   ```bash
   git checkout main
   git pull
   git tag -a v0.2.1 -m "Release v0.2.1"
   git push origin v0.2.1
   ```

### Option 2: Manual

1. [ ] Run version bump script:
   ```bash
   ./tools/dev/bump-version.sh 0.2.1
   ```
2. [ ] Review changes: `git diff`
3. [ ] Run verification script:
   ```bash
   ./tools/dev/verify-release.sh 0.2.1
   ```
4. [ ] Fix any issues reported by verification
5. [ ] Commit changes:
   ```bash
   git add -A
   git commit -m "chore: bump version to 0.2.1"
   git push origin main
   ```
6. [ ] Create and push tag:
   ```bash
   git tag -a v0.2.1 -m "Release v0.2.1"
   git push origin v0.2.1
   ```

## Post-Tag

After pushing the tag, the automated release workflow will:

1. [ ] Build binaries for all platforms (Linux x86_64/aarch64, macOS x86_64/aarch64)
2. [ ] Sign artifacts with Sigstore cosign
3. [ ] Generate SHA256 checksums
4. [ ] Create GitHub release with changelog
5. [ ] Upload all artifacts

## Verification

Once the release workflow completes:

1. [ ] Verify release appears on GitHub releases page
2. [ ] Check all artifacts are present:
   - [ ] `bazbom-x86_64-unknown-linux-gnu.tar.gz`
   - [ ] `bazbom-aarch64-unknown-linux-gnu.tar.gz`
   - [ ] `bazbom-x86_64-apple-darwin.tar.gz`
   - [ ] `bazbom-aarch64-apple-darwin.tar.gz`
   - [ ] `.sig` files for each archive
   - [ ] `.sha256` files for each archive
3. [ ] Verify signatures (optional but recommended):
   ```bash
   cosign verify-blob --signature bazbom-*.tar.gz.sig bazbom-*.tar.gz
   ```
4. [ ] Test installation on at least one platform:
   ```bash
   # Download and extract
   tar -xzf bazbom-*.tar.gz
   ./bazbom --version
   ```

## Post-Release

1. [ ] Update Homebrew formula (if applicable)
   - [ ] Update version in formula
   - [ ] Update SHA256 checksums
   - [ ] Test formula: `brew install --build-from-source bazbom`
2. [ ] Announce release:
   - [ ] GitHub Discussions
   - [ ] Project README badge (if needed)
3. [ ] Monitor for issues:
   - [ ] Check GitHub issues for bug reports
   - [ ] Review GitHub Actions for any workflow failures
4. [ ] Update project board/milestone
   - [ ] Close release milestone
   - [ ] Create next milestone (if needed)

## Rollback Procedure

If a critical issue is found after release:

1. [ ] Create hotfix branch from release tag:
   ```bash
   git checkout -b hotfix-0.2.2 v0.2.1
   ```
2. [ ] Fix the issue
3. [ ] Follow version bump process for hotfix version (0.2.2)
4. [ ] Tag and release hotfix
5. [ ] Consider yanking problematic release if necessary

## Pre-Release (Alpha/Beta/RC)

For pre-releases, follow the same process but:

1. [ ] Use pre-release version format: `0.3.0-alpha.1`, `0.3.0-beta.1`, `0.3.0-rc.1`
2. [ ] GitHub will automatically mark it as pre-release
3. [ ] Document known issues or limitations
4. [ ] Specify target testers/audience
5. [ ] Set clear expiration or upgrade timeline

## Automation Notes

- Version bump workflow creates a PR for review
- Release workflow triggers automatically on tag push
- Changelog is auto-generated from commit messages
- Pre-release detection is automatic (alpha/beta/rc suffixes)
- All actions are SHA-pinned for security
- Artifacts are signed with Sigstore keyless signing

## Tools

- **bump-version.sh** - Automate version updates
- **verify-release.sh** - Pre-release verification
- **GitHub Actions** - Automated workflows
- **cosign** - Artifact signing
- **cargo** - Build and test

## References

- [Versioning Guide](versioning.md)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Sigstore Cosign](https://docs.sigstore.dev/cosign/overview/)
