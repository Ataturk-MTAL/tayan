# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Ortaokul/lise öğretmeni — kendi dersinin soru bankasını kuran, sınavını hazırlayan,
bastıran ve sonucunu değerlendiren kişi.

v1 tek kullanıcı içindir: hesap, giriş, rol ve sahiplik yoktur. Uzun vadeli hedef
zümre/bölüm ortak bankasıdır, ancak paylaşım v1 kapsamı dışında bırakılmıştır
(bkz. Capabilities and Constraints — açık kararlar).

## Product Purpose

Öğretmenin soru bankası kurmasını, o bankadan sınav derlemesini, baskıya hazır A4
çıktı almasını, sonuçları girmesini ve madde analiziyle bankayı iyileştirmesini
sağlar. Başarı: öğretmenin sınav hazırlama süresi düşerken soru bankasının ölçülen
kalitesi turdan tura yükselir.

## Positioning

Üç şey aynı üründe ve eşit ağırlıkta:

1. **Dizgi kalitesi** — Typst gömülü derleyici ile matbaa kalitesinde çıktı.
   Kelime işlemcide bozulan matematik, hizalanmayan şıklar, kayan görseller sorunu
   ortadan kalkar.
2. **Madde analizi döngüsü** — sınav sonuçları soruya geri beslenir; güçlük ve
   ayırt edicilik indeksleri bankayı zamanla arıtır.
3. **Hız** — hazır bankadan baskıya hazır sınava kısa yoldan gidilir.

Hiçbiri diğerinin altına gömülmez. Ayırt edici olan, bu üçünün tek çevrimdışı
masaüstü uygulamasında birleşmesidir.

## Operating Context

- Öğretmenin kendi bilgisayarı; masaüstü uygulaması, internet gerektirmez.
- Veri yerel SQLite dosyasında (`tayan_dev.db`).
- Son çıktı kâğıttır: A4 baskı. Öğrenci nüshası ve cevap anahtarı ayrı çıktılardır.
- Kazanım kodları MEB formatındadır (örn. `MAT.9.1.2`).
- Arayüz dili Türkçedir; çoklu dil hedefi yoktur.

## Capabilities and Constraints

**Soru tipleri** (`crates/tayan-core/src/domain/exam_management/entities/`):
`multiple_choice`, `true_false`, `fill_in_blank`, `classic` (rubrikli).

**Madde analizi** (`value_objects/question_stats.rs`):
`difficulty_index`, `discrimination_index`; durum yordamları `is_too_easy`,
`is_too_hard`, `has_poor_discrimination`, `is_untested`.
`ScoreBadge` eşikleri — `Excellent` 80..=100, `Good` 50..=79, `Fair` 20..=49,
`Poor` altı.

**Teknik kısıtlar:**
- Typst gömülü: `typst 0.14.2`, `typst-pdf 0.14.2`, `typst-library 0.14.2`,
  `typst-assets 0.14.2` (features = ["fonts"]). `World` implementasyonu
  `crates/tayan-compiler/src/world.rs`.
- LaTeX matematiği Rust içinde Typst'e çevrilir
  (`crates/tayan-core/src/domain/shared/latex_to_typst.rs`, 97 test). MiTeX
  bağımlılığı kaldırılmıştır.
- Mevcut `compile_typst_preview` komutu base64 **PDF** döndürür. Tuş vuruşu
  hızında canlı önizleme için SVG çıktısı gerekir; bu ayrı bir komut olarak
  eklenecektir. PDF yolu dışa aktarma için korunur.
- Domain katmanı hexagonal: `domain/` + `application/{commands,ports,queries}`.
  Arayüz bu katmana 30 Tauri komutu üzerinden bağlanır.

**Açık kararlar (uydurulmayacak):**
- Zümre ortak bankası paylaşımı v1'de yoktur. Veri modeli sonradan kaynak/sahiplik
  alanı eklenebilecek şekilde bırakılır; paylaşım mekanizması (dosya aktarımı,
  ağ klasörü) henüz seçilmemiştir.
- Uygulama ikonu tasarlanmamıştır (`PLAN.md` T02 atlanmıştır).
- Fiyatlandırma, lisans, dağıtım kanalı ve kullanıcı sayısı belirlenmemiştir.

## Brand Commitments

Ürün adı **TAYAN**. Logo, marka rengi, tipografi seçimi ve ses tonu için
bağlayıcı bir kayıt yoktur.

## Evidence on Hand

- Çalışan domain kodu: `crates/` altında 6.097 satır Rust, 97 geçen test.
- Görev geçmişi: `PLAN.md` (gerçeğin gerisinde — T06 ve T17 tamamlanmış olmasına
  rağmen TODO görünüyor).
- Yerel geliştirme veritabanı: `tayan_dev.db`.
- Gerçek kullanıcı yoktur; referans, vaka çalışması, ölçüm, basın veya müşteri
  verisi yoktur. Bunlar uydurulmayacaktır.

## Product Principles

1. **Typst gizlenmez, öğretilir.** Öğretmen ham Typst yazmaya zorlanmaz; hazır
   bloklar, kısayollar ve otomatik tamamlama işi taşır. Ama kaynak görünürdür ve
   arayüz, öğretmenin Typst'in ne olduğunu zamanla anlamasına izin verir.
2. **Kâğıt son gerçektir.** Ekranda görünen her şey, kâğıda ne basılacağını
   dürüstçe temsil eder.
3. **Soru bir kere yazılmaz, ölçülerek iyileşir.** İstatistik soruya yapışıktır;
   ayrı bir rapor ekranına sürülmez.
4. **Çevrimdışı ve yerel.** Hiçbir akış ağ beklemez, hiçbir ekran hesap istemez.
5. **Üç çekirdek eşittir.** Dizgi, analiz ve hız — biri diğerinin alt sekmesi
   hâline gelmez.
