# TAYAN — Bilgi Mimarisi ve Yerleşim Şartnamesi

> Kaynak: Fable danışman ajanı, 2026-09-01. Kod tabanı okunarak üretildi.
> Durum: **öneri** — uygulanmadan önce onaylanmalı.

## 0. Teşhis

Doğrulanan kusurlar:

- **(a)** Dört tam genişlik şerit: `+layout.svelte` (nav) + `QuestionForm.svelte`
  (form başlığı) + `QuestionEditor.svelte` (künye) + `BlockPalette.svelte`.
- **(b)** `kazanım` üç yerde, `puan` iki, `soru tipi` iki yerde basılıyor.
- **(c)** `grid-cols-[minmax(320px,1fr)_minmax(360px,1.15fr)_240px]` — 794 px'lik
  A4 kalıcı olarak küçültülmüş; 240 px `MeasureRail` denenmemiş soruda boş.
- **(d)** Aynı sıkışık üç sütun `exams/[id]/+page.svelte`'de tekrarlanıyor.
- **(e)** Beş düz gezinme sekmesi, çok farklı kullanım sıklıklarında.
- **(f)** Liste↔editör tam sayfa turu; `goto("/questions")` süzgeç ve kaydırmayı siler.
- **(g)** Ana sayfa gezinmeyi kopyalıyor.

Ek teşhisler:

- **(h)** `routes/questions/+page.svelte` bankayı **ham Typst kaynağı** olarak
  listeliyor (`preview()` → `bodySource`, monospace). Öğretmen bankasında
  `$x^2 - 5x + 6 = 0$ #secenekler(dogru:...` görüyor. "Ürün kâğıttır" ilkesinin
  en sert ihlali editörde değil, listede.
- **(i)** `enter_exam_results` komutunun arayüzü yok. Ölçüm döngüsü kapanmadığı
  için `MeasureRail` **pratikte her zaman** boş — (c)'nin kök nedeni yerleşim
  değil, veri akışının kopukluğu.

## 1. IA ilkesi

**Kâğıt masadadır; geri kalan her şey kâğıdın kenarıdır.**

Tek gerçek nesne basılacak A4'tür. "Modern editör" yapısal olarak şu demektir:
tek kalıcı çalışma alanı, merkezde hep aynı nesne, seçim değiştikçe sayfa
*değişmez* içerik değişir, üstveri seçime bağlı kenar notudur, gezinme durumu
asla sıfırlanmaz. VS Code'u modern yapan sol rayı değil, **pencerenin hiç
yıkılmamasıdır**. TAYAN'daki karşılığı IDE değil öğretmenin masasıdır: ortada
kâğıt, solda müsvedde defteri, kenarda kırmızı kalemle şerhler. Typst kaynağı
kâğıdın eş ağırlıklı ikizi değil, **müsveddesidir** — hep görünür, ama ikincil.

Sildikleri: sayfa-geçişli liste↔detay turu; kâğıdı sıkıştıran eş ağırlıklı üç
sütun; boş dururken yer kaplayan sabit ölçüm rayı; aynı bilgiyi üç kez basan
şeritler; nav'ı kopyalayan ana sayfa; `soru tipi` `<select>`'i.

## 2. Nesne modeli

```
Soru (bankada yaşar; ölçüm tarihçesi taşır)
 ├─ gövde  = Typst kaynağı (yapının TEK doğru kaynağı: #secenekler, #bosluk…)
 ├─ künye  = kazanım[], (yedek puan), tip ← kaynaktan TÜRETİLİR
 └─ ölçüm  = QuestionStats (soruya yapışıktır, sınava değil)

Sınav (bir BASKI olayıdır)
 ├─ meta   = başlık, ders, sınıf, süre, tarih, talimat
 ├─ atıflar= ExamQuestionRef { question_id, display_order, points_override }
 └─ nüshalar (nesne DEĞİL, görünümdür): Kitapçık A/B/C/D × {öğrenci, anahtar}

Sınıf → Öğrenci → Sonuç (ExamResult: sınav × öğrenci)
Kâğıt/Sayfa: nesne değil; Soru'nun ve Sınav'ın derlenmiş GÖRÜNÜMÜ.
```

