# TYMM ders programları — kaynaklar

Çekim tarihi: 2026-09-08

`data/tymm/dersler.json` iki uç noktadan üretildi. Elle düzenlenmez;
yeniden üretmek için `python3 scripts/fetch_tymm_dersler.py`.

- `/Ders/GetDerslerBySinif` — ders dizini (kademe × sınıf)
- `/Chart/GetStackCharts` — ünite başına beceri ve öğrenme kanıtı eşlemesi

| Kademe | Ders | Beceri eşlemesi |
|---|---:|---:|
| Temel Eğitim | 57 | 15553 |
| Ortaöğretim | 30 | 10098 |
| Spor Ortaokulları | 5 | 1257 |
| Müzik Okulları | 19 | 5731 |
| **Toplam** | **111** | **32639** |

## Notlar

- Beceri kodları (`KB2.8`, `SDB1.2` …) `data/tymm/beceriler.json` ile
  doğrudan eşleşir; iki veri seti kod üzerinden birleştirilebilir.
- Kodsuz eşleme: 0/32639. Bunlarda kaynak
  yalnız beceri adını vermiştir; metin `name` alanında korunur.
- Ders programının tam metni PDF'tedir. PDF'ler indirilmez; adresleri
  `pdf_url` alanındadır.
- Öğrenme kanıtları `evidence` (ölçme kanıtı) ve `evaluation` (puanlama
  aracı) olarak ayrı tutulur; kaynakta ikincisi `Değerlendirme:` alt
  başlığı altındadır.
