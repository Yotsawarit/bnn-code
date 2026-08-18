import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from prototype.engine import CustomerServiceEngine
from prototype.rag import KnowledgeBase, PackageCatalog


class EngineTestCase(unittest.TestCase):
    def setUp(self):
        self.engine = CustomerServiceEngine()

    def test_greeting(self):
        result = self.engine.handle("สวัสดี", session_id="t1")
        self.assertEqual(result["intent"], "greeting")

    def test_bill_intent(self):
        result = self.engine.handle("ตรวจสอบบิลค่าบริการ", session_id="t2")
        self.assertEqual(result["intent"], "bill")
        self.assertIn("Auto Pay", result["reply"])

    def test_troubleshoot_returns_steps(self):
        result = self.engine.handle("เน็ตช้ามาก ทำยังไงดี", session_id="t3")
        self.assertEqual(result["intent"], "troubleshoot")
        self.assertIn("รีสตาร์ท", result["reply"])

    def test_handoff(self):
        result = self.engine.handle("ขอคุยกับเจ้าหน้าที่", session_id="t4")
        self.assertEqual(result["intent"], "handoff")
        self.assertIn("1212", result["reply"])

    def test_faq_answer(self):
        result = self.engine.handle("ชำระเงินค่าบริการช่องทางไหนบ้าง", session_id="t5")
        self.assertIn("พร้อมเพย์", result["reply"])

    def test_recommend_full_flow(self):
        r1 = self.engine.handle("แนะนำแพ็กเกจที่เหมาะกับฉัน", session_id="t6")
        self.assertEqual(r1["intent"], "recommend")
        r2 = self.engine.handle("30", session_id="t6")
        r3 = self.engine.handle("500", session_id="t6")
        r4 = self.engine.handle("ส่วนตัว", session_id="t6")
        self.assertEqual(r4["intent"], "recommend")
        self.assertIn("SAS", r4["reply"])
        self.assertTrue(r4["data"]["packages"])

    def test_package_selection(self):
        r1 = self.engine.handle("แนะนำแพ็กเกจ", session_id="t7")
        self.engine.handle("30", session_id="t7")
        self.engine.handle("500", session_id="t7")
        r4 = self.engine.handle("ส่วนตัว", session_id="t7")
        self.assertTrue(r4["data"]["packages"])
        r5 = self.engine.handle("1", session_id="t7")
        self.assertIn("บาท/เดือน", r5["reply"])

    def test_fallback(self):
        result = self.engine.handle("zzzzzzzzzzzzzzzz", session_id="t8")
        self.assertEqual(result["intent"], "fallback")

    def test_recommend_api_shape(self):
        profile = {"usage_gb": 50, "budget_max": 700, "family_members": 3, "needs": ["ครอบครัว"]}
        results = self.engine.recommender.recommend(profile, top_k=3)
        self.assertEqual(len(results), 3)
        self.assertGreaterEqual(results[0]["score"], 0)


class RagTestCase(unittest.TestCase):
    def test_kb_search_finds_faq(self):
        kb = KnowledgeBase()
        hits = kb.search("ชำระเงินยังไง")
        self.assertTrue(hits)
        top = kb.resolve(hits[:1])
        self.assertTrue(top)
        self.assertIn("พร้อมเพย์", top[0]["answer"])

    def test_package_catalog(self):
        catalog = PackageCatalog()
        self.assertGreaterEqual(len(catalog.packages), 5)


if __name__ == "__main__":
    unittest.main(verbosity=2)