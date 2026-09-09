# TYMM Ders Planı Veri Katmanı — Uygulama Planı

> **Ajan işçiler için:** GEREKLİ ALT BECERİ: bu planı görev görev uygulamak için
> `superpowers:subagent-driven-development` (önerilen) veya
> `superpowers:executing-plans` kullanın. Adımlar onay kutusu (`- [ ]`) taşır.

**Amaç:** Ders programı PDF'lerinden ünite düzeyinde tam ders planı verisini
çıkarıp her dersi ayrı bir JSON dosyası olarak üretmek.

**Yaklaşım:** MEB'in `/Chart/GetStackCharts` uç noktası ünite başına yalnız
beceri dağılımı veriyor; ders planının omurgası (öğrenme çıktıları, ders saati,
içerik çerçevesi, anahtar kavramlar, farklılaştırma) sadece ders programı
PDF'inde. Mevcut `scripts/fetch_tymm_ogrenme_ciktilari.py` çıktıları çıkarıyor;
bu plan onu ünite bölümlerinin tamamını okuyacak şekilde genişletir ve tek
büyük dosya yerine ders başına dosya üretir.

**Yığın:** Python 3 (yalnız standart kütüphane), `pdftotext` (poppler),
`unittest`.

## Global Kısıtlar

- Kod tanımlayıcıları ve veri dizin adları İngilizce; yorumlar, doküman adları
  ve kullanıcıya görünen metinler Türkçe.
- Yeni Python bağımlılığı YOK. Ayrıştırma standart kütüphaneyle yapılır.
- PDF'ler sürümlenmez (`.tymm-pdf/` `.gitignore`'da). Yalnız üretilen JSON
  sürümlenir.
- Kaynak metin BİREBİR aktarılır. Kaynaktaki yazım hataları düzeltilmez.
- Her ders için **kapsama denetimi** zorunlu: metinde bulunan kod sayısı ile
  ayrıştırılan sayı eşleşmiyorsa `report.gaps`'e yazılır. Sessiz kayıp yasak.
- Ön ek uzunluğu sabitlenmez (`FİZ`, `MAT`, `BEOSAB`, `T.C.İNK` gibi 3-9
  karakter arası değişir).

---

## Dosya Yapısı

**Oluşturulacak:**
- `scripts/tymm_pdf_sections.py` — PDF ünite bölümü ayrıştırıcısı (saf işlev,
  ağa çıkmaz, dosya yazmaz). Tek sorumluluk: metin → yapılandırılmış ünite.
- `tests/scripts/test_tymm_pdf_sections.py` — ayrıştırıcı birim testleri.
- `data/tymm/courses/<slug>.json` — ders başına bir dosya (üretilir).
- `data/tymm/courses/index.json` — ders dizini + sayımlar (üretilir).

**Değiştirilecek:**
- `scripts/fetch_tymm_ogrenme_ciktilari.py` — ayrıştırmayı yeni modüle devreder,
  çıktıyı ders başına dosyaya yazar.
- `.github/workflows/ci.yml` — Python testleri için iş eklenir.
- `data/tymm/README.md` — yeni dizin ve akış belgelenir.

Ayrıştırma mantığı ağ/dosya işinden ayrılıyor çünkü tek gerçek risk
ayrıştırmada ve yalnız saf bir işlev test edilebilir.

---

### Görev 1: Ünite bölümü ayrıştırıcısı

**Dosyalar:**
- Oluştur: `scripts/tymm_pdf_sections.py`
- Test: `tests/scripts/test_tymm_pdf_sections.py`

**Arayüzler:**
- Tüketir: yok (saf işlev).
- Üretir: `parse_units(text: str, prefix: str) -> list[dict]`. Her sözlük şu
  anahtarları taşır: `grade` (int | None), `unit` (int), `title` (str),
  `description` (str), `lesson_hours` (int | None), `sections` (dict[str, str]).

PDF'te bir ünite şu düzendedir (Fizik 9. sınıf 1. ünite, gerçek metin):