**Soru ve sınav aynı tür nesne DEĞİLDİR ve birleştirilmemelidir.** Gerekçe kodun
kendisinde: `QuestionForm.buildUpdated` kimliği ve `stats`'ı korumak için özenle
yazılmış — madde analizi tarihçesi soruya bağlıdır ve sorunun sınavlar arasında
yeniden kullanılması ürünün analitik çekirdeğidir. Sınav ise tarihli, tek
seferlik bir baskı olayıdır; puan bile soruya değil kenara (`points_override`)
aittir. Soru **fiş**tir (tekrar tekrar masaya gelir), sınav **zarf**tır (fişleri
bir kez diz, bas, kapat).

Üç yüzey, beş değil:

| Yüzey | Nesne | Sıklık |
|---|---|---|
| **Sorular** | fiş dizimi ⇄ masa (tek soru) | en yüksek |
| **Sınavlar** | sınav listesi → sınav masası (kitapçık) | orta |
| **Sonuçlar** | sınav × sınıf: sonuç girişi + analiz (+ sınıf yönetimi) | dönem sonu |

`Öğrenciler` bağımsız yüzey olmaktan çıkar → Sonuçlar'ın içine.
`Yardım` gezinme sekmesi olmaktan çıkar → her yerden çağrılan yan panel.
Ana sayfa silinir.

## 3. Yerleşim

### 3.1 Kabuk

```
┌──────────────────────────────────────────────────────────────────────────┐
│ TAYAN │ Sorular● │ Sınavlar │ Sonuçlar │                            [ ? ]│ 40px
└──────────────────────────────────────────────────────────────────────────┘
```

`[?]` sağdan Yardım panelini açar. Uygulama `/`'a değil `/questions`'a açılır.

### 3.2 Sorular — fiş dizimi (varsayılan)

Banka tablo değil, **dizilmiş fişler**: her soru derlenmiş SVG'siyle küçük bir
kâğıt parçası. Teşhis (h) çözülür. Liste sol ray değil, masanın **uzaktan
görünüşü**.

```
1440 px
┌──────────────────────────────────────────────────────────────────────────┐
│ TAYAN │ Sorular● │ Sınavlar │ Sonuçlar │                             [?] │
├──────────────────────────────────────────────────────────────────────────┤
│ Soru bankası · 34   [Tümü][Denenmemiş][Zayıf ayırt edici] [Ara____]      │
│                                        [Seçilenleri sınava ekle][Soru yaz]│ 40px
├──────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ dizgi (SVG)  │  │ dizgi (SVG)  │  │ dizgi (SVG)  │  │ dizgi (SVG)  │  │
│  │  x²−5x+6=0   │  │  ∫₀¹x²dx…    │  │ Boşluk: __   │  │ D/Y  ☐ ☐     │  │
│  ├──────────────┤  ├──────────────┤  ├──────────────┤  ├──────────────┤  │
│  │ MAT.9.1.2    │  │ MAT.9.2.1    │  │ (kazanım yok)│  │ MAT.9.1.4    │  │
│  │ ▬▬▬ zayıf .12│  │ ▬▬▬ iyi  .41 │  │ denenmemiş   │  │ ▬▬▬ orta .28 │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘  │
│  … (dikey kaydırma; 1280'de 3, 1440'ta 4, 1920'de 5 sütun; fiş ~300px)   │
└──────────────────────────────────────────────────────────────────────────┘
```

- Fişin alt çizgisi ölçümün rengi (`QuestionStrip`'teki 3px desen, `scoreBadge`
  renkleri). Kırmızı yalnız değerlendirme: zayıf fişin çizgisi kırmızıdır.
