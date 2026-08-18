#!/bin/bash
# ============================================
# 🏪 Build .deb Package for Deepin Store
# ============================================
# Creates a .deb package from the Rust binary
# compatible with Deepin 20+/23+, Debian, Ubuntu
#
# Usage:
#   ./build-deb.sh                     # Build .deb from local binary
#   ./build-deb.sh --version 0.1.0     # Custom version
#   ./build-deb.sh --install           # Build + install locally
# ============================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR"
BINARY_PATH="$PROJECT_DIR/target/release/bnn-code"
VERSION="0.1.0"
INSTALL_LOCAL=false

# ── Parse args ────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --version) VERSION="$2"; shift 2 ;;
        --install|-i) INSTALL_LOCAL=true; shift ;;
        --help|-h) echo "Usage: $0 [--version X.Y.Z] [--install]"; exit 0 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

# ── Colors ────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { echo -e "${CYAN}🔷${NC} $1"; }
ok()    { echo -e "${GREEN}✅${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠️${NC} $1"; }
err()   { echo -e "${RED}❌${NC} $1"; }

# ── Check binary ──────────────────────────────────────────────────
if [[ ! -f "$BINARY_PATH" ]]; then
    warn "Binary not found at $BINARY_PATH"
    info "Building release binary..."
    (cd "$PROJECT_DIR" && cargo build --release)
fi

if [[ ! -f "$BINARY_PATH" ]]; then
    err "Build failed!"
    exit 1
fi

BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
info "Binary: $BINARY_PATH ($BINARY_SIZE)"

# ── Create package structure ──────────────────────────────────────
PKG_NAME="bnn-code"
PKG_DIR="/tmp/${PKG_NAME}_${VERSION}_amd64"
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/applications"
mkdir -p "$PKG_DIR/usr/share/metainfo"
mkdir -p "$PKG_DIR/usr/share/doc/$PKG_NAME"
mkdir -p "$PKG_DIR/usr/share/bash-completion/completions"
mkdir -p "$PKG_DIR/usr/share/man/man1"

# ── Control file ──────────────────────────────────────────────────
cat > "$PKG_DIR/DEBIAN/control" << EOF
Package: $PKG_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.31), libgcc-s1 (>= 4.2)
Maintainer: BNN Code Team <team@bnn-code.dev>
Description: Terminal-native AI coding agent powered by Binarized Neural Networks
 BNN Code is a terminal-based AI coding assistant that helps you understand,
 refactor, and test your codebase using Binarized Neural Networks.
 .
 Features:
  • Codebase indexing with Tree-sitter AST parsing
  • Semantic code search and retrieval
  • AI-powered code explanation, refactoring, and test generation
  • Support for 15+ programming languages
  • Interactive TUI with ratatui
  • Fast binary inference with ONNX Runtime
Homepage: https://github.com/bnn-code/bnn-code
EOF

# ── Conffiles ─────────────────────────────────────────────────────
cat > "$PKG_DIR/DEBIAN/conffiles" << 'EOF'
/etc/bash_completion.d/bnn-code
EOF

# ── Post-installation script ─────────────────────────────────────
cat > "$PKG_DIR/DEBIAN/postinst" << 'POSTEOF'
#!/bin/bash
set -e

echo "🧠 BNN Code installed!"
echo ""
echo "Quick start:"
echo "  bnn init              # Initialize in current project"
echo "  bnn explain src/main.rs  # Explain a file"
echo "  bnn                   # Start interactive REPL"
echo ""
echo "To download AI model:"
echo "  bnn-code-models.sh --auto"

# Install bash completion
if [ -f /usr/share/bash-completion/completions/bnn-code ]; then
    echo "✓ Bash completion installed"
fi

exit 0
POSTEOF
chmod +x "$PKG_DIR/DEBIAN/postinst"

# ── Binary ────────────────────────────────────────────────────────
install -m 755 "$BINARY_PATH" "$PKG_DIR/usr/bin/bnn"

# ── Desktop entry (for Deepin Store) ──────────────────────────────
cat > "$PKG_DIR/usr/share/applications/bnn-code.desktop" << EOF
[Desktop Entry]
Name=BNN Code
Comment=Terminal-native AI coding agent powered by BNNs
Exec=bnn
Type=Application
Icon=utilities-terminal
Categories=Development;Utility;
Terminal=true
StartupNotify=false
Keywords=ai;coding;developer;terminal;
EOF

