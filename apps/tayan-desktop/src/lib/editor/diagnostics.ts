/**
 * Rust tarafındaki `format_diagnostic` şu biçimi üretir:
 *
 *   Typst derleme hatası:
 *   unknown variable: foo (satır 3, sütun 5)
 *   İpucu: ...
 *
 * Buradaki iş, o metni editörde kırmızı kalemle işaretlenebilir hâle
 * getirmektir. Satır numarası kaybolursa öğretmen hatanın nerede olduğunu
 * göremez, ve "arayüzden Typst'i anlama" vaadi orada biter.
 */

export type TypstDiagnostic = {
  line: number | null;
  column: number | null;
  message: string;
  hint: string | null;
};

const LOCATION = /\s*\(satır (\d+), sütun (\d+)\)\s*$/;
const HEADER = /^Typst (derleme|PDF oluşturma) hatası:?\s*/;

export function parseDiagnostics(raw: string): TypstDiagnostic[] {
  if (!raw) return [];

  const lines = raw.replace(HEADER, "").split("\n");
  const out: TypstDiagnostic[] = [];

  for (const rawLine of lines) {
    const text = rawLine.trim();
    if (!text) continue;

    if (text.startsWith("İpucu:")) {
      const hint = text.slice("İpucu:".length).trim();
      const last = out[out.length - 1];
      if (last) out[out.length - 1] = { ...last, hint };
      continue;
    }

    const match = text.match(LOCATION);
    if (match) {
      out.push({
        line: Number(match[1]),
        column: Number(match[2]),
        message: text.replace(LOCATION, "").trim(),
        hint: null,
      });
    } else {
      out.push({ line: null, column: null, message: text, hint: null });
    }
  }

  // Konumsuz tek satırlık gürültü kalırsa ham metni geri ver — hatayı
  // yutmaktansa ham hâlini göstermek yeğdir.
  if (out.length === 0) {
    return [{ line: null, column: null, message: raw.trim(), hint: null }];
  }

  return out;
}

export function errorText(err: unknown): string {
  if (typeof err === "string") return err;
  if (err instanceof Error) return err.message;
  return String(err);
}
