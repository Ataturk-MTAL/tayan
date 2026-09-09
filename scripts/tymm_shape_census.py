#!/usr/bin/env python3
"""Ders programı metnindeki kod ŞEKİLLERİNİ sayar — saf işlev.

Neden var: şema SEZGİYLE seçiliyordu ve azınlıkta kalan aile sessizce
düşüyordu. Eski `detect_scheme` (silindi) tek bir aileyi hacme göre kazandırıyordu;
Görsel Sanatlar'da `GS` kazanınca hazırlık sınıfının `GS.H` ailesi (48 geçiş)
hiç ayrıştırılmıyordu, Türk Dili'nde `TDE1.1.` kazanınca `TDE1.1.1.` (65
geçiş) düşüyordu. Kayıp hiçbir yerde görünmüyordu çünkü seçilen aile kendi
içinde tutarlıydı.

Çoklu aileyi SEZGİYLE açmak da denendi ve daha kötüydü: İngilizce'de beceri
çerçevesinin İngilizce adları (`CS` = Conflict Resolution Skill x542,
`SELS` = Social-Emotional Learning Skill x125) hacimde çıktı ailesiyle
yarışıyor ve kazanıyordu.

Çözüm sezgi değil BEYAN. Bu modül HAM gerçeği ölçer:

    şekil = (ön ek, şema)      ör. ("GS", 4), ("GS.H", 3), ("CS", 2)

Her şeklin ne OLDUĞUNA manifest karar verir (`data/tymm/shape-manifest.json`).
Manifestte olmayan bir şekil HATA'dır — sessizce düşmez.

Bir eşleşmenin üç durumu var:

    nested     aynı konumda daha uzun bir kod da eşleşti
               "GS.9.1.1." hem noktalı-3 hem noktalı-4 eşler; kısa olan gölge
    truncated  eşleşmenin kuyruğu rakamla başlıyor — kod daha derin
               "FİZ.9.2.1" (son nokta yok) noktalı-3 eşler, gerçek kod 4 parçalı
    inner      eşleşme daha uzun bir kodun ORTASINDAN başlıyor
               "DE.5.1.SP3.1." içindeki "SP3.1." yapışık-2 eşler; SP ön ek değil
    labelled   satırda kodun ÖNÜNDE "a)" süreç bileşeni etiketi var
               "a) TDE1.1.1. Seçim yapar." — bu kod çıktı değil, BİLEŞEN
    free       hiçbiri değil — gerçek geçiş

Bir şeklin `free` sayısı 0 ise o şekil başka bir şeklin gölgesidir.
"""

from __future__ import annotations

import collections
import re

from tymm_outcomes import SCHEME_PARTS, SCHEME_PATTERNS, build_code

# Şema kodu -> insan okur ad. Manifest bu adları taşır; sayı taşımak
# incelemeyi gereksiz zorlaştırıyor.
SCHEME_NAMES = {
    4: "noktalı-4", 3: "noktalı-3", 5: "yapışık-3", 2: "yapışık-2", 6: "boşluklu-2",
    # Yabancı dil programlarında kodun bir parçası harf+sayı: ENG.9.1.L1.
    7: "alanlı-4", 8: "alanlı-5",
}
SCHEME_CODES = {v: k for k, v in SCHEME_NAMES.items()}

DIGIT_TAIL_RE = re.compile(r"^\d")
# Eşleşmeden HEMEN ÖNCE rakam (ya da rakamı izleyen nokta) varsa, bu eşleşme
# daha uzun bir kodun kuyruğudur: "DE.5.1.SP3.1." içinden kopan "SP3.1.".
# Kardeş kodları bozmaz: "KB2.4. Çözümleme, KB2.14." ikincisinden önce boşluk.
# \Z, $ DEĞİL. Python'da "$" son satırsonundan ÖNCE de eşleşir: bir önceki
# satır rakamla bittiğinde ("... Çıktı 1\n") sonraki satırın kodu "iç
# eşleşme" sayılıyordu. Gerçek çıktıların 12'de 11'i serbest sayılmıyordu.
CONTINUATION_RE = re.compile(r"\d\.?\Z")
# Süreç bileşeni etiketi: satır "a) " ile başlıyor ve hemen ardından kod geliyor.
# Türk Dili'nde çıktı TDE1.1., bileşen "a) TDE1.1.1." — daha DERİN kod, ama
# çıktı değil. "En derin şema çıktıdır" varsayımı burada 239 çıktıyı siliyordu.
COMPONENT_LABEL_RE = re.compile(r"^\s*[a-zçğöşü]\)\s*$")


