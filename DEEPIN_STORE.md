# 🏪 Deepin Store Submission Guide

This guide explains how to submit BNN Code to the [Deepin Store](https://www.deepin.org/en/store/)
for easy installation by Deepin Linux users.

## 📋 Prerequisites

- A Deepin 25 system (or Deepin 20.9+)
- A [Deepin Store Developer Account](https://developer.deepin.org/)
- The `.deb` package built with `build-deb.sh`

## 🚀 Step-by-Step Submission

### 1. Build the .deb Package

```bash
# Ensure binary is built
cargo build --release

# Build .deb package
./build-deb.sh --version 0.1.0
```

This creates: `bnn-code_0.1.0_amd64.deb`

### 2. Test the Package Locally

```bash
# Install
sudo dpkg -i bnn-code_0.1.0_amd64.deb

# Verify
bnn --help
bnn init

# Uninstall if needed
sudo dpkg -r bnn-code
```

### 3. Submit to Deepin Store

1. Go to [Deepin Developer Center](https://developer.deepin.org/)
2. Sign in with your developer account
3. Click **"Submit Application"**
4. Fill in the form:
   - **Application Name**: BNN Code
   - **Package Name**: bnn-code
   - **Category**: Development Tools
   - **Description**: Terminal-native AI coding agent powered by Binarized Neural Networks
   - **Screenshots**: Add screenshots of the TUI
   - **License**: MIT
   - **Source URL**: https://github.com/bnn-code/bnn-code
5. Upload `bnn-code_0.1.0_amd64.deb`
6. Submit for review

### 4. Update Automatically via GitHub Actions

The CI/CD pipeline in `.github/workflows/release.yml` automatically builds
`.deb` packages when you push a new tag. You can configure Deepin Store
to watch the GitHub releases for updates.

## 📦 Package Structure

```
bnn-code_0.1.0_amd64.deb
├── DEBIAN/
│   ├── control          # Package metadata
│   ├── conffiles        # Config files
│   └── postinst         # Post-install script
├── usr/
│   ├── bin/
│   │   ├── bnn                    # Main binary
│   │   └── bnn-code-models.sh     # Model downloader
│   ├── share/
│   │   ├── applications/
│   │   │   └── bnn-code.desktop   # Desktop entry
│   │   ├── metainfo/
│   │   │   └── bnn-code.appdata.xml  # AppStream metadata
│   │   ├── doc/bnn-code/          # Documentation
│   │   ├── bash-completion/       # Tab completion
│   │   └── man/man1/              # Man page
```

## ✅ Requirements Checklist

| Requirement | Status | Notes |
|------------|--------|-------|
| MIT License | ✅ | LICENSE file included |
| Desktop file | ✅ | `bnn-code.desktop` |
| AppStream metadata | ✅ | `bnn-code.appdata.xml` |
| Man page | ✅ | `bnn-code.1` |
| Bash completion | ✅ | Tab completion |
| Static binary | ✅ | Rust builds static by default |
| No network required | ✅ | Core functionality works offline |

## 🔄 Auto-Update Configuration

For Deepin Store to automatically detect updates from GitHub:

1. In your developer dashboard, set **Update Method** to **"GitHub Releases"**
2. Enter your GitHub repository URL
3. The store will check for new releases daily

## 📊 Store Listing Preview

| Field | Value |
|-------|-------|
| **Name** | BNN Code |
| **ID** | bnn-code |
| **Category** | Development |
| **Size** | ~3-5 MB |
| **Price** | Free (Open Source) |
| **License** | MIT |
| **Languages** | 15+ programming languages |
| **Platform** | amd64 |
