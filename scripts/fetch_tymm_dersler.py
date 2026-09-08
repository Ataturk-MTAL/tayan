#!/usr/bin/env python3
"""TYMM ders programlarını ünite–beceri eşlemesiyle birlikte çeker.

İki uç nokta kullanılır (ikisi de sayfadaki JS'ten alındı):

    /Ders/GetDerslerBySinif?sinifId=<id>&kademe=<id>   -> ders dizini
    /Chart/GetStackCharts?url=<ders-url>               -> ünite/beceri eşlemesi

Ders programının tam metni PDF'tedir; PDF indirilmez, yalnız adresi kaydedilir.

Çıktı: data/tymm/dersler.json + data/tymm/DERSLER.md

Kullanım:
    python3 scripts/fetch_tymm_dersler.py
    python3 scripts/fetch_tymm_dersler.py --kademe 3      # yalnız ortaöğretim
"""

from __future__ import annotations

import argparse
import datetime
import html
import json
import re
import sys
import urllib.parse
import urllib.request
from pathlib import Path

BASE_URL = "https://tymm.meb.gov.tr"
DERS_ENDPOINT = "/Ders/GetDerslerBySinif"
CHART_ENDPOINT = "/Chart/GetStackCharts"
PDF_PREFIX = "/assets/pdf/"

KADEME_NAMES = {
    2: "Temel Eğitim",
    3: "Ortaöğretim",
    9: "Spor Ortaokulları",
    11: "Müzik Okulları",
}
# Sınıf kimlikleri kademeye göre değişiyor ve hiçbir yerde toplu listelenmiyor;
# aralığı tarayıp boş dönenleri atmak, id'leri elle sabitlemekten sağlamdır.
SINIF_ID_RANGE = list(range(1, 25)) + [-1]

USER_AGENT = "Mozilla/5.0 (compatible; tayan-tymm-fetch/1.0)"
REQUEST_TIMEOUT_S = 30

# Grup başlığı adlandırması kademeye ve derse göre değişiyor: "1. Ünite:",
# "1. Tema:", "6. Öğrenme Alanı:", "36-48 Ay Eylül Ayı Planı", "Theme 1:",
# "Lektıon 1:". Metin kalıbı kovalamak kırılgan; yapı zaten kesin: her <p>
# bir grup başlığıdır, ardındaki <li> ögeleri o gruba aittir.
# Kod iki biçimde gelir: baştan "KB2.8.Sorgulama Becerisi" ya da sondan
# parantez içinde "Dinleme/İzleme Becerisi (TAB1)".
CODE_RE = re.compile(r"^([A-ZÇĞİÖŞÜ]{1,8}\d[\d.]*)\.\s*(.*)$")
TRAILING_CODE_RE = re.compile(r"^(.+?)\s*\(([A-ZÇĞİÖŞÜ]{1,8}\d[\d.]*)\)\s*$")
ASSESSMENT_LABEL = "Öğrenme Kanıtları"
EVALUATION_MARKER = "Değerlendirme"


def get_json(path: str, params: dict) -> object:
    url = f"{BASE_URL}{path}?{urllib.parse.urlencode(params)}"
    request = urllib.request.Request(
        url,
        headers={"User-Agent": USER_AGENT, "X-Requested-With": "XMLHttpRequest"},
    )
    with urllib.request.urlopen(request, timeout=REQUEST_TIMEOUT_S) as response:
        return json.loads(response.read().decode("utf-8", errors="replace"))


def text_of(markup: str) -> str:
    return re.sub(r"\s+", " ", html.unescape(re.sub(r"<[^>]+>", " ", markup))).strip()


def blocks_of(markup: str) -> list[tuple[str, str]]:
    """<p> ve <li> ögelerini belge sırasında (tür, metin) olarak döndürür.

    Sıra taşıyıcıdır: bir <p> "1. Ünite: ..." başlığıdır ve kendisinden sonraki
    <li> ögeleri o üniteye aittir.
    """
    blocks = []
    for match in re.finditer(r"(?is)<(p|li)\b[^>]*>(.*?)</\1>", markup or ""):
        content = text_of(match.group(2))
        if content:
            blocks.append((match.group(1).lower(), content))
    return blocks