```
9. SINIF
1. ÜNİTE: FİZİK BİLİMİ VE KARİYER KEŞFİ
Bu ünitede öğrencilerin temel bir bilim olan fizik bilimini tanımlamaları, ...
DERS SAATİ 8
ALAN
BECERİLERİ            FBAB2. Sınıflandırma, FBAB10. Tümevarımsal Akıl Yürütme
KAVRAMSAL
BECERİLER             KB2.8. Sorgulama, KB2.15. Yansıtma
EĞİLİMLER             E1.4. Kendine İnanma (Öz Yeterlilik), E1.6. Seçicilik, ...
PROGRAMLAR ARASI
BİLEŞENLER
Sosyal-Duygusal
Öğrenme Becerileri    SDB1.1. Kendini Tanıma (Öz Farkındalık), ...
Değerler              D3. Çalışkanlık, D19. Vatanseverlik
Okuryazarlık Becerileri OB1. Bilgi Okuryazarlığı, OB4. Görsel Okuryazarlığı
DİSİPLİNLER ARASI ...
İLİŞKİLER             KB2.7. Karşılaştırma, KB2.10. Çıkarım Yapma
ÖĞRENME ÇIKTILARI
VE SÜREÇ BİLEŞENLERİ  FİZ.9.1.1. Fizik biliminin tanımına yönelik ...
İÇERİK ÇERÇEVESİ      Fizik Bilimi
Anahtar Kavramlar     fizik bilimi, bilimsel araştırma merkezi
ÖĞRENME
KANITLARI             Öğrenme çıktıları; zihin haritası, test ...
FARKLILAŞTIRMA        ...
```

DİKKAT: `pdftotext -layout` çok kelimeli etiketleri satırlara böler
(`ALAN` ve `BECERİLERİ` ayrı satırlarda). Ayrıştırıcı etiketi tek satırda
aramamalıdır.

- [ ] **Adım 1: Başarısız testi yaz**

`tests/scripts/test_tymm_pdf_sections.py`:

```python
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))

from tymm_pdf_sections import parse_units

SAMPLE = """9. SINIF
1. ÜNİTE: FİZİK BİLİMİ VE KARİYER KEŞFİ
Bu ünitede öğrencilerin fizik bilimini tanımlamaları amaçlanmaktadır.
DERS SAATİ 8
ALAN
BECERİLERİ            FBAB2. Sınıflandırma, FBAB10. Tümevarımsal Akıl Yürütme
KAVRAMSAL
BECERİLER             KB2.8. Sorgulama, KB2.15. Yansıtma
EĞİLİMLER             E1.4. Kendine İnanma, E3.8. Soru Sorma
Değerler              D3. Çalışkanlık, D19. Vatanseverlik
Okuryazarlık Becerileri OB1. Bilgi Okuryazarlığı
İLİŞKİLER             KB2.7. Karşılaştırma
İÇERİK ÇERÇEVESİ      Fizik Bilimi
Anahtar Kavramlar     fizik bilimi, bilimsel araştırma merkezi
2. ÜNİTE: KUVVET VE HAREKET
Bu ünitede kuvvet ele alınmaktadır.
DERS SAATİ 24
"""


class ParseUnitsTest(unittest.TestCase):
    def test_iki_unite_bulunur(self):
        units = parse_units(SAMPLE, "FİZ")
        self.assertEqual(len(units), 2)

    def test_baslik_ve_sinif_okunur(self):
        unit = parse_units(SAMPLE, "FİZ")[0]
        self.assertEqual(unit["grade"], 9)
        self.assertEqual(unit["unit"], 1)
        self.assertEqual(unit["title"], "FİZİK BİLİMİ VE KARİYER KEŞFİ")

    def test_ders_saati_sayiya_cevrilir(self):
        self.assertEqual(parse_units(SAMPLE, "FİZ")[0]["lesson_hours"], 8)
        self.assertEqual(parse_units(SAMPLE, "FİZ")[1]["lesson_hours"], 24)

    def test_cok_satirli_etiket_birlestirilir(self):
        # "ALAN" ve "BECERİLERİ" ayrı satırlarda; tek bölüm olmalı.
        sections = parse_units(SAMPLE, "FİZ")[0]["sections"]
        self.assertIn("ALAN BECERİLERİ", sections)
        self.assertIn("FBAB2. Sınıflandırma", sections["ALAN BECERİLERİ"])

    def test_aciklama_baslik_ile_ders_saati_arasindadir(self):
        unit = parse_units(SAMPLE, "FİZ")[0]
        self.assertTrue(unit["description"].startswith("Bu ünitede"))
        self.assertNotIn("DERS SAATİ", unit["description"])

    def test_anahtar_kavramlar_okunur(self):
        sections = parse_units(SAMPLE, "FİZ")[0]["sections"]
        self.assertEqual(
            sections["Anahtar Kavramlar"], "fizik bilimi, bilimsel araştırma merkezi"
        )


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Adım 2: Testi çalıştır, başarısız olduğunu gör**

Çalıştır: `python3 -m unittest discover -s tests/scripts -v`
Beklenen: FAIL — `ModuleNotFoundError: No module named 'tymm_pdf_sections'`

- [ ] **Adım 3: Ayrıştırıcıyı yaz**

`scripts/tymm_pdf_sections.py`:

```python
#!/usr/bin/env python3
"""Ders programı PDF metninden ünite bölümlerini çıkarır.

Saf işlev: ağa çıkmaz, dosya okumaz/yazmaz. Tek sorumluluğu metni yapıya
çevirmek; indirme ve yazma işi çağıranındır.
"""

