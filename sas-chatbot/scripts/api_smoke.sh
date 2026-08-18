#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
PORT="${PORT:-8000}"
LOG=/tmp/opencode/uvicorn.log

uvicorn prototype.main:app --host 127.0.0.1 --port "$PORT" >"$LOG" 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null || true' EXIT
sleep 3

echo "== /health =="
curl -sf -m 5 "http://127.0.0.1:$PORT/health"
echo

echo "== /api/chat (greeting) =="
curl -sf -m 5 -X POST "http://127.0.0.1:$PORT/api/chat" \
  -H 'Content-Type: application/json' \
  -d '{"message":"สวัสดี"}'
echo

echo "== /api/chat (troubleshoot) =="
curl -sf -m 5 -X POST "http://127.0.0.1:$PORT/api/chat" \
  -H 'Content-Type: application/json' \
  -d '{"message":"เน็ตช้ามาก ทำยังไงดี"}'
echo

echo "== /api/chat (recommend full flow) =="
SID=$(curl -sf -m 5 -X POST "http://127.0.0.1:$PORT/api/chat" \
  -H 'Content-Type: application/json' \
  -d '{"message":"แนะนำแพ็กเกจที่เหมาะกับฉัน"}' | python3 -c 'import json,sys;print(json.load(sys.stdin)["session_id"])')
for msg in 30 500 ส่วนตัว 1; do
  curl -sf -m 5 -X POST "http://127.0.0.1:$PORT/api/chat" \
    -H 'Content-Type: application/json' \
    -d "{\"message\":\"$msg\",\"session_id\":\"$SID\"}" | python3 -c "import json,sys;d=json.load(sys.stdin);print('['+d['intent']+']', d['reply'].splitlines()[0])"
done

echo "== /api/faq/search =="
curl -sf -m 5 -X POST "http://127.0.0.1:$PORT/api/faq/search" \
  -H 'Content-Type: application/json' \
  -d '{"query":"ลืมจ่ายบิล","top_k":1}'
echo

echo "== /api/recommend =="
curl -sf -m 5 -X POST "http://127.0.0.1:$PORT/api/recommend" \
  -H 'Content-Type: application/json' \
  -d '{"usage_gb":80,"budget_max":1200,"family_members":1,"needs":["สตรีมมิง"]}'
echo

echo "== /api/voice =="
curl -sf -m 5 -X POST "http://127.0.0.1:$PORT/api/voice" \
  -H 'Content-Type: application/json' \
  -d '{"transcript":"ตรวจสอบบิลค่าบริการ"}'
echo

echo "== /api/packages (count) =="
curl -sf -m 5 "http://127.0.0.1:$PORT/api/packages" | python3 -c "import json,sys;print(len(json.load(sys.stdin)['packages']),'packages')"

echo "OK"