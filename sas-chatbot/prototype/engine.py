import re
from dataclasses import dataclass, field

from . import llm as llm_module
from .rag import KnowledgeBase, PackageCatalog
from .recommender import PackageRecommender


@dataclass
class Session:
    session_id: str
    profile: dict = field(default_factory=dict)
    profile_state: str = "idle"
    pending_recommendations: list = field(default_factory=list)
    history: list = field(default_factory=list)


SYSTEM_PROMPT = (
    "คุณคือ 'SAS Assist' ผู้ช่วยบริการลูกค้าของ SAS Thailand "
    "ตอบเป็นภาษาไทย กระชับ เป็นมิตร ให้ข้อมูลตามเอกสารอ้างอิงที่ให้มาเท่านั้น "
    "หากไม่มีข้อมูลในเอกสาร ให้แนะนำให้ลูกค้าติดต่อเจ้าหน้าที่สายด่วน 1212 แทนการเดา"
)

GREETINGS = [
    "สวัสดีค่ะ คุณต้องการให้ช่วยอะไรวันนี้บ้างคะ?\n"
    "สามารถพิมพ์เลือกได้เลย เช่น:\n"
    "1. แนะนำแพ็กเกจที่เหมาะกับฉัน\n"
    "2. ถามข้อมูล/โปรโมชัน\n"
    "3. แจ้งปัญหา เช่น เน็ตช้า\n"
    "4. ตรวจสอบบิล/ชำระเงิน\n"
    "5. ขอคุยกับเจ้าหน้าที่",
]

THANKS_REPLY = "ด้วยความยินดีค่ะ ถ้ามีอะไรเพิ่มเติม พิมพ์ถามได้ตลอด 24 ชั่วโมงนะคะ 🙏"
BYE_REPLY = "ขอบคุณที่ใช้บริการ SAS Thailand ค่ะ มีเรื่องอะไรเพิ่มเติมกลับมาพิมพ์หาเราได้เสมอ สวัสดีค่ะ 👋"
HANDOFF_REPLY = (
    "รับทราบค่ะ กำลังส่งเรื่องให้เจ้าหน้าที่ดูแลคุณโดยเฉพาะนะคะ\n"
    "ระหว่างนี้คุณสามารถติดต่อได้โดยตรงที่:\n"
    "• สายด่วน 1212 (24 ชม.)\n"
    "• แชต Facebook Messenger / แอป SAS Go\n"
    "• อีเมล care@sas-thailand.co.th\n"
    "หากต้องการให้เจ้าหน้าที่ติดต่อกลับ โปรดแจ้งเบอร์โทรศัพท์ของคุณด้วยค่ะ"
)
BILL_REPLY = (
    "สามารถตรวจสอบยอดค่าบริการได้ผ่านแอป SAS Go > 'จ่ายบิล' หรือกด *123# แล้วเลือกเมนู 'ยอดค่าใช้จ่าย' ค่ะ\n"
    "ช่องทางชำระเงิน: แอป SAS Go, ธนาคารออนไลน์/พร้อมเพย์, ตัวแทนรับชำระรายเดือน หรือเปิด Auto Pay เพื่อหักบัญชีอัตโนมัติทุกวันที่ 15\n"
    "หมายเหตุ: หากค้างชำระเกิน 15 วัน บริการจะถูกงดชั่วคราว และจะเปิดคืนภายใน 30 นาทีหลังชำระสำเร็จค่ะ"
)
ACCOUNT_REPLY = (
    "เรื่องการจัดการบัญชี ระบบช่วยได้หลายอย่างค่ะ:\n"
    "• เปลี่ยนแพ็กเกจ — ผ่านแอป SAS Go > 'จัดการแพ็กเกจ' (เปลี่ยนได้ทุกเดือน ไม่มีค่าธรรมเนียม)\n"
    "• สมัครแพ็กเกจใหม่ — แอป SAS Go, *123# หรือร้าน SAS (พกบัตรประชาชน)\n"
    "• ยกเลิก/ย้ายค่าย — แอป SAS Go หรือสายด่วน 1212\n"
    "• ลืมรหัสผ่าน — กด 'ลืมรหัสผ่าน' แล้วยืนยันด้วย OTP ทาง SMS\n"
    "ต้องการให้ช่วยเรื่องไหน พิมพ์ระบุได้เลยนะคะ"
)
FALLBACK_MSG = (
    "ขออภัยที่ตอบไม่ชัดเจนนะคะ 😅 เพื่อให้ช่วยคุณได้เร็วขึ้น เลือกหัวข้อด้านล่างได้เลย:\n"
    "1. แนะนำแพ็กเกจที่เหมาะกับฉัน\n"
    "2. ตรวจสอบบิล / ชำระเงิน\n"
    "3. แจ้งปัญหา (เน็ตช้า, ใช้ไม่ได้)\n"
    "4. ข้อมูลบัญชี (สมัคร/เปลี่ยน/ยกเลิก)\n"
    "5. ขอคุยกับเจ้าหน้าที่\n"
    "หรือพิมพ์คำถามของคุณมาได้เลยค่ะ"
)

