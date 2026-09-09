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

sys.path.insert(0, str(Path(__file__).resolve().parent))
from tymm_beceri_lines import (  # noqa: E402
    normalize_ws,
    split_code_line,
    split_indicator,
)

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
# Kodsuz gruplama kademesi: "Anlama Becerileri", "1.Alımlayıcı Beceriler".
# Hangi becerinin hangi öbeğe düştüğünü SADECE bu satır söylüyor; kodda iz
# yok. Cümle olmadığından gövdede nokta aramıyoruz.
GROUP_RE = re.compile(r"^(?:\d+\.)?\s*([A-ZÇĞİÖŞÜ][^.]{2,45}Becerile(?:r|ri))$")
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
    first_code = next((i for i, line in enumerate(body) if split_code_line(line)), None)
    if first_code is None:
        return body

    for i in range(first_code, len(body)):
        if body[i] == FOOTER_MARKER:
            return body[:i]
    return body


def page_title_of(lines: list[str]) -> str:
    for line in lines:
        if line and line != FOOTER_MARKER and "T.C." not in line:
            title = normalize_ws(line.removesuffix(TITLE_SUFFIX))
            # "Türkçe Alan Becerileri (TAB)" -> kod, set alanında zaten var.
            return TITLE_ABBR_RE.sub("", title).strip()
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
    heading_labels: dict[str, list[str]] = {}
    heading_groups: dict[str, str] = {}
    last_heading: tuple[str, str] | None = None
    pending_descriptions: dict[str, str] = {}
    awaiting_description_for: str | None = None
    group_label: str | None = None

    def describe(target: str, text: str) -> None:
        """Açıklamayı BİRİKTİRİR. Tek satır saklamak 9 düğümde açıklamayı ilk
        parçada kesiyordu ("... kendini korumayı" / "kapsar.")."""
        prior = pending_descriptions.get(target, "")
        pending_descriptions[target] = normalize_ws(prior + " " + text)

    for raw_line in body:
        line = normalize_ws(raw_line)
        heading = HEADING_RE.match(line)
        if heading and not split_code_line(line):
            code, label = heading.group(2), heading.group(1).strip()
            awaiting_description_for = code
            last_heading = (code, label)
            heading_nodes.setdefault(code, label)
            if group_label is not None:
                heading_groups.setdefault(code, group_label)
            if label not in heading_labels.setdefault(code, []):
                heading_labels[code].append(label)
            continue

        # "Yabancı Dil Alan Becerileri (YDAB)" — setin GERÇEK adı. Sayfa
        # başlığı iki seti birden anıyor ("... ve Yabancı Dil Destekleyici
        # Beceriler"), o yüzden YDAB yanlış adla çıkıyordu. TITLE_ABBR_RE bu iş
        # için tanımlanmıştı ama hiç çağrılmıyordu.
        abbr = TITLE_ABBR_RE.search(line)
        if abbr and not split_code_line(line) and not HEADING_RE.match(line):
            code = abbr.group(1)
            set_names[code] = TITLE_ABBR_RE.sub("", line).strip()
            if code in sets:
                sets[code]["name"] = set_names[code]
            awaiting_description_for = None
            continue

        header = SET_HEADER_RE.match(line)
        if header and not split_code_line(line):
            code, name = header.group(1), header.group(2).strip()
            set_names[code] = name
            if code in sets:
                sets[code]["name"] = name
            current, in_components = None, False
            awaiting_description_for = None
            continue

        # Süreç bileşeni iki biçimde gelir ve ikisi de bileşendir:
        #   KB2.1.SB1. ...  (.SB<k> ekli)
        #   D1.1.1. ...     ("Süreç bileşenleri" başlığı altında, düz numara)
        # DİKKAT: kodu COMPONENT_RE'den al. CODE_RE aynı satırda "KB2.1" verir
        # (harf olmayan kısımda durur) ve o kodla yapılan alt-düğüm sınaması
        # yanlışlıkla başarısız olur.
        # GÖSTERGE KADEMESİ. SDB sayfası iki sütunlu tablo; sağ hücredeki
        # <ul class="indicator-list"> ögeleri süreç bileşeninin ALTINDA ayrı
        # bir kademedir. Etiketler satıra düzleştirilince her gösterge üst
        # bileşenin kodunu taşıyor gibi görünüyordu: 36 bileşen 225 kayda
        # şişmiş, G kodları tamamen kaybolmuştu.
        indicator_code, indicator_text = split_indicator(line)
        if indicator_code and current is not None:
            awaiting_description_for = None
            owner_code = indicator_code.rsplit(".", 1)[0]
            for existing in current["components"]:
                if existing["code"] == owner_code:
                    existing.setdefault("indicators", []).append(
                        {"code": indicator_code, "text": indicator_text}
                    )
                    break
            else:
                anomalies.append({
                    "kind": "indicator_without_component",
                    "code": indicator_code,
                    "expected_parent": owner_code,
                    "source_line": line,
                })
            continue

        component = COMPONENT_RE.match(line)
        if component and current is not None:
            awaiting_description_for = None
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

        # AYIRAÇ NOKTA DA OLABİLİR BOŞLUK DA:
        #   KB2.8.Sorgulama Becerisi            (nokta)
        #   DAB3.1 Dinî Kavramları Ayırt Etme   (boşluk, akordiyon başlığı)
        # Yalnız noktayı bekleyen desen ikinci biçimdeki düğümleri atlıyordu.
        node = split_code_line(line)
        if node:
            code, name = node
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
            if group_label is not None:
                entry["group"] = group_label
            ensure_set(prefix)["skills"].append(entry)
            by_code[code] = entry
            current, in_components = entry, False
            awaiting_description_for = None
            continue

        if line == COMPONENT_MARKER:
            in_components = True
            awaiting_description_for = None
            continue

        group = GROUP_RE.match(line)
        if group and line != page_title:
            group_label = group.group(1).strip()
            awaiting_description_for = None
            continue

        if awaiting_description_for is not None:
            describe(awaiting_description_for, line)
            continue

        if current is not None and not in_components:
            current["description"] = normalize_ws(current["description"] + " " + line)

    # KANONİK AD. Kod satırı adı kısaltıyor ("FBAB1.Bilimsel Gözlem"), başlık
    # tam adı taşıyor ("Bilimsel Gözlem Becerisi (FBAB1)"). Ders programları TAM
    # adla atıf yapıyor, o yüzden eşleme kaçıyordu — 32 beceride.
    # Yalnız başlık, mevcut adı UZATIYORSA değiştir: "SDB2.Sosyal Yaşam
    # Becerileri" gibi kodu tekrarlayan başlıklar adı bozmasın.
    for code, entry in by_code.items():
        for label in heading_labels.get(code, []):
            if label != entry["name"] and entry["name"] and label.startswith(entry["name"]):
                entry["name"] = label
                break
        # Kaynak aynı kodu birden çok başlıkla anıyor (YDAB'de üç yaklaşım:
        # Bütüncül / Yarı Bütüncül-Yarı Tümevarımsal / Tümevarımsal). Düğüm
        # tek; diğer etiketler düşürülmek yerine yazılı kalır.
        others = [l for l in heading_labels.get(code, []) if l != entry["name"]]
        if others:
            entry["also_titled"] = others

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
        # Yalnız başlıkta geçen düğüm de gruplama kademesinin altındadır;
        # TAB1 "Anlama Becerileri" öbeğinde ama kod satırı olmadığı için
        # döngü içinde grup almıyordu.
        if code in heading_groups:
            entry["group"] = heading_groups[code]
        others = [l for l in heading_labels.get(code, []) if l != entry["name"]]
        if others:
            entry["also_titled"] = others
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


