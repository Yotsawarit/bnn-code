#!/bin/bash
# ============================================================
# 🔌 BNN Code - VS Code Extension Installer
# ============================================================
# Builds and installs the VS Code extension for BNN Code.
#
# Usage:
#   ./scripts/install-vscode.sh                    # Build + install
#   ./scripts/install-vscode.sh --skip-build       # Install pre-built .vsix
#   ./scripts/install-vscode.sh --help             # Show help
# ============================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
VSCODE_DIR="$PROJECT_DIR/vscode-extension"
SKIP_BUILD=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-build) SKIP_BUILD=true; shift ;;
        --help|-h) echo "Usage: $0 [--skip-build]"; exit 0 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

# ── Colors ────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { echo -e "${CYAN}🔷${NC} $1"; }
ok()    { echo -e "${GREEN}✅${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠️${NC} $1"; }
err()   { echo -e "${RED}❌${NC} $1"; }

# ── Check prerequisites ──────────────────────────────────────────
info "Checking prerequisites..."

if ! command -v code &>/dev/null; then
    err "VS Code not found. Please install VS Code first."
    err "  https://code.visualstudio.com/download"
    exit 1
fi

if ! command -v npm &>/dev/null; then
    err "npm not found. Please install Node.js and npm."
    err "  https://nodejs.org/en/download/"
    exit 1
fi

ok "VS Code: $(code --version 2>&1 | head -1)"
ok "npm: $(npm --version)"

# ── Install dependencies ─────────────────────────────────────────
if [[ "$SKIP_BUILD" != true ]]; then
    info "Installing npm dependencies..."
    cd "$VSCODE_DIR"
    npm install --silent 2>&1 | tail -5
    ok "Dependencies installed"

    # ── Compile TypeScript ──────────────────────────────────────────
    info "Compiling TypeScript..."
    npm run compile 2>&1 | tail -5
    ok "TypeScript compiled"

    # ── Package extension ───────────────────────────────────────────
    info "Packaging extension..."
    npm run package 2>&1 | tail -5
    ok "Extension packaged"
fi

# ── Find .vsix file ──────────────────────────────────────────────
VSIX_FILE=$(ls "$VSCODE_DIR"/*.vsix 2>/dev/null | head -1)
if [[ -z "$VSIX_FILE" ]]; then
    # Try to build
    warn "No .vsix found. Building..."
    cd "$VSCODE_DIR"
    npm run package 2>&1 | tail -3
    VSIX_FILE=$(ls "$VSCODE_DIR"/*.vsix 2>/dev/null | head -1)
fi

if [[ -z "$VSIX_FILE" ]]; then
    err "Failed to build extension. Check for errors above."
    exit 1
fi

# ── Install extension ────────────────────────────────────────────
info "Installing extension: $(basename "$VSIX_FILE")"
code --install-extension "$VSIX_FILE" --force 2>&1
ok "VS Code extension installed!"

echo ""
echo "🚀 Restart VS Code to activate BNN Code"
echo ""
echo "Quick start:"
echo "  1. Open a code file"
echo "  2. Select some code"
echo "  3. Right-click → BNN → Explain"
echo "  4. Or press Ctrl+Shift+E (macOS: Cmd+Shift+E)"
