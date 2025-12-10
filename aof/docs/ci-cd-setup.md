# CI/CD Pipeline Documentation

## Overview

This project uses GitHub Actions for continuous integration and deployment. The pipeline includes automated testing, linting, building, security scanning, and release management.

## Workflows

### 1. CI/CD Pipeline (`.github/workflows/ci.yml`)

**Triggers:**
- Push to `main` or `dev` branches
- Pull requests to `main` or `dev`
- Manual workflow dispatch

**Jobs:**

#### Format Check
- Runs on: `ubuntu-latest`
- Checks code formatting with `cargo fmt --check`
- Uses pre-task and post-task hooks for coordination

#### Lint
- Matrix: Ubuntu, macOS, Windows × Stable, Nightly Rust
- Runs `cargo clippy` with strict warnings
- Caches Cargo registry, index, and target directories
- Fail-fast: disabled (runs all combinations)

#### Build
- Matrix: Ubuntu, macOS, Windows × Stable, Nightly Rust
- Builds entire workspace with all features
- Timeout: 30 minutes
- Comprehensive caching strategy

#### Test
- Matrix: Ubuntu, macOS, Windows × Stable, Nightly Rust
- Runs all tests with `cargo test --workspace --all-features`
- Timeout: 30 minutes

#### Coverage
- Runs on: `ubuntu-latest`
- Uses `cargo-llvm-cov` for coverage generation
- Uploads to Codecov (requires `CODECOV_TOKEN` secret)
- Generates LCOV format reports

#### Build Release
- Matrix: Ubuntu, macOS, Windows × Stable Rust
- Builds optimized release binaries
- Uploads artifacts for each platform
- Retention: 7 days

### 2. Security Audit (`.github/workflows/security.yml`)

**Triggers:**
- Daily at 2 AM UTC (scheduled)
- Push to `main` or `dev`
- Pull requests to `main` or `dev`
- Manual workflow dispatch

**Jobs:**

#### Security Audit
- Runs `cargo audit` to check for vulnerabilities
- Fails on any security warnings

#### Dependency Check
- Uses `cargo-deny` for dependency policy enforcement
- Checks for banned/denied dependencies

#### Outdated Dependencies
- Uses `cargo-outdated` to check for outdated crates
- Continues on error (informational)

#### Supply Chain Security
- Analyzes dependency publishers
- Generates supply chain report (JSON)
- Retains report for 30 days

### 3. Release (`.github/workflows/release.yml`)

**Triggers:**
- Git tags matching semantic versioning:
  - `v1.2.3` (stable)
  - `v1.2.3-alpha.1` (alpha)
  - `v1.2.3-beta.1` (beta)
  - `v1.2.3-rc.1` (release candidate)
- Manual workflow dispatch with version input

**Jobs:**

#### Create Release
- Generates changelog from `CHANGELOG.md`
- Creates GitHub release
- Marks pre-releases appropriately

#### Build Release
- Cross-platform builds:
  - Linux: x86_64, ARM64
  - macOS: x86_64, ARM64
  - Windows: x86_64
- Strips binaries (Unix)
- Creates compressed archives (.tar.gz, .zip)
- Uploads to GitHub release

#### Publish Crate
- Publishes to crates.io (requires `CARGO_TOKEN` secret)
- Runs after successful builds

### 4. Documentation (`.github/workflows/docs.yml`)

**Triggers:**
- Push to `main`
- Pull requests to `main`
- Manual workflow dispatch

**Jobs:**

#### Build Documentation
- Generates rustdoc documentation
- Deploys to GitHub Pages (main branch only)
- Creates index redirect

#### Check Documentation
- Validates documentation with strict warnings
- Ensures no broken doc links

### 5. Dependabot (`.github/workflows/dependabot.yml`)

**Configuration:**
- Weekly updates on Mondays at 9 AM
- Separate configurations for:
  - Cargo dependencies
  - GitHub Actions versions
- Automatic PR creation with labels
- Configured reviewers

## Required Secrets

Configure these in GitHub repository settings → Secrets and variables → Actions:

### Essential Secrets

