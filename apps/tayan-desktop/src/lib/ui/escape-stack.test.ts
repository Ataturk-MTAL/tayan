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
    addEventListener: (type: string, fn: (e: KeyboardEvent) => void) => {
      if (type === "keydown") handler = fn;
    },
    removeEventListener: () => {},
  });
});

describe("pushEscapeLayer", () => {
  test("tek katman Esc ile kapanır", () => {
    const close = vi.fn();
    pushEscapeLayer(close);
    esc();
    expect(close).toHaveBeenCalledTimes(1);
  });

  test("yalnız EN ÜSTTEKİ katman kapanır — hepsi birden değil", () => {
    // Asıl hata buydu: palet ve tamamlama kutusu tek Esc'te birlikte kapanıyordu.
    const closeBottom = vi.fn();
    const closeTop = vi.fn();
    pushEscapeLayer(closeBottom);
    pushEscapeLayer(closeTop);

    esc();
    expect(closeTop).toHaveBeenCalledTimes(1);
    expect(closeBottom).not.toHaveBeenCalled();
  });

  test("üst katman kalkınca sıradaki devralır", () => {
    const closeBottom = vi.fn();
    const closeTop = vi.fn();
    pushEscapeLayer(closeBottom);
    const remove = pushEscapeLayer(closeTop);

    remove();
    esc();
    expect(closeBottom).toHaveBeenCalledTimes(1);
    expect(closeTop).not.toHaveBeenCalled();
  });

  test("Esc başkası tarafından harcanmışsa merdiven devreye GİRMEZ", () => {
    // CodeMirror tamamlama kutusunu kapatırken preventDefault çağırıyor.
    const close = vi.fn();
    pushEscapeLayer(close);
    esc({ defaultPrevented: true });
    expect(close).not.toHaveBeenCalled();
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
    const close = vi.fn();
    pushEscapeLayer(close);
    const remove = pushEscapeLayer(() => {});
    remove();
    remove();
    expect(escapeLayerCount()).toBe(1);
    esc();
    expect(close).toHaveBeenCalledTimes(1);
  });

  test("Escape dışındaki tuşlar yok sayılır", () => {
    const close = vi.fn();
    pushEscapeLayer(close);
    handler?.({
      key: "Enter",
      defaultPrevented: false,
      preventDefault: () => {},
    } as KeyboardEvent);
    expect(close).not.toHaveBeenCalled();
  });
});
