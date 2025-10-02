# Release Guide

This document outlines the release process for Marty.

## Prerequisites

- [cargo-release](https://github.com/crate-ci/cargo-release) installed
- [cargo-dist](https://github.com/axodotdev/cargo-dist) installed
- Push access to the main branch (or ability to merge PRs)

## Release Workflow

### Option 1: Direct Release (for maintainers with push access)

```bash
# 1. Ensure you're on the latest main branch
git checkout main
git pull

# 2. Update CHANGELOG.md
# Move items from [Unreleased] to a new version section

# 3. Commit changelog updates
git commit -am "chore: update changelog for v0.2.0"

# 4. Use cargo-release to handle versioning and tagging
cargo release 0.2.0 --workspace

# This will:
# - Update versions in all Cargo.toml files
# - Create a git tag (v0.2.0)
# - Push the tag to GitHub
# - Trigger the GitHub Actions workflow for building releases
```

### Option 2: Pull Request Workflow (recommended for teams)

```bash
# 1. Create a release branch
git checkout -b release-v0.2.0

# 2. Update CHANGELOG.md and commit
git commit -am "chore: prep release v0.2.0"

# 3. Use cargo-release to update versions (without tagging/publishing)
cargo release --no-publish --no-tag --allow-branch=release-v0.2.0 0.2.0

# 4. Push the branch and create a PR
git push -u origin release-v0.2.0

# 5. After PR is reviewed and merged to main:
git checkout main
git pull

# 6. Complete the release (creates tag and triggers CI)
cargo release
```

## What Happens Automatically

When a version tag (like `v0.2.0`) is pushed to GitHub:

1. **GitHub Actions** runs the release workflow
2. **Cross-platform binaries** are built for:
   - macOS (Intel & Apple Silicon)
   - Linux (x86_64 & ARM64, including MUSL)
   - Windows (x86_64)
3. **Installers** are generated:
   - Shell script for macOS/Linux
   - PowerShell script for Windows
   - Auto-updater binaries for each platform
4. **GitHub Release** is created with:
   - Release notes from CHANGELOG.md
   - All binary archives and installers
   - SHA256 checksums for verification

## Release Checklist

- [ ] Update CHANGELOG.md with new version
- [ ] Verify all tests pass locally (`cargo test`)
- [ ] Run `dist plan` to preview what will be built
- [ ] Follow the appropriate workflow (direct or PR-based)
- [ ] Verify GitHub Actions workflow completes successfully
- [ ] Test the installer scripts work correctly
- [ ] Update any deployment or installation documentation

## Troubleshooting

### If the release workflow fails:

1. Check the GitHub Actions logs for specific errors
2. Common issues:
   - Missing repository URL in Cargo.toml
   - Build failures on specific platforms
   - Missing or malformed CHANGELOG.md

### To fix a failed release:

1. Fix the underlying issue
2. Delete the problematic tag: `git tag -d v0.2.0 && git push origin :v0.2.0`
3. Re-run the release process

## Version Scheme

Marty follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for backwards-compatible functionality additions
- **PATCH** version for backwards-compatible bug fixes

Pre-release versions can use suffixes like `v0.2.0-beta.1`.