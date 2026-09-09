#!/usr/bin/env python3
"""Ders programı PDF'lerinden öğrenme çıktılarını ve süreç bileşenlerini çıkarır.

Öğrenme çıktıları sitenin `/Chart/GetStackCharts` uç noktasında YOKTUR; yalnız
ders programı PDF'inin içinde bulunur. `dersler.json` beceri eşlemesini verir,
bu script planın omurgasını —kodlu çıktıları ve onların a/b/c/ç süreç
bileşenlerini— getirir.

PDF'teki düzen:

    ÖĞRENME ÇIKTILARI
    VE SÜREÇ BİLEŞENLERİ  FİZ.9.1.1. Fizik biliminin tanımına yönelik ...
                             a) Fizik biliminin diğer disiplinlerle ...
                             b) Fizik bilimini belirlediği ilişkilerden ...

Kod konumu kendi taşır: FİZ.9.1.1 = ders FİZ, 9. sınıf, 1. ünite, 1. çıktı.
Ünite ADI için `dersler.json` ile eşleşilir.

Çıktı: data/tymm/ogrenme-ciktilari.json   (PDF'ler .tymm-pdf/ altında, sürümlenmez)

Kullanım:
    python3 scripts/fetch_tymm_ogrenme_ciktilari.py
    python3 scripts/fetch_tymm_ogrenme_ciktilari.py --only fizik-dersi
    python3 scripts/fetch_tymm_ogrenme_ciktilari.py --keep-pdf   # PDF'i silme
"""

from __future__ import annotations

import argparse
import collections
import datetime
import json
import re
import shutil
import subprocess
import sys
import urllib.parse
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from tymm_pdf_sections import parse_units  # noqa: E402
from fetch_tymm_dersler import KADEME_NAMES  # noqa: E402

USER_AGENT = "Mozilla/5.0 (compatible; tayan-tymm-fetch/1.0)"
REQUEST_TIMEOUT_S = 120

# Çıktı kodu: <ÖNEK>.<sınıf>.<ünite>.<sıra>. Ön ek uzunluğu derse göre değişir
# (FİZ, MAT, TÜR, T.C.İNK gibi), SABİTLENMEZ.
OUTCOME_RE = re.compile(r"([A-ZÇĞİÖŞÜ][A-ZÇĞİÖŞÜ.]{1,9})\.(\d+)\.(\d+)\.(\d+)\.\s*(.*)$")
CODE_SCAN_RE = re.compile(r"\b([A-ZÇĞİÖŞÜ][A-ZÇĞİÖŞÜ.]{1,9})\.(\d+)\.(\d+)\.(\d+)\b")
COMPONENT_RE = re.compile(r"^\s*([a-zçğöşü])\)\s*(.+)$")

# Çıktı bloğunu sonlandıran bölüm başlıkları.
SECTION_RE = re.compile(
    r"^\s*(İÇERİK ÇERÇEVESİ|Anahtar Kavramlar|ÖĞRENME|KANITLARI|FARKLILAŞTIRMA"
    r"|BECERİLER|İLİŞKİLER|EĞİLİMLER|DEĞERLER|PROGRAMLAR|SOSYAL|ALAN|KAVRAMSAL"
    r"|VE SÜREÇ|\d+\.\s*ÜNİTE)"
)
PAGE_NOISE_RE = re.compile(r"^\s*\d+\s*$|ÖĞRETİM PROGRAMI\s*$")


def collapse(text: str) -> str:
    """Satır kırılmalarını ve tire bölünmelerini toparlar."""
    joined = re.sub(r"\s+", " ", text).strip()
    return re.sub(r"-\s+(?=\w)", "", joined)


def download(url: str, target: Path) -> None:
    # PDF adlarında Türkçe karakter geçebiliyor; ham hâliyle istenirse
    # urllib "'ascii' codec can't encode character '\u0131'" ile düşüyor.
    safe = urllib.parse.quote(url, safe=":/?&=%#")
    request = urllib.request.Request(safe, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=REQUEST_TIMEOUT_S) as response:
        target.write_bytes(response.read())


def dominant_prefix(text: str) -> str | None:
    """Belgede en sık geçen çıktı ön ekini bulur; ders kısaltmasını sabitlemez."""
    counts = collections.Counter(m.group(1) for m in CODE_SCAN_RE.finditer(text))
    return counts.most_common(1)[0][0] if counts else None


