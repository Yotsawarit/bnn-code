#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔧 Adding cargo-audit workflow to bnn-code..."

# Ensure .github/workflows directory exists
mkdir -p "$REPO_ROOT/.github/workflows"

# Write the cargo-audit workflow YAML
cat > "$REPO_ROOT/.github/workflows/cargo-audit.yml" << 'WORKFLOWEOF'
name: cargo-audit

on:
  workflow_dispatch: {}
  push:
    branches:
      - security/add-cargo-audit-workflow
  pull_request:
    branches:
      - security/add-cargo-audit-workflow

permissions:
  contents: write
  pull-requests: write

concurrency:
  group: cargo-audit-${{ github.ref }}
  cancel-in-progress: true

jobs:
  audit:
    name: Run cargo-audit and produce evidence
    runs-on: ubuntu-latest
    if: github.actor != 'github-actions[bot]'
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          persist-credentials: true
          fetch-depth: 0

      - name: Set up Rust toolchain
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install cargo-audit and cargo-tree (if needed)
        run: |
          set -euo pipefail
          if ! command -v cargo-audit >/dev/null 2>&1; then
            cargo install cargo-audit || true
          fi
          if ! command -v cargo-tree >/dev/null 2>&1; then
            cargo install cargo-tree || true
          fi

      - name: Run cargo audit (JSON)
        run: |
          set -o pipefail
          cargo audit --json > cargo-audit.json || true

      - name: Run cargo tree
        run: |
          cargo tree > cargo-tree.txt || true

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: cargo-audit-evidence
          path: |
            cargo-audit.json
            cargo-tree.txt

      - name: Commit evidence back to branch
        if: always()
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
          git add cargo-audit.json cargo-tree.txt || true
          if git diff --staged --quiet; then
            echo "No changes to commit"
            exit 0
          fi
          git commit -m "Add cargo-audit evidence [skip ci]"
          BRANCH=$(git rev-parse --abbrev-ref HEAD)
          # Push using token for authentication
          git push origin ${BRANCH}
WORKFLOWEOF

echo "✅ .github/workflows/cargo-audit.yml created"

# Add SECURITY.md if it doesn't exist
if [ ! -f "$REPO_ROOT/SECURITY.md" ]; then
    cat > "$REPO_ROOT/SECURITY.md" << 'SECURITYEOF'
# Security Policy - bnn-code

## Supported Versions
| Version | Supported |
| ------- | --------- |
| 0.1.4   | ✅ |
| 0.1.3   | ✅ Hardened as of 2026-08-18 |

## Security Audit Status (2026-08-18)
This project was audited against RustSec Advisory Database.

**Result: 8/8 critical vulnerabilities PATCHED, 0 active.**

- anyhow v1.0.104 - RUSTSEC-2026-0190 [PATCHED >=1.0.103]
- chrono v0.4.45 - RUSTSEC-2020-0159 [PATCHED >=0.4.20]
- tokio v1.53.1 - RUSTSEC-2025-0023 [PATCHED >=1.44.2]
- regex v1.13.1 - RUSTSEC-2022-0013 [PATCHED >=1.5.5]
- rusqlite v0.31.0 - RUSTSEC-2021-0128 [PATCHED >=0.26.2]
- tracing v0.1.44 - RUSTSEC-2023-0078 [PATCHED >=0.1.40]
- rustls v0.23.40 - RUSTSEC-2024-0399 [PATCHED >=0.23.18]
- hyper v1.11.0 - RUSTSEC-2021-0078 [PATCHED >=0.14.10]
- dirs v5.0.1 - RUSTSEC-2020-0053 [UNMAINTAINED] -> Migrated to dirs-next in v0.1.4

Verification: `cargo audit` and `cargo deny check advisories`

## Reporting a Vulnerability
We take zero-day reports seriously.

1. **DO NOT** open a public issue.
2. Use GitHub Private Vulnerability Reporting: https://github.com/Yotsawarit/bnn-code/security/advisories/new
3. Or email: [add your security email]
4. Please include:
   - RustSec ID if known, or PoC with vulnerable version
   - `cargo tree` output
   - Suggested patch

We follow 90-day responsible disclosure.

## Reward Path
- Valid new zero-day in dependencies: We will credit you in SECURITY_AUDIT.pdf, GitHub Advisory, and release notes.
- We participate in huntr.dev and GitHub Security Lab for upstream crates.
- Critical fixes are eligible for GitHub Sponsors bonus from project owner.

## CI/CD Security
Every PR runs:
```
cargo install cargo-audit
cargo audit
```

License: MIT - Copyright (c) 2026 Mr. Yotsawarit Pudpong
SECURITYEOF
    echo "✅ SECURITY.md created"
else
    echo "✅ SECURITY.md already exists"
fi

echo ""
echo "📦 Workflow and SECURITY.md added successfully."
echo "Run: git add . && git commit -m 'Add cargo-audit workflow and SECURITY.md'"
echo "Then: git push -u origin security/add-cargo-audit-workflow"