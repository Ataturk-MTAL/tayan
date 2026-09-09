#!/usr/bin/env python3
"""Ders programı PDF metninden ünite bölümlerini çıkarır.

Saf işlev: ağa çıkmaz, dosya okumaz/yazmaz. Tek sorumluluğu metni yapıya
çevirmek; indirme ve yazma işi çağıranındır. Bu ayrım test edilebilirlik
içindir — bu veri hattındaki her sessiz kayıp ayrıştırmadan çıktı, indirmeden
değil.
"""

from __future__ import annotations

import re

# Ünite başlığı her derste "ÜNİTE" demiyor: Matematik "N. TEMA", Hayat
# Bilgisi "N. ÖĞRENME ALANI" kullanıyor. Sözcüğü sabitlemek 111 dersin
# çoğunda çıktıların üniteye bağlanamamasına yol açıyordu (3069/3877
# çıktı sahipsiz kalmıştı). Kullanılan sözcük `kind` alanında saklanır.
UNIT_RE = re.compile(
    r"^\s*(\d+)\.\s*(ÜNİTE|TEMA|ÖĞRENME ALANI)\s*:\s*(.+?)\s*$"
)
GRADE_RE = re.compile(r"^\s*(\d+)\.\s*SINIF\b")
HOURS_RE = re.compile(r"^\s*DERS SAATİ\s+(\d+)\b")

# Ünite içindeki bölüm etiketleri. pdftotext çok kelimeli etiketleri satırlara
# böldüğü için (ör. "ALAN" + "BECERİLERİ") etiket satır satır BİRİKTİRİLİR.
SECTION_LABELS = [
    "ALAN BECERİLERİ",
    "KAVRAMSAL BECERİLER",
    "EĞİLİMLER",
    "PROGRAMLAR ARASI BİLEŞENLER",
    "Sosyal-Duygusal Öğrenme Becerileri",
    "Değerler",
    "Okuryazarlık Becerileri",
    "DİSİPLİNLER ARASI İLİŞKİLER",
    "İLİŞKİLER",
    "ÖĞRENME ÇIKTILARI VE SÜREÇ BİLEŞENLERİ",
    "İÇERİK ÇERÇEVESİ",
    "Anahtar Kavramlar",
    "ÖĞRENME KANITLARI",
    "FARKLILAŞTIRMA",
]
# Uzun etiket önce denenmeli: "İLİŞKİLER", "DİSİPLİNLER ARASI İLİŞKİLER"in
# sonuna da uyar ve kısa olan önce denenirse yanlış bölüm açılır.
SECTION_LABELS.sort(key=len, reverse=True)

PAGE_NOISE_RE = re.compile(r"^\s*\d+\s*$|ÖĞRETİM PROGRAMI\s*$")


def collapse(text: str) -> str:
    """Satır kırılmalarını ve satır sonu tire bölünmelerini toparlar."""
    joined = re.sub(r"\s+", " ", text).strip()
    return re.sub(r"-\s+(?=\w)", "", joined)


def match_label(buffer: str) -> tuple[str, str] | None:
    """Biriken etiket tamponu bir bölüm etiketiyle başlıyorsa (etiket, kalan)."""
    normalized = re.sub(r"\s+", " ", buffer).strip()
    for label in SECTION_LABELS:
        if normalized.startswith(label):
            return label, normalized[len(label):].strip()
    return None


def parse_units(text: str, prefix: str) -> list[dict]:
    """Metni ünite listesine çevirir.

    `prefix` şimdilik kullanılmıyor; imzada duruyor çünkü çağıran ders
    kısaltmasını zaten biliyor ve ileride ek bölümdeki (EK 1) çıktı
    şemalarını ayırt etmek için gerekecek.
    """
    lines = text.splitlines()
    units: list[dict] = []
    grade: int | None = None
    current: dict | None = None
    section: str | None = None
    label_buffer = ""

    def flush() -> None:
        nonlocal current, section, label_buffer
        if current is not None:
            # Blok ham metni saklanır: çıktılar anahtarla değil KONUMLA
            # eşleştirilecek — çıktı hangi bloğun içindeyse o bloğa aittir.
            current["text"] = "\n".join(current["_lines"])
            del current["_lines"]
            current["description"] = collapse(current["description"])
            current["sections"] = {k: collapse(v) for k, v in current["sections"].items()}
            units.append(current)
        current, section, label_buffer = None, None, ""

    for line in lines:
        if PAGE_NOISE_RE.match(line):
            continue

        grade_hit = GRADE_RE.match(line)
        if grade_hit:
            grade = int(grade_hit.group(1))
            continue

        unit_hit = UNIT_RE.match(line)
        if unit_hit:
            flush()
            current = {
                "grade": grade,
                "unit": int(unit_hit.group(1)),
                "kind": unit_hit.group(2),
                "title": unit_hit.group(3).strip(),
                "description": "",
                "lesson_hours": None,
                "sections": {},
                "_lines": [],
            }
            continue

        if current is None:
            continue
        current["_lines"].append(line)

        hours_hit = HOURS_RE.match(line)
        if hours_hit:
            current["lesson_hours"] = int(hours_hit.group(1))
            section, label_buffer = None, ""
            continue

        stripped = line.strip()
        if not stripped:
            continue

        # Etiket tamponu: satırı ekleyip bir etikete uyuyor mu diye bak.
        candidate = (label_buffer + " " + stripped).strip() if label_buffer else stripped
        hit = match_label(candidate)
        if hit:
            label, remainder = hit
            section = label
            current["sections"].setdefault(section, "")
            if remainder:
                current["sections"][section] += " " + remainder
            label_buffer = ""
            continue

        # Etiketin yalnız ilk parçası gelmiş olabilir; tamponda tut.
        if any(lbl.startswith(candidate) for lbl in SECTION_LABELS):
            label_buffer = candidate
            continue
        label_buffer = ""

        if section is None:
            current["description"] += " " + stripped
        else:
            current["sections"][section] += " " + stripped

    flush()

    # Belgelerin başındaki "PROGRAMIN YAPISI" bölümü ÖRNEK bir ünite gösteriyor
    # ve gerçek üniteyle aynı (sınıf, ünite) anahtarını taşıyor. Fizik'te bu,
    # 15 yerine 16 ünite üretiyordu. Yapısal bir işaretle ayıklamak güvenilmez:
    # "SINIF DÜZEYLERİNE AİT" ifadesi 110 dersin yalnız 39'unda var. Bunun
    # yerine en dolgun blok kazanır — çıktı ve beceri ayrıştırıcılarında da
    # kullanılan strateji.
    # Sınıfı OLMAYAN bloklar tekilleştirilmez. Bazı belgelerde "N. SINIF"
    # satırı hiç geçmiyor; o zaman 5./6./7. sınıfın TEMA 1'leri aynı
    # (None, 1) anahtarına düşer ve üçü birden teke inerdi — gerçek ünite
    # blokları kaybolurdu.
    ungraded = [u for u in units if u["grade"] is None]

    richest: dict[tuple[int, int], dict] = {}
    for unit in units:
        if unit["grade"] is None:
            continue
        key = (unit["grade"], unit["unit"])
        score = len(unit["sections"]) * 1000 + len(unit["description"])
        previous = richest.get(key)
        if previous is None or score > previous["_score"]:
            richest[key] = {**unit, "_score": score}
    for unit in richest.values():
        unit.pop("_score", None)

    graded = sorted(richest.values(), key=lambda u: (u["grade"], u["unit"]))
    return graded + ungraded
