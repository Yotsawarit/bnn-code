#!/bin/bash
# ============================================
# 🛡️ BNN Router Security Daemon
# ============================================
# Periodic security daemon for router monitoring and alert handling.
# Runs in background, watches for security events, and triggers remediation.
# Usage: ./scripts/router_security_daemon.sh [--start|--stop|--status]
# ============================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
Iptables="iptables"
LOG_PREFIX="SEC-DROP"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; NC='\033[0m'

# Root check flag
SKIP_ROOT=false

info()  { echo -e "${CYAN}🔷${NC} $1"; }
ok()    { echo -e "${GREEN}✅${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠️${NC} $1"; }
err()   { echo -e "${RED}❌${NC} $1"; }

PID_FILE="/var/run/bnn-router-daemon.pid"
LOG_FILE="/var/log/bnn-router-security.log"

# Check if running as root
check_root() {
    if [[ "$EUID" -ne 0 ]]; then
        warn "Not running as root - daemon will run in limited mode"
        SKIP_ROOT=true
    else
        SKIP_ROOT=false
    fi
}

# Check if daemon is running
is_running() {
    if [[ -f "$PID_FILE" ]]; then
        local pid
        pid=$(cat "$PID_FILE" 2>/dev/null)
        if [[ -d "/proc/$pid" ]]; then
            return 0
        fi
    fi
    return 1
}

# Start the daemon
start_daemon() {
    if is_running; then
        warn "Daemon already running (PID: $(cat "$PID_FILE"))"
        exit 0
    fi

    if $SKIP_ROOT; then
        warn "Running daemon in limited mode (no iptables monitoring)"
        info "Start with sudo for full functionality"
    fi

    info "Starting BNN Router Security Daemon..."

    # Create log file if needed
    touch "$LOG_FILE" 2>/dev/null || true

    # Start daemon in background
    bash "$SCRIPT_DIR/security_monitor.sh" --interval 300 \
        >> "$LOG_FILE" 2>&1 &

    local pid=$!
    echo "$pid" > "$PID_FILE"

    ok "Daemon started (PID: $pid)"
    info "Log file: $LOG_FILE"
    if $SKIP_ROOT; then
        info "Scan interval: 300s (5 minutes) - limited mode"
    fi
}

# Stop the daemon
stop_daemon() {
    if ! is_running; then
        warn "Daemon not running"
        exit 0
    fi

    local pid
    pid=$(cat "$PID_FILE")
    info "Stopping BNN Router Security Daemon (PID: $pid)..."

    kill "$pid" 2>/dev/null || true
    rm -f "$PID_FILE"

    # Wait for process to stop
    local timeout=10
    while [[ $timeout -gt 0 ]]; do
        if ! is_running; then
            break
        fi
        sleep 0.5
        ((timeout--))
    done

    if is_running; then
        warn "Process still running, forcing kill..."
        kill -9 "$pid" 2>/dev/null || true
    fi

    ok "Daemon stopped"
}

# Check daemon status
status_daemon() {
    if is_running; then
        local pid
        pid=$(cat "$PID_FILE")
        ok "Daemon is running (PID: $pid)"
        info "Log file: $LOG_FILE"
        info "Last alerts (last 5):"
        tail -n 5 "$LOG_FILE" 2>/dev/null || echo "  No logs yet"
    else
        warn "Daemon is not running"
    fi
}

# Parse log alerts
parse_alerts() {
    if [[ ! -f "$LOG_FILE" ]]; then
        warn "No log file found at $LOG_FILE"
        exit 1
    fi

    info "Security alerts from $LOG_FILE:"
    grep -i "ALERT\|SEC-DROP" "$LOG_FILE" 2>/dev/null || echo "  No alerts found"
}

# Handle remediation trigger
trigger_remediation() {
    info "Triggering remediation procedure..."

    if command -v "$SCRIPT_DIR/remediate_huawei_router.sh" &>/dev/null; then
        "$SCRIPT_DIR/remediate_huawei_router.sh"
    else
        warn "remediate_huawei_router.sh not found"
    fi

    if command -v "$SCRIPT_DIR/router_recovery_procedure.sh" &>/dev/null; then
        "$SCRIPT_DIR/router_recovery_procedure.sh"
    else
        warn "router_recovery_procedure.sh not found"
    fi
}

# Main
main() {
    check_root

    case "${1:-status}" in
        --start|start)
            start_daemon
            ;;
        --stop|stop)
            stop_daemon
            ;;
        --status|status)
            status_daemon
            ;;
        --parse|parse)
            parse_alerts
            ;;
        --trigger|trigger)
            trigger_remediation
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --start, start      Start the security daemon"
            echo "  --stop, stop        Stop the security daemon"
            echo "  --status, status    Check daemon status"
            echo "  --parse, parse      Parse security alerts from log"
            echo "  --trigger, trigger  Trigger remediation procedures"
            echo "  --help, -h          Show this help message"
            ;;
        *)
            err "Unknown option: $1"
            echo "Usage: $0 [OPTIONS]"
            exit 1
            ;;
    esac
}

main "$@"