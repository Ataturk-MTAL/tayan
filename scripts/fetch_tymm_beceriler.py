#!/usr/bin/env python3
"""TYMM beceri çerçevesini tymm.meb.gov.tr'den çekip JSON'a dönüştürür.

Kaynak sayfalar statik HTML; yapı her sayfada aynı:

    KB2.Bütünleşik Beceriler            <- düğüm (level 1)
      açıklama
      KB2.1.Çelişki Giderme Becerisi    <- düğüm (level 2)
        açıklama
        Süreç bileşenleri
          KB2.1.SB1. ...                <- süreç bileşeni

Çıktı: data/tymm/beceriler.json + data/tymm/SOURCES.md

Kullanım:
    python3 scripts/fetch_tymm_beceriler.py
    python3 scripts/fetch_tymm_beceriler.py --cache-dir /tmp/tymm   # tekrar indirme
"""

from __future__ import annotations

import argparse
import datetime
import html
import json
import re
import sys
import urllib.request
from collections import Counter
from pathlib import Path

BASE_URL = "https://tymm.meb.gov.tr"

# Beceri çerçevesini taşıyan sayfalar. Sıra çıktıdaki set sırasını belirler:
# önce programlar arası (çapraz) beceriler, sonra disipline özgü alan becerileri.
PAGE_PATHS = [
    "/beceriler/kavramsal-beceriler",
    "/beceriler/sosyal-duygusal-ogrenme-becerileri",
    "/beceriler/egilimler",
    "/beceriler/erdem-deger-eylem-cercevesi",
    "/beceriler/okuryazarlik-becerileri",
    "/beceriler/fiziksel-beceriler",
    "/beceriler/turkce-alan-becerileri",
    "/beceriler/matematik-alan-becerileri",
    "/beceriler/fen-bilimleri-alan-becerileri",
    "/beceriler/sosyal-bilimler-alan-becerileri",
    "/beceriler/sanat-alan-becerileri",
    "/beceriler/beden-egitimi-oyun-ve-spor-alan-becerileri",
    "/beceriler/bilisim-teknolojileri-ve-yazilim-alan-becerileri",
    "/beceriler/tasarim-alan-becerileri",
    "/beceriler/din-egitimi-ve-ogretimi-alan-becerileri",
    "/beceriler/yabanci-dil-alan-becerileri",
]

USER_AGENT = "Mozilla/5.0 (compatible; tayan-tymm-fetch/1.0)"
REQUEST_TIMEOUT_S = 30

# Kod deseni. Ön ek 1-8 harf arasında değişir (KB, E, D, OB, SDB, TAB, MAB,
# FBAB, SBAB, YDAB, YDDB, BTYAB, BEOSAB, TSRMAB) — uzunluğu SABİTLEME.
CODE_RE = re.compile(r"^([A-ZÇĞİÖŞÜ]{1,8}\d[\d.]*)\.\s*(.*)$")
# "SB8" ve hatalı "SB.8" biçimlerinin ikisini de kabul et (kaynakta ikisi de var).
# Kaynakta üç yazım da geçiyor: "SB8", hatalı "SB.8", ve boşluklu ". SB5".
COMPONENT_RE = re.compile(r"^([A-ZÇĞİÖŞÜ]{1,8}[\d.]*\.)\s*SB\.?\s*(\d+)\.\s*(.*)$")
# Alan sayfalarında açıklama kod satırından ÖNCE, kodu parantezde taşıyan bir
# başlıkla gelir: "Hareketi Sergileme Becerisi (BEOSAB1)".
HEADING_RE = re.compile(r"^(.+?)\s*\(([A-ZÇĞİÖŞÜ]{1,8}\d[\d.]*)\)\s*$")
PREFIX_RE = re.compile(r"^([A-ZÇĞİÖŞÜ]{1,8})")
TITLE_ABBR_RE = re.compile(r"\(([A-ZÇĞİÖŞÜ]{2,8})\)\s*$")
SET_HEADER_RE = re.compile(r"^([A-ZÇĞİÖŞÜ]{2,8})\.\s*(\D.*)$")
TITLE_SUFFIX = " - Türkiye Yüzyılı Maarif Modeli"

COMPONENT_MARKER = "Süreç bileşenleri"
FOOTER_MARKER = "Türkiye Yüzyılı Maarif Modeli"
READ_MORE_MARKER = "Daha fazla oku"


def fetch(url: str) -> str:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=REQUEST_TIMEOUT_S) as response:
        return response.read().decode("utf-8", errors="replace")