def parse_outcomes(text: str, prefix: str) -> list[dict]:
    lines = text.splitlines()
    best: dict[str, dict] = {}
    index = 0
    while index < len(lines):
        # DİKKAT: search, match DEĞİL. Her ünitenin İLK çıktısı bölüm etiketiyle
        # aynı satırı paylaşır ("VE SÜREÇ BİLEŞENLERİ FİZ.9.1.1. ..."); satır
        # başına çakılı bir desen onları sessizce düşürür.
        head = OUTCOME_RE.search(lines[index])
        if not head or head.group(1) != prefix:
            index += 1
            continue

        code = f"{head.group(1)}.{head.group(2)}.{head.group(3)}.{head.group(4)}"
        title_parts = [head.group(5)]
        components: list[dict] = []
        current: dict | None = None
        index += 1

        while index < len(lines):
            line = lines[index]
            if OUTCOME_RE.search(line) or SECTION_RE.match(line):
                break
            if PAGE_NOISE_RE.match(line):
                index += 1
                continue
            component = COMPONENT_RE.match(line)
            if component:
                current = {"label": component.group(1), "text": [component.group(2)]}
                components.append(current)
            elif line.strip():
                (current["text"] if current else title_parts).append(line.strip())
            index += 1

        entry = {
            "code": code,
            "grade": int(head.group(2)),
            "unit": int(head.group(3)),
            "order": int(head.group(4)),
            "text": collapse(" ".join(title_parts)),
            "components": [
                {"label": c["label"], "text": collapse(" ".join(c["text"]))} for c in components
            ],
        }
        # Aynı kod belgede birden çok geçer (tablo + özet + atıf). En dolgun
        # geçişi sakla; çıplak atıflar düşük puan alıp elenir.
        score = len(entry["text"]) + 50 * len(entry["components"])
        if code not in best or score > best[code]["_score"]:
            best[code] = {**entry, "_score": score}

    for entry in best.values():
        entry.pop("_score", None)
    return sorted(best.values(), key=lambda e: (e["grade"], e["unit"], e["order"]))


