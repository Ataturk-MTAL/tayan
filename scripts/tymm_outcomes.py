#!/usr/bin/env python3
"""Ders programı PDF metninden öğrenme çıktılarını çıkarır.

Saf işlev: ağa çıkmaz, dosya okumaz/yazmaz.

TYMM beş kod şeması kullanıyor:

    noktalı-4    FİZ.9.1.2.   ön ek + sınıf + ünite + sıra
    noktalı-3    T.O.5.2.     ön ek + sınıf + sıra   (ünite kademesi yok)
    yapışık-3    RK2.4.1.     rakam ön eke yapışık, üç sayı
    yapışık-2    TDE1.1.      rakam ön eke yapışık, iki sayı
    boşluklu-2   TKMT 3.1.    ön ek ile sayı arasında boşluk

Ön ek nokta içerebilir (`T.O` = Türkçe/Okuma, `T.D` = Dinleme). Ön ek
uzunluğunu ya da şemayı SABİTLEMEK 111 dersin 27'sinde hiç çıktı
bulunamamasına yol açmıştı; TEK şema SEZMEK ise azınlıkta kalan aileyi
sessizce düşürüyordu.

Hangi şeklin çıktı olduğu artık SEZİLMİYOR: `data/tymm/shape-manifest.json`
ders başına beyan ediyor, bu modül kendisine söylenen aileleri ayrıştırıyor.
Beyan edilmeyen şekil hatadır — bkz. tymm_shape_census.
"""

from __future__ import annotations

import collections
import re