def to_lines(markup: str) -> list[str]:
    """HTML'i görünür metin satırlarına indirger."""
    without_scripts = re.sub(r"(?is)<(script|style).*?</\1>", " ", markup)
    text = html.unescape(re.sub(r"(?s)<[^>]+>", "\n", without_scripts))
    return [line.strip() for line in text.splitlines() if line.strip()]


def content_slice(lines: list[str]) -> list[str]:
    """Navigasyon ve altbilgiyi atar, yalnız içerik gövdesini bırakır.

    Gövde "Daha fazla oku" bağlantısından sonra başlar; altbilgi, ilk kod
    satırından SONRA gelen ilk FOOTER_MARKER ile başlar.
    """
    start = 0
    if READ_MORE_MARKER in lines:
        start = lines.index(READ_MORE_MARKER) + 1

    body = lines[start:]
    first_code = next((i for i, line in enumerate(body) if CODE_RE.match(line)), None)
    if first_code is None:
        return body

    for i in range(first_code, len(body)):
        if body[i] == FOOTER_MARKER:
            return body[:i]
    return body


def page_title_of(lines: list[str]) -> str:
    for line in lines:
        if line and line != FOOTER_MARKER and "T.C." not in line:
            return line.removesuffix(TITLE_SUFFIX).strip()
    return ""


def level_of(code: str) -> int:
    """Kodun hiyerarşi derinliği: KB2 -> 1, KB2.1 -> 2, TSRMAB1.1 -> 2."""
    return len(code.split("."))


def parent_of(code: str) -> str | None:
    parts = code.split(".")
    return ".".join(parts[:-1]) if len(parts) > 1 else None


def is_descendant(code: str, ancestor: str) -> bool:
    return code.startswith(ancestor + ".")


def clean_name(name: str, set_code: str) -> str:
    """Ad sonundaki tekrar eden kısaltmayı atar: 'X Becerisi (YDAB2)' -> 'X Becerisi'."""
    return re.sub(r"\s*\([A-ZÇĞİÖŞÜ]{2,8}\d*\)\s*$", "", name).strip()