from __future__ import annotations

import re

UNIT_RE = re.compile(r"^\s*(\d+)\.\s*ÜNİTE\s*:\s*(.+?)\s*$")
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
    lines = text.splitlines()
    units: list[dict] = []
    grade: int | None = None
    current: dict | None = None
    section: str | None = None
    label_buffer = ""

    def flush() -> None:
        nonlocal current, section, label_buffer
        if current is not None:
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
                "title": unit_hit.group(2).strip(),
                "description": "",
                "lesson_hours": None,
                "sections": {},
            }
            continue

        if current is None:
            continue

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
    return units
```

- [ ] **Adım 4: Testi çalıştır, geçtiğini gör**

Çalıştır: `python3 -m unittest discover -s tests/scripts -v`
Beklenen: 6 test PASS

- [ ] **Adım 5: Gerçek PDF üzerinde duman testi**

```bash
python3 scripts/fetch_tymm_ogrenme_ciktilari.py --only fizik-dersi --keep-pdf
python3 - <<'PY'
import sys; sys.path.insert(0, "scripts")
from tymm_pdf_sections import parse_units
text = open(".tymm-pdf/dersler/fizik-dersi.txt", encoding="utf-8").read()
units = parse_units(text, "FİZ")
print("ünite:", len(units))
for u in units[:3]:
    print(f"  {u['grade']}/{u['unit']} {u['title'][:40]:<42} saat={u['lesson_hours']} bölüm={len(u['sections'])}")
