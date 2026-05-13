# TAYAN — Görev Planı

## Şablon

```
## T{N} — {Başlık}
**Durum:** ⬜ TODO | 🔄 IN PROGRESS | ✅ DONE
**Faz:** {0-5}
**Kabul kriterleri:**
- [ ] ...
**Commit:** `—` (tamamlanınca hash)
```

---

## Mevcut Durum (Tamamlananlar)

| # | Konu | Durum |
|---|------|-------|
| — | Cargo workspace yapısı | ✅ |
| — | tayan-macros (proc macro + macro_rules!) | ✅ |
| — | tayan-core domain katmanı (ContentNode, Question tipleri, Exam, QuestionBank, QuestionStats) | ✅ |
| — | tayan-compiler (TypstGenerator — string çıktı) | ✅ |
| — | tayan-db (SQLite repos + migrations) | ✅ |
| — | tayan-desktop Tauri iskelet (commands, state, capabilities) | ✅ |
| — | Frontend iskelet (SvelteKit SPA, layout, sidebar, dashboard) | ✅ |
| — | macOS 26 crash fix (`devtools: false`) | ✅ |

---

## FAZ 0 — Altyapı & Git

### T01 — Git reposu başlat ve ilk commit al
**Durum:** ✅ DONE
**Kabul kriterleri:**
- [x] `git init` yapıldı
- [x] `.gitignore` doğru (target/, node_modules/, .svelte-kit/, dist/, *.db)
- [x] `cargo check` temiz
- [x] İlk commit alındı: `chore: initial commit — workspace + all crates + Tauri scaffold`
**Commit:** `f8d4dd5`

### T02 — İkon dosyalarını düzelt
**Durum:** ⏭ SKIP — ikon tasarımı ayrı yapılacak, son aşamaya bırakıldı

---

## FAZ 1 — Soru Bankası UI

### T03 — Soru listesi sayfası (`/questions`)
**Durum:** ✅ DONE
**Kabul kriterleri:**
- [x] `/questions` route'u var, boş sayfa değil
- [x] `list_questions` komutu çağrılıyor, sorular listeleniyor
- [x] Her soru satırında: tip etiketi, puan, ScoreBadge (Untested/Good/Excellent…)
- [x] "Soru Ekle" butonu görünür, `/questions/new` açıyor
**Commit:** `43589ff`

### T04 — Çoktan seçmeli soru ekleme formu
**Durum:** ✅ DONE
**Kabul kriterleri:**
- [x] Form: soru gövdesi (plaintext, T06'da math eklenecek), 5 seçenek, doğru cevap seçimi, puan, kazanım kodu
- [x] `add_multiple_choice_question` Tauri komutu başarılı dönüyor
- [x] Yeni soru listede görünüyor
**Commit:** `876418d`

### T05 — Diğer soru tipleri (TF, FillInBlank, Classic)
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] Soru tipi seçici (tab veya dropdown)
- [ ] `add_true_false_question` formu çalışıyor
- [ ] `add_fill_in_blank_question` formu çalışıyor (blank placeholder ekleme)
- [ ] `add_classic_question` formu çalışıyor (rubric config dahil)
**Commit:** `—`

### T06 — KaTeX math preview (soru editörüne entegrasyon)
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] Soru gövdesinde `$...$` veya `$$...$$` sözdizimi tanınıyor
- [ ] Yazarken KaTeX ile render preview gösteriliyor (split view veya inline)
- [ ] Geçersiz LaTeX için hata göstergesi var
- [ ] Math node `ContentNode::Math` olarak serileştiriliyor
**Commit:** `—`

---

## FAZ 2 — Sınav Yönetimi

### T07 — Sınav listesi sayfası (`/exams`)
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] `list_exams` çağrılıyor, sınavlar listeleniyor
- [ ] Durum etiketi: Taslak / Yayında / Arşiv
- [ ] "Yeni Sınav" butonu çalışıyor
**Commit:** `—`

### T08 — Sınav oluşturma formu (`/exams/new`)
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] Form: başlık, ders, sınıf, öğretmen, süre, tarih, talimatlar
- [ ] `create_exam` komutu çağrılıyor
- [ ] Oluşturma sonrası `/exams/{id}` sayfasına yönlendirme
**Commit:** `—`

### T09 — Sınava soru ekleme & sıralama
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] Sınav detay sayfasında soru bankası listesi (panel veya modal)
- [ ] Soru seçme → `add_question_to_exam` çağrısı
- [ ] Seçili sorular sıralı gösteriliyor
- [ ] Sürükle-bırak veya yukarı/aşağı ok ile sıralama
- [ ] Toplam puan otomatik hesaplanıyor
**Commit:** `—`