- Fişe tıklamak **masayı açar**; dizim bileşeni yıkılmaz (kaydırma + süzgeç
  korunur). Dizim `routes/questions/+layout.svelte`'e taşınır, masa çocuğu olur
  (veya tek route + `$state`).
- Çoklu seçim (fişte onay köşesi) → "Seçilenleri sınava ekle" → taslak sınav
  seç/yeni oluştur (`api.exams.addQuestion` döngüsü).

### 3.3 Sorular — masa (tek soru editörü)

Dört şerit **bire** iner. Kaynak soldaki "müsvedde sayfası": sabit genişlik,
kalıp düğmeleri **kendi başlığında** — uygulama genişliğinde şerit değil.

```
1920 px
┌────────────────────────────────────────────────────────────────────────────────┐
│ TAYAN │ Sorular● │ Sınavlar │ Sonuçlar │                                   [?] │ 40
├────────────────────────────────────────────────────────────────────────────────┤
│ ◂ Fişler   ‹ 12/34 ›   ÇOKTAN SEÇMELİ · Kazanım: MAT.9.1.2 ✎ · 5 puan ✎        │
│                                                    ●değişti     [Kaydet ⌘S]    │ 40
├───────────────────────────┬────────────────────────────────────┬───────────────┤
│ MÜSVEDDE          560px   │            KÂĞIT  794px (%100)     │ KENAR  ~300px │
│┌─────────────────────────┐│      ┌──────────────────────────┐  │               │
││Kalıp: Şıklar·Mat·Kesir·⋯││      │ (A4 SVG, .sheet gölgesi) │  │  Güçlük  %78  │
│├─────────────────────────┤│      │  1. Aşağıdaki denklemin  │  │  Ayırt   0.12 │
││ CodeMirror              ││      │     köklerini bulunuz.   │  │  ZAYIF        │
││ Aşağıdaki denklemin     ││      │     x² − 5x + 6 = 0      │  │  "İyi ve zayıf│
││ köklerini bulunuz.      ││      │  A) x=1   B) x=2         │  │  öğrenciyi    │
││                         ││      │  C) x=2 ve x=3 …         │  │  ayırmıyor."  │
││ $ x^2 - 5x + 6 = 0 $    ││      │                          │  │  3 kez, son:  │
││                         ││      │                          │  │  12.03.2026   │
││ #secenekler(dogru: "C", ││      └──────────────────────────┘  │               │
││   [$x = 1$], …          ││       %100 · [−][+][Sığdır]        │ (denenmemişse │
│└─────────────────────────┘│                                    │  bu sütun YOK)│
└───────────────────────────┴────────────────────────────────────┴───────────────┘
```

```
1280 px  (kenar sütunu yok; şerh künyeye rozet olarak biner)
┌──────────────────────────────────────────────────────────────────┐
│ ◂ Fişler ‹12/34›  ÇOKTAN SEÇMELİ · MAT.9.1.2 ✎  [ZAYIF ▾] [Kaydet]│ 40
├─────────────────────────┬────────────────────────────────────────┤
│ MÜSVEDDE      ~470px    │      KÂĞIT  ~730px → ölçek ~%92        │
│ Kalıp: Şıklar · Mat · ⋯ │      (Sığdır varsayılan; PreviewZoom   │
│ CodeMirror              │       davranışı aynen korunur)         │
└─────────────────────────┴────────────────────────────────────────┘
```

Kurallar:

1. **Künye tek satır, her gerçek bir kez.** Tip = kaynaktan türetilen ibre
   (stamp, tıklanamaz). Kazanım = satır içi düzenlenen tek alan (`✎`). Puan =
   küçük, düzenlenebilir, "sınavda değişebilir" ipucuyla. `derleniyor…` metni
   silinir; `SheetPreview`'deki 2px kırmızı çizgi tek doğru göstergedir.
2. **Kenar şerhi = eski `MeasureRail`, koşullu ve kırmızı.** `times_used === 0`
   iken sütun **hiç yoktur**. Ölçüm varsa kâğıdın sağ kenarına `margin-rule` ile
   ayrılmış `annot` şerhler düşer. 1280–1560 arası: künyedeki renkli rozete
   katlanır, tıklayınca popover.
