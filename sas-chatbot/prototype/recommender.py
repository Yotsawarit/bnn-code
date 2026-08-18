class PackageRecommender:
    def __init__(self, catalog):
        self.catalog = catalog

    def _clamp(self, value, lo, hi):
        return max(lo, min(hi, value))

    def _usage_score(self, pkg, usage_gb):
        profile = pkg.get("ideal_profile", {})
        lo = profile.get("min_usage_gb", 0)
        hi = profile.get("max_usage_gb", 99999)
        if usage_gb is None:
            return 0.5
        mid = (lo + hi) / 2
        distance = abs(usage_gb - mid) / max(1, (hi - lo) / 2)
        return max(0.0, 1.0 - distance * 0.7)

    def _budget_score(self, pkg, budget_max):
        price = pkg.get("price", 0)
        profile = pkg.get("ideal_profile", {})
        cap = profile.get("max_budget", 0)
        if budget_max is None:
            return 0.5
        if price <= budget_max:
            if price <= budget_max * 0.7:
                return 1.0
            return 0.9
        overflow = (price - budget_max) / max(1, price)
        return max(0.0, 0.6 - overflow)

    def _tag_score(self, pkg, needs):
        if not needs:
            return 0.5
        profile = pkg.get("ideal_profile", {})
        tags = profile.get("tags", [])
        hits = 0
        for tag in tags:
            tag_norm = tag.lower().replace(" ", "")
            for need in needs:
                need_norm = need.lower().replace(" ", "")
                if tag_norm in need_norm or need_norm in tag_norm:
                    hits += 1
        return min(1.0, hits / len(needs) * 1.5)

    def _benefit_score(self, pkg, benefits):
        if not benefits:
            return 0.5
        blobs = " ".join(pkg.get("benefits", [])).lower().replace(" ", "")
        hits = sum(1 for b in benefits if b.lower().replace(" ", "") in blobs)
        return min(1.0, hits / len(benefits) * 1.2)

    def recommend(self, profile, top_k=3):
        usage = profile.get("usage_gb")
        budget = profile.get("budget_max")
        needs = profile.get("needs", [])
        family = profile.get("family_members", 1)
        results = []
        for pkg in self.catalog.packages:
            ip = pkg.get("ideal_profile", {})
            score = (
                0.35 * self._usage_score(pkg, usage)
                + 0.30 * self._budget_score(pkg, budget)
                + 0.20 * self._tag_score(pkg, needs)
                + 0.15 * self._benefit_score(pkg, needs)
            )
            if family and family >= 2:
                if ip.get("family_members"):
                    score += 0.10
            results.append({"package": pkg, "score": round(min(1.0, score), 3)})
        results.sort(key=lambda x: x["score"], reverse=True)
        return results[:top_k]

    def build_reply(self, profile, results):
        lines = ["ขอบคุณค่ะ นี่คือแพ็กเกจที่เหมาะกับคุณมากที่สุด 3 อันดับ:"]
        for i, item in enumerate(results, start=1):
            pkg = item["package"]
            benefits = ", ".join(pkg.get("benefits", []))
            price = pkg.get("price", 0)
            lines.append(
                f"{i}) {pkg['name']} — {price:,} บาท/เดือน\n"
                f"   เหตุผล: เข้ากับการใช้งานและงบประมาณของคุณ (คะแนน {int(item['score'] * 100)}%)\n"
                f"   สิทธิประโยชน์: {benefits}"
            )
        lines.append("สนใจแพ็กเกจไหน กดตอบเป็นตัวเลขเพื่อดูรายละเอียดเพิ่มเติม หรือกด 'เปลี่ยนแพ็กเกจ' ได้เลยค่ะ")
        return "\n".join(lines)