# Ön ek harf ve nokta içerir, tembel eşleşir: "T.O.5.2." için ön ek "T.O",
# "FİZ.9.1.2." için "FİZ". Tembel olmazsa ön ek sayıları da yutar.
# re.M şart: şekil sayımı desenleri TÜM metne uyguluyor ve MULTILINE olmadan
# "$" yalnız metnin en sonunda eşleşir — hiçbir kod bulunamaz.
FOUR_RE = re.compile(r"([A-ZÇĞİÖŞÜ][A-ZÇĞİÖŞÜ.]*?)\.(\d+)\.(\d+)\.(\d+)\.\s*(.*)$", re.M)
THREE_RE = re.compile(r"([A-ZÇĞİÖŞÜ][A-ZÇĞİÖŞÜ.]*?)\.(\d+)\.(\d+)\.\s*(.*)$", re.M)
# Üçüncü şema: rakam ön eke YAPIŞIK — "TDE1.1." = TDE + tema 1 + sıra 1.
# Beceri kodlarıyla (SDB1.2, E3.3) aynı biçimde; ayırt edici tek şey ön ekin
# SKILL_PREFIXES içinde olmaması.
# Yapışık biçim iki sayılı da olabilir üç sayılı da, ve ön ek ile sayı
# arasında BOŞLUK bulunabilir ("TKMT 3.1."). Üçü de aynı aileden.
GLUED_RE = re.compile(r"([A-ZÇĞİÖŞÜ]{2,8})(\d+)\.(\d+)\.\s*(.*)$", re.M)
GLUED3_RE = re.compile(r"([A-ZÇĞİÖŞÜ]{2,8})(\d+)\.(\d+)\.(\d+)\.\s*(.*)$", re.M)
# Boşluklu yazım ("TKMT 3.1.") ayrı desen ve ön ek EN FAZLA 5 HARF. Sınır şart:
# boşluğa serbest izin verilince "ESASLAR 3.1.", "PROGRAMI 1.2." gibi Türkçe
# kelimeler ön ek sanılıyor.
# Ayıraç \s DEĞİL [ \t]: \s satır sonunu da yutuyordu ve kendi satırında duran
# bir Türkçe/İngilizce kelime, BİR SONRAKİ satırdaki bölüm numarasıyla birleşip
# ön ek sanılıyordu — "SÜRE\n 1.3.", "MODÜL\n 1.1.", "SIZES\n 1.6.", "EBADI",
# "NIN". Altı sahte aile bu tek karakter yüzündendi.
SPACED_RE = re.compile(
    r"(?:^|(?<=[^A-ZÇĞİÖŞÜ]))([A-ZÇĞİÖŞÜ]{2,5})[ \t]+(\d+)\.(\d+)\.\s*(.*)$", re.M
)
# ALAN BECERİSİ KADEMESİ. Yabancı dil programlarında kodun bir parçası SAYI
# DEĞİL, harf+sayı: ENG.9.1.L1. = ders ENG, 9. sınıf, 1. tema, Listening 1.
# Harfler alan becerisini gösterir — İngilizce'de L/R/S/W (Listening, Reading,
# Speaking, Writing) ve G/V/P (Grammar, Vocabulary, Pronunciation), Almanca'da
# H/L/S/SP (Hören, Lesen, Schreiben, Sprechen).
# Bu biçimi tanıyan desen YOKTU: sekiz ders SIFIR çıktıyla duruyordu ve
# manifest bunu göremiyordu — hiçbir desenin eşlemediği kod sayıma da girmez.
FIELD_RE = re.compile(
    r"([A-ZÇĞİÖŞÜ][A-ZÇĞİÖŞÜ.]*?)\.(\d+)\.(\d+)\.([A-Z]{1,2}\d+)\.\s*(?![\d\s])(.*)$", re.M
)
# Almanca beş parçalı: DE.5.1.H1.1. — alan bloğundan SONRA bir sıra numarası var.
FIELD5_RE = re.compile(
    r"([A-ZÇĞİÖŞÜ][A-ZÇĞİÖŞÜ.]*?)\.(\d+)\.(\d+)\.([A-Z]{1,2}\d+)\.(\d+)\.\s*(.*)$", re.M
)
COMPONENT_RE = re.compile(r"^\s*([a-zçğöşü])\)\s*(.+)$")
# Gösterge kademesi süreç bileşeninin ALTINDADIR ve kodu ".SB<n>." taşır:
#   SDB2.1.SB1.G1. Başkalarından gelen iletileri fark eder.
# .SB<n>. ŞARTI KRİTİK. Desen önce yalnız "X.G<n>." bekliyordu ve yabancı dil
# programlarının GRAMMAR çıktılarını gösterge sanıp çalıyordu — belgenin kendi
# lejantına göre (satır 110-113) G/V/P destekleyici alan becerileridir
# (grammaring, vocabulary, pronunciation), gösterge değil. Ölçüm: "X.G<n>."
# 111 dersin YALNIZ 5'inde geçiyor ve beşi de yabancı dil; gerçek gösterge
# (".SB<n>.G<n>.") yalnız okul öncesinde var (121 geçiş).
INDICATOR_RE = re.compile(
    r"^\s*([A-ZÇĞİÖŞÜ][A-ZÇĞİÖŞÜ.]*[\d.]*\.SB\.?\s*\d+)\.G(\d+)\.\s*(.*)$", re.M
)

# Şema kodu -> desen. TEK KAYNAK: hem ayrıştırma hem kapsama denetimi burayı
# kullanır. Ayrı ayrı yazıldığında denetim, ayrıştırıcının tanıdığı biçimleri
# tanımıyordu ve 8 derste sessizce işlemsiz kalıyordu.
SCHEME_PATTERNS = {}

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


