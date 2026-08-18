#!/bin/bash
# ============================================
# 📊 BNN Router Monitor — Periodic Status Check
# ============================================
# Periodic router security status monitor with alerting.
# Usage: ./scripts/router_monitor.sh [--interval SECONDS] [--once]
# ============================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
Iptables="iptables"

# 5 vulnerable ports to monitor
VULN_PORTS=(23 2323 4840 9000 5000)
PORTS_DESC=(
    "Telnet (TCP/23)"
    "Huawei Telnet (TCP/2323)"
    "Huawei TR-069 (TCP/4840)"
    "Management (TCP/9000)"
    "SSH brute force (TCP/5000)"
)

# Log prefix
LOG_PREFIX="SEC-DROP"

# Default scan interval
DEFAULT_INTERVAL=60

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; NC='\033[0m'

# Root check flag
SKIP_ROOT=false

info()  { echo -e "${CYAN}🔷${NC} $1"; }
ok()    { echo -e "${GREEN}✅${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠️${NC} $1"; }
err()   { echo -e "${RED}❌${NC} $1"; }

# Check if running as root
check_root() {
    if [[ "$EUID" -ne 0 ]]; then
        warn "Not running as root - iptables checks will be skipped"
        SKIP_ROOT=true
    else
        SKIP_ROOT=false
    fi
}

# Check iptables rules for a port
check_iptables_rule() {
    local port="$1"
    "$Iptables" -C INPUT -p tcp --dport "$port" -j DROP 2>/dev/null && return 0 || return 1
}

# Check if port is open locally
check_port_open() {
    local port="$1"
    local result
    if command -v nmap &>/dev/null; then
        result=$("nmap" -p "$port" -T4 -F -oG - localhost 2>/dev/null | grep -oP "Ports: .*?/${port}(?=/)?" || true)
    else
        result=$(timeout 2 bash -c "echo > /dev/tcp/localhost/$port" 2>/dev/null && echo "OPEN" || echo "CLOSED")
    fi
    [[ "$result" == "OPEN" ]] || echo "$result" | grep -q "/${port}"
}

# Show status of all vulnerable ports
show_status() {
    info "BNN Router Security Status"
    echo "─────────────────────────────────────"
    local all_secure=true

    for port in "${VULN_PORTS[@]}"; do
        local desc="${PORTS_DESC[$((port - 23))]:-$port}"

        local iptables_ok=false
        local port_open=false

        if check_iptables_rule "$port"; then
            iptables_ok=true
            echo -e "${GREEN}✅${NC} ${desc}: iptables DROP rule active"
        else
            echo -e "${RED}❌${NC} ${desc}: iptables DROP rule NOT active"
            all_secure=false
        fi

        if check_port_open "$port"; then
            port_open=true
            echo -e "${RED}⚠️${NC} ${desc}: PORT IS OPEN!"
            all_secure=false
        else
            echo -e "${GREEN}✅${NC} ${desc}: port is CLOSED"
        fi
    done

    echo "─────────────────────────────────────"
    if $all_secure; then
        ok "All vulnerable ports are secure"
    else
        warn "Some vulnerable ports need attention"
    fi
}

# Block all vulnerable ports via iptables
block_all_ports() {
    info "Blocking all 5 vulnerable ports via iptables..."

    for port in "${VULN_PORTS[@]}"; do
        "$Iptables" -A INPUT -p tcp --dport "$port" -j DROP
        ok "Blocked TCP/${port}"
    done

    ok "All vulnerable ports blocked"
}

# Unblock all vulnerable ports (for recovery)
unblock_all_ports() {
    info "Unblocking all 5 vulnerable ports via iptables..."

    for port in "${VULN_PORTS[@]}"; do
        "$Iptables" -D INPUT -p tcp --dport "$port" -j DROP
        ok "Unblocked TCP/${port}"
    done

    ok "All vulnerable ports unblocked"
}

# Show help
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --interval SECONDS  Check interval in seconds (default: ${DEFAULT_INTERVAL}s)"
    echo "  --once              Run once and exit"
    echo "  --block             Block all vulnerable ports"
    echo "  --unblock           Unblock all vulnerable ports"
    echo "  --help              Show this help message"
}

# Main
main() {
    check_root
    local interval="$DEFAULT_INTERVAL"
    local once=false
    local block=false
    local unblock=false

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --interval)
                interval="$2"
                shift 2
                ;;
            --once)
                once=true
                shift
                ;;
            --block)
                block=true
                shift
                ;;
            --unblock)
                unblock=true
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                err "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    if $block; then
        block_all_ports
        exit 0
    fi

    if $unblock; then
        unblock_all_ports
        exit 0
    fi

    if $once; then
        info "Running router security status check..."
        show_status
        exit $?
    fi

    info "Starting BNN Router Monitor (interval: ${interval}s)"
    info "Press Ctrl+C to stop"

    while true; do
        show_status
        info "Waiting ${interval}s until next check..."
        sleep "$interval"
    done
}

main "$@"