PY
```

Beklenen: 15 ünite (9/1..9/4, 10/1..10/4, 11/1..11/3, 12/1..12/4);
9. sınıf 1. ünite `saat=8`, `bölüm` en az 8.

- [ ] **Adım 6: Commit**

```bash
git add scripts/tymm_pdf_sections.py tests/scripts/test_tymm_pdf_sections.py
git commit -m "feat: ders programı PDF'inden ünite bölümü ayrıştırıcısı"
```

---

### Görev 2: Ders başına JSON üretimi

**Dosyalar:**
- Değiştir: `scripts/fetch_tymm_ogrenme_ciktilari.py`

**Arayüzler:**
- Tüketir: Görev 1'in `parse_units(text, prefix)` işlevi.
- Üretir: `data/tymm/courses/<slug>.json` dosyaları ve
  `data/tymm/courses/index.json`.

Ders başına dosya iki sorunu birden çözer: 6 MB'lık tek dosyanın git diff'i
okunmuyordu, ve bir dersin verisi değiştiğinde yalnız o dosya değişiyor.

Dosya şeması:

```json
{
  "course": {
    "id": 17,
    "name": "Fizik Dersi",
    "slug": "fizik-dersi",
    "prefix": "FİZ",
    "kademe": 3,
    "kademe_name": "Ortaöğretim",
    "pdf_url": "https://tymm.meb.gov.tr/assets/pdf/fizik-dersi_20260819_142915_270.pdf"
  },
  "units": [
    {
      "grade": 9,
      "unit": 1,
      "title": "FİZİK BİLİMİ VE KARİYER KEŞFİ",
      "description": "Bu ünitede öğrencilerin ...",
      "lesson_hours": 8,
      "sections": { "ALAN BECERİLERİ": "FBAB2. Sınıflandırma, ..." },
      "outcomes": [
        {
          "code": "FİZ.9.1.2",
          "order": 2,
          "text": "Fizik biliminin alt dallarını sınıflandırabilme",
          "components": [
            { "label": "a", "text": "Fizik biliminin alt dallarının niteliklerini belirler." }
          ]
        }
      ]
    }
  ],
  "report": { "units": 15, "outcomes": 106, "components": 310, "gaps": [] }
}
```

- [ ] **Adım 1: Ayrıştırmayı yeni modüle devret**

`scripts/fetch_tymm_ogrenme_ciktilari.py` içindeki `parse_outcomes` KALIR
(çıktı ayrıştırması orada doğru çalışıyor, Fizik'te 106/106). İçe aktarma
bloğuna ekle:

```python
sys.path.insert(0, str(Path(__file__).resolve().parent))
from tymm_pdf_sections import parse_units
from fetch_tymm_dersler import KADEME_NAMES
```

`KADEME_NAMES`'i kopyalamayın; iki yerde duran bir sözlük er ya da geç ayrışır.

- [ ] **Adım 2: Çıktıları ünitelere bağla**

`main()` içinde, `outcomes = parse_outcomes(text, prefix)` satırından sonra:

```python
        units = parse_units(text, prefix)
        by_unit = {(u["grade"], u["unit"]): u for u in units}
        for unit in units:
            unit["outcomes"] = []
        orphan_outcomes = []
        for outcome in outcomes:
            target = by_unit.get((outcome["grade"], outcome["unit"]))
            if target is None:
                # Ünite başlığı bulunamadı; çıktı KAYBOLMAZ, ayrı listede durur.
                orphan_outcomes.append(outcome)
                continue
            target["outcomes"].append(
                {
                    "code": outcome["code"],
                    "order": outcome["order"],
                    "text": outcome["text"],
                    "components": outcome["components"],
                }
            )
        if orphan_outcomes:
            gaps.append({
                "course": course["name"],
                "reason": "ünitesi bulunamayan çıktı",
                "codes": [o["code"] for o in orphan_outcomes],
            })
```

Ardından `results.append(...)` çağrısına `"units": units` alanını ekleyin.

- [ ] **Adım 3: Ders başına dosya yaz**

`main()` içinde, tek büyük `ogrenme-ciktilari.json` yazan bloğun yerine:

```python
    courses_dir = args.data_dir / "courses"
    courses_dir.mkdir(parents=True, exist_ok=True)
    index = []
    for result in results:
        document = {
            "course": {
                "id": result["course_id"],
                "name": result["course"],
                "slug": result["course_url"],
                "prefix": result["prefix"],
                "kademe": result["kademe"],
                "kademe_name": KADEME_NAMES.get(result["kademe"], str(result["kademe"])),
                "pdf_url": result["pdf_url"],
            },
            "units": result["units"],
            "report": {
                "units": len(result["units"]),
                "outcomes": sum(len(u["outcomes"]) for u in result["units"]),
                "components": sum(
                    len(o["components"]) for u in result["units"] for o in u["outcomes"]
                ),
                "gaps": [g for g in gaps if g["course"] == result["course"]],
            },
        }
        path = courses_dir / f"{result['course_url']}.json"
        path.write_text(
            json.dumps(document, ensure_ascii=False, indent=1) + "\n", encoding="utf-8"
        )
        index.append({
            "slug": result["course_url"],
            "name": result["course"],
            "kademe": result["kademe"],
            "prefix": result["prefix"],
            "file": f"courses/{result['course_url']}.json",
            "units": document["report"]["units"],
            "outcomes": document["report"]["outcomes"],
            "components": document["report"]["components"],
        })

    (courses_dir / "index.json").write_text(
        json.dumps(
            {
                "source": {
                    "name": "Türkiye Yüzyılı Maarif Modeli — Ders Programları",
                    "fetched_at": datetime.date.today().isoformat(),
                    "generator": "scripts/fetch_tymm_ogrenme_ciktilari.py",
                },
                "report": {
                    "courses": len(index),
                    "units": sum(c["units"] for c in index),
                    "outcomes": sum(c["outcomes"] for c in index),
                    "components": sum(c["components"] for c in index),
                    "gaps": gaps,
                },
                "courses": sorted(index, key=lambda c: (c["kademe"], c["name"])),
            },
            ensure_ascii=False,
            indent=1,
        )
        + "\n",
        encoding="utf-8",
    )
