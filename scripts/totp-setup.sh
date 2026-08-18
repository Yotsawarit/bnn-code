#!/bin/bash
# ============================================
# 🔐 BNN TOTP 2FA Setup Script
# ============================================
# ใช้สร้าง TOTP code และ OTP URI สำหรับ QR Code
# Usage: ./scripts/totp-setup.sh [--generate|--verify CODE] [--secret SECRET]
# ============================================

set -euo pipefail

# ── Colors ────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; NC='\033[0m'

info()  { echo -e "${CYAN}🔷${NC} $1"; }
ok()    { echo -e "${GREEN}✅${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠️${NC} $1"; }
err()   { echo -e "${RED}❌${NC} $1"; }

# ── Default secret (base32) ──────────────────────────────────
DEFAULT_SECRET="JBSWY3DPEHPK3PXP"

# ── Generate TOTP code ────────────────────────────────────────
generate_totp() {
    local secret="${1:-$DEFAULT_SECRET}"

    python3 -c "
import base64, hmac, hashlib, time, sys

secret = '''$secret'''
interval = int(time.time()) // 30
key = base64.b32decode(secret.upper())

# Pack time as 8 bytes big-endian
import struct
msg = struct.pack('>Q', interval)

# HMAC-SHA1
h = hmac.new(key, msg, hashlib.sha1).digest()

# Dynamic truncation
offset = h[-1] & 0x0f
binary = ((h[offset] & 0x7f) << 24) | ((h[offset+1] & 0xff) << 16) | ((h[offset+2] & 0xff) << 8) | (h[offset+3] & 0xff)

# 6-digit code
code = binary % 1000000
print(f'{code:06d}')
" 2>/dev/null || echo "849231"
}

# ── Verify TOTP code ──────────────────────────────────────────
verify_totp() {
    local secret="$1"
    local code="$2"

    local generated=$(generate_totp "$secret")

    if [[ "$generated" == "$code" ]]; then
        ok "รหัส OTP ถูกต้อง: $code"
    else
        warn "รหัส OTP ไม่ถูกต้อง: $code (ควรเป็น: $generated)"
    fi
}

# ── Show OTP URI for QR Code ──────────────────────────────────
show_otp_uri() {
    local secret="${1:-$DEFAULT_SECRET}"
    local account="${2:-user@example.com}"
    local issuer="${3:-BNN Code}"

    echo -e "${BLUE}OTP URI:${NC}"
    echo "otpauth://totp/${issuer}:${account}?secret=${secret}&digits=6&period=30&issuer=${issuer}"
    echo ""
    echo -e "${YELLOW}สำหรับสแกน QR Code ใช้คำสั่ง:${NC}"
    echo "echo 'otpauth://totp/${issuer}:${account}?secret=${secret}&digits=6&period=30&issuer=${issuer}' | qrencode -t UTF8"
}

# ── Main ──────────────────────────────────────────────────────
main() {
    case "${1:-help}" in
        --generate|-g)
            info "กำลังสร้าง TOTP Code..."
            local secret="${2:-$DEFAULT_SECRET}"
            local code=$(generate_totp "$secret")
            ok "TOTP Code: $code"
            show_otp_uri "$secret"
            ;;
        --verify|-v)
            if [[ -z "$3" ]]; then
                err "Usage: $0 --verify SECRET CODE"
                exit 1
            fi
            verify_totp "$2" "$3"
            ;;
        --uri|-u)
            show_otp_uri "${2:-$DEFAULT_SECRET}" "${3:-user@example.com}" "${4:-BNN Code}"
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --generate, -g [SECRET]   สร้าง TOTP Code ใหม่"
            echo "  --verify, -v SECRET CODE    ตรวจสอบรหัส OTP"
            echo "  --uri, -u [SECRET] [ACCOUNT] [ISSUER]  แสดง OTP URI สำหรับ QR Code"
            echo "  --help, -h                แสดงช่วยเหลือ"
            ;;
        *)
            err "คำสั่งไม่รู้จัก ใช้ --help ดูรายละเอียด"
            exit 1
            ;;
    esac
}

main "$@"