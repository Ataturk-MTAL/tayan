#!/usr/bin/env python3
"""Ders programı PDF metninden öğrenme çıktılarını çıkarır.

Saf işlev: ağa çıkmaz, dosya okumaz/yazmaz.

TYMM iki kod şeması kullanıyor ve hangisinin geçerli olduğu BELGEDEN
ANLAŞILIR, ders adından değil:

    dört sayılı   FİZ.9.1.2.   ön ek + sınıf + ünite + sıra
    üç sayılı     T.O.5.2.     ön ek + sınıf + sıra   (ünite kademesi yok)

Ön ek nokta içerebilir (`T.O` = Türkçe/Okuma, `T.D` = Dinleme). Ön ek
uzunluğunu ya da şemayı sabitlemek 111 dersin 27'sinde hiç çıktı bulunamamasına
yol açmıştı.
"""

from __future__ import annotations

import collections
import re

# Ön ek harf ve nokta içerir, tembel eşleşir: "T.O.5.2." için ön ek "T.O",
# "FİZ.9.1.2." için "FİZ". Tembel olmazsa ön ek sayıları da yutar.
# re.M şart: detect_scheme desenleri TÜM metne uyguluyor ve MULTILINE olmadan
# "$" yalnız metnin en sonunda eşleşir — hiçbir kod bulunamaz.
FOUR_RE = re.compile(r"([A-ZÇĞİÖŞÜ][A-ZÇĞİÖŞÜ.]*?)\.(\d+)\.(\d+)\.(\d+)\.\s*(.*)$", re.M)
THREE_RE = re.compile(r"([A-ZÇĞİÖŞÜ][A-ZÇĞİÖŞÜ.]*?)\.(\d+)\.(\d+)\.\s*(.*)$", re.M)
COMPONENT_RE = re.compile(r"^\s*([a-zçğöşü])\)\s*(.+)$")

# Beceri çerçevesinin kodları ÖĞRENME ÇIKTISI DEĞİLDİR. Ders programları
# ünite bloklarında bu kodlara atıf yapıyor; ön ek tespitine karışırlarsa
# çıktı sanılıyorlar (Okul Öncesi'nde "D" değer kodu çıktı ön eki seçilmişti).
SKILL_PREFIXES = frozenset({
    "KB", "SDB", "E", "D", "OB",
    "TAB", "MAB", "FBAB", "SBAB", "SAB", "DAB",
    "BEOSAB", "BTYAB", "TSRMAB", "YDAB", "YDDB",
})

SECTION_RE = re.compile(
    r"^\s*(İÇERİK ÇERÇEVESİ|Anahtar Kavramlar|ÖĞRENME|KANITLARI|FARKLILAŞTIRMA"
    r"|BECERİLER|İLİŞKİLER|EĞİLİMLER|DEĞERLER|PROGRAMLAR|SOSYAL|ALAN|KAVRAMSAL"
    r"|VE SÜREÇ|\d+\.\s*(ÜNİTE|TEMA|ÖĞRENME ALANI))"
)
PAGE_NOISE_RE = re.compile(r"^\s*\d+\s*$|ÖĞRETİM PROGRAMI\s*$")


def collapse(text: str) -> str:
    joined = re.sub(r"\s+", " ", text).strip()
    return re.sub(r"-\s+(?=\w)", "", joined)


def detect_scheme(text: str) -> tuple[list[str], int]:
    """(ön ek listesi, sayı kademesi) döndürür; belgede baskın şema kazanır.

    Dört sayılı kodlar üç sayılı desene de uyar, bu yüzden önce dört sayılılar
    sayılır ve onların kapsadığı yerler üç sayılı sayımından düşülür.
    """
    four = collections.Counter()
    four_spans = []
    for match in FOUR_RE.finditer(text):
        four[match.group(1)] += 1
        four_spans.append(match.span())

    three = collections.Counter()
    for match in THREE_RE.finditer(text):
        start = match.start()
        if any(a <= start < b for a, b in four_spans):
            continue
        # Sonunda nokta OLMAYAN dört sayılı atıflar ("FB.5.1.1", tablolarda
        # bol) üç sayılı desene de uyar: FB.5.1 + metin "1". Bu sahte
        # eşleşmeler Fen Bilimleri'nde şemayı 182'ye 181 ile düşürüyor ve
        # bütün süreç bileşenlerini kaybettiriyordu. İmza: metin RAKAMLA
        # başlar — gerçek bir çıktı metni rakamla başlamaz.
        if re.match(r"^\d", match.group(4).strip()):
            continue
        three[match.group(1)] += 1

    four = collections.Counter({k: v for k, v in four.items() if k not in SKILL_PREFIXES})
    three = collections.Counter({k: v for k, v in three.items() if k not in SKILL_PREFIXES})

    # Kazanan ŞEMA seçilir, sonra o şemadaki TÜM ön ekler alınır. Bir ders
    # birden fazla ön ek kullanabiliyor (Türkçe: T.D, T.O, T.Y, T.K).
    if four and (not three or sum(four.values()) >= sum(three.values())):
        return sorted(four), 4
    if three:
        return sorted(three), 3
    return [], 0


def parse_outcomes(text: str, prefixes: str | list[str], segments: int) -> list[dict]:
    """Çıktıları ve a/b/c/ç süreç bileşenlerini çıkarır.

    `segments` 4 ise kodda ünite kademesi vardır; 3 ise `unit` None kalır —
    uydurulmaz.
    """
    if isinstance(prefixes, str):
        prefixes = [prefixes]
    wanted = set(prefixes)
    pattern = FOUR_RE if segments == 4 else THREE_RE
    lines = text.splitlines()
    best: dict[str, dict] = {}
    index = 0

    while index < len(lines):
        # DİKKAT: search, match DEĞİL. Her ünitenin ilk çıktısı bölüm etiketiyle
        # aynı satırı paylaşır ("VE SÜREÇ BİLEŞENLERİ FİZ.9.1.1. ...").
        head = pattern.search(lines[index])
        if not head or head.group(1) not in wanted:
            index += 1
            continue

        if segments == 4:
            grade, unit, order = int(head.group(2)), int(head.group(3)), int(head.group(4))
            code = f"{head.group(1)}.{head.group(2)}.{head.group(3)}.{head.group(4)}"
            title_parts = [head.group(5)]
        else:
            grade, unit, order = int(head.group(2)), None, int(head.group(3))
            code = f"{head.group(1)}.{head.group(2)}.{head.group(3)}"
            title_parts = [head.group(4)]

        components: list[dict] = []
        current: dict | None = None
        index += 1
        while index < len(lines):
            line = lines[index]
            if pattern.search(line) or SECTION_RE.match(line):
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
            "grade": grade,
            "unit": unit,
            "order": order,
            "text": collapse(" ".join(title_parts)),
            "components": [
                {"label": c["label"], "text": collapse(" ".join(c["text"]))} for c in components
            ],
        }
        # Aynı kod belgede birden çok geçer (tablo + özet + atıf). En dolgun
        # geçiş kazanır; çıplak atıflar düşük puan alıp elenir.
        score = len(entry["text"]) + 50 * len(entry["components"])
        if code not in best or score > best[code]["_score"]:
            best[code] = {**entry, "_score": score}

    for entry in best.values():
        entry.pop("_score", None)
    return sorted(
        best.values(), key=lambda e: (e["grade"], e["unit"] if e["unit"] is not None else 0, e["order"])
    )
