import { beforeEach, describe, expect, test, vi } from "vitest";
import { escapeLayerCount, pushEscapeLayer, resetEscapeStack } from "./escape-stack";

/**
 * Modül tek bir pencere dinleyicisi kuruyor. Testte gerçek DOM yok; `window`
 * taklit ediliyor ve yakalanan dinleyici elle çağrılıyor. Sınanan şey KARARIN
 * kendisi: hangi katman kapanır, hangisi kapanmaz.
 */
let handler: ((e: KeyboardEvent) => void) | null = null;

function esc(opts: { defaultPrevented?: boolean } = {}) {
  let prevented = opts.defaultPrevented ?? false;
  const e = {
    key: "Escape",
    get defaultPrevented() {
      return prevented;
    },
    preventDefault: () => {
      prevented = true;
    },
  } as unknown as KeyboardEvent;
  handler?.(e);
  return e;
}

beforeEach(() => {
  resetEscapeStack();
  handler = null;
  vi.stubGlobal("window", {
    addEventListener: (tip: string, fn: (e: KeyboardEvent) => void) => {
      if (tip === "keydown") handler = fn;
    },
    removeEventListener: () => {},
  });
});

describe("pushEscapeLayer", () => {
  test("tek katman Esc ile kapanır", () => {
    const kapat = vi.fn();
    pushEscapeLayer(kapat);
    esc();
    expect(kapat).toHaveBeenCalledTimes(1);
  });

  test("yalnız EN ÜSTTEKİ katman kapanır — hepsi birden değil", () => {
    // Asıl hata buydu: palet ve tamamlama kutusu tek Esc'te birlikte kapanıyordu.
    const alt = vi.fn();
    const ust = vi.fn();
    pushEscapeLayer(alt);
    pushEscapeLayer(ust);

    esc();
    expect(ust).toHaveBeenCalledTimes(1);
    expect(alt).not.toHaveBeenCalled();
  });

  test("üst katman kalkınca sıradaki devralır", () => {
    const alt = vi.fn();
    const ust = vi.fn();
    pushEscapeLayer(alt);
    const kaldir = pushEscapeLayer(ust);

    kaldir();
    esc();
    expect(alt).toHaveBeenCalledTimes(1);
    expect(ust).not.toHaveBeenCalled();
  });

  test("Esc başkası tarafından harcanmışsa merdiven devreye GİRMEZ", () => {
    // CodeMirror tamamlama kutusunu kapatırken preventDefault çağırıyor.
    const kapat = vi.fn();
    pushEscapeLayer(kapat);
    esc({ defaultPrevented: true });
    expect(kapat).not.toHaveBeenCalled();
  });

  test("katman kapatınca olay tüketilir", () => {
    pushEscapeLayer(() => {});
    expect(esc().defaultPrevented).toBe(true);
  });

  test("yığın boşsa olay tüketilmez", () => {
    pushEscapeLayer(() => {})();
    expect(esc().defaultPrevented).toBe(false);
  });

  test("kaldırma iki kez çağrılsa yığın bozulmaz", () => {
    const kapat = vi.fn();
    pushEscapeLayer(kapat);
    const kaldir = pushEscapeLayer(() => {});
    kaldir();
    kaldir();
    expect(escapeLayerCount()).toBe(1);
    esc();
    expect(kapat).toHaveBeenCalledTimes(1);
  });

  test("Escape dışındaki tuşlar yok sayılır", () => {
    const kapat = vi.fn();
    pushEscapeLayer(kapat);
    handler?.({
      key: "Enter",
      defaultPrevented: false,
      preventDefault: () => {},
    } as KeyboardEvent);
    expect(kapat).not.toHaveBeenCalled();
  });
});
