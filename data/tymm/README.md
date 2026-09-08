# TYMM veri seti

MEB Türkiye Yüzyılı Maarif Modeli'nden çekilmiş beceri çerçevesi, ders
programları ve ölçme-değerlendirme rehberi.

## Sürümlenen dosyalar

| Dosya | Boyut | Kaynak |
|---|---:|---|
| `beceriler.json` | 392 KB | 16 statik sayfa — 17 set, 462 beceri, 1512 süreç bileşeni |
| `dersler.json` | 6.0 MB | 2 uç nokta, 111 ders — 32 639 beceri eşlemesi |
| `olcme-rehberi.json` | 40 KB | 2 PDF'ten elle çıkarılmış ölçme kuralları |

`dersler.json` yeniden çekmek 111 HTTP çağrısı demek; bu yüzden sürümlenir.

## Üretilen, sürümlenmeyen dosyalar

`ders-beceri.jsonl` (5.9 MB) + `ders-beceri.report.json` iki kaynak
dosyadan tamamen türetilir; ağa çıkmadan saniyeler içinde üretilir. 5.9 MB'lık
türetilmiş veriyi sürümlemek yerine üretim adımı olarak durur:

```bash
python3 scripts/build_tymm_ders_beceri.py --jsonl
```

JSONL satır satır okunur; 32 639 satırı belleğe topluca almadan akıtabilirsin.
Satır sayısı, çözülemeyen kodlar ve ad uyuşmazlığı sayısı JSONL'e sığmadığı için
yan dosya `ders-beceri.report.json`'a yazılır.

## Yeniden üretim

```bash
python3 scripts/fetch_tymm_beceriler.py        # ağ
python3 scripts/fetch_tymm_dersler.py          # ağ
python3 scripts/build_tymm_ders_beceri.py      # ağ yok
python3 scripts/fetch_tymm_kilavuzlar.py       # ağ; PDF'ler .tymm-pdf/ (sürümlenmez)
```

`olcme-rehberi.json` mekanik üretilmez — kaynak serbest metindir, içerik
PDF'lerden elle çıkarılmıştır. Kapsam dışı bırakılanlar ve kaynak
tutarsızlıkları dosyanın `notes` alanındadır.

## Birleştirme

Beceri kodları (`KB2.8`, `D9`, `SBAB1.1` …) üç dosyada da ortaktır.
32 639 eşlemenin 32 612'si çözülür (%99,92). Çözülemeyen `KB2.16.1/.2/.3`
kaynak tutarsızlığıdır; satırlar `resolved: false` ile durur, atılmaz.

Ayrıntı: `SOURCES.md` (beceri çerçevesi), `DERSLER.md` (ders programları).
