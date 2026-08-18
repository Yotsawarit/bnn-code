import json
import os
import urllib.request


def _env(name, default=None):
    return os.environ.get(name, default)


class VoiceUnavailableError(Exception):
    pass


class VoiceGateway:
    def __init__(self):
        self.stt_url = _env("STT_URL")
        self.stt_key = _env("STT_KEY", _env("LLM_API_KEY"))
        self.stt_model = _env("STT_MODEL", "whisper-1")
        self.tts_url = _env("TTS_URL")

    @property
    def stt_configured(self):
        return bool(self.stt_url) and bool(self.stt_key)

    @property
    def tts_configured(self):
        return bool(self.tts_url)

    def transcribe(self, audio_bytes, filename="voice.webm"):
        if not self.stt_configured:
            raise VoiceUnavailableError("STT ไม่ได้ตั้งค่า ใช้ transcript จากฝั่งผู้เรียกแทน")
        boundary = "----voice-boundary"
        body = self._multipart_body(boundary, filename, audio_bytes)
        request = urllib.request.Request(
            self.stt_url,
            data=body,
            headers={
                "Content-Type": f"multipart/form-data; boundary={boundary}",
                "Authorization": f"Bearer {self.stt_key}",
            },
            method="POST",
        )
        with urllib.request.urlopen(request, timeout=60) as response:
            data = json.loads(response.read().decode("utf-8"))
        return data.get("text", "").strip()

    def synthesize(self, text, voice="th-TH-Premium"):
        if not self.tts_configured:
            return None
        payload = {"text": text, "voice": voice, "format": "audio/wav"}
        request = urllib.request.Request(
            self.tts_url,
            data=json.dumps(payload).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(request, timeout=60) as response:
            return response.read()

    def _multipart_body(self, boundary, filename, audio_bytes):
        parts = []
        parts.append(f"--{boundary}".encode())
        parts.append(f'Content-Disposition: form-data; name="file"; filename="{filename}"'.encode())
        parts.append(b"Content-Type: application/octet-stream")
        parts.append(b"")
        parts.append(audio_bytes)
        parts.append(f"--{boundary}".encode())
        parts.append(b'Content-Disposition: form-data; name="model"')
        parts.append(b"")
        parts.append(self.stt_model.encode())
        parts.append(f"--{boundary}--".encode())
        parts.append(b"")
        return b"\r\n".join(parts)
