#!/bin/bash
# ============================================
# 🔍 BNN Security Monitor — Periodic nmap/TCP Scans
# ============================================
# Performs periodic nmap/TCP scans with alerting on vulnerable port detection.
# Usage: ./scripts/security_monitor.sh [--interval SECONDS] [--once]
# ============================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
Iptables="iptables"
Nmap="nmap"

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

# Default scan interval in seconds (15 minutes)
DEFAULT_INTERVAL=900

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; NC='\033[0m'

info()  { echo -e "${CYAN}🔷${NC} $1"; }
ok()    { echo -e "${GREEN}✅${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠️${NC} $1"; }
err()   { echo -e "${RED}❌${NC} $1"; }

# Check dependencies
check_deps() {
    if ! command -v "$Nmap" &>/dev/null; then
        warn "nmap not found - some features will be limited"
    fi
    if ! command -v "$Iptables" &>/dev/null; then
        warn "iptables not found - alerting may be limited"
    fi
}

# Scan a single port - check if it's open
scan_port() {
    local port="$1"
    local desc="${PORTS_DESC[$((port - 23))]:-$port}"

    local result
    if command -v "$Nmap" &>/dev/null; then
        result=$("$Nmap" -p "$port" -T4 -F -oG - "$1" 2>/dev/null | grep -oP "Ports: .*?/${port}(?=/)?" || true)
    else
        # Fallback: simple bash check using /proc or timeout
        result=$(timeout 3 bash -c "echo > /dev/tcp/localhost/$port" 2>/dev/null && echo "OPEN" || echo "CLOSED")
    fi

    if [[ "$result" == "OPEN" ]] || echo "$result" | grep -q "/${port}"; then
        echo -e "${RED}⚠️ ALERT: ${desc} is OPEN!${NC}"
        log_alert "PORT_OPEN" "$desc" "$port"
        return 1
    fi

    echo -e "${GREEN}✅ ${desc} is CLOSED${NC}"
    return 0
}

# Log security alert
log_alert() {
    local alert_type="$1"
    local description="$2"
    local port="$3"

    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)

    # Write to security log
    echo "[$timestamp] ALERT type=${alert_type} description=\"${description}\" port=${port} prefix=${LOG_PREFIX}" \
        >> "/var/log/bnn-security-alerts.log" 2>/dev/null || \
        echo "[$timestamp] ALERT type=${alert_type} description=\"${description}\" port=${port} prefix=${LOG_PREFIX}"

    # Also log via syslog if available
    logger -t "bnn-security" "ALERT: ${alert_type} - ${description} port=${port}" 2>/dev/null || true
}

# Scan all vulnerable ports
scan_all_ports() {
    info "Scanning ${#VULN_PORTS[@]} vulnerable ports..."

    local all_closed=true
    for port in "${VULN_PORTS[@]}"; do
        if ! scan_port "$port"; then
            all_closed=false
        fi
    done

    if $all_closed; then
        ok "All vulnerable ports are CLOSED - no alerts"
    fi

    return 0
}

# Show help
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --interval SECONDS  Scan interval in seconds (default: ${DEFAULT_INTERVAL}, i.e. 15 min)"
    echo "  --once              Run a single scan and exit"
    echo "  --help              Show this help message"
}

# Main
main() {
    local interval="$DEFAULT_INTERVAL"
    local once=false

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

    check_deps

    if $once; then
        info "Running single security scan..."
        scan_all_ports
        exit $?
    fi

    info "Starting BNN Security Monitor (interval: ${interval}s / $(($interval / 60))m)"
    info "Press Ctrl+C to stop"

    while true; do
        scan_all_ports
        info "Waiting ${interval}s until next scan..."
        sleep "$interval"
    done
}

main "$@"