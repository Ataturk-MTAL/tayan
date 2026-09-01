import { getCurrentWebview } from "@tauri-apps/api/webview";

/**
 * Sayfa zoom'unu kilitler.
 *
 * SORUN: `SheetPreview` kendi zoom'unu `⌘+tekerlek` ve `⌘ +/-/0` ile
 * yönetiyor, ama dinleyicileri YALNIZCA önizleme kaydırıcısına bağlı. Fare
 * panelin ya da editörün üzerindeyken aynı hareket hiç yakalanmıyor ve
 * doğrudan WKWebView'e düşüyor: webview tüm belgeyi ölçekliyor. Sonuç, kâğıt
 * değil ARAYÜZ büyüyor — panel, gezinme bandı, satır numaraları, hepsi.
 *
 * macOS'ta trackpad kıstırması da buraya düşer: WebKit onu `ctrlKey: true`
 * olan bir `wheel` olayı olarak üretir, ayrı bir olay türü değildir.
 *
 * ÇÖZÜM: yakalama evresinde, pencere düzeyinde varsayılanı iptal et. Yayılım
 * DURDURULMAZ — `SheetPreview`'in kendi dinleyicileri sonrasında normal
 * çalışmaya devam eder. Yani kâğıdın zoom'u yaşar, sayfanın zoom'u ölür.
 */

/** Sayfa ölçeğini bire döndürür. Zaten kaymış bir pencereyi kurtarmak için. */
export async function resetPageZoom(): Promise<void> {
  try {
    await getCurrentWebview().setZoom(1);
  } catch {
    // Tarayıcıda (tauri dev dışında) webview yok; sessiz geçmek doğru.
    // Yetki verilmemişse de aynısı: koruma zaten yeni kaymayı engelliyor.
  }
}

/** Sayfa zoom'unu tetikleyen tuşlar. `_` ve `=` kaydırmalı düzenler içindir. */
const ZOOM_KEYS = new Set(["+", "=", "-", "_", "0"]);

/**
 * Korumayı kurar ve kaldırma fonksiyonunu döndürür.
 *
 * `passive: false` şart: tarayıcılar `wheel`'i varsayılan olarak edilgen kabul
 * eder ve edilgen bir dinleyicide `preventDefault()` sessizce yok sayılır.
 */
export function installPageZoomGuard(): () => void {
  const onWheel = (event: WheelEvent) => {
    if (event.ctrlKey || event.metaKey) event.preventDefault();
  };

  const onKeydown = (event: KeyboardEvent) => {
    if (!event.metaKey && !event.ctrlKey) return;
    if (ZOOM_KEYS.has(event.key)) event.preventDefault();
  };

  window.addEventListener("wheel", onWheel, { passive: false, capture: true });
  window.addEventListener("keydown", onKeydown, { capture: true });

  return () => {
    window.removeEventListener("wheel", onWheel, { capture: true });
    window.removeEventListener("keydown", onKeydown, { capture: true });
  };
}
