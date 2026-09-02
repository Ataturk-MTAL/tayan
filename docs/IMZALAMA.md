# Paket İmzalama

> **Karar: şimdilik yapılmayacak.** Maliyet bir okul projesi için gerçekçi değil.
>
> Bu dosya kararı ve sebebini kayda geçirir — altı ay sonra biri aynı araştırmayı
> baştan yapmasın. Hiçbir anahtar, sertifika veya parola içermez.

## Sorun

Paketler imzasız çıkıyor. Kullanıcının gördüğü:

| Platform | Uyarı |
|---|---|
| macOS | "TAYAN açılamıyor çünkü geliştiricisi doğrulanamadı." |
| Windows | SmartScreen: "Windows bilgisayarınızı korudu." |
| Linux | Uyarı yok; `.deb` / `.rpm` / `.AppImage` imza beklemiyor. |

Bir öğretmen için bu uyarı çoğu zaman kurulumun bittiği yerdir. Uygulama
çalışsa bile "virüs mü acaba" diye vazgeçilir.

## Maliyet — neden yapılmıyor

| Kalem | Fiyat |
|---|---|
| Apple Developer Program | **99 USD / yıl**, her yıl yenilenir |
| Windows OV kod imzalama | ~200-400 USD / yıl + donanım anahtarı (2023'ten beri zorunlu) |
| Windows EV kod imzalama | Daha pahalı, ama SmartScreen itibarını anında verir |
| Azure Trusted Signing | ~10 USD / ay, donanım anahtarı gerekmez |

En ucuz gerçekçi paket bile yılda **200 USD'nin üstünde** ve her yıl tekrarlıyor.
Ücretsiz, açık kaynak, tek kişilik bir proje için karşılığı yok.

Ek olarak Windows OV sertifikasında SmartScreen itibarı **zamanla ve indirme
sayısıyla** birikir; ilk kullanıcılar parayı verdikten sonra bile uyarı görür.
Yani 200 USD tek başına sorunu çözmüyor.

## Şimdi ne yapılıyor

**Kurulum adımları release notunda açıkça yazılıyor.** İmzalamanın yerini
tutmaz ama kullanıcıyı yalnız bırakmaz — uyarıyı bekleniyor bir adıma çevirir:

- **macOS:** indirdikten sonra uygulamaya **sağ tıklayıp Aç**, çıkan pencerede
  tekrar **Aç**. Yalnız ilk açılışta gerekir.
- **Windows:** SmartScreen uyarısında **Ek bilgi → Yine de çalıştır**.

## İleride yapılırsa

`release.yml` **zaten hazır**. Apple imzalama ortam değişkenleri okunuyor;
sırlar tanımlı değilse boş geliyor ve imzasız paket üretiliyor. Sırlar eklendiği
gün imzalama kendiliğinden devreye girer, iş akışında değişiklik gerekmez.

Eklenecek GitHub sırları (Settings → Secrets and variables → Actions):

| Sır adı | Nereden |
|---|---|
| `APPLE_CERTIFICATE` | Developer ID Application `.p12` dosyasının base64'ü |
| `APPLE_CERTIFICATE_PASSWORD` | `.p12` dışa aktarılırken verilen parola |
| `APPLE_SIGNING_IDENTITY` | `security find-identity -v -p codesigning` çıktısındaki tam ad |
| `APPLE_ID` | Apple hesabı e-postası |
| `APPLE_PASSWORD` | appleid.apple.com → Uygulamaya Özel Parolalar (hesap parolası DEĞİL) |
| `APPLE_TEAM_ID` | developer.apple.com → Membership, 10 karakter |

Windows tarafı **yapılandırılmadı**: hangi yolun seçileceği maliyet kararı ve
seçime göre `tauri.conf.json`'daki `bundle.windows` ayarları tamamen değişiyor.
Karar verildiğinde yapılandırılır.
