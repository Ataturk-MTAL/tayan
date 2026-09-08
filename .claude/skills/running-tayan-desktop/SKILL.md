---
name: running-tayan-desktop
description: Use when launching, restarting, or screenshotting the TAYAN desktop app in apps/tayan-desktop (Tauri v2 + SvelteKit + vite) — especially on "Port 5173 is already in use", when `pnpm tauri dev` exits 0 with no window from a non-interactive or agent shell, or when a screenshot captures the terminal instead of the app window.
---

# TAYAN masaüstünü çalıştırma

## Genel bakış

`apps/tayan-desktop` bir Tauri v2 uygulaması: Rust arka uç
(`target/debug/tayan-desktop`) + vite dev sunucusu (SvelteKit, HMR).
İnsan terminalinde tek komut yeter. İki tuzak var ve ikisi de sessiz
başarısızlıkla biter — sabitlenmiş `5173` portu, ve TTY'siz kabukta
pencereyi açıp hemen çıkan dev süreci.

## Hızlı başvuru

| Durum | Yol |
|---|---|
| İnteraktif terminal, `5173` boş | `cd apps/tayan-desktop && pnpm tauri dev` |
| `Port 5173 is already in use` | `--config` ile port yaması |
| TTY yok (ajan, arkaplan kabuğu, hook) | vite + binary'yi ayrı ayrı `nohup … & disown` |
| Pencereyi görüntüle | önce öne al, sonra `screencapture` |
| Durdur | `pkill -f 'target/debug/tayan-desktop'` + vite süreci |

İlk soğuk derleme ~682 crate, `1m 17s` (Apple Silicon, `dev` profili).
Sonraki başlatmalar saniyeler.

Süreç ve port komutları (`lsof`, `pgrep`, `pkill`) taşınabilir. Pencereyi öne
alma ve ekran görüntüsü bölümü macOS'a özgüdür (`osascript`, `screencapture`);
başka platformda o adımların karşılığını kullan.

## Normal yol

```bash
cd apps/tayan-desktop && pnpm tauri dev
```

Çalıştığının kanıtı, pencerenin açılması değil — pencere boş da açılabilir.
Kanıt şu üçü:

1. Uygulama logunda `font kaydı hazır: <süre>` satırı.
2. `pgrep -f 'target/debug/tayan-desktop'` bir PID veriyor.
3. Pencerede "Bugün ne yapacaksın?" ve üç kart **sayılarla** dolu
   (`BANKADAKİ SORU`, `TASLAK SINAV`, `AYIRT EDİCİLİĞİ DÜŞÜK SORU`).
   Sayılar SQLite'tan gelir; `0/0/0` görürsen arka uç komutları değil,
   veritabanı yolu şüphelidir.

## Tuzak 1 — port `5173` iki yerde sabit

```
apps/tayan-desktop/vite.config.ts   server: { port: 5173, strictPort: true }
apps/tayan-desktop/src-tauri/tauri.conf.json   "devUrl": "http://localhost:5173"
```

`strictPort: true` olduğu için vite başka porta kaçmaz, ölür.
**Sadece birini değiştirme** — Tauri ölü porta bağlanır, pencere beyaz açılır.

Port doluysa önce **kimin tuttuğuna bak**, körlemesine öldürme:

```bash
lsof -nP -iTCP:5173 -sTCP:LISTEN
ps -p <PID> -o pid,ppid,lstart,command | cat
```

`5173` yaygın bir vite portudur. Onu tutan süreç çoğu zaman TAYAN değil,
aynı makinede açık duran başka bir projenin dev sunucusu veya orchestrator'ıdır;
körlemesine öldürmek o projenin çalışan yığınını düşürür.

Çakışmayı kalıcı dosya değişikliği yapmadan aşmak için config'i çalışma
anında yamala — tek komutta hem `devUrl` hem vite portu:

```bash
cd apps/tayan-desktop && pnpm tauri dev --config '{"build":{"devUrl":"http://localhost:5174","beforeDevCommand":"npm run dev -- --port 5174 --strictPort"}}'
```

