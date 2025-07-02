# Automated Release Process

This project uses an automated release process based on [Conventional Commits](https://www.conventionalcommits.org/) and [Release Please](https://github.com/googleapis/release-please).

## How It Works

### 1. Commit Message Format

Use conventional commit messages in your pull requests:

- `fix:` - Bug fixes (patch version bump)
- `feat:` - New features (minor version bump)
- `feat!:` or `BREAKING CHANGE:` - Breaking changes (major version bump)
- `docs:` - Documentation changes
- `style:` - Code style changes
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `build:` - Build system changes
- `ci:` - CI configuration changes

### 2. Release Process

1. **Automatic PR Creation**: When commits with `fix:`, `feat:`, or breaking changes are pushed to `master`, Release Please will automatically create a PR with:
   - Updated version in `Cargo.toml`
   - Generated `CHANGELOG.md`
   - Proper semantic version bump

2. **Release Creation**: When the Release Please PR is merged:
   - A new git tag is created
   - A GitHub release is published with auto-generated release notes
   - Binaries are built for all supported platforms
   - Docker images are built and pushed to GitHub Container Registry
   - SHA256 checksums are generated

### 3. Supported Platforms

The release process automatically builds binaries for:

- `x86_64-unknown-linux-gnu` (Linux x64)
- `x86_64-unknown-linux-musl` (Linux x64 static)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-pc-windows-msvc` (Windows x64)

### 4. Docker Images

Docker images are automatically built and pushed to:
- `ghcr.io/0x79de/polymarket-mcp:latest`
- `ghcr.io/0x79de/polymarket-mcp:v{version}`
- `ghcr.io/0x79de/polymarket-mcp:{major}.{minor}`
- `ghcr.io/0x79de/polymarket-mcp:{major}`

## Examples

### Bug Fix (Patch Release)
```
fix: resolve connection timeout issues

Fixes timeout when API is slow to respond.
Closes #123
```
This will bump version from `1.0.0` → `1.0.1`

### New Feature (Minor Release)
```
feat: add new market analysis tool

- Implement sentiment analysis
- Add volume trend detection
- Include risk assessment

Closes #124
```
This will bump version from `1.0.1` → `1.1.0`

### Breaking Change (Major Release)
```
feat!: redesign API interface

BREAKING CHANGE: The API now uses different parameter names
for market queries. See migration guide for details.

Closes #125
```
This will bump version from `1.1.0` → `2.0.0`

## Manual Override

If you need to force a specific version bump, you can use:

- `fix!:` - Force major version bump for a fix
- `feat!:` - Force major version bump for a feature
- Include `BREAKING CHANGE:` in the commit body

## Configuration

The release process is configured via:

- `release-please-config.json` - Release Please configuration
- `.release-please-manifest.json` - Current version tracking
- `.github/workflows/release.yml` - GitHub Actions workflow

## Troubleshooting

### Release Please PR Not Created

1. Check that commits follow conventional commit format
2. Ensure commits are on the `master` branch
3. Verify no unreleased changes are already in a Release Please PR

### Build Failures

1. Check the GitHub Actions logs
2. Ensure all tests pass locally
3. Verify cross-compilation dependencies are available

### Docker Build Issues

1. Check Dockerfile syntax
2. Ensure all required files are included in build context
3. Verify base image compatibility