### T10 — Sınav yayınlama
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] "Yayınla" butonu görünür (en az 1 soru varsa aktif)
- [ ] `publish_exam` çağrısı başarılı
- [ ] Durum etiketi "Yayında" olarak güncelleniyor
**Commit:** `—`

---

## FAZ 3 — Öğrenci Yönetimi

### T11 — Sınıf oluşturma & listeleme (`/students`)
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] `list_classrooms` çağrılıyor
- [ ] "Yeni Sınıf" formu: isim → `create_classroom`
- [ ] Sınıflar kart veya liste olarak görünüyor
**Commit:** `—`

### T12 — Öğrenci ekleme & listeleme
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] Sınıf detayında öğrenci listesi (`list_students_by_classroom`)
- [ ] "Öğrenci Ekle" formu: ad, soyad, numara → `add_student`
- [ ] Toplu ekleme (CSV yapıştırma): satır başına `isim,numara`
**Commit:** `—`

---

## FAZ 4 — Sınav Analizi

### T13 — Sınav sonuçları girişi
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] Yayında sınav + sınıf seçimi
- [ ] Her öğrenci için soru soru cevap girişi (MC: seçenek, TF: T/F, Classic: puan)
- [ ] `enter_exam_results` komutu çağrılıyor
- [ ] Kaydedilen sonuçlar üzerine yazılabiliyor (UNIQUE constraint tolere ediliyor)
**Commit:** `—`

### T14 — Analiz dashboard (`/analysis`)
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] Sınav + sınıf seçici
- [ ] **Puan dağılımı histogramı** (LayerCake + D3) — öğrenci puanları
- [ ] **Soru × Öğrenci heatmap** — doğru/yanlış matrisi
- [ ] Sınıf ortalaması, en yüksek/en düşük puan gösterimi
**Commit:** `—`

### T15 — Soru istatistik badge'leri
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] Soru bankasında her soruda difficulty_index gösterimi
- [ ] discrimination_index uyarısı (< 0.2 → sarı bayrak)
- [ ] ScoreBadge renk kodlu: Excellent (yeşil) / Good (mavi) / Fair (sarı) / Poor (kırmızı)
- [ ] Filtreleme: outcome'a göre, badge'e göre, tipe göre
**Commit:** `—`

---

## FAZ 5 — PDF Çıktısı

### T16 — Typst kaynak görüntüleme (ara adım)
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] Sınav detayında "PDF Önizle" butonu
- [ ] `generate_exam_pdf` çağrısı → Typst kaynak string döndürüyor
- [ ] Kaynak, modal içinde `<pre>` ile gösteriliyor (ara adım olarak yeterli)
- [ ] Cevap anahtarlı / öğrenci versiyonu toggle
**Commit:** `—`

### T17 — typst-bake entegrasyonu (binary PDF)
**Durum:** ⬜ TODO
**Kabul kriterleri:**
- [ ] `typst-bake` crate'i workspace'e eklendi
- [ ] `TypstGenerator` Typst string → `Vec<u8>` PDF byte dönüşümü yapıyor
- [ ] Tauri komutu PDF byte'larını Base64 olarak frontend'e iletiyor
- [ ] Frontend PDF.js ile render ediyor
- [ ] "PDF Kaydet" ile dosya seçici açılıyor
**Commit:** `—`

---

## Özet Tablo

| Faz | Görev | Durum |
|-----|-------|-------|
| 0 | T01 Git init | ✅ |
| 0 | T02 İkon fix | ⏭ SKIP |
| 1 | T03 Soru listesi | ✅ |
| 1 | T04 MC soru formu | ✅ |
| 1 | T05 Diğer soru tipleri | ⬜ |
| 1 | T06 KaTeX preview | ⬜ |
| 2 | T07 Sınav listesi | ⬜ |
| 2 | T08 Sınav formu | ⬜ |
| 2 | T09 Soru ekleme & sıralama | ⬜ |
| 2 | T10 Sınav yayınlama | ⬜ |
| 3 | T11 Sınıf yönetimi | ⬜ |
| 3 | T12 Öğrenci yönetimi | ⬜ |
| 4 | T13 Sonuç girişi | ⬜ |
| 4 | T14 Analiz dashboard | ⬜ |
| 4 | T15 Soru istatistikleri | ⬜ |
| 5 | T16 Typst kaynak görüntüleme | ⬜ |
| 5 | T17 typst-bake / binary PDF | ⬜ |