3. **Kalıp paleti** müsvedde sütununun başlık satırı olur; taşanlar `⋯` menüsünde.
   Bileşen yaşar, yerleşimi değişir.
4. **Tip türetme kuralı — evet, ama korkuluklu.** Öncelik sırası: `#secenekler(`
   → çoktan seçmeli, `#dogru-yanlis(` → doğru/yanlış, `#bosluk(` → boşluk
   doldurma, hiçbiri → klasik. Birden fazlası varsa kaydetme kilitlenir:
   "Gövdede hem `#secenekler` hem `#bosluk` var — birini sil." Boş yeni belge:
   masa dört tipin **derlenmiş mini fişleriyle** açılır ("Nasıl bir soru?");
   seçim ilgili kalıbı kaynağa yazar. Pinlenmiş 4. maddenin en güçlü taşıyıcısı.
   `<select>` ölür.
5. **Kaydet gezinmez.** `goto("/questions")` silinir; künyede `●değişti` söner.
   `‹ ›` süzgeçteki komşu fişe geçer (kaydedilmemiş değişiklikte sorar).

### 3.4 Sınavlar — sınav masası

```
1440 px
┌──────────────────────────────────────────────────────────────────────────────┐
│ 1. Dönem 2. Yazılı · Matematik · 9-A · 12.06 · TASLAK                        │
│   Kitapçık:[Tek▾]  ☐ Cevap anahtarı   [Typst dışa aktar][PDF kaydet][Yayınla]│ 40
├──────────────────────────────────────────────────────────────────────────────┤
│ Sınav │ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 7 │ 8 │ + Soru ekle │ 8 soru · 85/100 ▓▓▓░  │ 44
├──────────────────────────────┬───────────────────────────────────────────────┤
│  (yalnız bir soru SEÇİLİYKEN)│            KİTAPÇIK (A4 sayfalar)             │
│  KENAR — 3. soru             │   ┌──────────────────────────┐                │
│  Çoktan Seçmeli              │   │  sayfa 1                 │                │
│  Puan: [10] ✎                │   │                          │                │
│  MAT.9.1.2                   │   └──────────────────────────┘                │
│  Ayırt: 0.12 ZAYIF           │   ┌──────────────────────────┐                │
│  [Soruyu aç] [Sınavdan çıkar]│   │  sayfa 2 …               │                │
│  (seçim yoksa bu sütun YOK,  │   └──────────────────────────┘                │
│   kitapçık ortalanır)        │                                               │
└──────────────────────────────┴───────────────────────────────────────────────┘

"+ Soru ekle" → sağdan ÇEKMECE (overlay, ~420px): fiş dizimi süzgeçli,
fişe tıkla → eklenir, kitapçık yeniden derlenir. Esc/dışarı tık kapatır.
```

- Puan **burada** düzenlenir (`points_override`) — model zaten böyle.
- Kitapçık uyarıları ve `missing` (bankada yok) uyarıları şeridin altında tek
  satır `annot`; mevcut metinler aynen taşınır.
- Şerit tıklaması v1'de **seçer**. Kitapçık içinde soruya kaydırma → §6.4 riski.
- `/exams` listesi tablo kalır (sınav azdır, tablo doğrudur); `exams/new` meta
  formu kalır, `Oluştur` sonrası doğrudan masaya iner.

### 3.5 Sonuçlar ve Yardım

**Sonuçlar** (`/results`): üstte iki seçici (Sınav, Sınıf) + özet sayılar; altta
iki sekmeli tek yüzey — **Sonuç girişi** (öğrenci × soru ızgarası;
`enter_exam_results`'ın eksik arayüzü, `AnswerGrid`'in yazılabilir hali) ve
**Analiz** (`ScoreHistogram` + `AnswerGrid` + kazanım başarısı). Sınıf/öğrenci
yönetimi "Sınıfları düzenle" ile açılan ikincil yüzey (mevcut
`students/+page.svelte` içeriği korunur).

