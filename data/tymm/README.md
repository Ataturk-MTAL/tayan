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
ölçümü: **108 ders · 1597 ünite · 6006 öğrenme çıktısı · 12 057 süreç bileşeni**.

Çıktılar üniteye ANAHTARLA değil KONUMLA bağlanır: çıktı fiziksel olarak hangi
ünite bloğunun içindeyse ona aittir. Anahtar eşleştirme iki yerde çuvallıyordu —
üç sayılı kodda ünite kademesi yok, bazı belgelerde de "N. SINIF" satırı hiç
geçmiyor.

Kaynak ÜÇ kod şeması kullanıyor ve hangisinin geçerli olduğu belgeden anlaşılır:

| Şema | Örnek | Anlamı |
|---|---|---|
| dört sayılı | `FİZ.9.1.2.` | ön ek + sınıf + ünite + sıra |
| üç sayılı | `T.O.5.2.` | ön ek + sınıf + sıra (ünite kademesi yok) |
| yapışık | `TDE1.1.` | rakam ön eke bitişik; beceri kodlarıyla aynı biçimde |

Bir ders birden fazla ön ek kullanabilir (Türkçe: `T.D`, `T.O`, `T.Y`, `T.K`).
Ünite başlığı sözcüğü de değişir (`ÜNİTE`, `TEMA`, `ÖĞRENME ALANI`); kullanılan
sözcük her ünitenin `kind` alanındadır.

Eksikler gizlenmez, sayıyla durur:

- **3 derste hiç kodlu çıktı çıkarılamadı.** En az biri (Türk Kültür ve
  Medeniyet Tarihi) kaynakta gerçekten kodlu çıktı taşımıyor; belgede yalnız
  `SDB`/`SBAB` beceri kodları geçiyor.
- **16 derste 241 çıktı** hiçbir ünite bloğunun içinde değil; dosyalarda
  `unassigned_outcomes` altında durur, atılmaz.
- **19 tür beceri kodu çözülemiyor.** İkiye ayrılır: seti VAR ama kodu yok
  (`KB2.16.1/.2/.3`, `E3.11`, `DAB3.1` — MEB'in iki belgesi birbirini tutmuyor)
  ve seti hiç yok (`SBD1`, `SDBS3`, `SBSB5` — 1-5 geçişlik, görünüşe göre
  kaynaktaki yazım hataları).

Aynı çıktının birden çok ünitede görünmesi tekrar değil, veridir: Ortaokul
Türkçe'de 418 benzersiz çıktı 800 kayıt üretir, çünkü aynı çıktı birkaç temada
işlenir. Ünite bloğu "nerede işleniyor"u, belge geneli kanonik tanımı verir;
ikisi birleştirilir.
