# TYMM beceri çerçevesi — kaynaklar

Çekim tarihi: 2026-09-09

`data/tymm/beceriler.json` bu sayfalardan üretildi. Elle düzenlenmez;
yeniden üretmek için `python3 scripts/fetch_tymm_beceriler.py`.

| Set | Ad | Beceri | Bileşen | Kaynak |
|---|---|---:|---:|---|
| `KB` | Kavramsal Beceriler | 38 | 78 | https://tymm.meb.gov.tr/beceriler/kavramsal-beceriler https://tymm.meb.gov.tr/beceriler/turkce-alan-becerileri https://tymm.meb.gov.tr/beceriler/matematik-alan-becerileri https://tymm.meb.gov.tr/beceriler/sosyal-bilimler-alan-becerileri https://tymm.meb.gov.tr/beceriler/sanat-alan-becerileri https://tymm.meb.gov.tr/beceriler/bilisim-teknolojileri-ve-yazilim-alan-becerileri https://tymm.meb.gov.tr/beceriler/din-egitimi-ve-ogretimi-alan-becerileri |
| `SDB` | Sosyal-Duygusal Öğrenme Becerileri | 12 | 36 | https://tymm.meb.gov.tr/beceriler/sosyal-duygusal-ogrenme-becerileri |
| `E` | Eğilimler | 24 | 0 | https://tymm.meb.gov.tr/beceriler/egilimler |
| `D` | Erdem-Değer-Eylem Çerçevesi | 90 | 405 | https://tymm.meb.gov.tr/beceriler/erdem-deger-eylem-cercevesi |
| `OB` | Okuryazarlık Becerileri | 60 | 149 | https://tymm.meb.gov.tr/beceriler/okuryazarlik-becerileri |
| `—` | Fiziksel Beceriler | 0 | 0 | https://tymm.meb.gov.tr/beceriler/fiziksel-beceriler |
| `TAB` | Türkçe Alan Becerileri | 18 | 60 | https://tymm.meb.gov.tr/beceriler/turkce-alan-becerileri |
| `MAB` | Matematik Alan Becerileri | 14 | 22 | https://tymm.meb.gov.tr/beceriler/matematik-alan-becerileri |
| `FBAB` | Fen Bilimleri Alan Becerileri | 13 | 44 | https://tymm.meb.gov.tr/beceriler/fen-bilimleri-alan-becerileri |
| `SBAB` | Sosyal Bilimler Alan Becerileri | 77 | 198 | https://tymm.meb.gov.tr/beceriler/sosyal-bilimler-alan-becerileri |
| `SAB` | Sanat Alan Becerileri | 30 | 60 | https://tymm.meb.gov.tr/beceriler/sanat-alan-becerileri |
| `BEOSAB` | Beden Eğitimi, Oyun ve Spor Alan Becerileri | 10 | 27 | https://tymm.meb.gov.tr/beceriler/beden-egitimi-oyun-ve-spor-alan-becerileri |
| `BTYAB` | Bilişim Teknolojileri ve Yazılım Alan Becerileri | 28 | 68 | https://tymm.meb.gov.tr/beceriler/bilisim-teknolojileri-ve-yazilim-alan-becerileri |
| `TSRMAB` | Tasarım Alan Becerileri | 8 | 23 | https://tymm.meb.gov.tr/beceriler/tasarim-alan-becerileri |
| `DAB` | Din Eğitimi ve Öğretimi Alan Becerileri | 28 | 85 | https://tymm.meb.gov.tr/beceriler/din-egitimi-ve-ogretimi-alan-becerileri |
| `YDAB` | Yabancı Dil Alan Becerileri | 4 | 0 | https://tymm.meb.gov.tr/beceriler/yabanci-dil-alan-becerileri |
| `YDDB` | Yabancı Dil Destekleyici Becerileri | 11 | 66 | https://tymm.meb.gov.tr/beceriler/yabanci-dil-alan-becerileri |
| | **Toplam** | **465** | **1321** | |

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
- Alan sayfaları ödünç aldıkları `KB` kodlarının süreç bileşenlerini KENDİ
  disiplinlerine göre yeniden ifade ediyor (`Toplanan bilgiler üzerinde` /
  `üzerinden çıkarım yapmak`). Bu geçişler düşürülmez: bileşen metni
  farklıysa düğümün `variants` alanına yazılır, aynıysa yalnız sayılır
  (`report.repeated_nodes`).
- Kod satırı adı kısaltabiliyor (`FBAB1.Bilimsel Gözlem`); tam ad sayfa
  başlığındadır (`Bilimsel Gözlem Becerisi (FBAB1)`). Başlık adı mevcut adı
  UZATIYORSA kanonik ad odur; kalan başlıklar `also_titled` altındadır.
- Kodsuz gruplama kademesi (`Anlama Becerileri`, `Alımlayıcı Beceriler`)
  düğümün `group` alanına yazılır; kaynakta kodda izi yoktur.
- Kaynağın kendi tutarsızlıkları `anomalies` altındadır: bir süreç bileşeni
  içinde bulunduğu düğümden başka bir kod taşıyor (`DAB4.4` altında
  `DAB4.3.SB4`). Kod OLDUĞU GİBİ saklanır, sıra esas alınır, anomali
  bildirilir.
