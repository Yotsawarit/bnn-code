import json
import math
from pathlib import Path

import numpy as np

DATA_DIR = Path(__file__).resolve().parent.parent / "data"


def tokenize_thai(text):
    text = str(text).lower().strip()
    tokens = []
    for ch in text:
        if ch.isalnum():
            tokens.append(ch)
    tokens = "".join(tokens)
    if len(tokens) < 2:
        return [tokens] if tokens else []
    return [tokens[i : i + 2] for i in range(len(tokens) - 1)]


class KnowledgeBase:
    def __init__(self, kb_path=None):
        self.kb_path = Path(kb_path) if kb_path else DATA_DIR / "knowledge_base.json"
        self.faqs = []
        self.guides = []
        self._vocab = {}
        self._idf = {}
        self._doc_vectors = []
        self._doc_keys = []
        self.load()

    def load(self):
        raw = json.loads(self.kb_path.read_text(encoding="utf-8"))
        self.faqs = raw.get("faqs", [])
        self.guides = raw.get("guides", [])
        self._build_index()

    def _build_index(self):
        docs = []
        for faq in self.faqs:
            text = faq["question"] + " " + " ".join(faq.get("keywords", []))
            docs.append((("faq", faq["id"]), text))
        for guide in self.guides:
            text = guide["title"] + " " + " ".join(guide.get("keywords", []))
            docs.append((("guide", guide["id"]), text))
        vocab = set()
        for _, text in docs:
            vocab.update(tokenize_thai(text))
        self._vocab = {tok: i for i, tok in enumerate(sorted(vocab))}
        n_docs = len(docs)
        df = [0] * len(self._vocab)
        for _, text in docs:
            seen = set(tokenize_thai(text))
            for tok in seen:
                if tok in self._vocab:
                    df[self._vocab[tok]] += 1
        self._idf = {}
        for tok, idx in self._vocab.items():
            self._idf[tok] = math.log((1 + n_docs) / (1 + df[idx])) + 1
        self._doc_vectors = []
        self._doc_keys = []
        for key, text in docs:
            vec = self._vectorize(text)
            self._doc_vectors.append(vec)
            self._doc_keys.append(key)

    def _vectorize(self, text):
        vec = np.zeros(len(self._vocab), dtype=np.float64)
        tf = {}
        for tok in tokenize_thai(text):
            if tok in self._vocab:
                tf[tok] = tf.get(tok, 0) + 1
        for tok, count in tf.items():
            vec[self._vocab[tok]] = count * self._idf[tok]
        norm = np.linalg.norm(vec)
        return vec / norm if norm > 0 else vec

    def search(self, query, top_k=2):
        if not self._doc_vectors:
            return []
        qvec = self._vectorize(query)
        if not np.any(qvec):
            return []
        scores = [(self._doc_keys[i], float(np.dot(qvec, v))) for i, v in enumerate(self._doc_vectors)]
        scores.sort(key=lambda x: x[1], reverse=True)
        return [{"key": k, "score": s} for k, s in scores[:top_k]]

    def get_faq(self, faq_id):
        for faq in self.faqs:
            if faq["id"] == faq_id:
                return faq
        return None

    def get_guide(self, guide_id):
        for guide in self.guides:
            if guide["id"] == guide_id:
                return guide
        return None

    def resolve(self, search_results):
        out = []
        for item in search_results:
            kind, obj_id = item["key"]
            obj = self.get_faq(obj_id) if kind == "faq" else self.get_guide(obj_id)
            if obj is None:
                continue
            if kind == "faq":
                out.append({"kind": "faq", "id": obj_id, "question": obj["question"], "answer": obj["answer"], "score": item["score"]})
            else:
                out.append({"kind": "guide", "id": obj_id, "title": obj["title"], "steps": obj["steps"], "score": item["score"]})
        return out


class PackageCatalog:
    def __init__(self, packages_path=None):
        self.path = Path(packages_path) if packages_path else DATA_DIR / "packages.json"
        self.packages = []
        self.load()

    def load(self):
        raw = json.loads(self.path.read_text(encoding="utf-8"))
        self.packages = raw.get("packages", [])

    def get(self, package_id):
        for pkg in self.packages:
            if pkg["id"] == package_id:
                return pkg
        return None