```

- [ ] **Adım 4: Tek ders üzerinde çalıştır**

```bash
python3 scripts/fetch_tymm_ogrenme_ciktilari.py --only fizik-dersi
python3 -c "
import json
d = json.load(open('data/tymm/courses/fizik-dersi.json', encoding='utf-8'))
print(d['report'])
u = d['units'][0]
print(u['grade'], u['unit'], u['title'], 'saat=', u['lesson_hours'], 'çıktı=', len(u['outcomes']))
"
```

Beklenen: `{'units': 15, 'outcomes': 106, 'components': 310, 'gaps': []}`
ve ilk ünite `9 1 FİZİK BİLİMİ VE KARİYER KEŞFİ saat= 8 çıktı= 4`.

- [ ] **Adım 5: Commit**

```bash
git add scripts/fetch_tymm_ogrenme_ciktilari.py data/tymm/courses/fizik-dersi.json data/tymm/courses/index.json
git commit -m "feat: ders programı verisi ders başına JSON dosyasına ayrıldı"
```

---

### Görev 3: Tam koşum ve doğrulama

**Dosyalar:**
- Oluştur: `scripts/check_tymm_courses.py`
- Değiştir: `data/tymm/README.md`

**Arayüzler:**
- Tüketir: `data/tymm/courses/index.json`, `data/tymm/beceriler.json`.
- Üretir: yok (doğrulama script'i; çıkış kodu 1 = sorun var).

- [ ] **Adım 1: Doğrulama script'ini yaz**

`scripts/check_tymm_courses.py`:

```python
#!/usr/bin/env python3
"""Üretilen ders dosyalarını doğrular; sorun varsa çıkış kodu 1.

Üç şeye bakar:
  1. index.json'daki her ders dosyası gerçekten var mı,
  2. hiçbir derste `report.gaps` dolu mu (dolu = sessiz kayıp),
  3. ünite bölümlerinde geçen beceri kodları beceriler.json'da çözülüyor mu.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

DATA = Path("data/tymm")
CODE_RE = re.compile(r"\b([A-ZÇĞİÖŞÜ]{1,8}\d[\d.]*)\.")


def main() -> int:
    index = json.loads((DATA / "courses" / "index.json").read_text(encoding="utf-8"))
    beceriler = json.loads((DATA / "beceriler.json").read_text(encoding="utf-8"))
    known = {k["code"] for s in beceriler["sets"] for k in s["skills"]}
    known |= {
        c["code"]
        for s in beceriler["sets"]
        for k in s["skills"]
        for c in k["components"]
    }

    problems: list[str] = []
    unresolved: dict[str, int] = {}

    for entry in index["courses"]:
        path = DATA / entry["file"]
        if not path.exists():
            problems.append(f"eksik dosya: {entry['file']}")
            continue
        document = json.loads(path.read_text(encoding="utf-8"))
        for gap in document["report"]["gaps"]:
            problems.append(f"{entry['slug']}: {gap['reason']} {gap.get('codes', '')}")
        for unit in document["units"]:
            for text in unit["sections"].values():
                for match in CODE_RE.finditer(text):
                    code = match.group(1)
                    if code not in known:
                        unresolved[code] = unresolved.get(code, 0) + 1

    print(f"ders: {index['report']['courses']}  ünite: {index['report']['units']}")
    print(f"çıktı: {index['report']['outcomes']}  bileşen: {index['report']['components']}")
    print(f"çözülemeyen beceri kodu: {len(unresolved)}")
    for code, count in sorted(unresolved.items(), key=lambda kv: -kv[1])[:10]:
        print(f"   {code} x{count}")
    for problem in problems:
        print(f"  SORUN {problem}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
```

- [ ] **Adım 2: 111 ders için tam koşum**

```bash
python3 scripts/fetch_tymm_ogrenme_ciktilari.py
```

Beklenen: her satırda `kayıp=0`. Sıfırdan büyük bir değer görülürse Görev 1'e
dönülür; kapsama denetimi bunun için var.

- [ ] **Adım 3: Doğrula**

```bash
python3 scripts/check_tymm_courses.py; echo "çıkış=$?"
```

Beklenen: `çıkış=0`, `SORUN` satırı yok. Çözülemeyen beceri kodları sıfır
olmayabilir; bilinen `KB2.16.1/.2/.3` tutarsızlığı buraya da düşer, kabul
edilir ve README'de not edilir.

- [ ] **Adım 4: README'yi güncelle**

`data/tymm/README.md`'deki "Sürümlenen dosyalar" tablosuna satır ekle:

```markdown
| `courses/<slug>.json` | 111 dosya | Ders programı PDF'lerinden ünite, ders saati, öğrenme çıktısı ve süreç bileşenleri |
| `courses/index.json`  | 1 dosya   | Ders dizini + toplam sayımlar |
```

ve "Yeniden üretim" bloğuna ekle:

```bash
python3 scripts/fetch_tymm_ogrenme_ciktilari.py   # ağ; PDF indirir, metne çevirir, siler
python3 scripts/check_tymm_courses.py             # doğrulama
```

- [ ] **Adım 5: Commit**

```bash
git add data/tymm/courses scripts/check_tymm_courses.py data/tymm/README.md
git commit -m "feat: 111 dersin ünite ve öğrenme çıktısı verisi"
```

---

### Görev 4: Python testleri CI'a bağlanır

**Dosyalar:**
- Değiştir: `.github/workflows/ci.yml`

Test yazıp CI'a bağlamamak, testin çalışmadığı anlamına gelir. Bu depoda CI
yalnız `dev`/`main` hedefli PR'larda koşar; özellik dalları arası PR'da
"no checks reported" çıkar.

- [ ] **Adım 1: İş ekle**

`.github/workflows/ci.yml` içindeki `jobs:` altına:

```yaml
  python:
    name: Python
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      # Bağımlılık kurulumu YOK: script'ler yalnız standart kütüphane kullanır.
      - name: Birim testleri
        run: python3 -m unittest discover -s tests/scripts -v
```

Mevcut işlerin `uses:` satırları commit SHA'sına sabitlenmişse (bkz. `775b7a5`)
bu iki eylemi de aynı biçimde sabitleyin.

- [ ] **Adım 2: Yerelde doğrula**

```bash
python3 -m unittest discover -s tests/scripts -v
```

Beklenen: 6 test PASS.

- [ ] **Adım 3: Commit ve PR**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: Python birim testleri için iş eklendi"
git push -u origin feat/tymm-ders-plani
gh pr create --base dev --title "feat: TYMM ders planı veri katmanı"
```

CI yeşil olunca `dev`'e merge, sonra `dev` → `main`, sonra sürüm etiketi
(`main`'e her birleşmeden sonra sürüm yayımlanır).

---

## Kapsam dışı

Bu plan yalnız VERİ katmanını üretir. Aşağıdakiler ayrı planlara aittir:

- tayan arayüzünde ders planı ekranı.
- Taslak yıllık planların (`/assets/file/*-taslak-yillik-planlar_*.zip`)
  indirilip ayrıştırılması.
- `dersler.json`'un emekliye ayrılması. Beceri yüzdeleri (`share`) ve öğrenme
  kanıtı listeleri hâlâ yalnız orada; ders dosyalarıyla birleştirme kararı
  verilene kadar iki kaynak da durur.
