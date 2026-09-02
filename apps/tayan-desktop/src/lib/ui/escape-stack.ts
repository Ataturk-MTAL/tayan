/**
 * Esc merdiveni — tek pencere dinleyicisi, en üstteki katmanı kapatır.
 *
 * NEDEN GEREKLİ. Her katmanın kendi pencere dinleyicisi olduğunda tek bir Esc
 * hepsini birden kapatıyor. Somut örnek: kalıp paleti açıkken editörde Esc'e
 * basmak CodeMirror'ın tamamlama kutusunu kapatıyor, olay pencereye çıkıyor ve
 * paleti DE kapatıyordu. Öğretmen bir şey kapatmak isterken iki şey kaybediyor.
 *
 * ÇÖZÜM iki parçalı:
 *   1. Tek dinleyici, yığının EN ÜSTÜNDEKİ katmanı kapatır — VS Code'un
 *      davranışı. İç içe katmanlar içten dışa kapanır, hepsi birden değil.
 *   2. `defaultPrevented` kontrolü. CodeMirror gibi kendi Esc'ini işleyen
 *      bileşenler işlediklerinde preventDefault çağırır; o durumda merdiven
 *      hiç devreye girmez.
 *
 * Kayıtsız katmanlar (CodeMirror) yığında görünmez ama (2) sayesinde
 * çakışmazlar. Kendi Esc'ini işleyip preventDefault ÇAĞIRMAYAN bir bileşen
 * eklenirse burası yeniden düşünülmeli.
 *
 * SelectBox merdivene KAYITLI DEĞİL ve olmamalı: onun Esc'i kendi girdisinin
 * üzerinde, stopPropagation ile duruyor — pencereye hiç ulaşmıyor. Odak
 * içindeyken doğru katman zaten odur.
 */

type Layer = { close: () => void };

const stack: Layer[] = [];
let installed = false;

function onKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape") return;
  // Başka biri bu Esc'i zaten harcadı (CodeMirror tamamlaması gibi).
  if (event.defaultPrevented) return;

  const top = stack.at(-1);
  if (!top) return;

  event.preventDefault();
  top.close();
}

function ensureListener() {
  if (installed || typeof window === "undefined") return;
  window.addEventListener("keydown", onKeydown);
  installed = true;
}

/**
 * Katmanı yığına ekler, kaldırma fonksiyonunu döndürür.
 *
 * Kaldırma, katman kapanırken MUTLAKA çağrılmalı — yoksa yığında ölü bir
 * katman kalır ve Esc görünmeyen bir şeyi "kapatmaya" çalışır.
 */
export function pushEscapeLayer(close: () => void): () => void {
  ensureListener();
  const layer: Layer = { close };
  stack.push(layer);

  return () => {
    const i = stack.lastIndexOf(layer);
    if (i !== -1) stack.splice(i, 1);
  };
}

/** Test içindir: yığındaki katman sayısı. */
export function escapeLayerCount(): number {
  return stack.length;
}

/**
 * Test içindir: yığını boşaltır VE dinleyiciyi kaldırır.
 *
 * Dinleyiciyi bırakmak eksik bir sıfırlamaydı: `installed` bayrağı modül
 * düzeyinde olduğu için ikinci testte dinleyici bir daha kaydolmuyor ve yığın
 * sessizce sağır kalıyordu. Sıfırlama, adının söylediğini yapmalı.
 */
export function resetEscapeStack(): void {
  stack.length = 0;
  if (installed && typeof window !== "undefined") {
    window.removeEventListener("keydown", onKeydown);
  }
  installed = false;
}
