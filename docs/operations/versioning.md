# Versioning and Release Process

BazBOM follows [Semantic Versioning 2.0.0](https://semver.org/) for all releases.

## Version Format

Versions follow the format: `MAJOR.MINOR.PATCH[-PRERELEASE]`

- **MAJOR**: Incremented for incompatible API changes or breaking changes
- **MINOR**: Incremented for new features in a backwards-compatible manner
- **PATCH**: Incremented for backwards-compatible bug fixes
- **PRERELEASE** (optional): Alpha, beta, or release candidate identifiers (e.g., `0.3.0-beta.1`)

## Release Types

### Stable Releases

Stable releases are tagged as `vX.Y.Z` (e.g., `v0.2.1`, `v1.0.0`).

### Pre-releases

Pre-release versions include a suffix:
- **Alpha**: `vX.Y.Z-alpha.N` - Early testing, unstable
- **Beta**: `vX.Y.Z-beta.N` - Feature complete, testing phase
- **Release Candidate**: `vX.Y.Z-rc.N` - Final testing before stable release

## Automated Release Process

### Option 1: Using the Bump Version Workflow (Recommended)

1. Navigate to **Actions** → **Version Bump Automation** in GitHub
2. Click **Run workflow**
3. Enter the new version (e.g., `0.2.1` or `0.3.0-beta`)
4. Select whether to create a GitHub release
5. The workflow will:
   - Create a new branch
   - Update all `Cargo.toml` files
   - Update `CHANGELOG.md`
   - Create a Pull Request
6. Review and merge the PR
7. After merging, manually create and push the tag:
   ```bash
   git tag -a v0.2.1 -m "Release v0.2.1"
   git push origin v0.2.1
   ```
8. The release workflow will automatically trigger and:
   - Build binaries for all platforms
   - Sign artifacts with Sigstore/cosign
   - Generate checksums
   - Create a GitHub release with release notes

### Option 2: Manual Version Bump

1. Use the bump version script:
   ```bash
   ./tools/dev/bump-version.sh 0.2.1
   ```

2. Review the changes:
   ```bash
   git diff
   ```

3. Commit and push:
   ```bash
   git add -A
   git commit -m "chore: bump version to 0.2.1"
   git push origin main
   ```

4. Create and push the tag:
   ```bash
   git tag -a v0.2.1 -m "Release v0.2.1"
   git push origin v0.2.1
   ```

5. The release workflow will automatically build and publish artifacts

## Version Management Scripts

### Version Bump Script

The `tools/dev/bump-version.sh` script automates version updates:

```bash
./tools/dev/bump-version.sh <new-version>
```

This script:
- Validates version format
- Updates all `Cargo.toml` files in `crates/`
- Updates `CHANGELOG.md` with the release date
- Regenerates `Cargo.lock`
- Provides next-step instructions

### Release Verification Script

The `tools/dev/verify-release.sh` script checks if a version is ready for release:

```bash
./tools/dev/verify-release.sh [version]
```

This script verifies:
- Version consistency across all crates
- Cargo.lock is up to date
- CHANGELOG.md has an entry for the version
- Working directory is clean (no uncommitted changes)
- Git tag doesn't already exist
- Project builds without errors
- Documentation is properly linked

Run this script before creating a release tag to ensure everything is in order.

## Release Workflow

When a tag matching `v*` is pushed, the release workflow:

1. **Build**: Compiles binaries for:
   - Linux x86_64
   - Linux aarch64
   - macOS x86_64 (Intel)
   - macOS aarch64 (Apple Silicon)

2. **Sign**: Uses Sigstore cosign for keyless signing
   - Generates `.sig` signature files
   - Creates `.sha256` checksum files

3. **Release**: Creates a GitHub release with:
   - All binary artifacts
   - Signature and checksum files
   - Auto-generated changelog from git commits
   - Pre-release flag for alpha/beta/rc versions

## Changelog Maintenance

### Unreleased Changes

All changes should be documented under the `## [Unreleased]` section in `CHANGELOG.md` as they are made:

```markdown
## [Unreleased]

### Added
- New feature description

### Changed
- Modified behavior description

### Fixed
- Bug fix description

### Security
- Security-related change description
```

### Release Preparation

When preparing a release, the version bump script automatically:
1. Adds a new version section with the current date
2. Moves unreleased changes to the version section
3. Creates a new empty `[Unreleased]` section

## Commit Message Conventions

For better changelog generation, follow conventional commit formats:

- `feat:` - New features (MINOR version bump)
- `fix:` - Bug fixes (PATCH version bump)
- `docs:` - Documentation changes
- `chore:` - Maintenance tasks
- `refactor:` - Code refactoring
- `test:` - Test updates
- `security:` - Security fixes or improvements
- `breaking:` or `BREAKING CHANGE:` - Breaking changes (MAJOR version bump)

Example:
```bash
git commit -m "feat: add Maven shade plugin support"
git commit -m "fix: resolve reachability analysis deadlock"
git commit -m "security: update vulnerable dependencies"
```

## Pre-release Testing

Before creating a stable release:

1. Create a pre-release:
   ```bash
   ./tools/dev/bump-version.sh 0.3.0-rc.1
   ```

2. Test thoroughly across all supported platforms

3. If issues are found, create additional release candidates (rc.2, rc.3, etc.)

4. Once stable, create the final release:
   ```bash
   ./tools/dev/bump-version.sh 0.3.0
   ```

## Version Verification

After a version bump, verify consistency:

```bash
# Check all crate versions
grep -r "^version = " crates/*/Cargo.toml

# Verify Cargo.lock is up to date
cargo check --workspace
```

All crates should have the same version number.

## Additional Resources

- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Sigstore Cosign](https://docs.sigstore.dev/cosign/overview/)