**Yardım paneli**: `[?]` her yüzeyden sağdan ~420px panel açar; içerik
`yardim/+page.svelte`'nin bölümleri, bağlama göre açılır (masadayken "Soru
kalıpları" açık gelir) ve her kod örneğinin yanında `[Ekle]` düğmesi imlece
yerleştirir (mevcut `TypstSource.insert` API'si). `/yardim` route'u genişletilmiş
görünüm olarak yaşar; nav sekmesi ölür.

## 4. Etkileşim modeli

**Gece, yarınki yazılı, 20 soru:** Fiş dizimi açılır → `Soru yaz` → dört mini
fişten tip seç → müsveddeye yaz, kalıp düğmeleriyle yapı ekle (120 ms sonra kâğıt
oturur) → künyede kazanım → `Kaydet` (yerinde) → `‹ ›` veya `◂ Fişler` →
sıradaki. 20 sorunun hiçbirinde süzgeç/kaydırma kaybolmaz. Sonra: fişleri seç →
"Seçilenleri sınava ekle" → sınav masası → puanları kenardan ayarla →
`PDF kaydet` (öğrenci nüshası) → `☑ Cevap anahtarı` → `PDF kaydet` → yazıcı.

Klavye — hepsi **hızlandırıcı**, hiçbiri tek yol değil:

| Kısayol | İş | Düğme karşılığı |
|---|---|---|
| `⌘S` | kaydet | künyedeki `Kaydet` |
| `⌘N` | yeni soru | dizimdeki `Soru yaz` |
| `⌘←` / `⌘→` | önceki/sonraki fiş | künyedeki `‹ ›` |
| `Esc` | masadan dizime / çekmeceyi kapat | `◂ Fişler` |
| `⌘+/−/0`, `⌘+tekerlek` | zoom (mevcut) | `PreviewZoom` |
| `F1` / `?` | yardım paneli | `[?]` |

Klavyeye hiç dokunmayan öğretmen için akış tamamen tıklamayla yürür.

## 5. Silme listesi

| Silinen | Dosya | İşlevi nereye gitti |
|---|---|---|
| Ana sayfa ("Bugün ne yapacaksın?") | `routes/+page.svelte` | Silindi; açılış `/questions`. Sayaçlar süzgeç çiplerine rozet ("Zayıf ayırt edici · 3"); "hiç uygulanmadı" notu Sonuçlar'a |
| Nav'dan `Öğrenciler`, `Analiz`, `Yardım` | `routes/+layout.svelte` | `Sonuçlar` tek sekme; Yardım → `[?]` paneli |
| Soru tipi `<select>` | `QuestionForm.svelte` | Kaynaktan türetilen künye ibresi + boş belgede tip seçici + kalıp düğmeleri |
| Form başlığı şeridi (kazanım input + Kaydet) | `QuestionForm.svelte` | Künye satırına (tek şerit) |
| Künye şeridi (tip/puan/kazanım/`derleniyor…`) | `QuestionEditor.svelte` ~119-131 | Aynı künye satırına; `derleniyor…` tamamen silinir |
| `BlockPalette` tam genişlik şeridi | `BlockPalette.svelte` yerleşimi | Müsvedde sütununun başlık satırı (bileşen yaşar) |
| Sabit 240px `MeasureRail` + "Bu soru hiç uygulanmadı" | `MeasureRail.svelte`, `QuestionEditor.svelte` grid | Koşullu kenar şerhi (`times_used > 0`); Puan/Kazanım blokları silinir |
| `grid-cols-[minmax(320px,1fr)_minmax(360px,1.15fr)_240px]` | `QuestionEditor.svelte` | `560px_1fr(_~300px koşullu)` — kâğıt esneyen sütun |
| Banka tablosu (ham kaynak önizlemeli) | `routes/questions/+page.svelte` | Fiş dizimi (derlenmiş SVG kartlar); süzgeçler çip |
| `goto("/questions")` + tam sayfa liste↔detay gezinmesi | `QuestionForm.svelte` `save()`, `questions/+page.svelte` | Yerinde kaydet; dizim kalıcı layout'ta monte kalır |
| Sınav masasının üç sütunu | `routes/exams/[id]/+page.svelte` grid | Sorular → `QuestionStrip` + seçim kenarı; banka → çekmece; kitapçık → merkez |
| `BudgetGauge` ayrı şeridi | `exams/[id]/+page.svelte` | `QuestionStrip` satırının sağ ucuna birleşir |
| `routes/students` nav girişi | — | Sonuçlar içinde "Sınıfları düzenle" |

## 6. Maliyet ve risk

1. **Ucuz, risksiz** — nav sadeleştirme, ana sayfa silme, künye satırı, palet
   taşıma, `MeasureRail`'in koşullu şerhe dönüşmesi, `goto` silme, sınav başlığı
   birleştirme, yardım paneli (içerik hazır).
2. **Orta** — (a) Kalıcı çalışma alanı: dizimi `questions/+layout.svelte`'e
   taşımak; URL↔durum senkronu özen ister. (b) **Tip türetme**: parser'lar hazır;
   öncelik + çakışma hatası + boş-belge tip seçici eklenir. (c) **Sınav
   çekmecesi**: mevcut "Bankadan ekle" listesinin overlay'e taşınması.
   (d) Sonuç girişi ızgarası: `enter_exam_results` API'si hazır, yüzey yeni.
3. **Pahalı / dikkat — fiş dizimi.** Soru başına `compile_question_preview_svg`
   (4-30 ms). Ölçek sorunu derleme değil **bellek**: SVG ~50-150 KB × yüzlerce
   soru. Zorunlu önlemler:
   - `IntersectionObserver` ile tembel derleme
   - eşzamanlılık sınırı (mevcut tek-kuyruk deseni genellenir)
   - bellek içi önbellek, anahtar = gövde kaynağının hash'i
     (`Question`'da `updated_at` **yok** — hash şart)
   - görünümden çıkan fişlerin SVG'sinin bırakılması
4. **Tuzak — bilinçli erteleme.** Sınav şeridinden kitapçık içinde soruya
   kaydırma, soru→sayfa/konum eşlemesi ister; `WorldExt::range` teknik olarak var
   ama yeni Tauri komutu demektir. v1: şerit tıklaması yalnız seçer. Aynı sebeple
   kâğıda tıkla→kaynağa git eşlemesi kapsam dışı.
5. **Kaydedilmemiş değişiklik.** Yerinde gezinme geldiği an "değişti/kaydet"
   disiplini gerekir (`●değişti` + geçişte onay). **Otomatik kaydet YAPILMAZ:**
   `parseOptions` hatalıyken kaydetmek yanlış cevap anahtarı üretebilir; mevcut
   "açık hata > sessiz yanlış" kararı korunur.

**Kırmızı Kalem kararı: aynen korunur.** Hiçbir token, renk veya kural değişmez.
Bu IA dünyayı *güçlendirir* — `margin-rule` yorumu ("bu çizginin sağı
değerlendirmedir") kenar şerhinde ilk kez harfiyen gerçekleşir, kırmızı yalnız
ölçümde kalır, fişler `.sheet` gölgesiyle "masadan kalkan tek nesne" kuralına
uyar. Eklenen tek şey iki bileşen deseni: **fiş kartı** ve **kenar şerhi bloğu**
— ikisi de mevcut `ruled` / `annot` / `stamp` sınıflarıyla kurulur.

## 7. Yapılmayacaklar

- **WYSIWYG / form-tabanlı soyutlama yok.** Şıkları ayrı inputlara bölmek
  pinlenmiş kısıtın ihlali: Typst görünmez olur, cevap anahtarı ile kâğıt
  ayrışabilir. Kaynak tek doğru kaynak kalır; UI onu okur, saklamaz.
- **Komut paleti yok.** Ayda birkaç kez açılan uygulamada `⌘K` keşfedilmez;
  keşfedilmeyen palet, düğmesi silinmiş işlev demektir. Her işlev görünür bir
  düğmedir; kısayollar yalnız hızlandırıcı.
- **Kaynak↔dizgi tıklama eşlemesi yok (v1).** En "modern editör" görünümlü hamle
  ve en büyük tuzak: backend değişikliği + `preview_line_offset` türü satır
  kaydırma sorunlarının iki yönlüsü. İki-sayfa yakınlığı + 120 ms canlı derleme
  aynı ihtiyacın %90'ını eşleme olmadan karşılar.
- **Sınavı tek yazım yüzeyi yapmak yok.** Soru kimliğinin ve ölçüm tarihçesinin
  bankada yaşadığı modelde banka birincil kalmalı; aksi, yeniden kullanımı ve
  madde analizini ikinci sınıfa iter.
- **Otomatik kaydet yok** (§6.5).

## 8. Açık çatal

Fiş diziminde SVG önizleme yerine **metin-dizgili kart** (matematiği ham metinle
yaklaşıklayan ucuz kart) ara kademe olabilir. **Tavsiye: girme.** SVG fişler bu
IA'nın kimliğidir ve maliyeti §6.3 önlemleriyle yönetilebilir; yaklaşık dizgi
"kâğıt üründür" tezini daha ilk ekranda yalanlar.


---

# 9. DÜZELTME — yüzen gruplu kalıp paneli

> Bu bölüm §3.3/3'ü ve §5'teki "BlockPalette sütun başlığına taşınır"
> satırını **geçersiz kılar**.
>
> Fable'ın "yüzer ada = her panel gölgeyle yüzen kâğıt tabakası" yorumu
> yanlış anlamaydı ve **iptal edildi**. `app.css`'teki "masadan kalkan tek
> nesne basılacak sayfadır" kuralı **değişmiyor** — tek istisna aşağıdaki
> kalıp panelidir.

## 9.1 Sorun

`BlockPalette.svelte` bugün tam genişlik bir şeritte **17 düğme** sarmalıyor
(`templates.ts` → `BY_TYPE` 1 + `COMMON` 15 + Görsel dosya seçici). Hiçbir
gruplama yok: `Kesir` ile `Şekil + başlık` yan yana, aynı ağırlıkta.

## 9.2 Gruplar

`COMMON` düz listeden gruplu yapıya geçer:

| Grup | Bloklar |
|---|---|
| **Kalıp** | tipe göre: Şıklar / Doğru-Yanlış / Boşluk / Cevap alanı (`BY_TYPE`) |
| **Matematik** | Matematik · Blok matematik · Üs · Alt indis · Kesir · Kök · Türev · Kısmi türev · Toplam · İntegral |
| **Yerleşim** | Tablo · Şekil + başlık · Metin \| görsel · Boşluk bırak · Kalın |
| **Ekle** | Görsel (dosya seçici) |

## 9.3 Yüzen panel

Tam genişlik şerit ölür. Yerine kaynak bölmesinin üzerinde **yüzen, köşesiz,
gölgeli bir kâğıt tabakası** — grup sekmeli.

```
┌─ kaynak bölmesi ───────────────────────────┐
│ Aşağıdaki denklemin köklerini bulunuz.     │
│                                            │
│ $ x^2 - 5x + 6 = 0 $                       │
│                                            │
│ #secenekler(dogru: "C",                    │
│   [$x = 1$],           ┌───────────────────┴──┐
│   [$x = 2$],           │ Kalıp │Matematik│ ⋯  │
│ )                      ├──────────────────────┤
│                        │ Üs        Alt indis  │
│                        │ Kesir     Kök        │
│                        │ Türev     Kısmi tür. │
│                        │ Toplam    İntegral   │
│                        │ Matematik Blok mat.  │
│                        └──────────────────────┘
└────────────────────────────────────────────┘

kapalı hâl:            ┌──────────┐
                       │ ⊞  Ekle  │   ← tek pill, köşede
                       └──────────┘
```

Kurallar:

- **Köşesiz, yuvarlatmasız.** `border: 1px solid var(--color-rule-strong)`,
  `background: var(--color-paper-lift)`, hafif gölge. Kırmızı yalnız
  değerlendirmede kalır — panelde kırmızı yok.
- **Kapalı başlar**, tek `⊞ Ekle` pill'i olarak. Açılınca son kullanılan grup
  seçili gelir.
- Kaynak bölmesinin **sağ-alt köşesinde** yüzer; metni kapatmaz çünkü yazma
  imleci genelde sol-üstte. Sürüklenebilir olması v1'de gerekmez.
- `Esc` kapatır (§9.5 merdiveni).

**Tasarım sistemi notu:** Kırmızı Kalem "kart yok" der. Bu panel kart değil,
`.sheet` ile aynı dili konuşan bir **kâğıt tabakası**dır: köşesiz, cetvelli
kenarlı. Dünyanın tek istisnası budur ve kullanıcı tarafından açıkça istendi.

## 9.4 Uygulama deseni — `signex-iced`'ten

Kullanıcının `signex-iced` projesinde bu işin olgun hâli var
(`crates/signex-app/src/app/view/context_menu/`). Alınacak kararlar:

- **Veri-görünüme (`#269`, `ADR-0003`).** Menü/palet saf bir **veri dizisidir**,
  widget ağacı değil. TEK paylaşılan çizici hepsini basar.
- **Saf kurucu / ince kabuk ayrımı.** `*_entries` kurucuları etiketleri,
  eylemleri, etkin-durumu taşır ve **DOM olmadan birim testi yapılır**; ince
  kabuklar durumu çözüp çiziciyi çağırır.
- **Satır kromu çiziciye aittir** — ikon sütunu, etiket, ipucu, hover,
  grileştirme. Çağıran taraf elle düğme kurmaz.

TAYAN karşılığı:

```ts
// lib/question/templates.ts  (mevcut Block tipi korunur)
export type BlockGroup = { id: string; label: string; blocks: Block[] };
export function groupsFor(type: QuestionType): BlockGroup[];
```

`groupsFor` saf ve vitest ile test edilir. `FloatingPalette.svelte` yalnız çizer.
Aynı çizici ileride sağ-tık menüsü gerekirse tekrar kullanılır.

## 9.5 Esc merdiveni — zorunlu

`signex-iced`/`view/overlays/bars.rs`'teki `has_blocking_modal` yorumu, `#547`
kaydı: bir modal kendi penceresine taşınınca kart çizilmeyi bıraktı ama yüklem
hâlâ "yığın bloke" diyordu; `collect_overlays` erken dönüş yaptı ve **beş
oluşturucunun hepsi hiçbir şey döndürmedi.** Ctrl+G, çıkış onayı, Preferences,
kütüphane modalleri: hepsi durumunu açtı, hiçbiri görünmedi, ekranda sebebini
açıklayan hiçbir şey yoktu.

Kural, aynı dosyadan:

> Esc merdiveninin kopyası bu yüklemle **terim terim uyuşmalı, yoksa Esc
> ekranda olmayan bir yığına karşı çözülür.**

TAYAN'da eşzamanlı olabilecek katmanlar: yüzen kalıp paneli, sınav çekmecesi,
yardım paneli, boş-belge tip seçici, kaydedilmemiş-değişiklik onayı.

**Esc'i bileşenlere dağıtmak yasak.** Tek sıralı merdiven + tek açık-katman
kaydı, baştan kurulur.

## 9.6 Silme güncellemesi

| Silinen | Nereye gitti |
|---|---|
| `BlockPalette.svelte` tam genişlik şeridi (§5'te "sütun başlığına taşınır" deniyordu — artık **panel oluyor**) | `FloatingPalette.svelte` |
| `COMMON` düz listesi | `groupsFor()` gruplu yapısı |