def census(text: str) -> list[dict]:
    """Metindeki tüm (ön ek, şema) şekillerini sayar. Süzgeç YOK.

    Sayım ham olmalı: süzgeç uygulanırsa manifest, ayrıştırıcının zaten
    görmediği bir şekli beyan edemez ve "beyan edilmeyen şekil hatadır"
    değişmezi anlamını yitirir.
    """
    hits = []
    for scheme, pattern in SCHEME_PATTERNS.items():
        for match in pattern.finditer(text):
            hits.append({
                "start": match.start(),
                "prefix": match.group(1),
                "scheme": scheme,
                "code": build_code(match, scheme),
                "tail": match.groups()[-1].strip(),
                "inner": bool(CONTINUATION_RE.search(text[max(0, match.start() - 2):match.start()])),
                "labelled": bool(COMPONENT_LABEL_RE.match(
                    text[text.rfind("\n", 0, match.start()) + 1:match.start()]
                )),
                "line": match.group(0).strip()[:80],
            })

    by_start: dict[int, list[dict]] = collections.defaultdict(list)
    for hit in hits:
        by_start[hit["start"]].append(hit)
    for group in by_start.values():
        longest = max(group, key=lambda h: len(h["code"]))
        for hit in group:
            if hit is not longest and longest["code"].startswith(hit["code"]):
                hit["nested_under"] = (longest["prefix"], longest["scheme"])

    shapes: dict[tuple[str, int], dict] = {}
    for hit in hits:
        key = (hit["prefix"], hit["scheme"])
        shape = shapes.setdefault(key, {
            "prefix": hit["prefix"],
            "scheme": SCHEME_NAMES[hit["scheme"]],
            "count": 0,
            "nested": 0,
            "truncated": 0,
            "inner": 0,
            "labelled": 0,
            "free": 0,
            "sample": hit["line"],
        })
        shape["count"] += 1
        if "nested_under" in hit:
            shape["nested"] += 1
            shape.setdefault("shadow_of", "{}/{}".format(
                hit["nested_under"][0], SCHEME_NAMES[hit["nested_under"][1]]
            ))
        elif hit["inner"]:
            shape["inner"] += 1
        elif hit["labelled"]:
            shape["labelled"] += 1
            shape["free"] += 1
        elif DIGIT_TAIL_RE.match(hit["tail"]):
            shape["truncated"] += 1
        else:
            shape["free"] += 1
            if shape["free"] == 1:
                shape["sample"] = hit["line"]

    return sorted(shapes.values(), key=lambda s: (-s["free"], -s["count"], s["prefix"]))


def shape_key(shape: dict) -> str:
    """Manifest anahtarı: 'GS.H/noktalı-3'. Ön ek nokta içerebilir, şema adı içermez."""
    return f"{shape['prefix']}/{shape['scheme']}"


# Beceri çerçevesinin ders programlarında geçen ön ekleri. Türkçe adlar
# tymm_outcomes.SKILL_PREFIXES'te; İngilizce ders programları çerçeveyi
# ÇEVİRİYOR ve bu adlar hacimde çıktı ailesiyle yarışıyor:
#   CS   = Conflict Resolution / Conceptual Skill   x542 (İngilizce 9-12)
#   SELS = Social-Emotional Learning Skill          x125
# Sezgisel çoklu aile denemesinin bozulma sebebi tam olarak buydu.
# YALNIZ belgede DOĞRULANMIŞ olanlar. Spekülatif takma ad doğrudan veri
# kaybı: "DS" tahmini, Ortaokul Dijital Sanatlar dersinin kendi çıktı
# ailesini (DS.1.1.1., 25 serbest geçiş) beceri atıfı sanıp sıfırlamıştı.
SKILL_ALIASES = frozenset({"CS", "SELS"})

# Bir ailenin "gerçek" sayılması için gereken en az SERBEST geçiş.
# 1 = KAPSAYICI. Bilerek düşük: eksik kapsayan taslak veriyi SESSİZCE kaybeder,
# fazla kapsayan taslak GÖRÜNÜR çöp üretir ve inceleme onu manifestte
# "noise"a çeker. Eşiği 3 yapmak Çağdaş Türk ve Dünya Tarihi'nin tek çıktı
# ailesini (serbest 2) siliyordu — ders komple çıktısız kalıyordu.
FREE_MIN = 1
# Hayalet ön ek eşiği: son ek ilişkisindeki çiftte nadir olan artefakttır.
GHOST_RATIO = 5


def bootstrap_roles(
    shapes: list[dict],
    skill_prefixes: frozenset[str],
    free_min: int = FREE_MIN,
) -> dict[str, str]:
    """Şekillere ADLANDIRILMIŞ kurallarla taslak rol atar.

    Taslaktır: manifest gözden geçirilip elle düzeltilebilir. Kuralın değeri
    otomatik olması değil, kararın DOSYADA YAZILI ve denetlenebilir olması.
    Sıra önemli — ilk uyan kural kazanır.
    """
    roles: dict[str, str] = {}
    by_scheme: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    for shape in shapes:
        by_scheme[shape["scheme"]][shape["prefix"]] = shape["free"]

    # Aynı ön ekin EN DERİN şeması çıktı kademesidir; daha sığ olanı bir üst
    # kademedir. "MAT.1.1. Sayılar" öğrenme alanı başlığı, çıktı "MAT.1.1.1.".
    # Sığ olanı çıktı saymak ilkokul matematiğe 30 sahte çıktı sokuyordu.
    # Bileşen kademesi derinlik yarışına GİRMEZ. Türk Dili'nde "a) TDE1.1.1."
    # daha derin ama bileşendir; derinlik yarışına sokulunca çıktı kademesi
    # TDE1.1. "başlık" sayılıp dersin 239 çıktısı sıfırlanıyordu.
    deepest: dict[str, int] = {}
    for shape in shapes:
        if shape["free"] == 0 or _is_component(shape):
            continue
        parts = SCHEME_PARTS[SCHEME_CODES[shape["scheme"]]]
        deepest[shape["prefix"]] = max(deepest.get(shape["prefix"], 0), parts)

    for shape in shapes:
        key = shape_key(shape)
        prefix = shape["prefix"]
        parts = SCHEME_PARTS[SCHEME_CODES[shape["scheme"]]]
        if shape["free"] == 0:
            roles[key] = "shadow"
        elif prefix in skill_prefixes or prefix in SKILL_ALIASES:
            roles[key] = "skill_reference"
        elif _is_component(shape):
            roles[key] = "component"
        elif _is_ghost(prefix, by_scheme[shape["scheme"]]):
            roles[key] = "noise"
        elif shape["free"] < free_min:
            roles[key] = "noise"
        elif parts < deepest.get(prefix, 0):
            roles[key] = "heading"
        else:
            roles[key] = "outcome"
    return roles