RECOMMEND_QUESTIONS = {
    "ask_usage": "เพื่อแนะนำแพ็กเกจที่ตรงใจ เริ่มจาก \"การใช้งานเน็ตเฉลี่ยต่อเดือนกี่ GB คะ?\" เช่น พิมพ์ 30",
    "ask_budget": "รับทราบค่ะ แล้ว \"งบประมาณต่อเดือนที่ยินดีจ่ายสูงสุดเท่าไหร่คะ?\" เช่น พิมพ์ 500 (บาท)",
    "ask_family": "สุดท้ายนี้ \"ใช้งานคนเดียว หรือหลายคน/ครอบครัว/ธุรกิจ คะ?\" ตอบเช่น 'ส่วนตัว' 'ครอบครัว' 'ธุรกิจ' หรือพิมพ์จำนวนคน",
}

NUMBER_RE = re.compile(r"\d+")
FAMILY_KEYS = {
    "ครอบครัว": "ครอบครัว",
    "ธุรกิจ": "ธุรกิจ",
    "บริษัท": "ธุรกิจ",
    "ส่วนตัว": "ส่วนตัว",
    "คนเดียว": "ส่วนตัว",
}


class CustomerServiceEngine:
    def __init__(self, kb=None, catalog=None, gateway=None):
        self.kb = kb or KnowledgeBase()
        self.catalog = catalog or PackageCatalog()
        self.recommender = PackageRecommender(self.catalog)
        self.gateway = gateway
        self.sessions = {}

    def get_session(self, session_id):
        if session_id not in self.sessions:
            self.sessions[session_id] = Session(session_id)
        return self.sessions[session_id]

    def classify_intent(self, text):
        t = str(text).lower().strip()
        if any(k in t for k in ["สวัสดี", "hello", " hi", "หวัดดี", "สวัสดีครับ", "good morning", "อรุณสวัสดิ์"]):
            return "greeting"
        if any(k in t for k in ["ขอบคุณ", "ขอบใจ", "thank", "bye", "ลาก่อน", "บ้ายบาย"]):
            return "thanks"
        if any(k in t for k in ["เจ้าหน้าที่", "พนักงาน", "คนคุย", "operator", "ขอคุย", "ติดต่อ", "สายด่วน", "human"]):
            return "handoff"
        if any(k in t for k in ["เน็ตช้า", "ช้า", "ไม่ได้", "ใช้ไม่ได้", "หลุด", "สัญญาณ", "ปัญหา", "ซ่อม", "ทำไง", "แก้อะไร", "ต่อเน็ต", "buffering", "โทรไม่ออก"]):
            return "troubleshoot"
        if any(k in t for k in ["บิล", "ยอด", "ค่าบริการ", "ค้าง", "ชำระ", "จ่ายเงิน", "จ่าย", "autopay", "auto pay"]):
            return "bill"
        if any(k in t for k in ["เปลี่ยนแพ็กเกจ", "สมัคร", "ยกเลิก", "ย้ายค่าย", "ลืมรหัส", "รหัสผ่าน", "แก้ไขข้อมูล", "ข้อมูลส่วนตัว", "เลิกใช้"]):
            return "account"
        if any(k in t for k in ["แนะนำแพ็กเกจ", "แพ็กเกจไหน", "แพ็กเกจที่เหมาะ", "เสนอแพ็กเกจ", "เหมาะกับ", "package", "โปรโมชัน", " โปร", "แพ็กเกจ"]):
            return "recommend"
        return "fallback"

    def handle(self, message, session_id="default"):
        session = self.get_session(session_id)
        session.history.append({"role": "user", "content": message})
        text = str(message).strip()

        if session.profile_state != "idle":
            result = self._collect_profile(text, session)
            reply = result["reply"] if result["complete"] else result["reply"]
            return self._out(session, result["intent"], reply, data=result.get("data"))

        number = NUMBER_RE.fullmatch(text)
        if number and session.pending_recommendations:
            return self._package_detail(session, int(number.group()))

        intent = self.classify_intent(text)
        reply = None
        data = None

        if intent == "greeting":
            reply = GREETINGS[0]
        elif intent == "thanks":
            reply = THANKS_REPLY
        elif intent == "handoff":
            reply = HANDOFF_REPLY
        elif intent == "bill":
            reply = BILL_REPLY
        elif intent == "account":
            reply = ACCOUNT_REPLY
        elif intent == "recommend":
            return self._start_recommendation(session)
        elif intent == "troubleshoot":
            reply, data = self._troubleshoot(text)
        else:
            reply, data = self._faq_lookup(text)

        session.history.append({"role": "assistant", "content": reply})
        return self._out(session, intent, reply, data=data)

    def _out(self, session, intent, reply, data=None):
        return {
            "session_id": session.session_id,
            "intent": intent,
            "reply": reply,
            "data": data,
            "speech": self._speech_suffix(reply),
        }

    def _speech_suffix(self, reply):
        return None

    def _start_recommendation(self, session):
        session.pending_recommendations = []
        if "usage_gb" in session.profile and "budget_max" in session.profile:
            return self._run_recommendation(session)
        session.profile_state = "ask_usage"
        return self._out(session, "recommend", RECOMMEND_QUESTIONS["ask_usage"])

    def _collect_profile(self, text, session):
        state = session.profile_state
        if state == "ask_usage":
            m = NUMBER_RE.search(text)
            if not m:
                return {"intent": "recommend", "reply": "ขออภัยค่ะ ไม่พบตัวเลข ตัวอย่างเช่น พิมพ์ 30 (GB) ค่ะ", "complete": False}
            session.profile["usage_gb"] = int(m.group())
            session.profile_state = "ask_budget"
            return {"intent": "recommend", "reply": RECOMMEND_QUESTIONS["ask_budget"], "complete": False}
        if state == "ask_budget":
            m = NUMBER_RE.search(text)
            if not m:
                return {"intent": "recommend", "reply": "ขออภัยค่ะ ไม่พบตัวเลข ตัวอย่างเช่น พิมพ์ 500 (บาท) ค่ะ", "complete": False}
            session.profile["budget_max"] = int(m.group())
            session.profile_state = "ask_family"
            return {"intent": "recommend", "reply": RECOMMEND_QUESTIONS["ask_family"], "complete": False}
        if state == "ask_family":
            lowered = text.lower()
            members = 1
            need_tag = None
            for key, tag in FAMILY_KEYS.items():
                if key in lowered:
                    need_tag = tag
                    break
            m = NUMBER_RE.search(text)
            if m:
                members = int(m.group())
                if members > 1:
                    need_tag = "ครอบครัว"
            session.profile["family_members"] = members
            session.profile["needs"] = [need_tag] if need_tag else []
            session.profile_state = "idle"
            return self._run_recommendation(session)
        return {"intent": "recommend", "reply": FALLBACK_MSG, "complete": False}

    def _run_recommendation(self, session):
        profile = session.profile
        results = self.recommender.recommend(profile)
        session.pending_recommendations = [item["package"]["id"] for item in results]
        reply = self.recommender.build_reply(profile, results)
        session.history.append({"role": "assistant", "content": reply})
        return {
            "intent": "recommend",
            "reply": reply,
            "complete": True,
            "data": {
                "profile": profile,
                "packages": [
                    {"id": item["package"]["id"], "name": item["package"]["name"], "price": item["package"]["price"], "score": item["score"]}
                    for item in results
                ],
            },
        }

    def _package_detail(self, session, choice):
        ids = session.pending_recommendations
        if choice < 1 or choice > len(ids):
            return self._out(session, "recommend", "กรุณาเลือกหมายเลขที่ถูกต้อง (1-{}) ค่ะ".format(len(ids)))
        pkg = self.catalog.get(ids[choice - 1])
        if not pkg:
            return self._out(session, "recommend", FALLBACK_MSG)
        shared = f"\n• {pkg.get('shared')}" if pkg.get("shared") else ""
        benefits = "\n".join(f"• {b}" for b in pkg.get("benefits", []))
        reply = (
            f"รายละเอียด {pkg['name']} ค่ะ\n"
            f"ราคา {pkg['price']:,} บาท/เดือน{shared}\n"
            f"สิทธิประโยชน์:\n{benefits}\n\n"
            "ต้องการเปลี่ยนแพ็กเกจเป็นรายการนี้เลยไหมคะ? พิมพ์ 'เปลี่ยนแพ็กเกจ' เพื่อดำเนินการ"
        )
        session.history.append({"role": "assistant", "content": reply})
        return self._out(session, "recommend", reply, data={"package": pkg})

    def _troubleshoot(self, text):
        results = self.kb.search(text, top_k=3)
        for item in results:
            if item["key"][0] == "guide" and item["score"] >= 0.15:
                guide = self.kb.get_guide(item["key"][1])
                steps = "\n".join(f"{i}) {s}" for i, s in enumerate(guide["steps"], start=1))
                reply = (
                    f"รับทราบค่ะ เกี่ยวกับ \"{guide['title']}\" นี่คือขั้นตอนการแก้ปัญหาเบื้องต้น:\n"
                    f"{steps}\n\n"
                    f"หากทำครบแล้วยังไม่หาย ให้{guide['escalate']}ผ่านแอป SAS Go หรือโทร 1212 ค่ะ"
                )
                return reply, {"guide_id": guide["id"], "escalate": guide["escalate"]}
        resolved = self.kb.resolve(results)
        if resolved and resolved[0]["score"] >= 0.15:
            return resolved[0]["answer"], {"source": resolved[0]["id"]}
        return FALLBACK_MSG, None

    def _faq_lookup(self, text):
        results = self.kb.search(text, top_k=2)
        resolved = self.kb.resolve(results)
        if not resolved or resolved[0]["score"] < 0.15:
            if llm_module.is_configured():
                generated = llm_module.generate(SYSTEM_PROMPT, text)
                if generated:
                    return generated, {"source": "llm"}
            return FALLBACK_MSG, None
        best = resolved[0]
        reply = best["answer"]
        if llm_module.is_configured():
            grounded = llm_module.generate(
                SYSTEM_PROMPT,
                f"เอกสารอ้างอิง:\n{best['answer']}\n\nคำถามลูกค้า: {text}\n\nตอบโดยใช้ข้อมูลจากเอกสารอ้างอิงเท่านั้น",
            )
            if grounded:
                reply = grounded
        return reply, {"source": best["id"], "score": best["score"]}
