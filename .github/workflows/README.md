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

Builds VCV Rack and the rack-cli tool for Linux with comprehensive testing.

**Triggers:**
- Push to `main`, `develop`, or `claude/**` branches (when source files change)
- Pull requests to `main` or `develop` (when source files change)
- Manual dispatch
- GitHub releases (to attach binaries)

**Jobs:**

#### `build-linux`
Builds VCV Rack and CLI client for Linux (x86_64)
- Uses Ubuntu latest
- Builds dependencies from source
- Builds Rack executable and libRack.so
- Builds CLI client
- Static linking for portability
- Includes all resources (fonts, presets, translations)
- Creates comprehensive `.tar.gz` archive

**What's included in the build:**
- `Rack` - VCV Rack standalone application
- `libRack.so` - VCV Rack shared library
- `rack-cli` - CLI client for HTTP API
- `res/` - Resources (fonts, component SVGs, Core module panels)
- `presets/` - Module presets
- `translations/` - Internationalization files
- `cacert.pem` - SSL certificates for HTTPS
- `Core.json` - Core module metadata
- `template.vcv` - Default patch template
- Documentation and license files

#### `test-cli`
Integration testing (depends on `build-linux`)
- Downloads Linux build artifact
- Starts Rack with Xvfb (virtual display)
- Creates test patch via API (VCO → VCF → VCA)
- Tests all CLI commands:
  - `list-modules`
  - `show-module <id>`
  - `list-connections`
  - `list-connections --sort-by module`
  - `list-connections --sort-by type`

**Artifacts:**

For regular builds (retention: 30 days):
- `vcv-rack-linux-x64.tar.gz` - Complete VCV Rack build with CLI client

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
3. Download `vcv-rack-linux-x64.tar.gz` from **Assets**

### Using the Build

#### Linux
```bash
# Extract archive
tar -xzf vcv-rack-linux-x64.tar.gz
cd vcv-rack-linux-x64

# Run VCV Rack with HTTP API
./Rack --httpapi

# In another terminal, use CLI client
./rack-cli list-modules
./rack-cli show-module 1
./rack-cli list-connections --sort-by type
```

The archive includes everything needed to run VCV Rack and the CLI client.

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

### Test Full Build Locally

#### Linux
```bash
# Build dependencies
make dep

# Build Rack
make all

# Build CLI client
make cli

# Test
./Rack --httpapi &
./tools/rack-cli list-modules
```

---

## CI/CD Best Practices

### Caching

Both workflows use GitHub Actions caching to speed up builds:
- Dependencies are cached based on `dep.mk` and `dep/Makefile` hashes
- Cache is automatically invalidated when dependencies change
- Typical cache hit: 2-3 minutes vs 10-15 minutes for full dependency build

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
| test-cli | 5-10 min | N/A | 5-10 min |
| **Total** | **15-25 min** | **7-13 min** | **15-25 min** |

### Optimization Tips

1. **Enable caching** - Already implemented
2. **Use parallel builds** - `-j$(nproc)` flag used throughout
3. **Limit triggers** - Only runs on relevant file changes
4. **Path-based triggers** - Skips builds when only docs change
5. **Artifact retention** - 30 days keeps storage costs low

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