def merge_sets(sets: list[dict]) -> tuple[dict[str, dict], dict]:
    """Sayfa başına ayrıştırılmış setleri kod bazında birleştirir.

    Aynı set birden fazla sayfada geçer: alan sayfaları kendi becerilerinin
    yanında atıfta bulundukları KB kodlarını da basar. Kod bazında birleştir,
    hangi sayfalarda geçtiğini kaydet.

    DÜŞÜRME YOK. Aynı kodun ikinci geçişi eskiden sessizce atılıyordu; ölçüm:
    34 düğüm ve 99 süreç bileşeni. Bunların 8'inde bileşen metni kanonik
    sürümden FARKLIYDI — alan sayfası KB bileşenini kendi disiplinine göre
    yeniden ifade ediyor ("Tarihsel kanıtı sorgulanacak duruma bağlamak").
    Metin farklıysa varyant olarak saklanır, aynıysa yalnız sayılır.
    """
    merged: dict[str, dict] = {}
    report = {"repeated_nodes": 0, "variant_nodes": 0}
    shapes: dict[int, set[tuple[str, ...]]] = {}

    for item in sets:
        key = item["code"]
        target = merged.get(key)
        if target is None:
            merged[key] = {
                "code": key,
                "name": item["name"],
                "source_urls": [item["source_url"]],
                "skills": list(item["skills"]),
                # Anomaliler birleştirmede DÜŞÜYORDU: parse_page üretiyor ama
                # main() kopyalamıyordu, çıktıda "anomalies" hiç görünmüyordu.
                "anomalies": list(item.get("anomalies", [])),
            }
            continue

        if item["source_url"] not in target["source_urls"]:
            target["source_urls"].append(item["source_url"])
        target["anomalies"].extend(item.get("anomalies", []))

        by_code = {}
        for existing in target["skills"]:
            by_code.setdefault(existing["code"], existing)

        for skill in item["skills"]:
            canonical = by_code.get(skill["code"])
            if canonical is None:
                by_code[skill["code"]] = skill
                target["skills"].append(skill)
                continue
            # Aynı biçim ikinci kez gelirse yeni bilgi yok. Sayfa KB2.8'i üç
            # kez basabiliyor; üçünü de saklamak varyantı gürültüye boğar.
            shape = tuple(c["text"] for c in skill["components"])
            seen_shapes = shapes.setdefault(id(canonical), {
                tuple(c["text"] for c in canonical["components"])
            })
            if shape in seen_shapes:
                report["repeated_nodes"] += 1
                continue
            seen_shapes.add(shape)
            variant = {
                "source_url": item["source_url"],
                "name": skill["name"],
                "components": skill["components"],
            }
            if "context" in skill:
                variant["context"] = skill["context"]
            canonical.setdefault("variants", []).append(variant)
            report["variant_nodes"] += 1

    return merged, report


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

    merged, merge_report = merge_sets(sets)

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
            # Kaynağın kendi tutarsızlıkları: yanlış üst koda yazılmış süreç
            # bileşenleri, göstergesi sahipsiz kalan satırlar. Sayfa başına
            # üretiliyordu ama birleştirme kopyalamadığı için çıktıya HİÇ
            # ulaşmıyordu; tüketici yanlış kodu uyarısız doğru sanıyordu.
            "anomalies": sum(len(s.get("anomalies", [])) for s in sets),
            # Aynı kodun başka sayfadaki geçişi: metni aynıysa "repeated",
            # farklıysa düğümün "variants" alanına yazıldı.
            "repeated_nodes": merge_report["repeated_nodes"],
            "variant_nodes": merge_report["variant_nodes"],
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
        "- Alan sayfaları ödünç aldıkları `KB` kodlarının süreç bileşenlerini KENDİ",
        "  disiplinlerine göre yeniden ifade ediyor (`Toplanan bilgiler üzerinde` /",
        "  `üzerinden çıkarım yapmak`). Bu geçişler düşürülmez: bileşen metni",
        "  farklıysa düğümün `variants` alanına yazılır, aynıysa yalnız sayılır",
        "  (`report.repeated_nodes`).",
        "- Kod satırı adı kısaltabiliyor (`FBAB1.Bilimsel Gözlem`); tam ad sayfa",
        "  başlığındadır (`Bilimsel Gözlem Becerisi (FBAB1)`). Başlık adı mevcut adı",
        "  UZATIYORSA kanonik ad odur; kalan başlıklar `also_titled` altındadır.",
        "- Kodsuz gruplama kademesi (`Anlama Becerileri`, `Alımlayıcı Beceriler`)",
        "  düğümün `group` alanına yazılır; kaynakta kodda izi yoktur.",
        "- Kaynağın kendi tutarsızlıkları `anomalies` altındadır: bir süreç bileşeni",
        "  içinde bulunduğu düğümden başka bir kod taşıyor (`DAB4.4` altında",
        "  `DAB4.3.SB4`). Kod OLDUĞU GİBİ saklanır, sıra esas alınır, anomali",
        "  bildirilir.",
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
