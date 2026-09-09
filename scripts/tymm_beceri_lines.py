#!/usr/bin/env python3
"""Beceri sayfası satırlarının ayrıştırılması — saf işlevler.

Ayrı modül: bu mantık iki kez sessiz veri kaybına yol açtı ve ancak bağımsız
bir araçla çapraz doğrulayınca görüldü. Saf işlev olarak testlenebilir.

Kaynağın iki tuzağı:

1. GÖSTERGE KADEMESİ. SDB sayfası iki sütunlu tablo:
       <td>SDB2.1.SB1. Başkalarını etkin şekilde dinlemek</td>
       <td><ul class="indicator-list">
             <li>SDB2.1.SB1.G1. Başkalarından gelen iletileri fark eder.</li>
   Etiketler satıra düzleştirilince gösterge satırı üst bileşenin kodunu
   taşıyor gibi görünüyor. 36 bileşen 225 kayda şişmiş, G kodları kaybolmuştu.

2. AYIRAÇ. Kod ile ad arasında nokta da olabiliyor boşluk da:
       KB2.8.Sorgulama Becerisi          (nokta)
       DAB3.1 Dinî Kavramları Ayırt Etme (boşluk, akordiyon başlığı)
   Yalnız noktayı bekleyen desen ikinci biçimdeki düğümleri atlıyordu.
"""

from __future__ import annotations

import re

# Gösterge: bileşen kodunun ardından .G<n> gelir.
INDICATOR_RE = re.compile(
    r"^([A-ZÇĞİÖŞÜ]{1,8}[\d.]*\.SB\.?\s*\d+\.G\d+)\.\s*(.*)$"
)
# Kod satırı: ayıraç NOKTA ya da BOŞLUK.
CODE_LINE_RE = re.compile(r"^([A-ZÇĞİÖŞÜ]{1,8}\d[\d.]*)(?:\.\s*|\s+)(\S.*)$")


def split_indicator(line: str) -> tuple[str | None, str]:
    """Gösterge satırıysa (kod, metin), değilse (None, satır)."""
    match = INDICATOR_RE.match(line.strip())
    if not match:
        return None, line
    return match.group(1), match.group(2).strip()


def split_code_line(line: str) -> tuple[str, str] | None:
    """"KOD.Ad" ya da "KOD Ad" satırını (kod, ad) olarak böler; değilse None.

    Kod ucundaki nokta ayıraca dâhildir: "OB1. Bilgi Okuryazarlığı" -> ("OB1",
    "Bilgi Okuryazarlığı"). Niceleyici AÇGÖZLÜ olmalı ve geri izlemeye
    bırakılmalı; tembel olursa "KB2.8.Sorgulama" kodu "KB2" diye kesilir ve
    "8." ada kayar.
    """
    match = CODE_LINE_RE.match(line.strip())
    if not match:
        return None
    code = match.group(1).rstrip(".")
    return code, match.group(2).strip()
