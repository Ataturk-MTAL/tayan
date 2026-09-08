#!/usr/bin/env python3
"""beceriler.json + dersler.json -> düzleştirilmiş ders×beceri satırları.

Ağa çıkmaz; yalnız iki üretilmiş dosyayı birleştirir.

Her satır bir eşlemedir: bir dersin bir sınıfındaki bir ünitede geçen bir
beceri. Beceri TANIMI ve SÜREÇ BİLEŞENLERİ satıra kopyalanmaz — onların tek
kaynağı `beceriler.json`'dur; kopyalamak 32 bin satırda ikinci bir doğruluk
kaynağı yaratır ve kaynak metin değişince sessizce ayrışır.

Çıktı: data/tymm/ders-beceri.json

Kullanım:
    python3 scripts/build_tymm_ders_beceri.py
    python3 scripts/build_tymm_ders_beceri.py --jsonl   # satır başına bir kayıt
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

PREFIX_RE = re.compile(r"^([A-ZÇĞİÖŞÜ]{1,8})")


def index_skills(beceriler: dict) -> tuple[dict, dict]:
    """kod -> (set, kayıt) dizinleri. İkincisi süreç bileşenleri içindir."""
    skills, components = {}, {}
    for skill_set in beceriler["sets"]:
        for skill in skill_set["skills"]:
            skills[skill["code"]] = (skill_set["code"], skill)
            for component in skill["components"]:
                components[component["code"]] = (skill_set["code"], skill["code"])
    return skills, components


SKILL_SUFFIX_RE = re.compile(r"\s*beceri(si|leri)$")


def normalize(text: str) -> str:
    """Ad karşılaştırması için sadeleştirir.

    Ders sayfaları becerinin adına "... Becerisi" ekliyor, beceri çerçevesi
    eklemiyor (`FBAB2. Sınıflandırma` ile `Sınıflandırma Becerisi (FBAB2)`).
    Bu ek elenmezse rapor gerçek sürüklenmeyi gösteremeyecek kadar gürültülü
    olur.
    """
    collapsed = re.sub(r"\s+", " ", (text or "")).strip().casefold()
    return SKILL_SUFFIX_RE.sub("", collapsed).strip()


def build_rows(dersler: dict, skills: dict, components: dict,
               mismatches: list[dict]) -> list[dict]:
    rows = []
    for course in dersler["courses"]:
        for grade in course["grades"]:
            for category in grade["categories"]:
                for unit in category["units"]:
                    for entry in unit["skills"]:
                        code = entry["code"]
                        set_code, record = skills.get(code, (None, None))
                        if record is None and code in components:
                            # Ders, süreç bileşeni düzeyine atıf yapmış.
                            set_code, parent = components[code]
                            record = {"name": entry["name"], "level": None}
                            parent_code = parent
                        else:
                            parent_code = record["parent"] if record else None
                        if set_code is None and code:
                            prefix = PREFIX_RE.match(code)
                            set_code = prefix.group(1) if prefix else None
                        # Ders sayfasındaki ad ile beceri çerçevesindeki kanonik
                        # ad karşılaştırılır ama satıra YAZILMAZ: 32 bin satırda
                        # 4 MB tutar ve ikinci bir doğruluk kaynağı olur. Yalnız
                        # uyuşmazlıklar raporlanır — kaynak metin değişirse buradan
                        # görülür.
                        if record is not None and record.get("name"):
                            if normalize(entry["name"]) != normalize(record["name"]):
                                mismatches.append(
                                    {
                                        "code": code,
                                        "source_name": entry["name"],
                                        "canonical_name": record["name"],
                                        "course": course["name"],
                                    }
                                )
                        rows.append(
                            {
                                "course_id": course["id"],
                                "grade": grade["name"],
                                "unit": unit["unit"],
                                "category": category["label"],
                                "set": set_code,
                                "code": code,
                                "level": record["level"] if record else None,
                                "parent": parent_code,
                                "resolved": record is not None,
                            }
                        )
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data-dir", type=Path, default=Path("data/tymm"))
    parser.add_argument(
        "--jsonl",
        action="store_true",
        help="Satır başına bir kayıt yaz (git diff'i satır satır okunur).",
    )
    args = parser.parse_args()

    beceriler_path = args.data_dir / "beceriler.json"
    dersler_path = args.data_dir / "dersler.json"
    for path in (beceriler_path, dersler_path):
        if not path.exists():
            print(f"HATA: {path} yok. Önce çekme script'lerini çalıştır.", file=sys.stderr)
            return 1

    beceriler = json.loads(beceriler_path.read_text(encoding="utf-8"))
    dersler = json.loads(dersler_path.read_text(encoding="utf-8"))

    skills, components = index_skills(beceriler)
    mismatches: list[dict] = []
    rows = build_rows(dersler, skills, components, mismatches)

    resolved = sum(1 for row in rows if row["resolved"])
    unresolved_codes = sorted({row["code"] for row in rows if not row["resolved"]})

    if args.jsonl:
        out_path = args.data_dir / "ders-beceri.jsonl"
        with out_path.open("w", encoding="utf-8") as handle:
            for row in rows:
                handle.write(json.dumps(row, ensure_ascii=False) + "\n")
    else:
        out_path = args.data_dir / "ders-beceri.json"
        document = {
            "report": {
                "rows": len(rows),
                "resolved": resolved,
                "unresolved": len(rows) - resolved,
                # Ders programı bu kodlara atıf yapıyor ama beceri çerçevesinde
                # karşılıkları yok. Satırlar atılmaz; işaretli durur.
                "unresolved_codes": unresolved_codes,
                "courses": len({row["course_id"] for row in rows}),
                "name_mismatches": len(mismatches),
                "name_mismatch_samples": mismatches[:20],
            },
            "source": {
                "generator": "scripts/build_tymm_ders_beceri.py",
                "inputs": [str(beceriler_path), str(dersler_path)],
                "fetched_at": dersler["source"]["fetched_at"],
                "note": "Satırlar sadeleştirilmiştir: ders adı/kademe course_id ile dersler.json'dan, beceri adı/tanımı/süreç bileşenleri code ile beceriler.json'dan çözülür.",
            },
            "rows": rows,
        }
        out_path.write_text(
            json.dumps(document, ensure_ascii=False, separators=(",", ":")) + "\n", encoding="utf-8"
        )

    print(
        f"yazıldı: {out_path} — {len(rows)} satır, {resolved} çözüldü, "
        f"{len(rows) - resolved} çözülemedi {unresolved_codes}, "
        f"ad uyuşmazlığı {len(mismatches)}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