# Bir şeklin bileşen sayılması için etiketli geçişlerin serbest geçişlere oranı.
COMPONENT_LABEL_RATIO = 0.5


def _is_component(shape: dict) -> bool:
    """Geçişlerinin çoğu "a)" etiketliyse bu şekil süreç bileşeni kademesidir."""
    return shape["free"] > 0 and shape["labelled"] >= shape["free"] * COMPONENT_LABEL_RATIO


def _is_ghost(prefix: str, siblings: collections.Counter) -> bool:
    """pdftotext bir satırı kestiğinde ön ekin kuyruğu ayrı ön ek gibi görünür.

    "İTA" 20 kez geçerken kırpılmış tek bir "TA" ayrı aile sanılıyordu. İlişki
    iki yönlü: TA ⊂ İTA (baştan kırpık) ve ENG ⊂ EENG (başa harf yapışmış).
    Hangisinin artefakt olduğu uzunluktan değil SIKLIKTAN anlaşılır.
    """
    mine = siblings[prefix]
    for other, count in siblings.items():
        if other == prefix or mine * GHOST_RATIO >= count:
            continue
        if not (prefix.endswith(other) or other.endswith(prefix)
                or prefix.startswith(other) or other.startswith(prefix)):
            continue
        # Fark NOKTA içeriyorsa bu bir alt aile işaretidir, kırpma değil:
        # "MAT.H" (hazırlık sınıfı) MAT'in gölgesi DEĞİLDİR ve seyrek geçtiği
        # için elenirse dersin hazırlık çıktıları tümden kaybolur.
        longer, shorter = (prefix, other) if len(prefix) > len(other) else (other, prefix)
        if "." in longer[len(shorter):] or "." in longer[:len(longer) - len(shorter)]:
            continue
        return True
    return False


def check_manifest(shapes: list[dict], declared: dict[str, list[str]]) -> list[dict]:
    """Sayımdaki her şekil manifestte rol almış mı? Almamışsa HATA.

    Değişmez budur: kaynak yeni bir şekil getirdiğinde ayrıştırıcı onu sessizce
    düşürmez, koşum hata verir. eski `detect_scheme` sezgisinin yerini bu alıyor.
    """
    known = {key: role for role, keys in declared.items() for key in keys}
    errors = []
    for shape in shapes:
        key = shape_key(shape)
        if key not in known:
            errors.append({
                "kind": "undeclared_shape",
                "shape": key,
                "count": shape["count"],
                "free": shape["free"],
                "sample": shape["sample"],
            })
    for key in known:
        if key not in {shape_key(s) for s in shapes}:
            errors.append({"kind": "stale_declaration", "shape": key})
    return errors


def declared_families(declared: dict[str, list[str]]) -> list[tuple[list[str], int]]:
    """Manifestin `outcome` rollü şekillerini ayrıştırıcının beklediği biçime çevirir."""
    families: dict[int, list[str]] = collections.defaultdict(list)
    for key in declared.get("outcome", []):
        prefix, _, scheme_name = key.rpartition("/")
        families[SCHEME_CODES[scheme_name]].append(prefix)
    return [(sorted(prefixes), scheme) for scheme, prefixes in sorted(families.items())]


def outcome_families(
    text: str,
    skill_prefixes: frozenset[str],
    free_min: int = FREE_MIN,
) -> list[tuple[list[str], int]]:
    """Sayım + taslak rol -> ayrıştırılacak aileler. Manifest YOKKEN kullanılır.

    Üretimde aileler manifestten gelir; bu işlev manifest taslağını üretirken
    ve testlerde kullanılır. Küçük örneklerde `free_min=1` verilmelidir:
    varsayılan eşik gerçek belgelere göre ayarlıdır.
    """
    shapes = census(text)
    roles = bootstrap_roles(shapes, skill_prefixes, free_min)
    declared: dict[str, list[str]] = collections.defaultdict(list)
    for key, role in roles.items():
        declared[role].append(key)
    return declared_families(declared)
