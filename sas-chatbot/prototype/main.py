import base64
import uuid

from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel

from . import llm as llm_module
from .engine import CustomerServiceEngine
from .rag import KnowledgeBase, PackageCatalog
from .voice import VoiceGateway, VoiceUnavailableError

app = FastAPI(
    title="SAS Assist API",
    description="Customer Service chatbot for SAS Thailand (Generative AI + voice)",
    version="0.1.0",
)

kb = KnowledgeBase()
catalog = PackageCatalog()
gateway = VoiceGateway()
engine = CustomerServiceEngine(kb=kb, catalog=catalog, gateway=gateway)


class ChatRequest(BaseModel):
    message: str
    session_id: str | None = None
    customer_id: str | None = None


class RecommendRequest(BaseModel):
    usage_gb: int | None = None
    budget_max: int | None = None
    family_members: int = 1
    needs: list[str] = []


class FaqSearchRequest(BaseModel):
    query: str
    top_k: int = 2


class VoiceRequest(BaseModel):
    transcript: str | None = None


@app.get("/")
def root():
    return {"service": "SAS Assist", "provider": "SAS Thailand", "version": "0.1.0"}


@app.get("/health")
def health():
    return {"status": "ok", "llm_configured": llm_module.is_configured(), "stt_configured": gateway.stt_configured, "tts_configured": gateway.tts_configured}


@app.post("/api/chat")
def chat(request: ChatRequest):
    session_id = request.session_id or uuid.uuid4().hex[:12]
    result = engine.handle(request.message, session_id=session_id)
    result["customer_id"] = request.customer_id
    return result


@app.post("/api/faq/search")
def faq_search(request: FaqSearchRequest):
    results = kb.search(request.query, top_k=request.top_k)
    return {"results": kb.resolve(results)}


@app.post("/api/recommend")
def recommend(request: RecommendRequest):
    profile = {
        "usage_gb": request.usage_gb,
        "budget_max": request.budget_max,
        "family_members": request.family_members,
        "needs": request.needs,
    }
    results = engine.recommender.recommend(profile)
    return {
        "profile": profile,
        "packages": [
            {
                "id": item["package"]["id"],
                "name": item["package"]["name"],
                "price": item["package"]["price"],
                "benefits": item["package"]["benefits"],
                "score": item["score"],
            }
            for item in results
        ],
    }


@app.get("/api/packages")
def packages():
    return {"packages": catalog.packages}


@app.post("/api/voice")
def voice(request: VoiceRequest):
    transcript = request.transcript
    return _voice_reply(transcript, "voice.json")


@app.post("/api/voice/audio")
async def voice_audio(request: Request):
    audio_bytes = await request.body()
    if not audio_bytes:
        return JSONResponse(status_code=400, content={"error": "ต้องส่ง audio payload"})
    try:
        transcript = gateway.transcribe(audio_bytes, filename="voice.webm")
    except VoiceUnavailableError as exc:
        return JSONResponse(status_code=503, content={"error": str(exc)})
    return _voice_reply(transcript, "voice.webm")


def _voice_reply(transcript, source):
    if not transcript:
        return JSONResponse(status_code=400, content={"error": "ต้องระบุ transcript หรือ audio"})
    session_id = uuid.uuid4().hex[:12]
    result = engine.handle(transcript, session_id=session_id)
    tts_audio = None
    if gateway.tts_configured and result["reply"]:
        tts_audio = gateway.synthesize(result["reply"])
    payload = {
        "session_id": session_id,
        "source": source,
        "transcript": transcript,
        "reply": result["reply"],
        "intent": result["intent"],
        "tts_text": result["reply"],
    }
    if tts_audio:
        payload["audio_base64"] = base64.b64encode(tts_audio).decode()
    return payload