#!/usr/bin/env python3
"""TYMM kaynaklarının değişip değişmediğini bildirir.

MEB sayfaları ve PDF'leri zamanla güncellenir; PDF'ler repoda tutulmaz.
Bu script kaynakları çeker, parmak izlerini `data/tymm/sources.lock.json`
ile karşılaştırır ve NE DEĞİŞTİĞİNİ söyler. Hiçbir veri dosyasının üzerine
yazmaz — güncelleme kararı insana aittir.

İki parmak izi tutulur, çünkü tek hash yanıltır:

  sha256_raw   ham bayt. Site yeniden yayınlandığında CSS sürüm damgası
               (output1.css?v=...) değiştiği için içerik aynı kalsa bile döner.
  sha256_text  görünür metin (script/style atılmış, boşluk sadeleştirilmiş).
               Asıl içerik sinyali budur.

Yalnız raw değiştiyse: kozmetik, görmezden gel.
text de değiştiyse: içerik değişmiş, çekme script'lerini yeniden çalıştır.

Kullanım:
    python3 scripts/check_tymm_sources.py            # karşılaştır ve raporla
    python3 scripts/check_tymm_sources.py --update   # kilidi tazele
    python3 scripts/check_tymm_sources.py --deep     # 111 ders grafiğini de tara

Çıkış kodu: içerik değişmişse veya kaynak kaybolmuşsa 1, aksi hâlde 0.
Böylece bir cron işi veya CI adımı olarak kullanılabilir.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import html
import json
import re
import sys
import urllib.parse
import urllib.request
from pathlib import Path

BASE_URL = "https://tymm.meb.gov.tr"
USER_AGENT = "Mozilla/5.0 (compatible; tayan-tymm-check/1.0)"
REQUEST_TIMEOUT_S = 60
LOCK_PATH = Path("data/tymm/sources.lock.json")

# Beceri çerçevesi sayfaları — fetch_tymm_beceriler.py ile aynı liste olmalı.
BECERI_PATHS = [
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

KILAVUZ_PATHS = [
    "/upload/kilavuz/coktan-secmeli-soru-yazim-kilavuzu.pdf",
    "/assets/pdf/modul-5.pdf",
    "/assets/pdf/modul-5-yayin-2.pdf",
    "/upload/brosur/ortak_metin.pdf",
    # Menüde duruyor ama sunucu 500 veriyor. Listede KALIYOR: geri gelirse
    # rapor bunu "YENİDEN YAYINDA" diye gösterir.
    "/upload/kilavuz/performans-gelisim-cercevesi.pdf",
]

DERS_ENDPOINT = "/Ders/GetDerslerBySinif"
CHART_ENDPOINT = "/Chart/GetStackCharts"
KADEME_IDS = [2, 3, 9, 11]
SINIF_ID_RANGE = list(range(1, 25)) + [-1]


def fetch(url: str) -> bytes | None:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(request, timeout=REQUEST_TIMEOUT_S) as response:
            return response.read()
    except Exception:
        return None


def visible_text(payload: bytes) -> bytes:
    markup = payload.decode("utf-8", errors="replace")
    without_scripts = re.sub(r"(?is)<(script|style).*?</\1>", " ", markup)
    text = html.unescape(re.sub(r"(?s)<[^>]+>", " ", without_scripts))
    return re.sub(r"\s+", " ", text).strip().encode("utf-8")


def digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def fingerprint(path: str, is_binary: bool) -> dict:
    payload = fetch(BASE_URL + path)
    if payload is None:
        return {"status": "missing"}
    entry = {
        "status": "ok",
        "bytes": len(payload),
        "sha256_raw": digest(payload),
    }
    if not is_binary:
        entry["sha256_text"] = digest(visible_text(payload))
    return entry


def course_urls() -> list[str]:
    """Ders dizinini tarar; ders sayısı ve url listesi de bir değişim sinyalidir."""
    urls: dict[str, None] = {}
    for kademe in KADEME_IDS:
        for sinif_id in SINIF_ID_RANGE:
            query = urllib.parse.urlencode({"sinifId": sinif_id, "kademe": kademe})
            payload = fetch(f"{BASE_URL}{DERS_ENDPOINT}?{query}")
            if not payload:
                continue
            try:
                data = json.loads(payload.decode("utf-8", errors="replace"))
            except json.JSONDecodeError:
                continue
            for item in data or []:
                urls[item["url"]] = None
    return sorted(urls)


def collect(deep: bool) -> dict:
    sources: dict[str, dict] = {}
    for path in BECERI_PATHS:
        sources[path] = fingerprint(path, is_binary=False)
    for path in KILAVUZ_PATHS:
        sources[path] = fingerprint(path, is_binary=True)

    urls = course_urls()
    sources[DERS_ENDPOINT] = {
        "status": "ok" if urls else "missing",
        "course_count": len(urls),
        "sha256_text": digest("\n".join(urls).encode("utf-8")),
    }

    if deep:
        for url in urls:
            query = urllib.parse.urlencode({"url": url})
            payload = fetch(f"{BASE_URL}{CHART_ENDPOINT}?{query}")
            key = f"{CHART_ENDPOINT}?url={url}"
            sources[key] = (
                {"status": "missing"}
                if payload is None
                else {"status": "ok", "bytes": len(payload), "sha256_raw": digest(payload)}
            )
    return sources


def compare(old: dict, new: dict) -> tuple[list[str], list[str], list[str]]:
    """(içerik değişimi, kozmetik değişim, yapısal değişim) döndürür."""
    content, cosmetic, structural = [], [], []
    for key, entry in new.items():
        previous = old.get(key)
        if previous is None:
            structural.append(f"YENİ           {key}")
            continue
        if entry["status"] != previous.get("status"):
            label = "YENİDEN YAYINDA" if entry["status"] == "ok" else "KAYBOLDU"
            structural.append(f"{label:<14} {key}")
            continue
        if entry["status"] != "ok":
            continue
        if "course_count" in entry and entry["course_count"] != previous.get("course_count"):
            structural.append(
                f"DERS SAYISI    {key}: {previous.get('course_count')} -> {entry['course_count']}"
            )
        text_key = "sha256_text" if "sha256_text" in entry else "sha256_raw"
        if entry.get(text_key) != previous.get(text_key):
            content.append(f"İÇERİK         {key}")
        elif entry.get("sha256_raw") != previous.get("sha256_raw"):
            cosmetic.append(f"kozmetik       {key}")
    for key in old:
        if key not in new:
            structural.append(f"LİSTEDEN ÇIKTI {key}")
    return content, cosmetic, structural


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--update", action="store_true", help="Kilidi bulunan durumla tazele.")
    parser.add_argument("--deep", action="store_true", help="111 ders grafiğini de parmakla.")
    parser.add_argument("--lock", type=Path, default=LOCK_PATH)
    args = parser.parse_args()

    print("kaynaklar çekiliyor...", file=sys.stderr)
    new_sources = collect(args.deep)

    if not args.lock.exists():
        args.lock.parent.mkdir(parents=True, exist_ok=True)
        args.lock.write_text(
            json.dumps(
                {
                    "checked_at": datetime.date.today().isoformat(),
                    "deep": args.deep,
                    "sources": new_sources,
                },
                ensure_ascii=False,
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
        print(f"kilit oluşturuldu: {args.lock} ({len(new_sources)} kaynak)", file=sys.stderr)
        return 0

    lock = json.loads(args.lock.read_text(encoding="utf-8"))
    content, cosmetic, structural = compare(lock.get("sources", {}), new_sources)

    print(f"\nson kontrol: {lock.get('checked_at')}   kaynak: {len(new_sources)}")
    for line in structural + content:
        print("  " + line)
    if cosmetic:
        print(f"  ({len(cosmetic)} kozmetik değişim — içerik aynı, yok sayılabilir)")
    if not (structural or content or cosmetic):
        print("  değişiklik yok.")

    if args.update:
        lock["checked_at"] = datetime.date.today().isoformat()
        lock["deep"] = args.deep or lock.get("deep", False)
        lock["sources"] = new_sources
        args.lock.write_text(json.dumps(lock, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"\nkilit tazelendi: {args.lock}", file=sys.stderr)
    elif content or structural:
        print(
            "\nİçerik değişmiş. Sırayla: çekme script'lerini çalıştır, üretilen veriyi\n"
            "gözden geçir, sonra --update ile kilidi tazele.",
            file=sys.stderr,
        )

    return 1 if (content or structural) else 0


if __name__ == "__main__":
    sys.exit(main())
