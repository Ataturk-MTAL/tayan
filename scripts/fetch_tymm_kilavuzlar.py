#!/usr/bin/env python3
"""TYMM ölçme-değerlendirme kılavuzlarını indirir ve metne çevirir.

Bu belgeler VERİ değil KURAL taşır: soru yazım ölçütleri, hata örnekleri,
soru tipi bazında puanlama esasları. Mekanik olarak tam ayrıştırılamazlar;
`data/tymm/olcme-rehberi.json` bunlardan elle çıkarılmış yapılandırılmış
özettir ve kaynak sayfa numaralarını taşır.

PDF'ler repoya girmez (toplam ~55 MB); bu script onları yerel bir dizine
indirir. Metin çıkarımı için `pdftotext` (poppler) gerekir.

Kullanım:
    python3 scripts/fetch_tymm_kilavuzlar.py                 # .tymm-pdf/ altına
    python3 scripts/fetch_tymm_kilavuzlar.py --out-dir /tmp/x
    python3 scripts/fetch_tymm_kilavuzlar.py --no-text       # yalnız indir
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path

BASE_URL = "https://tymm.meb.gov.tr"
USER_AGENT = "Mozilla/5.0 (compatible; tayan-tymm-fetch/1.0)"
REQUEST_TIMEOUT_S = 120

# (dosya adı, yol, ne olduğu)
DOCUMENTS = [
    (
        "coktan-secmeli-soru-yazim-kilavuzu.pdf",
        "/upload/kilavuz/coktan-secmeli-soru-yazim-kilavuzu.pdf",
        "Bağlam temelli çoktan seçmeli soru yazım kılavuzu (123 s.)",
    ),
    (
        "modul-5.pdf",
        "/assets/pdf/modul-5.pdf",
        "5. Modül — Öğrenme Kanıtları (Ölçme ve Değerlendirme) 1 (62 s.)",
    ),
    (
        # 6. modül sayfası (/dokuman/6/5-modul) bu dosyaya işaret eder;
        # adı "modul-6.pdf" DEĞİLDİR — o ayrı ve başka bir belgedir.
        "modul-5-yayin-2.pdf",
        "/assets/pdf/modul-5-yayin-2.pdf",
        "5. Modül — Ölçme ve Değerlendirme Uygulamaları 2 (142 s.)",
    ),
    (
        "ortak_metin.pdf",
        "/upload/brosur/ortak_metin.pdf",
        "Öğretim Programları Ortak Metni; §1.4.7 Öğrenme Kanıtları (158 s.)",
    ),
]

# Menüde duruyor ama sunucu 500 veriyor — yayından kaldırılmış görünüyor.
# Kayıt olarak burada durur ki bir dahaki sefere yeniden aranmasın.
UNAVAILABLE = [("performans-gelisim-cercevesi.pdf", "/upload/kilavuz/performans-gelisim-cercevesi.pdf")]


def download(path: str, target: Path) -> int:
    request = urllib.request.Request(BASE_URL + path, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=REQUEST_TIMEOUT_S) as response:
        payload = response.read()
    target.write_bytes(payload)
    return len(payload)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out-dir", type=Path, default=Path(".tymm-pdf"))
    parser.add_argument("--no-text", action="store_true", help="pdftotext adımını atla")
    args = parser.parse_args()

    args.out_dir.mkdir(parents=True, exist_ok=True)
    has_pdftotext = shutil.which("pdftotext") is not None
    if not args.no_text and not has_pdftotext:
        print("uyarı: pdftotext bulunamadı; yalnız PDF indirilecek.", file=sys.stderr)
        print("       macOS: brew install poppler", file=sys.stderr)

    failures = 0
    for name, path, description in DOCUMENTS:
        target = args.out_dir / name
        try:
            size = download(path, target)
        except Exception as error:
            print(f"  HATA {name}: {error}", file=sys.stderr)
            failures += 1
            continue

        note = ""
        if not args.no_text and has_pdftotext:
            result = subprocess.run(
                ["pdftotext", "-layout", str(target), str(target.with_suffix(".txt"))],
                capture_output=True,
            )
            if result.returncode != 0:
                note = " (metin çıkarımı başarısız)"
                failures += 1
            else:
                note = f" + {target.with_suffix('.txt').name}"

        print(f"  {name:<42} {size / 1e6:6.1f} MB{note}\n      {description}", file=sys.stderr)

    for name, path in UNAVAILABLE:
        print(f"  {name:<42} YAYINDA DEĞİL ({BASE_URL}{path} -> 500)", file=sys.stderr)

    print(f"\ndizin: {args.out_dir.resolve()}", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
