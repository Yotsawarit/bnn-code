#!/bin/bash
# ============================================
# 🔄 BNN Router Recovery Procedure
# ============================================
# Recovery procedures for compromised/restored Huawei routers.
# Restores legitimate services, validates configuration, and monitors for re-infection.
# Usage: ./scripts/router_recovery_procedure.sh [--auto] [--service SERVICE] [--help]
# ============================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
Iptables="iptables"
Nmap="nmap"

# 5 vulnerable ports that should remain blocked
VULN_PORTS=(23 2323 4840 9000 5000)

# Legitimate services that should be allowed
LEGITIMATE_SERVICES=(
    "22:SSH"
    "80:HTTP"
    "443:HTTPS"
)

# Log prefix
LOG_PREFIX="SEC-DROP"

# Colors
RED='\033[0;31m'; YELLOW='\033[1;33m'; GREEN='\033[0;32m'
CYAN='\033[0;36m'; NC='\033[0m'

# Root check flag
SKIP_ROOT=false

info()  { echo -e "${CYAN}🔷${NC} $1"; }
ok()    { echo -e "${GREEN}✅${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠️${NC} $1"; }
err()   { echo -e "${RED}❌${NC} $1"; }

# Check if running as root
check_root() {
    if [[ "$EUID" -ne 0 ]]; then
        warn "Not running as root - some operations will be limited"
        SKIP_ROOT=true
    else
        SKIP_ROOT=false
    fi
}

# Verify all vulnerable ports are blocked
verify_ports_blocked() {
    info "Verifying vulnerable ports are blocked..."

    local all_blocked=true
    for port in "${VULN_PORTS[@]}"; do
        if "$Iptables" -C INPUT -p tcp --dport "$port" -j DROP 2>/dev/null; then
            ok "Port TCP/${port}: BLOCKED"
        else
            warn "Port TCP/${port}: NOT blocked - re-applying rule"
            "$Iptables" -A INPUT -p tcp --dport "$port" -j DROP
            all_blocked=false
        fi
    done

    if $all_blocked; then
        ok "All vulnerable ports are confirmed blocked"
    else
        warn "Some ports could not be blocked"
    fi
}

# Restore legitimate services (allow SSH, HTTP, HTTPS)
restore_legitimate_services() {
    info "Restoring legitimate services..."

    for rule in "${LEGITIMATE_SERVICES[@]}"; do
        local port="${rule%%:*}"
        local name="${rule#*:}"

        # Check if rule already exists; if not, add ACCEPT rule
        if ! "$Iptables" -C INPUT -p tcp --dport "$port" -j ACCEPT 2>/dev/null; then
            "$Iptables" -A INPUT -p tcp --dport "$port" -j ACCEPT
            ok "Allowed ${name} (TCP/${port})"
        else
            ok "${name} (TCP/${port}) already allowed"
        fi
    done

    ok "Legitimate services restored"
}

# Run security scan to validate router state
validate_router_state() {
    info "Validating router security state..."

    local scan_results=""

    if command -v "$Nmap" &>/dev/null; then
        scan_results=$("$Nmap" -p "23,2323,4840,9000,5000" -T4 -F localhost 2>/dev/null || true)
        info "Port scan results:"
        echo "$scan_results" | while IFS= read -r line; do
            echo "  $line"
        done
    else
        warn "nmap not available - skipping state validation scan"
    fi

    # Check iptables rules
    info "Current iptables INPUT rules:"
    "$Iptables" -L INPUT -n --line-numbers 2>/dev/null | head -20 || warn "Could not read iptables rules"

    ok "Router state validation complete"
}

# Check for signs of re-compromise
check_recompromise() {
    info "Checking for signs of re-compromise..."

    local alert_log="/var/log/bnn-security-alerts.log"

    if [[ -f "$alert_log" ]]; then
        local recent_alerts
        recent_alerts=$(tail -n 20 "$alert_log" 2>/dev/null || true)

        if echo "$recent_alerts" | grep -qi "PORT_OPEN\|SEC-DROP"; then
            warn "Recent security alerts detected - possible re-compromise"
            echo "$recent_alerts"
        else
            ok "No recent security alerts - system appears clean"
        fi
    else
        warn "No alert log found at $alert_log"
        info "Run security_monitor.sh to generate alerts log"
    fi
}

# Generate recovery report
generate_report() {
    local report_file="/tmp/bnn_recovery_report_$(date +%Y%m%d_%H%M%S).txt"

    info "Generating recovery report: $report_file"

    {
        echo "=== BNN Router Recovery Report ==="
        echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo ""
        echo "Vulnerable Ports Status:"
        for port in "${VULN_PORTS[@]}"; do
            if "$Iptables" -C INPUT -p tcp --dport "$port" -j DROP 2>/dev/null; then
                echo "  TCP/${port}: BLOCKED"
            else
                echo "  TCP/${port}: OPEN (UNBLOCKED)"
            fi
        done
        echo ""
        echo "Legitimate Services Status:"
        for rule in "${LEGITIMATE_SERVICES[@]}"; do
            local port="${rule%%:*}"
            local name="${rule#*:}"
            if "$Iptables" -C INPUT -p tcp --dport "$port" -j ACCEPT 2>/dev/null; then
                echo "  TCP/${port} (${name}): ALLOWED"
            else
                echo "  TCP/${port} (${name}): NOT allowed"
            fi
        done
        echo ""
        echo "=== End of Report ==="
    } > "$report_file"

    ok "Report saved to $report_file"
    echo ""
    info "Review the report and verify all settings are correct"
}

# Main
main() {
    check_root

    local auto_mode=false
    local action=""

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --auto|-a)
                auto_mode=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --auto, -a          Run full recovery procedure"
                echo "  --help, -h          Show this help message"
                ;;
            *)
                err "Unknown option: $1"
                echo "Usage: $0 [OPTIONS]"
                exit 1
                ;;
        esac
    done

    info "=== BNN Router Recovery Procedure ==="
    echo ""

    # Step 1: Verify vulnerable ports are blocked
    verify_ports_blocked
    echo ""

    # Step 2: Restore legitimate services
    restore_legitimate_services
    echo ""

    # Step 3: Validate router state
    validate_router_state
    echo ""

    # Step 4: Check for re-compromise
    check_recompromise
    echo ""

    # Step 5: Generate recovery report (if not auto mode, ask)
    if ! $auto_mode; then
        generate_report
    else
        generate_report
    fi

    echo ""
    ok "Recovery procedure complete"
    info "Next steps:"
    info "  1. Review the recovery report"
    info "  2. Monitor with: ./scripts/security_monitor.sh --once"
    info "  3. Check alerts: ./scripts/router_security_daemon.sh --status"
}

main "$@"