#!/bin/bash
# ============================================
# 🛡️ BNN Security Logging Setup
# ============================================
# Installs LOG rules (prefix SEC-DROP-*) + security-alert-watcher.service.
# Usage: ./scripts/setup_security_logging.sh
# ============================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
Iptables="iptables"
SYSTEMD_DIR="/etc/systemd/system"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; NC='\033[0m'

# Log prefix
LOG_PREFIX="SEC-DROP"

# Service unit content
WATCHER_SERVICE="security-alert-watcher.service"

info()  { echo -e "${CYAN}🔷${NC} $1"; }
ok()    { echo -e "${GREEN}✅${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠️${NC} $1"; }
err()   { echo -e "${RED}❌${NC} $1"; }

# Check if running as root
check_root() {
    if [[ "$EUID" -ne 0 ]]; then
        warn "Not running as root - iptables LOG rules will be skipped"
        warn "Run with sudo for full functionality"
        SKIP_ROOT=true
    else
        SKIP_ROOT=false
    fi
}

# Check if iptables is available
check_iptables() {
    if ! command -v "$Iptables" &>/dev/null; then
        err "iptables not found. Install it first."
        exit 1
    fi
}

# Install SEC-DROP-* LOG rules for the 5 vulnerable ports
install_iptables_rules() {
    info "Installing SEC-DROP iptables LOG rules..."

    local ports=(23 2323 4840 9000 5000)
    local descriptions=(
        "Telnet"
        "Huawei Telnet"
        "Huawei TR-069"
        "Management"
        "SSH brute force"
    )

    for i in "${!ports[@]}"; do
        local port="${ports[$i]}"
        local desc="${descriptions[$i]}"

        # Add LOG rule with prefix SEC-DROP-<port>
        "$Iptables" -A INPUT -p tcp --dport "$port" -m state --state NEW -j LOG \
            --log-prefix "${LOG_PREFIX}-${port}" \
            --log-level 4

        ok "Added LOG rule: ${LOG_PREFIX}-${port} for port ${port} (${desc})"
    done
}

# Install the security-alert-watcher systemd service
install_watcher_service() {
    local service_path="${SYSTEMD_DIR}/${WATCHER_SERVICE}"

    info "Installing ${WATCHER_SERVICE} systemd service..."

    mkdir -p "$SYSTEMD_DIR"

    cat > "$service_path" << 'EOF'
[Unit]
Description=BNN Security Alert Watcher
After=network.target
Wants=network-online.target
Wants=iptables.service
Wants=nmap.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/security-alert-watcher
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

    ok "Created ${WATCHER_SERVICE} at ${service_path}"
}

# Enable and start the watcher service
enable_service() {
    info "Enabling and starting ${WATCHER_SERVICE}..."

    systemctl daemon-reload
    systemctl enable "$WATCHER_SERVICE" 2>/dev/null || true
    systemctl start "$WATCHER_SERVICE" 2>/dev/null || true

    ok "${WATCHER_SERVICE} enabled and started"
}

# Display current iptables LOG rules
show_rules() {
    info "Current SEC-DROP iptables LOG rules:"
    "$Iptables" -L INPUT -n -v --log-prefix "${LOG_PREFIX}-" 2>/dev/null || \
        warn "No SEC-DROP LOG rules found (may need to run apply_router_security.sh first)"
}

main() {
    check_root
    check_iptables

    if $SKIP_ROOT; then
        warn "Skipping iptables LOG rules (not running as root)"
        ok "Limited mode: watcher service setup only"
        info "Use 'sudo $0' for full iptables LOG rules functionality"
        echo ""
        # Still install the watcher service even without root
        install_watcher_service
        echo ""
        ok "setup_security_logging.sh completed (limited mode)"
        return 0
    fi

    info "BNN Security Logging Setup"
    info "Log prefix: ${LOG_PREFIX}-*"

    install_iptables_rules
    echo ""

    install_watcher_service
    echo ""

    enable_service
    echo ""

    show_rules

    echo ""
    ok "Security logging setup complete!"
    info "LOG rules installed with prefix: ${LOG_PREFIX}-<port>"
    info "Watcher service: ${WATCHER_SERVICE} active"
    info "Use 'apply_router_security.sh' to block the 5 vulnerable ports"
}

main "$@"