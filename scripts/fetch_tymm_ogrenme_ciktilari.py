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
from tymm_outcomes import (  # noqa: E402
    SCHEME_PATTERNS,
    build_code,
    detect_scheme,
    parse_outcomes,
)
from fetch_tymm_dersler import KADEME_NAMES  # noqa: E402

USER_AGENT = "Mozilla/5.0 (compatible; tayan-tymm-fetch/1.0)"
REQUEST_TIMEOUT_S = 120

# Çıktı kodu: <ÖNEK>.<sınıf>.<ünite>.<sıra>. Ön ek uzunluğu derse göre değişir
# (FİZ, MAT, TÜR, T.C.İNK gibi), SABİTLENMEZ.
# Desenler ve çıktı ayrıştırması tymm_outcomes modülünde; burada
# tekrarlanmaz. Bu dosyanın işi indirme, birleştirme ve yazma.

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
        prefixes, segments = detect_scheme(text)
        prefix = prefixes[0] if prefixes else None
        if not prefixes:
            gaps.append({"course": course["name"], "reason": "kodlu çıktı bulunamadı"})
            print(f"  [{position}/{len(courses)}] {course['name'][:40]:<42} kod YOK", file=sys.stderr)
            if not args.keep_pdf:
                pdf_path.unlink(missing_ok=True)
            continue

        outcomes = parse_outcomes(text, prefixes, segments)

        # KAPSAMA DENETİMİ. Metinde geçen kod kümesinden ayrıştırılan küme
        # çıkarılır; fark raporlanır. Bu oturumdaki her sessiz kaybı bu
        # denetim yakaladı — kaldırılmamalı.
        # Denetim, AYRIŞTIRICININ deseninin AYNISINI kullanır. Önceden
        # "FOUR_RE if segments == 4 else THREE_RE" yazıyordu; yapışık (2, 5) ve
        # boşluklu (6) şemalarda THREE_RE hiçbir kodu eşlemiyor, scanned boş
        # kalıyor ve denetim 8 derste sessizce işlemsiz oluyordu.
        pattern = SCHEME_PATTERNS.get(segments)
        wanted = set(prefixes)
        scanned = {
            build_code(m, segments)
            for m in (pattern.finditer(text) if pattern else ())
            if m.group(1) in wanted
        }
        missed = sorted(scanned - {o["code"] for o in outcomes})
        if missed:
            gaps.append({
                "course": course["name"],
                "reason": "ayrıştırılamayan kod",
                "codes": missed[:20],
            })

        units = parse_units(text, prefix or "")

        # KONUMA DAYALI EŞLEŞTİRME. Çıktı, fiziksel olarak hangi ünite
        # bloğunun içindeyse ona aittir. Önceki hâl (grade, unit) anahtarıyla
        # eşleştiriyordu ve iki yerde çuvallıyordu: üç sayılı kodda ünite
        # kademesi yok, bazı belgelerde de "N. SINIF" satırı hiç geçmediği için
        # blokların sınıfı None kalıyordu. 1837 çıktı bu yüzden sahipsizdi.
        # Blok = çıktının NEREDE işlendiği. Belge geneli = KANONİK tanım.
        # Türkçe gibi derslerde tema bloğunda çıktı yalnız kod+ad olarak anılır,
        # a/b/c süreç bileşenleri EK 1'de tanımlıdır. İkisi birleştirilmezse ya
        # eşleme ya da bileşenler kaybolur.
        canonical = {o["code"]: o for o in outcomes}

        assigned: set[str] = set()
        for unit in units:
            unit_outcomes = parse_outcomes(unit["text"], prefixes, segments)
            merged = []
            for o in unit_outcomes:
                source = canonical.get(o["code"], o)
                # Blok metni daha dolgunsa onu tercih et; değilse kanonik olan.
                text = o["text"] if len(o["text"]) >= len(source["text"]) else source["text"]
                components = o["components"] or source["components"]
                merged.append(
                    {
                        "code": o["code"],
                        "order": o["order"],
                        "text": text,
                        "components": components,
                    }
                )
            unit["outcomes"] = merged
            assigned.update(o["code"] for o in unit_outcomes)
            # Bloğun sınıfı yazılı değilse içindeki çıktının kodundan alınır;
            # uydurulmaz, kodun kendisi söyler (MBU.5.1.1 -> 5. sınıf).
            if unit["grade"] is None and unit_outcomes:
                unit["grade"] = unit_outcomes[0]["grade"]
            unit.pop("text", None)

        orphan_outcomes = [o for o in outcomes if o["code"] not in assigned]
        if orphan_outcomes:
            gaps.append({
                "course": course["name"],
                "reason": "hiçbir ünite bloğunun içinde değil",
                "codes": [o["code"] for o in orphan_outcomes][:20],
            })

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
            f"{(prefix or '-'):<8} çıktı={len(outcomes):<4} bileşen="
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