def unit_titles(courses: list[dict], course_url: str) -> dict[tuple[int, int], str]:
    """dersler.json'daki ünite adlarını (sınıf, ünite no) ile eşler."""
    titles: dict[tuple[int, int], str] = {}
    course = next((c for c in courses if c["url"] == course_url), None)
    if not course:
        return titles
    for grade in course["grades"]:
        grade_no = re.match(r"(\d+)", grade["name"] or "")
        if not grade_no:
            continue
        for category in grade["categories"]:
            for unit in category["units"]:
                number = re.match(r"\s*(\d+)\s*\.", unit["unit"] or "")
                if number:
                    titles[(int(grade_no.group(1)), int(number.group(1)))] = unit["unit"]
    return titles


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data-dir", type=Path, default=Path("data/tymm"))
    parser.add_argument("--pdf-dir", type=Path, default=Path(".tymm-pdf/dersler"))
    parser.add_argument("--only", help="Yalnız bu ders url'si (ör. fizik-dersi)")
    parser.add_argument("--keep-pdf", action="store_true", help="PDF'leri silme")
    args = parser.parse_args()

    if not shutil.which("pdftotext"):
        print("HATA: pdftotext bulunamadı (macOS: brew install poppler)", file=sys.stderr)
        return 1

    dersler_path = args.data_dir / "dersler.json"
    if not dersler_path.exists():
        print(f"HATA: {dersler_path} yok. Önce fetch_tymm_dersler.py çalıştır.", file=sys.stderr)
        return 1
    dersler = json.loads(dersler_path.read_text(encoding="utf-8"))
    courses = dersler["courses"]
    if args.only:
        courses = [c for c in courses if c["url"] == args.only]
        if not courses:
            print(f"HATA: '{args.only}' adlı ders yok.", file=sys.stderr)
            return 1

    args.pdf_dir.mkdir(parents=True, exist_ok=True)
    courses_dir = args.data_dir / "courses"
    courses_dir.mkdir(parents=True, exist_ok=True)
    index: list[dict] = []
    gaps: list[dict] = []
    parsed_count = 0

    for position, course in enumerate(courses, 1):
        if not course.get("pdf_url"):
            gaps.append({"course": course["name"], "reason": "pdf_url yok"})
            continue

        pdf_path = args.pdf_dir / f"{course['url']}.pdf"
        txt_path = pdf_path.with_suffix(".txt")
        try:
            if not pdf_path.exists():
                download(course["pdf_url"], pdf_path)
            if not txt_path.exists():
                subprocess.run(
                    ["pdftotext", "-layout", str(pdf_path), str(txt_path)],
                    check=True, capture_output=True,
                )
        except Exception as error:
            gaps.append({"course": course["name"], "reason": f"indirme/metin: {error}"})
            print(f"  [{position}/{len(courses)}] HATA {course['url']}: {error}", file=sys.stderr)
            continue

        text = txt_path.read_text(encoding="utf-8", errors="replace")
        prefix = dominant_prefix(text)
        if not prefix:
            gaps.append({"course": course["name"], "reason": "kodlu çıktı bulunamadı"})
            print(f"  [{position}/{len(courses)}] {course['name'][:40]:<42} kod YOK", file=sys.stderr)
            if not args.keep_pdf:
                pdf_path.unlink(missing_ok=True)
            continue

        outcomes = parse_outcomes(text, prefix)
        units = parse_units(text, prefix)
        by_unit = {(u["grade"], u["unit"]): u for u in units}
        for unit in units:
            unit["outcomes"] = []

        orphan_outcomes = []
        for outcome in outcomes:
            target = by_unit.get((outcome["grade"], outcome["unit"]))
            if target is None:
                # Ünite bloğu bulunamadı; çıktı KAYBOLMAZ, boşluk raporlanır.
                orphan_outcomes.append(outcome)
                continue
            target["outcomes"].append(
                {
                    "code": outcome["code"],
                    "order": outcome["order"],
                    "text": outcome["text"],
                    "components": outcome["components"],
                }
            )
        if orphan_outcomes:
            gaps.append({
                "course": course["name"],
                "reason": "ünitesi bulunamayan çıktı",
                "codes": [o["code"] for o in orphan_outcomes],
            })

        # KAPSAMA DENETİMİ. Metinde geçen kod sayısı ile ayrıştırılan sayı
        # eşleşmiyorsa sessiz kayıp var demektir — raporlanır, gizlenmez.
        scanned = {
            f"{m.group(1)}.{m.group(2)}.{m.group(3)}.{m.group(4)}"
            for m in CODE_SCAN_RE.finditer(text)
            if m.group(1) == prefix
        }
        parsed = {o["code"] for o in outcomes}
        missed = sorted(scanned - parsed)
        if missed:
            gaps.append({"course": course["name"], "reason": "ayrıştırılamayan kod", "codes": missed})

        # Ders dosyası HEMEN yazılır, bellekte biriktirilmez. 111 dersin
        # tamamını tutup sonda yazmak tepe belleği gereksiz şişiriyordu ve
        # sıkışık bir makinede koşum işletim sistemi tarafından öldürüldü.
        report = {
            "units": len(units),
            "outcomes": sum(len(u["outcomes"]) for u in units),
            "components": sum(len(o["components"]) for u in units for o in u["outcomes"]),
            "orphan_outcomes": len(orphan_outcomes),
            "gaps": [g for g in gaps if g["course"] == course["name"]],
        }
        (courses_dir / f"{course['url']}.json").write_text(
            json.dumps(
                {
                    "course": {
                        "id": course["id"],
                        "name": course["name"],
                        "slug": course["url"],
                        "prefix": prefix,
                        "kademe": course["kademe"],
                        "kademe_name": KADEME_NAMES.get(
                            course["kademe"], str(course["kademe"])
                        ),
                        "pdf_url": course["pdf_url"],
                    },
                    "units": units,
                    # Ünitesine bağlanamayan çıktılar ATILMAZ; burada durur.
                    "unassigned_outcomes": orphan_outcomes,
                    "report": report,
                },
                ensure_ascii=False,
                indent=1,
            )
            + "\n",
            encoding="utf-8",
        )
        index.append({
            "slug": course["url"],
            "name": course["name"],
            "kademe": course["kademe"],
            "prefix": prefix,
            "file": f"courses/{course['url']}.json",
            "units": report["units"],
            "outcomes": report["outcomes"],
            "components": report["components"],
        })
        parsed_count += 1
        print(
            f"  [{position}/{len(courses)}] {course['name'][:40]:<42} "
            f"{prefix:<8} çıktı={len(outcomes):<4} bileşen="
            f"{sum(len(o['components']) for o in outcomes):<5} kayıp={len(missed)}",
            file=sys.stderr,
        )

        if not args.keep_pdf:
            pdf_path.unlink(missing_ok=True)

    total_outcomes = sum(c["outcomes"] for c in index)
    total_components = sum(c["components"] for c in index)
    (courses_dir / "index.json").write_text(
        json.dumps(
            {
                "source": {
                    "name": "Türkiye Yüzyılı Maarif Modeli — Ders Programları",
                    "fetched_at": datetime.date.today().isoformat(),
                    "generator": "scripts/fetch_tymm_ogrenme_ciktilari.py",
                    "note": "Ders programı PDF'lerinden çıkarıldı; PDF'ler sürümlenmez.",
                },
                "report": {
                    "courses": len(index),
                    "units": sum(c["units"] for c in index),
                    "outcomes": total_outcomes,
                    "components": total_components,
                    "courses_without_outcomes": len(courses) - parsed_count,
                    "gaps": gaps,
                },
                "courses": sorted(index, key=lambda c: (c["kademe"], c["name"])),
            },
            ensure_ascii=False,
            indent=1,
        )
        + "\n",
        encoding="utf-8",
    )

    print(
        f"\nyazıldı: {courses_dir}/ — {parsed_count} ders dosyası, {total_outcomes} çıktı, "
        f"{total_components} süreç bileşeni, {len(gaps)} boşluk",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