`beforeDevCommand`'daki `-- --port` şart: vite CLI bayrağı config'teki
`server.port`'u ezer, yoksa vite yine `5173`'e gider.

## Tuzak 2 — TTY yoksa `pnpm tauri dev` sessizce çıkar

Ajan aracı, arkaplan kabuğu veya hook içinde `pnpm tauri dev` derlemeyi
bitirir, binary'yi çalıştırır, pencere açılır, sayfalar derlenir — sonra
**exit code 0** ile kapanır. Hata satırı yok. Log dolu ve düzgün görünür.

Çözüm: `tauri dev` sarmalayıcısını atla, iki parçayı ayrı ayrı ayrık başlat.

Önce vite, sonra binary — bu sıra önemlidir; binary açılışta dev URL'e bağlanır:

```bash
# 1) vite dev sunucusu
cd apps/tayan-desktop
nohup npm run dev -- --port 5174 --strictPort > /tmp/tayan-vite.log 2>&1 < /dev/null & disown
sleep 6
lsof -nP -iTCP:5174 -sTCP:LISTEN     # LISTEN satırı görmeden devam etme

# 2) derlenmiş binary
cd ../..
nohup ./target/debug/tayan-desktop > /tmp/tayan-app.log 2>&1 < /dev/null & disown
sleep 8
pgrep -fl 'target/debug/tayan-desktop'
```

Binary dev URL'i **derleme anında** içine gömer. Yukarıdaki `--config`
yamasıyla derlendiyse `5174` arar; düz derlendiyse `5173`. Portu
değiştirdiysen binary'yi o config ile bir kez derlemiş olman gerekir.

## Pencereyi görüntüleme

Binary bundle dışından çalıştığı için öne gelmez. Doğrudan `screencapture`
çekersen terminali yakalarsın — bu "uygulama açılmadı" demek değildir.
Önce süreç canlı mı diye `pgrep` ile ayır, sonra öne al:

```bash
osascript -e 'tell application "System Events" to tell process "tayan-desktop" to set frontmost to true'
sleep 3
screencapture -x /tmp/tayan-shot.png
```

Ekran görüntüsüne **bak**. Boş beyaz kare = webview dev URL'e bağlanamadı;
vite portunu ve binary'nin derlendiği `devUrl`'i karşılaştır.

## Durdurma

```bash
pkill -f 'target/debug/tayan-desktop'
pkill -f 'vite.*--port 5174'
```

## Sık yapılan hatalar

| Hata | Sonuç | Doğrusu |
|---|---|---|
| `5173`'ü tutan süreci sormadan öldürmek | Başka projenin yığını düşer | Önce `lsof` + `ps`, sahibini tanı |
| Sadece `devUrl`'i değiştirmek | Beyaz pencere, hata yok | İkisini birden yamala |
| `beforeDevCommand`'a `-- --port` koymamak | vite yine `5173`, yine çakışır | CLI bayrağı config'i ezer, şart |
| Arkaplanda `pnpm tauri dev` | exit 0, pencere kaybolur | vite + binary ayrı, `nohup … & disown` |
| Ekran görüntüsünde terminal görüp "açılmadı" demek | Yanlış teşhis | `pgrep` ile süreci doğrula, sonra öne al |
| Pencere açıldı diye "çalışıyor" demek | Boş kabuk da açılır | Kartlardaki sayıları oku |
| Süreç kaybolunca çökme sanmak | Boşuna hata avı | Pencere elle kapatıldıysa Tauri süreci de biter; log temizse yeniden başlat |

## Bilinen gürültü

Derlemede `QuestionForm.svelte` için 12 adet Svelte uyarısı çıkar:

```
src/lib/components/question/QuestionForm.svelte:55:42 This reference only captures the initial value of `initialType`.
https://svelte.dev/e/state_referenced_locally
```

Başlatmayı engellemez. Ayrı bir iş; başlatma sorunu ararken bu izi kovalama.
