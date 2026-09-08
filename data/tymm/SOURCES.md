# TYMM beceri çerçevesi — kaynaklar

Çekim tarihi: 2026-09-08

`data/tymm/beceriler.json` bu sayfalardan üretildi. Elle düzenlenmez;
yeniden üretmek için `python3 scripts/fetch_tymm_beceriler.py`.

| Set | Ad | Beceri | Bileşen | Kaynak |
|---|---|---:|---:|---|
| `KB` | Kavramsal Beceriler | 38 | 78 | https://tymm.meb.gov.tr/beceriler/kavramsal-beceriler https://tymm.meb.gov.tr/beceriler/turkce-alan-becerileri https://tymm.meb.gov.tr/beceriler/matematik-alan-becerileri https://tymm.meb.gov.tr/beceriler/sosyal-bilimler-alan-becerileri https://tymm.meb.gov.tr/beceriler/sanat-alan-becerileri https://tymm.meb.gov.tr/beceriler/bilisim-teknolojileri-ve-yazilim-alan-becerileri https://tymm.meb.gov.tr/beceriler/din-egitimi-ve-ogretimi-alan-becerileri |
| `SDB` | Sosyal-Duygusal Öğrenme Becerileri | 12 | 225 | https://tymm.meb.gov.tr/beceriler/sosyal-duygusal-ogrenme-becerileri |
| `E` | Eğilimler | 24 | 0 | https://tymm.meb.gov.tr/beceriler/egilimler |
| `D` | Erdem-Değer-Eylem Çerçevesi | 90 | 405 | https://tymm.meb.gov.tr/beceriler/erdem-deger-eylem-cercevesi |
| `OB` | Okuryazarlık Becerileri | 60 | 149 | https://tymm.meb.gov.tr/beceriler/okuryazarlik-becerileri |
| `—` | Fiziksel Beceriler | 0 | 0 | https://tymm.meb.gov.tr/beceriler/fiziksel-beceriler |
| `TAB` | Türkçe Alan Becerileri (TAB) | 18 | 60 | https://tymm.meb.gov.tr/beceriler/turkce-alan-becerileri |
| `MAB` | Matematik Alan Becerileri (MAB) | 14 | 22 | https://tymm.meb.gov.tr/beceriler/matematik-alan-becerileri |
| `FBAB` | Fen Bilimleri Alan Becerileri (FBAB) | 13 | 44 | https://tymm.meb.gov.tr/beceriler/fen-bilimleri-alan-becerileri |
| `SBAB` | Sosyal Bilimler Alan Becerileri (SBAB) | 77 | 198 | https://tymm.meb.gov.tr/beceriler/sosyal-bilimler-alan-becerileri |
| `SAB` | Sanat Alan Becerileri (SAB) | 30 | 60 | https://tymm.meb.gov.tr/beceriler/sanat-alan-becerileri |
| `BEOSAB` | Beden Eğitimi, Oyun ve Spor Alan Becerileri (BEOSAB) | 10 | 27 | https://tymm.meb.gov.tr/beceriler/beden-egitimi-oyun-ve-spor-alan-becerileri |
| `BTYAB` | Bilişim Teknolojileri ve Yazılım Alan Becerileri (BTYAB) | 28 | 68 | https://tymm.meb.gov.tr/beceriler/bilisim-teknolojileri-ve-yazilim-alan-becerileri |
| `TSRMAB` | Tasarım Alan Becerileri (TSRMAB) | 8 | 23 | https://tymm.meb.gov.tr/beceriler/tasarim-alan-becerileri |
| `DAB` | Din Eğitimi ve Öğretimi Alan Becerileri (DAB) | 25 | 87 | https://tymm.meb.gov.tr/beceriler/din-egitimi-ve-ogretimi-alan-becerileri |
| `YDAB` | Yabancı Dil Alan Becerileri ve Yabancı Dil Destekleyici Beceriler | 4 | 0 | https://tymm.meb.gov.tr/beceriler/yabanci-dil-alan-becerileri |
| `YDDB` | Yabancı Dil Destekleyici Becerileri | 11 | 66 | https://tymm.meb.gov.tr/beceriler/yabanci-dil-alan-becerileri |
| | **Toplam** | **462** | **1512** | |

## Notlar

- Alan beceri sayfaları başka setlerin kodlarına atıf yapar (ör. matematik
  sayfasındaki `KB` kodları). Her düğüm KENDİ ön ekinin setine yazılır;
  aynı set birden çok sayfada geçerse `source_urls` hepsini listeler ve
  beceriler koda göre tekilleştirilir.
- Kaynakta aynı kod birden fazla kez tanımlanabiliyor (ör. `YDDB1.1` üç
  öğretim yaklaşımı için). Hiçbiri atılmaz; ayırt edici bilgi `context`
  alanındadır ve tamamı `report.duplicate_codes` altında listelenir.
- Kaynakta süreç bileşeni kodları üç ayrı yazımla geçiyor: `SB8`, hatalı
  `SB.8` ve boşluklu `. SB5`. Ayrıştırıcı üçünü de tanır, kodu `SB8`
  biçimine normalleştirir.
- Açıklaması boş düğümlerin çoğunda kaynakta da açıklama yoktur (ör. `OB`,
  `SBAB`); bu bir çekim kaybı değildir.
- Fiziksel Beceriler sayfası kodlanmış liste içermez; düz anlatımdır.
- Kod ön ekleri 1-6 harf arasında değişir (`E`, `KB`, `SDB`, `BEOSAB`,
  `TSRMAB`); ayrıştırıcı ön ek uzunluğunu sabitlemez.
