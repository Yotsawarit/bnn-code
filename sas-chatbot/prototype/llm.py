import json
import os
import urllib.request


def _env(name, default=None):
    return os.environ.get(name, default)


def is_configured():
    return bool(_env("LLM_API_URL")) and bool(_env("LLM_API_KEY"))


def generate(system_prompt, user_message, max_tokens=512, temperature=0.7):
    url = _env("LLM_API_URL")
    api_key = _env("LLM_API_KEY")
    model = _env("LLM_MODEL", "gpt-4o-mini")
    if not url or not api_key:
        return None
    payload = {
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_message},
        ],
        "max_tokens": max_tokens,
        "temperature": temperature,
    }
    request = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {api_key}",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            body = json.loads(response.read().decode("utf-8"))
        return body["choices"][0]["message"]["content"].strip()
    except Exception:
        return None