def parse_page(path: str, lines: list[str]) -> list[dict]:
    """Bir sayfayı ayrıştırır. Sayfa birden fazla set taşıyabilir (ör. YDAB + YDDB)."""
    body = content_slice(lines)
    page_title = page_title_of(lines)

    sets: dict[str, dict] = {}
    by_code: dict[str, dict] = {}
    set_names: dict[str, str] = {}
    current: dict | None = None
    in_components = False

    def ensure_set(code: str) -> dict:
        if code not in sets:
            sets[code] = {
                "code": code,
                "name": set_names.get(code, page_title),
                "source_url": BASE_URL + path,
                "skills": [],
            }
        return sets[code]

    anomalies: list[dict] = []
    heading_nodes: dict[str, str] = {}
    last_heading: tuple[str, str] | None = None
    pending_descriptions: dict[str, str] = {}
    awaiting_description_for: str | None = None

    for line in body:
        heading = HEADING_RE.match(line)
        if heading and not CODE_RE.match(line):
            awaiting_description_for = heading.group(2)
            last_heading = (heading.group(2), heading.group(1).strip())
            heading_nodes[heading.group(2)] = heading.group(1).strip()
            continue

        header = SET_HEADER_RE.match(line)
        if header and not CODE_RE.match(line):
            code, name = header.group(1), header.group(2).strip()
            set_names[code] = name
            if code in sets:
                sets[code]["name"] = name
            current, in_components = None, False
            continue

        # Süreç bileşeni iki biçimde gelir ve ikisi de bileşendir:
        #   KB2.1.SB1. ...  (.SB<k> ekli)
        #   D1.1.1. ...     ("Süreç bileşenleri" başlığı altında, düz numara)
        # DİKKAT: kodu COMPONENT_RE'den al. CODE_RE aynı satırda "KB2.1" verir
        # (harf olmayan kısımda durur) ve o kodla yapılan alt-düğüm sınaması
        # yanlışlıkla başarısız olur.
        component = COMPONENT_RE.match(line)
        if component and current is not None:
            code = component.group(1) + "SB" + component.group(2)
            text = component.group(3).strip()
            if is_descendant(code, current["code"]):
                current["components"].append({"code": code, "text": text})
                continue
            # Kod, içinde bulunduğu düğümü göstermiyor (kaynakta yazım hatası).
            # Sırayı esas al, ama kodu OLDUĞU GİBİ sakla ve anomaliyi bildir.
            current["components"].append({"code": code, "text": text})
            anomalies.append({
                "kind": "component_code_mismatch",
                "code": code,
                "expected_parent": current["code"],
                "source_line": line,
                "source_url": BASE_URL + path,
            })
            continue

        node = CODE_RE.match(line)
        if node:
            code, name = node.group(1), node.group(2).strip()
            prefix = PREFIX_RE.match(code).group(1)

            if in_components and current is not None and is_descendant(code, current["code"]):
                current["components"].append({"code": code, "text": name})
                continue

            entry = {
                "code": code,
                "name": clean_name(name, prefix),
                "description": pending_descriptions.get(code, ""),
                "parent": parent_of(code),
                "level": level_of(code),
                "components": [],
            }
            if last_heading is not None and (
                code == last_heading[0] or is_descendant(code, last_heading[0])
            ):
                entry["context"] = last_heading[1]
            ensure_set(prefix)["skills"].append(entry)
            by_code[code] = entry
            current, in_components = entry, False
            awaiting_description_for = None
            continue

        if line == COMPONENT_MARKER:
            in_components = True
            continue

        if awaiting_description_for is not None:
            pending_descriptions.setdefault(awaiting_description_for, line)
            awaiting_description_for = None
            continue

        if current is not None and not in_components and not current["description"]:
            current["description"] = line

    # Bazı üst düzey beceriler (TAB1, MAB1, SBAB2, YDDB1 …) sayfada YALNIZ
    # başlıkta geçer, kendi kod satırları yoktur. Ders programları bu kodlara
    # atıf yaptığı için düğüm olarak var edilmeleri şart; yoksa ders→beceri
    # birleştirmesinde binlerce eşleme karşılıksız kalır.
    for code, name in heading_nodes.items():
        if code in by_code:
            continue
        prefix_match = PREFIX_RE.match(code)
        if not prefix_match:
            continue
        prefix = prefix_match.group(1)
        if prefix not in sets:
            continue
        entry = {
            "code": code,
            "name": clean_name(name, prefix),
            "description": pending_descriptions.get(code, ""),
            "parent": parent_of(code),
            "level": level_of(code),
            "components": [],
            "from_heading": True,
        }
        sets[prefix]["skills"].append(entry)
        by_code[code] = entry

    for code, item in sets.items():
        item["name"] = set_names.get(code, item["name"])
        item["anomalies"] = [a for a in anomalies if a["code"].startswith(code)]

    # Sayfa hiç kod içermiyorsa (ör. Fiziksel Beceriler) yine de kaydını tut.
    if not sets:
        return [{
            "code": "",
            "name": page_title,
            "source_url": BASE_URL + path,
            "skills": [],
        }]

    return list(sets.values())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cache-dir",
        type=Path,
        help="HTML'i buradan oku/buraya yaz; ağa tekrar çıkmaz.",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("data/tymm"),
        help="Çıktı dizini (varsayılan: data/tymm)",
    )
    args = parser.parse_args()

    if args.cache_dir:
        args.cache_dir.mkdir(parents=True, exist_ok=True)

    sets = []
    for path in PAGE_PATHS:
        cache_file = None
        if args.cache_dir:
            cache_file = args.cache_dir / (path.strip("/").replace("/", "_") + ".html")

        if cache_file and cache_file.exists():
            markup = cache_file.read_text(encoding="utf-8", errors="replace")
        else:
            url = BASE_URL + path
            print(f"çekiliyor: {url}", file=sys.stderr)
            try:
                markup = fetch(url)
            except Exception as error:  # ağ hatası tek sayfayı düşürmemeli
                print(f"  HATA {path}: {error}", file=sys.stderr)
                return 1
            if cache_file:
                cache_file.write_text(markup, encoding="utf-8")

        for parsed in parse_page(path, to_lines(markup)):
            sets.append(parsed)
            print(
                f"  {parsed['code'] or '(kodsuz)':<8} "
                f"beceri={len(parsed['skills']):<4} "
                f"bileşen={sum(k['components'].__len__() for k in parsed['skills'])}",
                file=sys.stderr,
            )

    # Aynı set birden fazla sayfada geçebilir: alan sayfaları kendi becerilerinin
    # yanında atıfta bulundukları KB kodlarını da basar. Kod bazında birleştir,
    # beceriyi koduna göre tekilleştir, hangi sayfalarda geçtiğini kaydet.
    merged: dict[str, dict] = {}
    for item in sets:
        key = item["code"]
        target = merged.get(key)
        if target is None:
            merged[key] = {
                "code": key,
                "name": item["name"],
                "source_urls": [item["source_url"]],
                "skills": list(item["skills"]),
            }
            continue
        if item["source_url"] not in target["source_urls"]:
            target["source_urls"].append(item["source_url"])
        seen = {k["code"] for k in target["skills"]}
        for skill in item["skills"]:
            if skill["code"] in seen:
                continue
            seen.add(skill["code"])
            target["skills"].append(skill)

    for item in merged.values():
        item["skills"].sort(key=lambda k: [
            int(part) if part.isdigit() else part
            for part in re.split(r"[.]", re.sub(r"^[A-ZÇĞİÖŞÜ]+", "", k["code"]))
        ] if k["code"] else [])

    sets = list(merged.values())

    fetched_at = datetime.date.today().isoformat()
    duplicate_codes = []
    for item in sets:
        counts = Counter(k["code"] for k in item["skills"])
        for code, n in sorted(counts.items()):
            if n > 1:
                duplicate_codes.append({
                    "set": item["code"],
                    "code": code,
                    "count": n,
                    "contexts": [
                        k.get("context", "") for k in item["skills"] if k["code"] == code
                    ],
                })

    document = {
        "report": {
            "sets": len(sets),
            "skills": sum(len(k["skills"]) for k in sets),
            "components": sum(len(c["components"]) for k in sets for c in k["skills"]),
            # Kaynakta aynı kod birden fazla kez tanımlanabiliyor (ör. YDDB1.1
            # üç öğretim yaklaşımı için). Hiçbiri atılmaz; ayırt edici bilgi
            # "context" alanındadır.
            "duplicate_codes": duplicate_codes,
        },
        "source": {
            "name": "Türkiye Yüzyılı Maarif Modeli — Beceri Çerçevesi",
            "base_url": BASE_URL,
            "fetched_at": fetched_at,
            "generator": "scripts/fetch_tymm_beceriler.py",
        },
        "sets": sets,
    }

    args.out_dir.mkdir(parents=True, exist_ok=True)
    out_json = args.out_dir / "beceriler.json"
    out_json.write_text(
        json.dumps(document, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    total_skills = sum(len(s["skills"]) for s in sets)
    total_components = sum(len(k["components"]) for s in sets for k in s["skills"])

    sources = [
        "# TYMM beceri çerçevesi — kaynaklar",
        "",
        f"Çekim tarihi: {fetched_at}",
        "",
        "`data/tymm/beceriler.json` bu sayfalardan üretildi. Elle düzenlenmez;",
        "yeniden üretmek için `python3 scripts/fetch_tymm_beceriler.py`.",
        "",
        "| Set | Ad | Beceri | Bileşen | Kaynak |",
        "|---|---|---:|---:|---|",
    ]
    for item in sets:
        components = sum(len(k["components"]) for k in item["skills"])
        sources.append(
            f"| `{item['code'] or '—'}` | {item['name']} | {len(item['skills'])} "
            f"| {components} | {' '.join(item['source_urls'])} |"
        )
    sources += [
        f"| | **Toplam** | **{total_skills}** | **{total_components}** | |",
        "",
        "## Notlar",
        "",
        "- Alan beceri sayfaları başka setlerin kodlarına atıf yapar (ör. matematik",
        "  sayfasındaki `KB` kodları). Her düğüm KENDİ ön ekinin setine yazılır;",
        "  aynı set birden çok sayfada geçerse `source_urls` hepsini listeler ve",
        "  beceriler koda göre tekilleştirilir.",
        "- Kaynakta aynı kod birden fazla kez tanımlanabiliyor (ör. `YDDB1.1` üç",
        "  öğretim yaklaşımı için). Hiçbiri atılmaz; ayırt edici bilgi `context`",
        "  alanındadır ve tamamı `report.duplicate_codes` altında listelenir.",
        "- Kaynakta süreç bileşeni kodları üç ayrı yazımla geçiyor: `SB8`, hatalı",
        "  `SB.8` ve boşluklu `. SB5`. Ayrıştırıcı üçünü de tanır, kodu `SB8`",
        "  biçimine normalleştirir.",
        "- Açıklaması boş düğümlerin çoğunda kaynakta da açıklama yoktur (ör. `OB`,",
        "  `SBAB`); bu bir çekim kaybı değildir.",
        "- Fiziksel Beceriler sayfası kodlanmış liste içermez; düz anlatımdır.",
        "- Kod ön ekleri 1-6 harf arasında değişir (`E`, `KB`, `SDB`, `BEOSAB`,",
        "  `TSRMAB`); ayrıştırıcı ön ek uzunluğunu sabitlemez.",
    ]
    (args.out_dir / "SOURCES.md").write_text("\n".join(sources) + "\n", encoding="utf-8")

    print(
        f"\nyazıldı: {out_json} — {len(sets)} set, {total_skills} beceri, "
        f"{total_components} süreç bileşeni",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
