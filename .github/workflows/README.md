# GitHub Actions Workflows

This directory contains GitHub Actions workflows for building and testing VCV Rack and the CLI client.

## Workflows

### 1. Docker Build (`docker-build.yml`)

Builds VCV Rack and the CLI client using the Docker build environment (Ubuntu 24.04).

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main`
- Manual dispatch

**Jobs:**
- Builds VCV Rack dependencies
- Builds VCV Rack (libRack.so and Rack executable)
- Builds CLI client (rack-cli)
- Runs HTTP API tests
- Uploads Linux build artifacts

**Artifacts:**
- `vcv-rack-linux` - Contains Rack, libRack.so, and rack-cli

**Usage:**
```bash
# Download artifacts from GitHub Actions
# Extract and run
./Rack --httpapi &
./tools/rack-cli list-modules
```

---

### 2. Build CLI Client (`build-cli-client.yml`)

Builds the rack-cli tool for all supported platforms with comprehensive testing.

**Triggers:**
- Push to `main`, `develop`, or `claude/**` branches (when CLI files change)
- Pull requests to `main` or `develop` (when CLI files change)
- Manual dispatch
- GitHub releases (to attach binaries)

**Jobs:**

#### `build-linux`
Builds CLI client for Linux (x86_64)
- Uses Ubuntu latest
- Static linking for portability
- Creates `.tar.gz` archive

#### `build-macos`
Builds CLI client for macOS (x86_64 and ARM64)
- Uses macOS latest
- Matrix build for both architectures
- Universal binary support
- Creates `.tar.gz` archive

#### `build-windows`
Builds CLI client for Windows (x86_64)
- Uses Windows latest with MSYS2
- MinGW-w64 toolchain
- Includes required DLLs
- Creates `.zip` archive

#### `test-cli`
Integration testing (depends on `build-linux`)
- Downloads Linux binary
- Starts Rack with Docker
- Creates test patch via API
- Tests all CLI commands:
  - `list-modules`
  - `show-module <id>`
  - `list-connections`
  - `list-connections --sort-by type`

#### `create-release-summary`
Creates release documentation (only on releases)
- Generates release notes
- Uploads summary artifact

**Artifacts:**

For regular builds (retention: 30 days):
- `rack-cli-linux-x64.tar.gz`
- `rack-cli-macos-x86_64.tar.gz`
- `rack-cli-macos-arm64.tar.gz`
- `rack-cli-windows-x64.zip`

For releases:
- Automatically attached to GitHub release

---

## Usage

### Manual Workflow Dispatch

You can manually trigger workflows from the GitHub Actions tab:

1. Go to **Actions** tab in the repository
2. Select the workflow you want to run
3. Click **Run workflow**
4. Choose the branch
5. Click **Run workflow**

### Downloading Artifacts

#### From Workflow Runs

1. Go to **Actions** tab
2. Click on a workflow run
3. Scroll to **Artifacts** section
4. Download the artifact you need

#### From Releases

If the workflow was triggered by a release:
1. Go to **Releases** tab
2. Find the release
3. Download the binary for your platform from **Assets**

### Using the Binaries

#### Linux
```bash
tar -xzf rack-cli-linux-x64.tar.gz
chmod +x rack-cli
./rack-cli --help
```

#### macOS
```bash
tar -xzf rack-cli-macos-x86_64.tar.gz  # or arm64
chmod +x rack-cli
./rack-cli --help
```

#### Windows
```powershell
# Extract rack-cli-windows-x64.zip
.\rack-cli.exe --help
```

---

## Local Testing

### Test Docker Build Workflow

```bash
# Build Docker image
./docker-build.sh build-image

# Build dependencies
./docker-build.sh deps -j4

# Build VCV Rack
./docker-build.sh build -j4

# Build CLI client
./docker-build.sh exec make cli

# Run tests
./docker-build.sh run
# In another terminal:
./docker-build.sh exec ./tools/rack-cli list-modules
```

### Test CLI Client Build

#### Linux
```bash
make dep
make cli
./tools/rack-cli --help
```

#### macOS
```bash
make dep
make cli
./tools/rack-cli --help
```

#### Windows (MSYS2)
```bash
make dep
make cli
./tools/rack-cli.exe --help
```

---

## CI/CD Best Practices

### Caching

Both workflows use GitHub Actions caching to speed up builds:
- Dependencies are cached based on `dep.mk` and `dep/Makefile` hashes
- Cache is platform-specific
- Cache is automatically invalidated when dependencies change

### Matrix Builds

The macOS build uses a matrix strategy to build for both x86_64 and ARM64:
```yaml
strategy:
  matrix:
    arch: [x86_64, arm64]
```

### Conditional Steps

Release uploads only run when triggered by a release:
```yaml
if: github.event_name == 'release'
```

### Artifact Retention

- Regular builds: 30 days
- Release builds: Permanent (attached to release)

---

## Troubleshooting

### Build Failures

#### "Dependencies not found"
The dependency cache might be corrupted. Clear it:
1. Go to repository **Settings** > **Actions** > **Caches**
2. Delete the cache for the failing platform
3. Re-run the workflow

#### "Binary not executable"
For Linux/macOS, ensure the binary has execute permissions:
```bash
chmod +x rack-cli
```

#### "Missing DLLs" (Windows)
The workflow should include all required DLLs. If missing:
- `libgcc_s_seh-1.dll`
- `libstdc++-6.dll`
- `libwinpthread-1.dll`

Download them from the MSYS2 package or re-run the workflow.

### Test Failures

#### "HTTP API not ready"
The test gives Rack 60 seconds to start. If it still fails:
- Check Docker logs
- Increase timeout in workflow
- Check if port 8080 is available

#### "Module creation failed"
Ensure the Fundamental plugin is available:
- Check if dependencies were built correctly
- Verify Docker image includes required plugins

---

## Adding New Workflows

### Workflow File Structure

```yaml
name: My Workflow

on:
  push:
    branches: [ main ]
  workflow_dispatch:

jobs:
  my-job:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # ... your steps
```

### Best Practices

1. **Use caching** for dependencies
2. **Set appropriate triggers** to avoid unnecessary runs
3. **Use matrix builds** for multi-platform support
4. **Add artifact retention** for debugging
5. **Include tests** when possible
6. **Use conditional steps** for releases
7. **Document** the workflow in this README

---

## Security Considerations

### Secrets

These workflows don't require secrets except for:
- `GITHUB_TOKEN` - Automatically provided by GitHub Actions
- Used only for uploading release assets

### Permissions

The workflows use default permissions:
- Read access to repository
- Write access to artifacts
- Write access to releases (on release events)

### Dependencies

All dependencies are built from source using the project's `dep/` system:
- No untrusted binary downloads
- Checksums verified during build
- Dependency versions locked in `dep.mk`

---

## Performance

### Build Times (Approximate)

| Job | Time | Cache Hit | Cache Miss |
|-----|------|-----------|------------|
| build-linux | 10-15 min | 2-3 min | 10-15 min |
| build-macos (x86_64) | 15-20 min | 3-5 min | 15-20 min |
| build-macos (arm64) | 15-20 min | 3-5 min | 15-20 min |
| build-windows | 20-25 min | 5-7 min | 20-25 min |
| test-cli | 5-10 min | 2-3 min | 5-10 min |

### Optimization Tips

1. **Enable caching** - Already implemented
2. **Use parallel builds** - `-j$(nproc)`
3. **Limit triggers** - Only run on relevant file changes
4. **Use matrix builds** - Build multiple platforms in parallel
5. **Skip tests** for draft PRs (add conditional)

---

## Monitoring

### View Workflow Status

Add a badge to your README:
```markdown
![Build CLI Client](https://github.com/yourusername/Rack/workflows/Build%20CLI%20Client/badge.svg)
```

### Notifications

Configure notifications in **Settings** > **Notifications**:
- Email on failure
- Slack/Discord integration
- GitHub mobile app

---

## Contributing

When modifying workflows:

1. Test locally first (when possible)
2. Use workflow dispatch to test changes
3. Update this README with changes
4. Document new jobs/steps
5. Consider backward compatibility

---

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
- [Caching Dependencies](https://docs.github.com/en/actions/using-workflows/caching-dependencies-to-speed-up-workflows)
- [Upload/Download Artifacts](https://docs.github.com/en/actions/using-workflows/storing-workflow-data-as-artifacts)
