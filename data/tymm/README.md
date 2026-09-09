# TYMM veri seti

> **Haklar.** Bu dizindeki içerik T.C. Millî Eğitim Bakanlığı'nın Türkiye
> Yüzyılı Maarif Modeli yayınlarından türetilmiştir. İçeriğin ve "Türkiye
> Yüzyılı Maarif Modeli" adının marka, telif ve kullanım hakları MEB'e aittir;
> TAYAN'ın Apache-2.0 lisansı bu veriyi kapsamaz. TAYAN MEB ile bağlantılı
> değildir ve MEB tarafından onaylanmamıştır. Bağlayıcı metin için
> <https://tymm.meb.gov.tr>.

MEB Türkiye Yüzyılı Maarif Modeli'nden çekilmiş beceri çerçevesi, ders
programları ve ölçme-değerlendirme rehberi.

## Sürümlenen dosyalar

| Dosya | Boyut | Kaynak |
|---|---:|---|
| `beceriler.json` | 392 KB | 16 statik sayfa — 17 set, 462 beceri, 1512 süreç bileşeni |
| `dersler.json` | 6.0 MB | 2 uç nokta, 111 ders — 32 639 beceri eşlemesi |
| `olcme-rehberi.json` | 40 KB | 2 PDF'ten elle çıkarılmış ölçme kuralları |
| `courses/<slug>.json` | 80 dosya | Ders programı PDF'lerinden ünite, ders saati, öğrenme çıktısı ve süreç bileşenleri |
| `courses/index.json` | 1 dosya | Ders dizini + toplam sayımlar |

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
python3 scripts/fetch_tymm_ogrenme_ciktilari.py  # ağ; ders PDF'lerini indirir, metne çevirir, siler
python3 scripts/check_tymm_courses.py            # doğrulama; yapısal sorunda çıkış kodu 1
```

`olcme-rehberi.json` mekanik üretilmez — kaynak serbest metindir, içerik
PDF'lerden elle çıkarılmıştır. Kapsam dışı bırakılanlar ve kaynak
tutarsızlıkları dosyanın `notes` alanındadır.

## Güncelleme

MEB sayfaları ve PDF'leri zamanla değişir; PDF'ler repoda tutulmaz. Neyin
değiştiğini `sources.lock.json` üzerinden takip ederiz:

```bash
python3 scripts/check_tymm_sources.py            # karşılaştır, raporla
python3 scripts/check_tymm_sources.py --deep     # 111 ders grafiğini de tara
python3 scripts/check_tymm_sources.py --update   # kilidi tazele
```

Kilitte kaynak başına iki parmak izi durur. Tek hash yanıltıcıdır: sayfalardaki
CSS bağlantısı sürüm damgası taşır (`output1.css?v=...`), site yeniden
yayınlanınca içerik aynı kalsa bile ham hash döner.

| Alan | Anlamı |
|---|---|
| `sha256_raw` | ham bayt — yalnız bu değiştiyse kozmetik, yok sayılabilir |
| `sha256_text` | görünür metin — bu da değiştiyse İÇERİK değişmiş |
| `course_count` | ders sayısı; değişmesi ders eklenip çıkarıldığını gösterir |
| `status` | `ok` / `missing`; kaybolan kaynak geri gelirse "YENİDEN YAYINDA" |

Script hiçbir veri dosyasının üzerine yazmaz — güncelleme kararı insana aittir.
İçerik değişmişse veya kaynak kaybolmuşsa çıkış kodu 1 döner, böylece cron
işi veya CI adımı olarak kullanılabilir.

Sıra önemli: önce `check_tymm_sources.py` ile neyin değiştiğini gör, sonra
ilgili çekme script'ini çalıştır, sonra üretilen veriyi gözden geçir, en son
`--update` ile kilidi tazele. Kilidi önce tazelersen değişimin kaydı kaybolur.

`performans-gelisim-cercevesi.pdf` şu an yayında değil (sunucu 500 veriyor).
Kilitte `missing` olarak durur; listeden çıkarılmadı ki geri gelirse görülsün.

## Birleştirme

Beceri kodları (`KB2.8`, `D9`, `SBAB1.1` …) üç dosyada da ortaktır.
32 639 eşlemenin 32 612'si çözülür (%99,92). Çözülemeyen `KB2.16.1/.2/.3`
kaynak tutarsızlığıdır; satırlar `resolved: false` ile durur, atılmaz.

Ayrıntı: `SOURCES.md` (beceri çerçevesi), `DERSLER.md` (ders programları).

## Ders programları — bilinen eksikler

`courses/` altındaki veri ders programı PDF'lerinden çıkarılır. 2026-09-09
ölçümü: **111 ders · 5846 öğrenme çıktısı · 12 225 süreç bileşeni**. Çıktısı
çıkarılamayan ders yok.

Çıktılar üniteye ANAHTARLA değil KONUMLA bağlanır: çıktı fiziksel olarak hangi
ünite bloğunun içindeyse ona aittir.

Kaynak **beş** kod biçimi kullanıyor ve hangisinin geçerli olduğu belgeden
anlaşılır — ders adından değil:

| Biçim | Örnek | Ders |
|---|---|---|
| dört sayılı | `FİZ.9.1.2.` | Fizik, Matematik |
| üç sayılı | `T.O.5.2.` | Türkçe, Okuma Becerileri |
| yapışık, üç sayılı | `RK2.4.1.` | Robotik Kodlama |
| yapışık, iki sayılı | `TDE1.1.` | Türk Dili ve Edebiyatı |
| boşluklu | `TKMT 3.1.` | Türk Kültür ve Medeniyet Tarihi |

Biçim aileleri SIRAYLA değil HACME göre yarışır: Türk Dili'nde 10 kez geçen
noktalı `E.` atıfı, 382 kez geçen yapışık `TDE` şemasını gölgeliyordu.

Bir ders birden fazla ön ek kullanabilir (Türkçe: `T.D`, `T.O`, `T.Y`, `T.K`).
Ünite başlığı sözcüğü de değişir (`ÜNİTE`, `TEMA`, `ÖĞRENME ALANI`); kullanılan
sözcük her ünitenin `kind` alanındadır.

Kalan eksikler — ikisi de uydurmadan kapatılamaz:

- **19 tür beceri kodu çözülemiyor (98 geçiş).** 8 türü kaynak tutarsızlığı:
  `KB2.16.1/.2/.3` (64 geçiş), `E3.11`, `DAB3.1/.2`. MEB'in beceri sayfasında
  `KB2.16` için "Süreç bileşenleri" bloğu HİÇ YOK (canlı sayfada `KB2.16.SB`
  satırı 0, `KB2.17.SB` 4) ama ders programları o kodlara atıf yapıyor. 11 türü
  tanınmayan küme (`SBD1`, `SDBS3`, `SBSB5` — 1-5 geçişlik, görünüşe göre
  kaynaktaki dizgi hataları).
- **Bazı çıktılar hiçbir ünite bloğunun içinde değil**; dosyalarda
  `unassigned_outcomes` altında durur, atılmaz.

Aynı çıktının birden çok ünitede görünmesi tekrar değil, veridir: Ortaokul
Türkçe'de 418 benzersiz çıktı 800 kayıt üretir, çünkü aynı çıktı birkaç temada
işlenir. Ünite bloğu "nerede işleniyor"u, belge geneli kanonik tanımı verir.
