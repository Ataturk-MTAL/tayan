#!/usr/bin/env python3
"""Ders programı metinlerindeki kod şekillerini sayar ve şekil manifestini üretir.

İki dosya yazar:

    data/tymm/shape-census.json     ÖLÇÜLEN — hangi belgede hangi şekil, kaç kez
    data/tymm/shape-manifest.json   BEYAN EDİLEN — her şekil NE (rol)

Ayrım kasıtlı. Sayım PDF'lerden yeniden üretilir; kaynak değişince sayılar
oynar. Manifest elle gözden geçirilen karardır ve nadiren değişir.
Ayrıştırıcı manifesti okur, sayımı değil.

DEĞİŞMEZ: sayımdaki her şekil manifestte bir rol almalıdır. Almayan şekil
HATA'dır — `fetch_tymm_ogrenme_ciktilari.py` boşluk kaydı üretir. Bu, şemayı
sezgiyle seçen eski `detect_scheme` işlevinin (silindi) yerini alır: azınlıkta kalan aile artık
sessizce düşmez, beyan edilmemiş aile bağırır.

Manifest ZATEN VARSA rolleri korunur; yalnız yeni şekiller taslak rolle
eklenir ve kaybolan şekiller çıkarılır. Elle verilen kararlar ezilmez.

Kullanım:
    python3 scripts/build_tymm_shape_manifest.py
    python3 scripts/build_tymm_shape_manifest.py --text-dir .tymm-pdf/dersler
"""

from __future__ import annotations

import argparse
import collections
import datetime
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from tymm_outcomes import SKILL_PREFIXES  # noqa: E402
from tymm_shape_census import (  # noqa: E402
    bootstrap_roles,
    census,
    shape_key,
)

ROLE_NOTES = {
    "outcome": "Öğrenme çıktısı ailesi. Ayrıştırıcı YALNIZ bunları işler.",
    "skill_reference": "Beceri çerçevesi kodu (KB, SDB, CS, SELS…). Çıktı değil.",
    "component": "Süreç bileşeni kademesi: satırda kodun önünde 'a)' etiketi var. "
                 "Türk Dili'nde çıktı 'TDE1.1.', bileşen 'a) TDE1.1.1.'.",
    "heading": "Aynı ön ekin bir ÜST kademesi: ünite/öğrenme alanı/tema başlığı. "
               "'MAT.1.1.' başlıktır, çıktı 'MAT.1.1.1.'.",
    "shadow": "Başka bir şeklin gölgesi: aynı konumda daha uzun kod eşleşti "
              "ya da kuyruk rakamla başlıyor (kod daha derin).",
    "noise": "pdftotext artefaktı: kırpılmış ön ek ya da eşik altı geçiş.",
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--text-dir", type=Path, default=Path(".tymm-pdf/dersler"))
    parser.add_argument("--data-dir", type=Path, default=Path("data/tymm"))
    args = parser.parse_args()

    texts = sorted(args.text_dir.glob("*.txt"))
    if not texts:
        print(
            f"HATA: {args.text_dir} altında .txt yok. Önce "
            "fetch_tymm_ogrenme_ciktilari.py --keep-pdf çalıştır.",
            file=sys.stderr,
        )
        return 1

    manifest_path = args.data_dir / "shape-manifest.json"
    previous: dict[str, dict[str, list[str]]] = {}
    overrides: dict[str, dict[str, dict]] = {}
    if manifest_path.exists():
        stored = json.loads(manifest_path.read_text(encoding="utf-8"))
        previous = stored["courses"]
        overrides = stored.get("overrides", {})

    census_courses: dict[str, list[dict]] = {}
    manifest_courses: dict[str, dict[str, list[str]]] = {}
    kept = added = removed = overridden = 0

    for path in texts:
        slug = path.stem
        shapes = census(path.read_text(encoding="utf-8", errors="replace"))
        census_courses[slug] = shapes

        draft = bootstrap_roles(shapes, SKILL_PREFIXES)
        held = {key: role for role, keys in previous.get(slug, {}).items() for key in keys}

        # ELLE GEÇERSİZ KILMA taslağı DA korunan rolü DE ezer ve gerekçesi
        # manifestte yazılıdır. Kural kovalamak yerine tekil kararı yazmak,
        # sezgiyi yeniden üretmeden düzeltmenin tek dürüst yolu.
        forced = overrides.get(slug, {})

        roles: dict[str, list[str]] = collections.defaultdict(list)
        for shape in shapes:
            key = shape_key(shape)
            if key in forced:
                roles[forced[key]["role"]].append(key)
                overridden += 1
            elif key in held:
                roles[held[key]].append(key)
                kept += 1
            else:
                roles[draft[key]].append(key)
                added += 1
        removed += len(set(held) - {shape_key(s) for s in shapes})

        manifest_courses[slug] = {
            role: sorted(roles[role]) for role in ROLE_NOTES if roles[role]
        }

    args.data_dir.mkdir(parents=True, exist_ok=True)
    stamp = datetime.date.today().isoformat()

    (args.data_dir / "shape-census.json").write_text(
        json.dumps(
            {
                "source": {
                    "name": "TYMM ders programı PDF'lerindeki kod şekilleri",
                    "measured_at": stamp,
                    "generator": "scripts/build_tymm_shape_manifest.py",
                    "note": "ÖLÇÜLEN. Elle düzenlenmez; PDF'lerden yeniden üretilir.",
                },
                "report": {
                    "courses": len(census_courses),
                    "shapes": sum(len(s) for s in census_courses.values()),
                },
                "courses": census_courses,
            },
            ensure_ascii=False,
            indent=1,
        ) + "\n",
        encoding="utf-8",
    )

    manifest_path.write_text(
        json.dumps(
            {
                "source": {
                    "name": "TYMM kod şekli manifesti",
                    "updated_at": stamp,
                    "generator": "scripts/build_tymm_shape_manifest.py",
                    "note": "BEYAN EDİLEN. Elle düzenlenebilir; mevcut roller korunur. "
                            "Sayımda olup burada rolü olmayan şekil HATA'dır.",
                },
                "roles": ROLE_NOTES,
                "report": {
                    "courses": len(manifest_courses),
                    "declarations": sum(
                        len(v) for c in manifest_courses.values() for v in c.values()
                    ),
                    "kept": kept,
                    "overridden": overridden,
                    "added": added,
                    "removed": removed,
                },
                "overrides": overrides,
                "courses": manifest_courses,
            },
            ensure_ascii=False,
            indent=1,
        ) + "\n",
        encoding="utf-8",
    )

    counts = collections.Counter(
        role for c in manifest_courses.values() for role, keys in c.items() for _ in keys
    )
    print(
        f"yazıldı: {manifest_path} — {len(manifest_courses)} ders, "
        f"korunan={kept} eklenen={added} düşen={removed} elle={overridden}",
        file=sys.stderr,
    )
    for role in ROLE_NOTES:
        print(f"  {role:<16} {counts[role]}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
