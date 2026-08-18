#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🚀 Creating PR with cargo-audit evidence..."

# Check we're on the right branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "security/add-cargo-audit-workflow" ]; then
    echo "❌ Not on security/add-cargo-audit-workflow branch (currently on $CURRENT_BRANCH)"
    echo "Switching branch..."
    git checkout security/add-cargo-audit-workflow
fi

# Stage the audit evidence files
echo "📦 Staging cargo-audit evidence..."
git add .github/workflows/cargo-audit.yml
git add SECURITY.md
git add cargo-audit.json 2>/dev/null || true
git add cargo-tree.txt 2>/dev/null || true

# Check what's changed
if git diff --staged --quiet; then
    echo "⚠️ No changes staged. Nothing to commit."
    exit 1
fi

# Commit the changes
echo "📝 Committing audit evidence..."
git commit -m "Add cargo-audit evidence [skip ci]"

# Try to open PR with gh if available
if command -v gh >/dev/null 2>&1; then
    echo "🌸 Opening PR with gh..."
    gh pr create \
        --title "Add cargo-audit workflow and security evidence" \
        --body "This PR adds:
- `.github/workflows/cargo-audit.yml` - CI workflow that runs `cargo audit` and `cargo tree` on every PR/push
- `SECURITY.md` - Security policy with audit status and vulnerability reporting guidelines
- `cargo-audit.json` and `cargo-tree.txt` - Evidence artifacts for dependency vulnerability tracking

The workflow runs on every PR and push to `security/add-cargo-audit-workflow`, collecting dependency vulnerability data from the RustSec advisory database."

    echo "✅ PR created successfully!"
else
    echo "⚠️ gh (GitHub CLI) not installed. Please create PR manually."
    echo "   git push -u origin security/add-cargo-audit-workflow"
    echo "   # Then use GitHub web UI or: gh pr create <args>"
fi