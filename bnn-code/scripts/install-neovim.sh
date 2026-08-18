#!/bin/bash
# ============================================================
# 🎮 BNN Code - Neovim Plugin Installer
# ============================================================
# Installs the BNN Code Neovim plugin and detects your
# plugin manager (lazy.nvim, packer.nvim, vim-plug).
#
# Usage:
#   ./scripts/install-neovim.sh                    # Interactive install
#   ./scripts/install-neovim.sh --auto             # Auto-install to std dir
#   ./scripts/install-neovim.sh --help             # Show help
# ============================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
NVIM_DIR="$PROJECT_DIR/nvim-plugin"
AUTO=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --auto) AUTO=true; shift ;;
        --help|-h) echo "Usage: $0 [--auto]"; exit 0 ;;
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
if ! command -v nvim &>/dev/null; then
    err "Neovim not found. Please install Neovim first."
    err "  https://github.com/neovim/neovim/wiki/Installing-Neovim"
    exit 1
fi

NVIM_VERSION=$(nvim --version | head -1)
ok "Neovim: $NVIM_VERSION"

# ── Detect plugin manager ────────────────────────────────────────
LAZY_DIR="$HOME/.local/share/nvim/lazy"
PACKER_DIR="$HOME/.local/share/nvim/site/pack/packer"
PLUG_DIR="$HOME/.local/share/nvim/site/pack/plugins/start"

detect_plugin_manager() {
    if [ -d "$LAZY_DIR" ]; then
        echo "lazy"
    elif ls "$PACKER_DIR"/*/opt 2>/dev/null | head -1 >/dev/null 2>&1; then
        echo "packer"
    elif [ -f "$HOME/.config/nvim/init.vim" ] && grep -q "plug#" "$HOME/.config/nvim/init.vim" 2>/dev/null; then
        echo "vim-plug"
    else
        echo "unknown"
    fi
}

PM=$(detect_plugin_manager)

# ── Print installation instructions ─────────────────────────────
print_instructions() {
    local pm=$1
    echo ""
    echo "📋 Add the following to your Neovim configuration:"
    echo ""

    case "$pm" in
        lazy)
            echo "  {"
            echo "      dir = '$HOME/.local/share/nvim/lazy/bnn.nvim',"
            echo "      name = 'bnn-code/nvim-plugin',"
            echo "      config = function()"
            echo "          require('bnn').setup({"
            echo "              binary_path = 'bnn',"
            echo "              codebase_path = '.',"
            echo "          })"
            echo "      end,"
            echo "  }"
            ;;
        packer)
            echo "  use {"
            echo "      'bnn-code/nvim-plugin',"
            echo "      config = function()"
            echo "          require('bnn').setup()"
            echo "      end,"
            echo "  }"
            ;;
        vim-plug)
            echo "  Plug 'bnn-code/nvim-plugin'"
            echo ""
            echo "  lua << EOF"
            echo "  require('bnn').setup()"
            echo "  EOF"
            ;;
        *)
            echo "  -- Copy plugin files to: $PLUG_DIR/bnn/"
            echo "  git clone https://github.com/bnn-code/nvim-plugin $PLUG_DIR/bnn"
            ;;
    esac

    echo ""
    echo "Then restart Neovim and run:"
    echo "  :checkhealth bnn"
}

# ── Auto-install if requested ────────────────────────────────────
if [[ "$AUTO" == true ]]; then
    info "Auto-installing plugin to $PLUG_DIR/bnn ..."
    mkdir -p "$PLUG_DIR/bnn"
    cp -r "$NVIM_DIR"/* "$PLUG_DIR/bnn/"
    ok "Plugin installed to $PLUG_DIR/bnn"
    echo ""
    echo "🚀 Restart Neovim to activate"
    exit 0
fi

# ── Interactive mode ─────────────────────────────────────────────
echo ""
echo "🧠 BNN Code Neovim Plugin Installer"
echo "=================================="
echo ""

case "$PM" in
    lazy)
        info "Detected: lazy.nvim"
        print_instructions "lazy"
        ;;
    packer)
        info "Detected: packer.nvim"
        print_instructions "packer"
        ;;
    vim-plug)
        info "Detected: vim-plug"
        print_instructions "vim-plug"
        ;;
    *)
        warn "No recognized plugin manager found."
        echo ""
        echo "Options:"
        echo "  1. Manual install  — copy files to $PLUG_DIR/bnn/"
        echo "  2. Git clone       — clone from GitHub"
        echo "  3. Print instructions"
        echo ""
        read -rp "Choose [1-3]: " choice

        case "$choice" in
            1)
                mkdir -p "$PLUG_DIR/bnn"
                cp -r "$NVIM_DIR"/* "$PLUG_DIR/bnn/"
                ok "Plugin installed to $PLUG_DIR/bnn"
                ;;
            2)
                mkdir -p "$PLUG_DIR"
                if [ -d "$PLUG_DIR/bnn" ]; then
                    warn "Plugin already exists. Updating..."
                    cd "$PLUG_DIR/bnn" && git pull
                else
                    git clone https://github.com/bnn-code/nvim-plugin "$PLUG_DIR/bnn"
                fi
                ok "Plugin cloned to $PLUG_DIR/bnn"
                ;;
            3)
                print_instructions "unknown"
                ;;
            *)
                err "Invalid choice"
                exit 1
                ;;
        esac
        ;;
esac

echo ""
echo "✅ Installer complete!"
echo ""
echo "🚀 Restart Neovim to activate BNN Code"
echo ""
echo "Quick start:"
echo "  :BnnExplain     — Explain current file"
echo "  :BnnQuery ...   — Ask a question"
echo "  <leader>be      — Explain selection (visual mode)"
