# Üçüncü taraf bileşenler

TAYAN Apache-2.0 ile dağıtılıyor. Aşağıdaki bileşenler uygulamayla **birlikte
gönderiliyor** ve kendi lisansları altında kalıyor.

## Gömülü Typst paketleri

`crates/tayan-compiler/packages/preview/` altında kaynak olarak duruyorlar ve
`include_dir` ile uygulama ikilisine gömülüyorlar.

| Paket | Sürüm | Lisans | Ne için |
|---|---|---|---|
| [cetz](https://github.com/cetz-package/cetz) | 0.4.2 | LGPL-3.0 | Çizim ve grafik |
| [zap](https://github.com/janmalte/zap) | 0.5.0 | LGPL-3.0 | Devre şemaları |
| [oxifmt](https://github.com/PgBiel/typst-oxifmt) | 1.0.0 | Apache-2.0 / MIT | Dize biçimlendirme |

### Neden gömülüyorlar

TAYAN tamamen çevrimdışı çalışmak zorunda. Typst paketleri normalde ilk
kullanımda internetten indiriliyor; temiz kurulmuş bir okul bilgisayarında,
internet yokken, bu paketleri kullanan bir sınav kâğıdı **hiç basılamazdı**.
Gömülü paketler bu riski ortadan kaldırıyor.

Bir test bunu kanıtlıyor: `world.rs::gomulu_paket_testleri::cetz_onbelleksiz_derlenir`
paket önbelleğini var olmayan bir yola kurup cetz kullanan bir belgeyi derliyor.

### LGPL uyumu

cetz ve zap LGPL-3.0. Typst paketleri **kaynak** olarak dağıtılıyor (`.typ`
dosyaları), yani:

- **Kaynak erişilebilir.** Paketlerin tamamı okunabilir metin olarak depoda
  duruyor; ayrıca bir kaynak dağıtımı gerekmiyor.
- **Kullanıcı değiştirebilir.** `crates/tayan-compiler/packages/` altındaki
  dosyalar değiştirilip uygulama yeniden derlenebilir. Gömülü olmayan bir
  sürüm istenirse paket önbelleği yolu hâlâ çalışıyor.
- **Lisans metinleri korunuyor.** Her paketin kendi `LICENSE` dosyası olduğu
  gibi duruyor; bir test bunu da doğruluyor
  (`gomulu_paket_testleri::lisans_dosyalari_gomulu_kaldi`).

Paketler değiştirilmedi; olduğu gibi kopyalandı.

## Sürüm yükseltme

Gömülü bir paketi yükseltmek için:

1. Yeni sürümü `crates/tayan-compiler/packages/preview/<ad>/<sürüm>/` altına kopyala
2. `world.rs`'teki testte sürüm numarasını güncelle
3. Eski sürümü, hiçbir sınav onu içe aktarmıyorsa kaldır

Sürüm **kesin eşleşiyor**: gömülüde olmayan bir sürüm istendiğinde "yakın" bir
sürüm verilmiyor. Yakın sürüm vermek, öğretmenin kâğıdını başka bir kütüphaneyle
dizmek olurdu ve fark sessizce çıktıya yansırdı.