1. **CODECOV_TOKEN** (Optional but recommended)
   - For coverage reporting
   - Get from: https://codecov.io

2. **CARGO_TOKEN** (Required for publishing)
   - For publishing to crates.io
   - Get from: https://crates.io/settings/tokens

### API Keys for Testing (Not yet configured)

These should be added as you implement integrations:

3. **ANTHROPIC_API_KEY**
   - For Claude API tests
   - Get from: https://console.anthropic.com

4. **OPENAI_API_KEY**
   - For OpenAI API tests
   - Get from: https://platform.openai.com

5. **AWS_ACCESS_KEY_ID** and **AWS_SECRET_ACCESS_KEY**
   - For AWS Bedrock tests
   - Get from: AWS IAM Console

6. **AZURE_OPENAI_KEY** and **AZURE_OPENAI_ENDPOINT**
   - For Azure OpenAI tests
   - Get from: Azure Portal

## Caching Strategy

All workflows implement multi-level caching:

1. **Cargo Registry** (`~/.cargo/registry`)
2. **Cargo Index** (`~/.cargo/git`)
3. **Target Directory** (`target/`)

Cache keys include:
- OS identifier
- Rust toolchain version
- Cargo.lock hash

This significantly reduces build times on subsequent runs.

## Performance Optimization

- **Parallel Jobs**: Matrix strategies run multiple configurations simultaneously
- **Fail-fast: false**: All test combinations run even if one fails
- **Selective Caching**: Different cache keys for different job types
- **Artifact Compression**: Release binaries are compressed before upload

## Branch Protection Rules

Recommended settings for `main` and `dev` branches:

1. Require pull request reviews (1+ approvers)
2. Require status checks to pass:
   - Format Check
   - Lint (all matrix combinations)
   - Build (all matrix combinations)
   - Test (all matrix combinations)
   - Security Audit
3. Require branches to be up to date
4. Restrict who can push to matching branches
5. Require linear history (optional)

## Local Testing

Before pushing, run locally:

```bash
# Format check
cargo fmt --all -- --check

# Lint
cargo clippy --workspace --all-features --all-targets -- -D warnings

# Build
cargo build --workspace --all-features

# Test
cargo test --workspace --all-features

# Security audit
cargo install cargo-audit
cargo audit

# Dependency check
cargo install cargo-deny
cargo deny check

# Coverage (requires llvm-tools)
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --all-features
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with new version section
3. Commit changes to `dev` branch
4. Create PR to `main`
5. After merge, create and push tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

6. GitHub Actions will:
   - Create release
   - Build binaries for all platforms
   - Upload artifacts
   - Publish to crates.io

## Monitoring

- Check workflow runs: Repository → Actions
- View coverage: Codecov dashboard
- Review security: Repository → Security → Dependabot
- Monitor releases: Repository → Releases

## Troubleshooting

### Workflow Failures

1. **Cache Issues**: Clear cache in Actions settings
2. **Token Expiration**: Rotate secrets
3. **Build Failures**: Check Rust toolchain compatibility
4. **Test Timeouts**: Increase timeout or optimize tests

### Common Errors

- **Clippy Warnings**: Fix or allow specific lints
- **Format Violations**: Run `cargo fmt`
- **Security Vulnerabilities**: Update dependencies
- **Coverage Upload Fails**: Check CODECOV_TOKEN

## Continuous Improvement

- Monitor build times and optimize caching
- Update GitHub Actions versions regularly
- Review and update security policies
- Add new test scenarios as needed
- Optimize matrix strategies based on usage

## Integration with Claude Flow

All workflows integrate with `npx claude-flow@alpha hooks`:

- **Pre-task hooks**: Run before job execution
- **Post-task hooks**: Run after job completion
- **Session coordination**: Track workflow metrics
- **Memory storage**: Persist build information

This enables:
- Cross-workflow coordination
- Performance tracking
- Automated optimization
- Failure analysis
- Neural pattern training

## Next Steps

1. Configure repository secrets
2. Set up branch protection rules
3. Enable Codecov integration
4. Configure Dependabot alerts
5. Set up CODEOWNERS file
6. Enable GitHub Pages for docs
7. Configure notification preferences
