import json
import sys
import urllib.request

DEFAULT_URL = "http://127.0.0.1:8000"


def post(url, payload):
    request = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.loads(response.read().decode("utf-8"))


def main():
    base = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_URL
    session_id = None
    print("SAS Assist demo — พิมพ์ 'exit' เพื่อออก")
    print("-" * 60)
    while True:
        try:
            message = input("คุณ: ").strip()
        except (EOFError, KeyboardInterrupt):
            print()
            break
        if not message:
            continue
        if message.lower() in ("exit", "ออก"):
            break
        payload = {"message": message}
        if session_id:
            payload["session_id"] = session_id
        result = post(f"{base}/api/chat", payload)
        session_id = result["session_id"]
        print(f"SAS Assist [{result['intent']}]:")
        print(result["reply"])
        print("-" * 60)


if __name__ == "__main__":
    main()