# ── AppStream metadata (for Deepin Store) ─────────────────────────
cat > "$PKG_DIR/usr/share/metainfo/bnn-code.appdata.xml" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<component type="console-application">
  <id>bnn-code</id>
  <name>BNN Code</name>
  <summary>Terminal-native AI coding agent powered by Binarized Neural Networks</summary>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <description>
    <p>BNN Code is a terminal-based AI coding assistant that helps you understand,
    refactor, and test your codebase using Binarized Neural Networks.</p>
    <p>Key features:</p>
    <ul>
      <li>Codebase indexing with Tree-sitter AST parsing</li>
      <li>Semantic code search and retrieval</li>
      <li>AI-powered code explanation, refactoring, and test generation</li>
      <li>Support for 15+ programming languages</li>
      <li>Interactive TUI with ratatui</li>
      <li>Fast binary inference with ONNX Runtime</li>
    </ul>
  </description>
  <url type="homepage">https://github.com/bnn-code/bnn-code</url>
  <url type="bugtracker">https://github.com/bnn-code/bnn-code/issues</url>
  <screenshots>
    <screenshot type="default">
      <image>https://raw.githubusercontent.com/bnn-code/bnn-code/main/docs/screenshot.png</image>
    </screenshot>
  </screenshots>
  <provides>
    <binary>bnn</binary>
  </provides>
  <categories>
    <category>Development</category>
    <category>Utility</category>
  </categories>
  <releases>
    <release version="$VERSION" date="$(date +%Y-%m-%d)">
      <description>
        <p>Initial release of BNN Code</p>
      </description>
    </release>
  </releases>
</component>
EOF

# ── Bash completion ───────────────────────────────────────────────
cat > "$PKG_DIR/usr/share/bash-completion/completions/bnn-code" << 'BASH'
_bnn_code() {
    local i cur prev opts cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    opts="-h --help -V --version explain refactor test init"
    
    if [[ ${cur} == -* ]] || [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi
    
    case "${prev}" in
        explain|refactor|test)
            COMPREPLY=( $(compgen -f -- ${cur}) )
            return 0
            ;;
    esac
}
complete -F _bnn_code bnn-code bnn
BASH

# ── Man page ──────────────────────────────────────────────────────
cat > "$PKG_DIR/usr/share/man/man1/bnn-code.1" << 'MANEOF'
.TH BNN-CODE 1 "2026-06-28" "0.1.0" "BNN Code Manual"
.SH NAME
bnn-code, bnn \- Terminal-native AI coding agent powered by BNNs
.SH SYNOPSIS
.B bnn
[\fIOPTIONS\fR] [\fIQUERY\fR]
.SH DESCRIPTION
BNN Code is a terminal-based AI coding assistant that helps you understand,
refactor, and test your codebase using Binarized Neural Networks.
.SH OPTIONS
.TP
.BR \-h ", " \-\-help
Show help message
.TP
.BR \-V ", " \-\-version
Print version
.TP
.BR \-p ", " \-\-path "=" \fIPATH\fR
Path to codebase (default: .)
.TP
.BR \-\-model "=" \fIMODEL\fR
BNN model to use
.SH COMMANDS
.TP
.B explain \fIFILE\fR
Explain a file or function
.TP
.B refactor \fIFILE\fR
Suggest refactoring improvements
.TP
.B test \fIFILE\fR
Generate unit tests
.TP
.B init
Initialize BNN Code in current project
.SH EXAMPLES
.B bnn init
.br
Initialize BNN Code in current directory
.br
.B bnn explain src/main.rs
.br
Explain a Rust file
.br
.B bnn "how does this parser work?"
.br
Ask a question in REPL mode
.SH REPORTING BUGS
https://github.com/bnn-code/bnn-code/issues
MANEOF

# ── Model download helper ────────────────────────────────────────
install -m 755 "$PROJECT_DIR/scripts/download_model.sh" "$PKG_DIR/usr/bin/bnn-code-models.sh"

# ── Build .deb ────────────────────────────────────────────────────
DEB_FILE="${PKG_NAME}_${VERSION}_amd64.deb"
info "Building .deb package..."
fakeroot dpkg-deb --build "$PKG_DIR" "$DEB_FILE" 2>/dev/null || dpkg-deb --build "$PKG_DIR" "$DEB_FILE"

if [[ -f "$DEB_FILE" ]]; then
    DEB_SIZE=$(du -h "$DEB_FILE" | cut -f1)
    ok "Package built: $DEB_FILE ($DEB_SIZE)"
    
    # Verify package
    dpkg-deb --info "$DEB_FILE" 2>/dev/null | head -10
    
    # Install if requested
    if [[ "$INSTALL_LOCAL" == true ]]; then
        info "Installing package..."
        sudo dpkg -i "$DEB_FILE"
        ok "Package installed! Run 'bnn --help' to get started."
    fi
    
    # Cleanup
    rm -rf "$PKG_DIR"
else
    err "Package build failed"
    exit 1
fi
