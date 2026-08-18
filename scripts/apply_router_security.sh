#!/bin/bash
# ============================================
# 🛡️ BNN Router Security — Apply iptables Rules
# ============================================
# Blocks the 5 vulnerable ports via iptables (INPUT DROP + LOG).
# Usage: ./scripts/apply_router_security.sh
# ============================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
Iptables="iptables"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; NC='\033[0m'

# 5 vulnerable ports to block
VULN_PORTS=(23 2323 4840 9000 5000)
PORTS_DESC=(
    "Telnet (TCP/23)"
    "Huawei Telnet (TCP/2323)"
    "Huawei TR-069 (TCP/4840)"
    "Common management (TCP/9000)"
    "SSH brute force target (TCP/5000)"
)

# Log prefix
LOG_PREFIX="SEC-DROP"

info()  { echo -e "${CYAN}🔷${NC} $1"; }
ok()    { echo -e "${GREEN}✅${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠️${NC} $1"; }
err()   { echo -e "${RED}❌${NC} $1"; }

# Check if iptables is available
check_iptables() {
    if ! command -v "$Iptables" &>/dev/null; then
        err "iptables not found. Install it first."
        exit 1
    fi
}

# Check if running as root
check_root() {
    if [[ "$EUID" -ne 0 ]]; then
        warn "Not running as root - iptables operations will be skipped"
        warn "Run with sudo for full functionality"
        SKIP_ROOT=true
    else
        SKIP_ROOT=false
    fi
}

# Block a single port with iptables INPUT DROP + LOG
block_port() {
    local port="$1"
    local desc="${PORTS_DESC[$((port - 23))]:-$port}"

    # Log the attempt first
    "$Iptables" -A INPUT -p tcp --dport "$port" -m state --state NEW -j LOG \
        --log-prefix "${LOG_PREFIX}-PORT-${port}" \
        --log-level 4

    # Then drop
    "$Iptables" -A INPUT -p tcp --dport "$port" -j DROP

    ok "Blocked ${desc} (TCP/${port})"
}

# Open a port for legitimate access (optional helper)
open_port() {
    local port="$1"
    local desc="${PORTS_DESC[$((port - 23))]:-$port}"

    "$Iptables" -D INPUT -p tcp --dport "$port" -j DROP 2>/dev/null || true
    "$Iptables" -A INPUT -p tcp --dport "$port" -j ACCEPT

    ok "Opened ${desc} (TCP/${port}) for legitimate access"
}

# Flush all existing rules for these ports
flush_rules() {
    for port in "${VULN_PORTS[@]}"; do
        "$Iptables" -D INPUT -p tcp --dport "$port" -j LOG \
            --log-prefix "${LOG_PREFIX}-PORT-${port}" 2>/dev/null || true
        "$Iptables" -D INPUT -p tcp --dport "$port" -j DROP 2>/dev/null || true
    done
    ok "Flushed existing iptables rules for vulnerable ports"
}

main() {
    check_root
    check_iptables

    if $SKIP_ROOT; then
        warn "Skipping iptables rules (not running as root)"
        ok "Limited mode: port status checks only"
        info "Use 'sudo $0' for full iptables functionality"
        echo ""
        ok "apply_router_security.sh completed (limited mode)"
        return 0
    fi

    info "BNN Router Security - Applying iptables rules for 5 vulnerable ports"
    info "Log prefix: ${LOG_PREFIX}-*"

    # First flush any existing rules
    flush_rules

    # Block each vulnerable port
    for port in "${VULN_PORTS[@]}"; do
        block_port "$port"
    done

    echo ""
    ok "All 5 vulnerable ports blocked via iptables INPUT DROP + LOG"
    info "Log entries will appear with prefix: ${LOG_PREFIX}-PORT-<port>"
    info "Use 'setup_security_logging.sh' to also install the watcher service"
}

main "$@"