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
SPACED_RE = re.compile(r"(?:^|(?<=[^A-ZÇĞİÖŞÜ]))([A-ZÇĞİÖŞÜ]{2,5})\s+(\d+)\.(\d+)\.\s*(.*)$", re.M)
COMPONENT_RE = re.compile(r"^\s*([a-zçğöşü])\)\s*(.+)$")

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


def detect_scheme(text: str) -> tuple[list[str], int]:
    """(ön ek listesi, şema kodu) döndürür.

    Şema kodları: 4 dört sayılı, 3 üç sayılı, 5 yapışık üç sayılı,
    2 yapışık iki sayılı, 6 boşluklu. 0 = hiçbiri.

    Aileler SIRAYLA değil HACME göre yarışır. Sıralı denemede Türk Dili ve
    Edebiyatı'nda 10 kez geçen noktalı "E." atıfı, 382 kez geçen yapışık
    "TDE" şemasını gölgeliyordu.
    """
    def tally(pattern, exclude_spans=(), skill_filter=False, text_group=None):
        counter = collections.Counter()
        spans = []
        for match in pattern.finditer(text):
            if any(a <= match.start() < b for a, b in exclude_spans):
                continue
            if skill_filter and match.group(1) in SKILL_PREFIXES:
                continue
            if text_group and re.match(r"^\d", match.group(text_group).strip()):
                continue
            counter[match.group(1)] += 1
            spans.append(match.span())
        return counter, spans

    four, four_spans = tally(FOUR_RE)
    three, _ = tally(THREE_RE, exclude_spans=four_spans, text_group=4)
    glued3, glued3_spans = tally(GLUED3_RE, skill_filter=True)
    glued, _ = tally(GLUED_RE, exclude_spans=glued3_spans, skill_filter=True, text_group=4)
    spaced, _ = tally(SPACED_RE, skill_filter=True, text_group=4)

    def drop_ghosts(counter):
        """Kırpılmış ön ekleri eler.

        pdftotext bir satırı kestiğinde ön ekin KUYRUĞU ayrı bir ön ek gibi
        görünür: "İTA" 20 kez geçerken kırpılmış tek bir "TA" aileye üye
        oluyor ve sorted()[0] ile dersin kimliğini ele geçiriyordu. Bir ön ek
        başka bir ön ekin son ekiyse ve geçişi onun beşte birinden azsa
        artefakttır.
        """
        result = dict(counter)
        for a in list(counter):
            for b in counter:
                if a == b or a not in result:
                    continue
                # Son ek ilişkisi iki yönde de olabilir:
                #   TA  ⊂ İTA   -> baştan kırpılmış, KISA olan hayalet
                #   ENG ⊂ EENG  -> başa harf yapışmış, UZUN olan hayalet
                # Hangisi olduğu uzunluktan değil SIKLIKTAN anlaşılır: nadir
                # olan artefakttır.
                if not (a.endswith(b) or b.endswith(a)):
                    continue
                if counter[a] * 5 < counter[b]:
                    del result[a]
                    break
        return collections.Counter(result)

    def prefer_non_skill(counter):
        non_skill = {k: v for k, v in counter.items() if k not in SKILL_PREFIXES}
        return collections.Counter(non_skill or counter)

    four = prefer_non_skill(drop_ghosts(four))
    three = prefer_non_skill(drop_ghosts(three))

    # Eşitlikte daha ÖZGÜL şema kazanır: dört sayılı > üç sayılı > yapışık.
    glued3 = drop_ghosts(glued3)
    glued = drop_ghosts(glued)
    spaced = drop_ghosts(spaced)

    families = [(four, 4), (three, 3), (glued3, 5), (glued, 2), (spaced, 6)]
    best = max(
        (f for f in families if f[0]),
        key=lambda f: sum(f[0].values()),
        default=None,
    )
    if best is None:
        return [], 0
    return sorted(best[0]), best[1]


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
        elif segments == 3:
            grade, unit, order = int(head.group(2)), None, int(head.group(3))
            code = f"{head.group(1)}.{head.group(2)}.{head.group(3)}"
            title_parts = [head.group(4)]
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


SCHEME_PATTERNS.update({4: FOUR_RE, 3: THREE_RE, 2: GLUED_RE, 5: GLUED3_RE, 6: SPACED_RE})
# Şema başına kod parçası sayısı (kapsama denetimi kodu bundan kurar).
SCHEME_PARTS = {4: 4, 3: 3, 5: 4, 2: 3, 6: 3}


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
    # 2 (yapışık) ve 6 (boşluklu) aynı kodu üretir: ÖNEKn.m
    return f"{g[0]}{g[1]}.{g[2]}"