def parse_outcomes(text: str, prefixes: str | list[str], segments: int) -> list[dict]:
    """Çıktıları ve a/b/c/ç süreç bileşenlerini çıkarır.

    `segments` 4 ise kodda ünite kademesi vardır; 3 ise `unit` None kalır —
    uydurulmaz.
    """
    if isinstance(prefixes, str):
        prefixes = [prefixes]
    wanted = set(prefixes)
    if segments not in SCHEME_PATTERNS:
        return []
    pattern = SCHEME_PATTERNS[segments]
    lines = text.splitlines()
    best: dict[str, dict] = {}
    index = 0

    while index < len(lines):
        field = None
        # DİKKAT: search, match DEĞİL. Her ünitenin ilk çıktısı bölüm etiketiyle
        # aynı satırı paylaşır ("VE SÜREÇ BİLEŞENLERİ FİZ.9.1.1. ...").
        # "a)" ile başlayan satır SÜREÇ BİLEŞENİDİR, kod taşısa bile. Türk
        # Dili'nde bileşenin kendi kodu var ("a) TDE1.1.1. Seçim yapar.") ve
        # desen onu bir sonraki çıktı sanıyordu: ders 239 çıktı üretiyor ama
        # SIFIR bileşen — bileşenlerin hepsi çıktı olarak sayılmıştı.
        head = None if COMPONENT_RE.match(lines[index]) else pattern.search(lines[index])
        if not head or head.group(1) not in wanted:
            index += 1
            continue

        if segments == 4:
            grade, unit, order = int(head.group(2)), int(head.group(3)), int(head.group(4))
            code = f"{head.group(1)}.{head.group(2)}.{head.group(3)}.{head.group(4)}"
            title_parts = [head.group(5)]
        elif segments == 3:
            grade, unit, order = int(head.group(2)), None, int(head.group(3))
            code = f"{head.group(1)}.{head.group(2)}.{head.group(3)}"
            title_parts = [head.group(4)]
        elif segments == 7:
            # ENG.9.1.L1. -> sınıf 9, tema 1, alan "L1", sıra 1
            grade, unit = int(head.group(2)), int(head.group(3))
            field = head.group(4)
            order = int(re.search(r"\d+", field).group())
            code = f"{head.group(1)}.{head.group(2)}.{head.group(3)}.{field}"
            title_parts = [head.group(5)]
        elif segments == 8:
            # DE.5.1.H1.1. -> sınıf 5, tema 1, alan "H1", sıra 1
            grade, unit = int(head.group(2)), int(head.group(3))
            field = head.group(4)
            order = int(head.group(5))
            code = f"{head.group(1)}.{head.group(2)}.{head.group(3)}.{field}.{head.group(5)}"
            title_parts = [head.group(6)]
        elif segments == 5:
            # Yapışık, üç sayılı: RK2.4.1 -> ön ek RK, 2, ünite 4, sıra 1.
            grade, unit, order = int(head.group(2)), int(head.group(3)), int(head.group(4))
            code = f"{head.group(1)}{head.group(2)}.{head.group(3)}.{head.group(4)}"
            title_parts = [head.group(5)]
        else:
            # Yapışık ya da boşluklu, iki sayılı: TDE1.1 / TKMT 3.1 -> ön ek TDE, tema/sınıf 1, sıra 1.
            # Boşluklu yazım da buraya düşer ("TKMT 3.1" -> TKMT3.1).
            grade, unit, order = int(head.group(2)), None, int(head.group(3))
            code = f"{head.group(1)}{head.group(2)}.{head.group(3)}"
            title_parts = [head.group(4)]

        components: list[dict] = []
        current: dict | None = None
        index += 1
        while index < len(lines):
            line = lines[index]
            component = COMPONENT_RE.match(line)
            if not component and (pattern.search(line) or SECTION_RE.match(line)):
                break
            if PAGE_NOISE_RE.match(line):
                index += 1
                continue
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
            "field": field,
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
        best.values(),
        key=lambda e: (
            e["grade"],
            e["unit"] if e["unit"] is not None else 0,
            e["field"] or "",
            e["order"],
        ),
    )


SCHEME_PATTERNS.update({
    4: FOUR_RE, 3: THREE_RE, 2: GLUED_RE, 5: GLUED3_RE, 6: SPACED_RE,
    7: FIELD_RE, 8: FIELD5_RE,
})
# Şema başına kod parçası sayısı (kapsama denetimi kodu bundan kurar).
SCHEME_PARTS = {4: 4, 3: 3, 5: 4, 2: 3, 6: 3, 7: 4, 8: 5}


