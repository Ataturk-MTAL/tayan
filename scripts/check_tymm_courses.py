#!/usr/bin/env python3
"""Üretilen ders dosyalarını doğrular; sorun varsa çıkış kodu 1.

Dört şeye bakar:
  1. index.json'daki her ders dosyası gerçekten var mı,
  2. index'teki sayımlar dosyaların kendi raporlarıyla tutuyor mu,
  3. sahipsiz çıktı ve boşluk var mı (varsa sayısı ve dersi yazılır),
  4. ünite bölümlerinde geçen beceri kodları beceriler.json'da çözülüyor mu.

Boşluk bulunması BAŞARISIZLIK değildir — bilinen ve raporlanan eksikler var
(Türkçe programlarında çıktılar EK 1'de, Okul Öncesi üç parçalı kod
kullanıyor). Çıkış kodu 1 yalnız YAPISAL sorunlarda döner: eksik dosya veya
index ile dosya arasında sayım uyuşmazlığı. Eksikler sayıyla raporlanır ki
sessizce büyümesinler.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

DATA = Path("data/tymm")
COURSES = DATA / "courses"
CODE_RE = re.compile(r"\b([A-ZÇĞİÖŞÜ]{1,8}\d[\d.]*)\.")


def load_known_codes() -> set[str]:
    beceriler = json.loads((DATA / "beceriler.json").read_text(encoding="utf-8"))
    known = {k["code"] for s in beceriler["sets"] for k in s["skills"]}
    known |= {
        c["code"]
        for s in beceriler["sets"]
        for k in s["skills"]
        for c in k["components"]
    }
    return known


def main() -> int:
    index_path = COURSES / "index.json"
    if not index_path.exists():
        print(f"HATA: {index_path} yok. Önce fetch_tymm_ogrenme_ciktilari.py çalıştır.",
              file=sys.stderr)
        return 1

    index = json.loads(index_path.read_text(encoding="utf-8"))
    known = load_known_codes()

    problems: list[str] = []
    unresolved: dict[str, int] = {}
    orphans_total = 0
    orphan_courses: list[tuple[str, int]] = []
    sum_units = sum_outcomes = sum_components = 0

    for entry in index["courses"]:
        path = DATA / entry["file"]
        if not path.exists():
            problems.append(f"eksik dosya: {entry['file']}")
            continue

        document = json.loads(path.read_text(encoding="utf-8"))
        report = document["report"]

        # index ile dosyanın kendi raporu ayrışırsa biri bayat demektir.
        for field in ("units", "outcomes", "components"):
            if entry[field] != report[field]:
                problems.append(
                    f"{entry['slug']}: index.{field}={entry[field]} "
                    f"ama dosyada {report[field]}"
                )
        sum_units += report["units"]
        sum_outcomes += report["outcomes"]
        sum_components += report["components"]

        orphans = len(document["unassigned_outcomes"])
        if orphans:
            orphans_total += orphans
            orphan_courses.append((entry["name"], orphans))

        # Dersin KENDİ çıktı kodları bölüm metinlerinde bolca geçiyor
        # (Anahtar Kavramlar, İLİŞKİLER, Öğrenme Çıktıları). Bunlar beceri
        # referansı değil; ayıklanmazsa doğrulayıcı 625 yanlış pozitif üretir.
        own_prefix = document["course"].get("prefix") or ""
        for unit in document["units"]:
            for text in unit["sections"].values():
                for match in CODE_RE.finditer(text):
                    code = match.group(1)
                    if code in known:
                        continue
                    if own_prefix and code.startswith(own_prefix):
                        continue
                    unresolved[code] = unresolved.get(code, 0) + 1

    report = index["report"]
    print(f"ders: {report['courses']}   ünite: {sum_units}")
    print(f"çıktı (ünitede): {sum_outcomes}   süreç bileşeni: {sum_components}")
    print(f"sahipsiz çıktı: {orphans_total}   TOPLAM çıktı: {sum_outcomes + orphans_total}")
    print(f"çıktısı çıkarılamayan ders: {report['courses_without_outcomes']}")
    print(f"boşluk kaydı: {len(report['gaps'])}")

    if orphan_courses:
        print(f"\nsahipsiz çıktısı olan {len(orphan_courses)} ders:")
        for name, count in sorted(orphan_courses, key=lambda x: -x[1])[:10]:
            print(f"   {name[:44]:<46} {count}")

    if unresolved:
        # İki ayrı durum: seti VAR ama kodu yok (kaynak tutarsızlığı) ve
        # seti hiç yok (tanımadığımız beceri kümesi). Aynı torbaya konurlarsa
        # hangisinin düzeltilebilir olduğu görünmez.
        beceriler = json.loads((DATA / "beceriler.json").read_text(encoding="utf-8"))
        sets = {s["code"] for s in beceriler["sets"] if s["code"]}
        inconsistent: dict[str, int] = {}
        unknown_set: dict[str, int] = {}
        for code, count in unresolved.items():
            prefix = re.match(r"[A-ZÇĞİÖŞÜ]+", code)
            target = inconsistent if prefix and prefix.group(0) in sets else unknown_set
            target[code] = count

        print(f"\nçözülemeyen kod: {len(unresolved)} tür / {sum(unresolved.values())} geçiş")
        print(f"   seti VAR, kodu yok (kaynak tutarsızlığı): {len(inconsistent)} tür")
        for code, count in sorted(inconsistent.items(), key=lambda kv: -kv[1])[:6]:
            print(f"      {code:<14} x{count}")
        print(f"   seti HİÇ YOK (tanınmayan küme): {len(unknown_set)} tür")
        for code, count in sorted(unknown_set.items(), key=lambda kv: -kv[1])[:6]:
            print(f"      {code:<14} x{count}")

    for problem in problems:
        print(f"  SORUN {problem}")

    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
