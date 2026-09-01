# TAYAN

Öğretmenler için Typst tabanlı sınav ve soru bankası uygulaması. Tamamen
çevrimdışı çalışan bir masaüstü programı: hesap yok, bulut yok, veri kendi
bilgisayarında.

Üç şey aynı üründe ve eşit ağırlıkta:

1. **Dizgi kalitesi** — Typst derleyicisi uygulamanın içine gömülüdür. Kelime
   işlemcide bozulan matematik, hizalanmayan şıklar, kayan görseller sorunu
   ortadan kalkar.
2. **Madde analizi** — sınav sonuçları soruya geri beslenir; güçlük ve ayırt
   edicilik indeksleriyle banka zamanla arınır.
3. **Hız** — hazır bankadan baskıya hazır sınava kısa yoldan gidilir.

## Durum

Geliştirme aşamasında. Çalışan: soru bankası, dört soru tipi, canlı önizlemeli
soru editörü, sınav kurma, PDF çıktısı, analiz ekranı.

Eksik: sınav sonucu giriş ekranı (`enter_exam_results` komutu hazır, arayüzü
yok). Bu olmadan madde analizi veri bulamaz.

## Soru tipleri

`multiple_choice` · `true_false` · `fill_in_blank` · `classic` (rubrikli)

Sorunun yapısı ayrı bir form panelinde değil, Typst kaynağının kendisinde durur:

```typst
Aşağıdaki denklemin köklerini bulunuz.

$ x^2 - 5x + 6 = 0 $

#secenekler(dogru: "C",
  [$x = 1$],
  [$x = 2$],
  [$x = 2$ ve $x = 3$],
  [$x = 6$],
  [Hiçbiri],
)
```

`dogru` parametresi öğrenci nüshasında basılmaz; uygulamanın cevap anahtarını ve
madde analizini kurabilmesi için kaynakta durur. Öğretmen tek yere bakar ve cevap
anahtarının kâğıtta görünenden ayrı düşmesi imkânsız olur.

## Mimari

```
crates/
  tayan-core       alan katmanı (DDD): domain + application/{commands,ports,queries}
  tayan-compiler   gömülü Typst derleyicisi, World implementasyonu, SVG/PDF
  tayan-db         SQLite depoları ve göçler
  tayan-macros     yardımcı makrolar
apps/tayan-desktop
  src-tauri        Tauri komutları (alan katmanına açılan tek yüzey)
  src              SvelteKit arayüzü
```

Alan katmanı arayüzü tanımaz; bağlantı `application/ports` üzerinden kurulur.

**Yığın:** Rust · Tauri 2 · Typst 0.14.2 · SvelteKit 2 · Svelte 5 · Tailwind 4 ·
CodeMirror 6 · SQLite

## Geliştirme

Gerekenler: Rust (stable), Node 20+, pnpm.

```bash
cd apps/tayan-desktop
pnpm install
pnpm tauri dev
```

### Dil sunucusu (isteğe bağlı)

Editör kutudan çıktığı hâliyle Typst'in 560 sembolünü tanır ve internet
gerektirmez. İstenirse **tinymist** dil sunucusu Yardım ekranından kurulabilir;
içe aktarılan paketlerin sembollerini ve hover açıklamalarını ekler.

Uygulamayla paketlenmez: platform başına 60 MB ve dondurulmuş bir sürüm demek
olurdu. Kurulum açık bir kullanıcı eylemidir ve indirilen dosya sha256 ile
doğrulanır.

İlk derleme Typst'i de derlediği için uzun sürer ve belleği zorlar; sonraki
açılışlar artımlıdır.

```bash
cargo test                                    # 97 test
cd apps/tayan-desktop && pnpm run check       # svelte-check
```

### Ölçüm araçları

```bash
/usr/bin/time -l cargo run -p tayan-compiler --example mem_probe -- 30
cargo run -p tayan-compiler --example tpl_test
```

`mem_probe` art arda derlemede bellek büyümesini ölçer, `tpl_test` soru
kalıplarının derlendiğini doğrular.

## Veri

Yerel SQLite. Geliştirmede `tayan_dev.db`, dağıtımda `tayan.db`:

```
~/Library/Application Support/tayan/     (macOS)
```

Font künye indeksi de aynı dizinde önbelleklenir (`font-index.postcard`); dizin
içeriği değişince kendiliğinden yenilenir.

## Lisans

Apache License 2.0 — bkz. [LICENSE](LICENSE).

Üçüncü taraf bileşenler ve dağıtılan ikili dosyaya gömülen yazı tiplerinin
atıfları için [NOTICE](NOTICE).