def parse_skill_units(markup: str) -> list[dict]:
    """Beceri kategorisi HTML'ini ünite -> beceri listesine çevirir."""
    units: list[dict] = []
    current: dict | None = None
    for kind, content in blocks_of(markup):
        if kind == "p":
            current = {"unit": content, "skills": []}
            units.append(current)
            continue
        if kind == "li" and current is not None:
            leading = CODE_RE.match(content)
            trailing = TRAILING_CODE_RE.match(content)
            if leading:
                current["skills"].append(
                    {"code": leading.group(1), "name": leading.group(2).strip()}
                )
            elif trailing:
                current["skills"].append(
                    {"code": trailing.group(2), "name": trailing.group(1).strip()}
                )
            else:
                # Kodsuz madde: kaynakta kod verilmemiş; metni kaybetme.
                current["skills"].append({"code": None, "name": content})
    return [u for u in units if u["skills"]]


def parse_assessment_units(markup: str) -> list[dict]:
    """Öğrenme Kanıtları HTML'ini ünite -> (kanıt, değerlendirme) ayrımına çevirir.

    Blok içinde "Değerlendirme:" alt başlığı, ölçme KANITLARINI (poster, sunum)
    onları puanlayan ARAÇLARDAN (dereceli puanlama anahtarı) ayırır; ikisi tek
    listeye konursa bu ayrım kaybolur.
    """
    units: list[dict] = []
    current: dict | None = None
    in_evaluation = False
    for kind, content in blocks_of(markup):
        if kind == "p" and content.rstrip(":").strip() == EVALUATION_MARKER:
            in_evaluation = True
            continue
        if kind == "p":
            current = {"unit": content, "evidence": [], "evaluation": []}
            units.append(current)
            in_evaluation = False
            continue
        if kind == "li" and current is not None:
            target = "evaluation" if in_evaluation else "evidence"
            current[target].append(content)
    return [u for u in units if u["evidence"] or u["evaluation"]]


def collect_courses(kademeler: list[int]) -> list[dict]:
    courses: dict[tuple[int, int], dict] = {}
    for kademe in kademeler:
        for sinif_id in SINIF_ID_RANGE:
            try:
                data = get_json(DERS_ENDPOINT, {"sinifId": sinif_id, "kademe": kademe})
            except Exception as error:
                print(f"  uyarı: kademe={kademe} sinif={sinif_id}: {error}", file=sys.stderr)
                continue
            for item in data or []:
                courses[(item["kademe"], item["id"])] = item
    return [courses[key] for key in sorted(courses)]


