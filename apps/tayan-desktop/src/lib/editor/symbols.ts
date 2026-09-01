import { invoke } from "@tauri-apps/api/core";

/**
 * Typst sembol dökümü.
 *
 * Rust tarafında Typst'in KENDİ kütüphanesi taranıyor; burada elle yazılmış
 * hiçbir liste yok. Elle liste iki yönden bozulur: Typst sürümü değişince
 * eskir, ve baştan eksik kalır — kimse 554 sembolü tek tek yazmaz.
 */
export type TypstSymbol = {
  name: string;
  /** "function" | "symbol" | "type" | "module" | "value" | "tayan" */
  kind: string;
  /** Yalnızca matematik kipinde geçerli ($ … $ içinde). */
  math: boolean;
  params: string[];
  summary: string;
};

let cache: Promise<TypstSymbol[]> | null = null;

/**
 * Bir kez çekilir ve süreç boyunca saklanır. Döküm derleme zamanı sabiti:
 * uygulama çalışırken değişmez, her tamamlama açılışında yeniden istemenin
 * anlamı yok.
 */
export function typstSymbols(): Promise<TypstSymbol[]> {
  if (cache === null) {
    cache = invoke<TypstSymbol[]>("typst_symbols").catch(() => {
      // Döküm alınamazsa tamamlama sessizce boş kalır; editör çalışmaya
      // devam eder. Bir liste uğruna yazmayı bloklamak yanlış olur.
      cache = null;
      return [];
    });
  }
  return cache;
}

/**
 * İmleç matematik kipinde mi?
 *
 * Kaçırılmamış `$` sayısı tekse içerideyiz. Kaba ama bu editörde doğru:
 * gövdeler kısa ve dolar dengesi bozuksa zaten derleme hatası veriyor.
 */
export function inMathMode(textBeforeCursor: string): boolean {
  let count = 0;
  for (let i = 0; i < textBeforeCursor.length; i += 1) {
    if (textBeforeCursor[i] !== "$") continue;
    if (i > 0 && textBeforeCursor[i - 1] === "\\") continue;
    count += 1;
  }
  return count % 2 === 1;
}

/** Belgede tanımlanmış `#let ad(...)` işlevleri — öğretmenin kendi yazdıkları. */
export function localDefinitions(doc: string): TypstSymbol[] {
  const out: TypstSymbol[] = [];
  const re = /#let\s+([a-zA-Z][\w-]*)\s*(\(([^)]*)\))?/g;

  for (const m of doc.matchAll(re)) {
    const params = (m[3] ?? "")
      .split(",")
      .map((p) => p.trim().replace(/^\.\./, "").split(":")[0].trim())
      .filter(Boolean);

    out.push({
      name: m[1],
      kind: "tayan",
      math: false,
      params,
      summary: "Bu soruda tanımlandı",
    });
  }
  return out;
}