def build_code(match, segments: int) -> str:
    """Eşleşmeden kod dizesini kurar — TEK KAYNAK.

    Ayrıştırıcı ile kapsama denetimi bu işlevi paylaşır. Ayrı ayrı yazıldığında
    denetim, ayrıştırıcının ürettiğinden farklı kod dizeleri kurup sahte
    "ayrıştırılamayan kod" raporluyordu.
    """
    g = match.groups()
    if segments == 4:
        return f"{g[0]}.{g[1]}.{g[2]}.{g[3]}"
    if segments == 3:
        return f"{g[0]}.{g[1]}.{g[2]}"
    if segments == 5:
        return f"{g[0]}{g[1]}.{g[2]}.{g[3]}"
    if segments == 7:
        return f"{g[0]}.{g[1]}.{g[2]}.{g[3]}"
    if segments == 8:
        return f"{g[0]}.{g[1]}.{g[2]}.{g[3]}.{g[4]}"
    # 2 (yapışık) ve 6 (boşluklu) aynı kodu üretir: ÖNEKn.m
    return f"{g[0]}{g[1]}.{g[2]}"


# Bir belgede bir ailenin "gerçek" sayılması için gereken en az geçiş.
# Altında kalanlar pdftotext gürültüsü kabul edilir.
FAMILY_MIN = 3


# ÇOKLU AİLE ARTIK SEZGİ DEĞİL BEYAN. Aileler `data/tymm/shape-manifest.json`
# içinde ders başına yazılıdır; bu modül hangi ailelerin ayrıştırılacağını
# ARAMAZ, kendisine SÖYLENİR. Sezgisel deneme ölçülmüş ve bırakılmıştı:
# geri kazanım artıyordu (6119 -> 6560 çıktı) ama kesinlik çöküyordu
# (bileşen 12 215 -> 9435, sahipsiz 285 -> 1701) çünkü çerçeve atıfları
# ("CS" x542, "SELS" x125 — beceri adlarının İngilizcesi) hacimde gerçek
# çıktı ailesini geçiyordu. Manifest bu ikisini ayırt eder, sezgi edemez.


def parse_declared(text: str, families: list[tuple[list[str], int]]) -> list[dict]:
    """BEYAN EDİLEN ailelerin hepsini ayrıştırıp birleştirir.

    Kod bazında tekilleştirir — bir kod birden çok ailede eşleşebilir
    (ör. "SNAB1." hem yapışık hem noktalı desene uyar). En dolgun geçiş kazanır.
    Göstergeler (ENG.9.1.G1) çıktı sayılmaz, sahiplerine bağlanır.
    """
    merged: dict[str, dict] = {}
    for prefixes, scheme in families:
        for outcome in parse_outcomes(text, prefixes, scheme):
            previous = merged.get(outcome["code"])
            score = len(outcome["text"]) + 50 * len(outcome["components"])
            if previous is None or score > previous["_score"]:
                merged[outcome["code"]] = {**outcome, "_score": score}

    # Gösterge süreç bileşenine bağlıdır, bileşen de çıktıya: "SDB2.1.SB1.G1"
    # -> bileşen "SDB2.1.SB1" -> çıktı "SDB2.1". Sahip aranırken ".SB<n>"
    # kuyruğu atılır; atılmazsa hiçbir gösterge sahibini bulamaz.
    for match in INDICATOR_RE.finditer(text):
        component_code = match.group(1)
        parent = re.sub(r"\.SB\.?\s*\d+$", "", component_code)
        owner = merged.get(parent)
        if owner is None:
            continue
        owner.setdefault("indicators", []).append({
            "code": f"{component_code}.G{match.group(2)}",
            "component": component_code,
            "text": collapse(match.group(3)),
        })

    for outcome in merged.values():
        outcome.pop("_score", None)
    return sorted(
        merged.values(),
        key=lambda o: (o["grade"], o["unit"] if o["unit"] is not None else 0, o["order"]),
    )
