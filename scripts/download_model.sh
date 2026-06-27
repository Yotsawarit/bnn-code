#!/bin/bash
# ============================================
# 📥 BNN Model Downloader — Shell Wrapper
# ============================================
# Downloads a pre-converted ONNX model directly
# (no Python dependencies needed)
#
# Usage:
#   ./scripts/download_model.sh                    # Interactive menu
#   ./scripts/download_model.sh --auto             # Auto-download default
#   ./scripts/download_model.sh --list             # List available models
# ============================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MODELS_DIR="$(dirname "$SCRIPT_DIR")/models"
RELEASE_URL="https://github.com/bnn-code/models/releases/download/v0.1.0"

# ── Model configurations ──────────────────────────────────────────
declare -A MODELS
MODELS["codeberta-small"]="CodeBERTa-small-v1|codeberta-small.onnx|84M params, 6-layer RoBERTa"
MODELS["codebert-base"]="CodeBERT-base|codebert-base.onnx|125M params, 12-layer BERT"
MODELS["codebert-quantized"]="CodeBERT-base-quantized|codebert-base-quantized.onnx|Quantized (INT8) CodeBERT"

# ── Colors ────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; NC='\033[0m'

info()  { echo -e "${CYAN}🔷${NC} $1"; }
ok()    { echo -e "${GREEN}✅${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠️${NC} $1"; }
err()   { echo -e "${RED}❌${NC} $1"; }

# ── Functions ─────────────────────────────────────────────────────
list_models() {
    echo -e "\n${CYAN}Available Models:${NC}"
    echo "─────────────────────────────────────────────────"
    printf "  %-25s %-25s %s\n" "NAME" "FILE" "DESCRIPTION"
    echo "─────────────────────────────────────────────────"
    for key in "${!MODELS[@]}"; do
        IFS='|' read -r name file desc <<< "${MODELS[$key]}"
        printf "  ${GREEN}%-25s${NC} %-25s %s\n" "$key" "$file" "$desc"
    done
    echo "─────────────────────────────────────────────────"
}

download_model() {
    local model_key="$1"
    local entry="${MODELS[$model_key]:-}"
    
    if [[ -z "$entry" ]]; then
        err "Unknown model: $model_key"
        list_models
        exit 1
    fi

    IFS='|' read -r name file desc <<< "$entry"
    
    mkdir -p "$MODELS_DIR"
    local target="$MODELS_DIR/$file"

    if [[ -f "$target" ]]; then
        local size=$(du -h "$target" | cut -f1)
        info "Model already exists: $target ($size)"
        return 0
    fi

    local url="$RELEASE_URL/$file"
    info "Downloading ${CYAN}$name${NC} ($desc)..."
    info "URL: $url"
    
    echo -n "  "
    if command -v wget &>/dev/null; then
        wget -q --show-progress "$url" -O "$target"
    elif command -v curl &>/dev/null; then
        curl -#L "$url" -o "$target"
    else
        err "Need wget or curl"
        exit 1
    fi

    local size=$(du -h "$target" | cut -f1)
    ok "Downloaded: $file ($size)"
    
    # Also download tokenizer if available
    local tokenizer_url="$RELEASE_URL/tokenizer.json"
    if command -v wget &>/dev/null; then
        wget -q "$tokenizer_url" -O "$MODELS_DIR/tokenizer.json" 2>/dev/null && \
            ok "Tokenizer downloaded" || warn "Tokenizer not available separately"
    fi
    
    # Write metadata
    cat > "$MODELS_DIR/metadata.json" << EOF
{
  "model": "$name",
  "file": "$file",
  "description": "$desc",
  "downloaded_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "source": "$RELEASE_URL"
}
EOF
    ok "Metadata saved"
}

interactive_menu() {
    echo -e "\n${CYAN}╔══════════════════════════════╗${NC}"
    echo -e "${CYAN}║  📥 BNN Model Downloader     ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════╝${NC}"
    
    list_models
    
    echo -e "\n${YELLOW}Select model to download:${NC}"
    local i=1
    local keys=()
    for key in "${!MODELS[@]}"; do
        echo "  $i) $key"
        keys+=("$key")
        ((i++))
    done
    echo "  $i) Download ALL"
    echo "  q) Quit"
    echo ""
    
    read -rp "Choice [1-$i]: " choice
    
    if [[ "$choice" == "q" ]]; then
        exit 0
    fi
    
    if [[ "$choice" -eq "$i" ]]; then
        for key in "${keys[@]}"; do
            download_model "$key"
        done
    elif [[ "$choice" -ge 1 && "$choice" -lt "$i" ]]; then
        download_model "${keys[$((choice - 1))]}"
    else
        err "Invalid choice"
        exit 1
    fi
}

# ── Main ──────────────────────────────────────────────────────────
main() {
    case "${1:-interactive}" in
        --auto|-a)
            download_model "codeberta-small"
            ;;
        --list|-l)
            list_models
            ;;
        --help|-h)
            echo "Usage: $0 [--auto|--list|--help]"
            echo ""
            echo "  (no args)    Interactive menu"
            echo "  --auto, -a   Download default model"
            echo "  --list, -l   List available models"
            echo "  --help, -h   Show this help"
            ;;
        *)
            if [[ -n "${1:-}" ]]; then
                download_model "$1"
            else
                interactive_menu
            fi
            ;;
    esac
}

main "$@"