def build_course(item: dict) -> dict:
    chart = get_json(CHART_ENDPOINT, {"url": item["url"]})
    grades = []
    for series in chart.get("stackedChart") or []:
        categories, assessment = [], []
        for point in series.get("dataPoints") or []:
            label = point.get("label") or ""
            markup = point.get("beceriler") or ""
            if label.startswith(ASSESSMENT_LABEL):
                assessment = parse_assessment_units(markup)
                continue
            categories.append(
                {
                    "label": label,
                    "share": point.get("y"),
                    "units": parse_skill_units(markup),
                }
            )
        grades.append(
            {"name": series.get("name"), "categories": categories, "assessment": assessment}
        )

    return {
        "id": item["id"],
        "name": item["dersAdi"],
        "url": item["url"],
        "page_url": f"{BASE_URL}/ogretim-programlari/ders/{item['url']}",
        "kademe": item["kademe"],
        "kademe_name": KADEME_NAMES.get(item["kademe"], str(item["kademe"])),
        # PDF indirilmez; program metni gerekirse buradan çekilir.
        "pdf_url": f"{BASE_URL}{PDF_PREFIX}{item['pdf']}" if item.get("pdf") else None,
        "grades": grades,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--kademe",
        type=int,
        action="append",
        choices=sorted(KADEME_NAMES),
        help="Yalnız bu kademe(ler). Verilmezse hepsi.",
    )
    parser.add_argument("--out-dir", type=Path, default=Path("data/tymm"))
    args = parser.parse_args()

    kademeler = args.kademe or sorted(KADEME_NAMES)
    print(f"ders dizini çekiliyor (kademe: {kademeler})", file=sys.stderr)
    raw_courses = collect_courses(kademeler)
    print(f"  {len(raw_courses)} ders bulundu", file=sys.stderr)

    courses = []
    for index, item in enumerate(raw_courses, 1):
        try:
            course = build_course(item)
        except Exception as error:
            print(f"  HATA {item['url']}: {error}", file=sys.stderr)
            return 1
        courses.append(course)
        mappings = sum(
            len(unit["skills"])
            for grade in course["grades"]
            for category in grade["categories"]
            for unit in category["units"]
        )
        print(
            f"  [{index}/{len(raw_courses)}] {course['name'][:44]:<46} "
            f"sınıf={len(course['grades'])} eşleme={mappings}",
            file=sys.stderr,
        )

    total_mappings = sum(
        len(unit["skills"])
        for c in courses
        for g in c["grades"]
        for cat in g["categories"]
        for unit in cat["units"]
    )
    coded = sum(
        1
        for c in courses
        for g in c["grades"]
        for cat in g["categories"]
        for unit in cat["units"]
        for s in unit["skills"]
        if s["code"]
    )
    fetched_at = datetime.date.today().isoformat()

    document = {
        "report": {
            "courses": len(courses),
            "skill_mappings": total_mappings,
            "coded_mappings": coded,
            "uncoded_mappings": total_mappings - coded,
        },
        "source": {
            "name": "Türkiye Yüzyılı Maarif Modeli — Ders Programları",
            "base_url": BASE_URL,
            "fetched_at": fetched_at,
            "generator": "scripts/fetch_tymm_dersler.py",
            "endpoints": [DERS_ENDPOINT, CHART_ENDPOINT],
        },
        "courses": courses,
    }

    args.out_dir.mkdir(parents=True, exist_ok=True)
    (args.out_dir / "dersler.json").write_text(
        json.dumps(document, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    by_kademe: dict[str, list[dict]] = {}
    for course in courses:
        by_kademe.setdefault(course["kademe_name"], []).append(course)

    lines = [
        "# TYMM ders programları — kaynaklar",
        "",
        f"Çekim tarihi: {fetched_at}",
        "",
        "`data/tymm/dersler.json` iki uç noktadan üretildi. Elle düzenlenmez;",
        "yeniden üretmek için `python3 scripts/fetch_tymm_dersler.py`.",
        "",
        f"- `{DERS_ENDPOINT}` — ders dizini (kademe × sınıf)",
        f"- `{CHART_ENDPOINT}` — ünite başına beceri ve öğrenme kanıtı eşlemesi",
        "",
        "| Kademe | Ders | Beceri eşlemesi |",
        "|---|---:|---:|",
    ]
    for name, items in by_kademe.items():
        mapped = sum(
            len(u["skills"]) for c in items for g in c["grades"]
            for cat in g["categories"] for u in cat["units"]
        )
        lines.append(f"| {name} | {len(items)} | {mapped} |")
    lines += [
        f"| **Toplam** | **{len(courses)}** | **{total_mappings}** |",
        "",
        "## Notlar",
        "",
        "- Beceri kodları (`KB2.8`, `SDB1.2` …) `data/tymm/beceriler.json` ile",
        "  doğrudan eşleşir; iki veri seti kod üzerinden birleştirilebilir.",
        f"- Kodsuz eşleme: {total_mappings - coded}/{total_mappings}. Bunlarda kaynak",
        "  yalnız beceri adını vermiştir; metin `name` alanında korunur.",
        "- Ders programının tam metni PDF'tedir. PDF'ler indirilmez; adresleri",
        "  `pdf_url` alanındadır.",
        "- Öğrenme kanıtları `evidence` (ölçme kanıtı) ve `evaluation` (puanlama",
        "  aracı) olarak ayrı tutulur; kaynakta ikincisi `Değerlendirme:` alt",
        "  başlığı altındadır.",
    ]
    (args.out_dir / "DERSLER.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(
        f"\nyazıldı: {args.out_dir / 'dersler.json'} — {len(courses)} ders, "
        f"{total_mappings} beceri eşlemesi ({coded} kodlu)",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
