import type { RubricItem } from "$lib/types";

/**
 * Soru gövdesindeki `#rubrik((...))` bloğunu panele taşınabilir veriye çevirir.
 *
 * NE OLDUĞU KADAR NE OLMADIĞI DA ÖNEMLİ.
 *
 * Bu bir Typst çözümleyicisi DEĞİL ve asla not vermez. Tek işi, öğretmenin
 * elindeki eski cevap anahtarı dosyasından yapıştırdığı ölçütleri BİR KEZ,
 * gözünün önünde, panele taşımak. Taşındıktan sonra hüküm panelin ve
 * ClassicQuestion::validate()'in.
 *
 * Bu ayrım güvenliğin kendisi: kutucuklar not veriyor. Kutucukların kaynağı
 * bir metin çözümleyicisi olsaydı, yarım yazılmış bir makro yüzünden verilen
 * not yanlış olurdu. Burada en kötü ihtimalle "ayrıştıramadım" denir ve
 * öğretmen ölçütleri elle girer.
 *
 * YALNIZ DÜZ-DEĞİŞMEZ ALT KÜME kabul edilir:
 *
 *     #rubrik((
 *       ([ölçüt metni], 6),
 *       ([başka ölçüt], 4),
 *     ))
 *
 * Değişken (`#rubrik(olcutler)`), koşul, döngü ve hesaplanmış puan KABUL
 * EDİLMEZ. Typst tam bir dil; bunları doğru okumak derlemek demektir ve
 * derlemenin ürettiği şey kaynakta yazandan başka olabilir. Sınırın burada
 * olması bilinçli.
 */

export type RubricImport =
  | { ok: true; items: RubricItem[]; from: number; to: number }
  | { ok: false; reason: string; from: number; to: number };

const RUBRIC_CALL = "#rubrik(";

/**
 * Dengeli ayraç tarayıcı. `konum` açılışı gösterir; kapanışın indisini döndürür.
 * Kapanmıyorsa -1 — yarım yazılmış makro sessizce kabul edilmemeli.
 */
function findClosingBracket(source: string, start: number): number {
  let depth = 0;
  let inString: '"' | null = null;

  for (let i = start; i < source.length; i++) {
    const c = source[i];

    if (c === "\\") {
      i++;
      continue;
    }
    if (inString) {
      if (c === inString) inString = null;
      continue;
    }
    if (c === '"') {
      inString = '"';
      continue;
    }
    if (c === "(" || c === "[") depth++;
    else if (c === ")" || c === "]") {
      depth--;
      if (depth === 0) return i;
      if (depth < 0) return -1;
    }
  }
  return -1;
}

/** Gövdede `#rubrik(` var mı? Kaydetme kapısı ve uyarı şeridi bunu sorar. */
export function hasRubricCall(source: string): boolean {
  return source.includes(RUBRIC_CALL);
}

/**
 * `([içerik], puan)` demetlerini ayıklar.
 *
 * İçerik ham bırakılır: matematik ve biçimlendirme ölçüt metninin parçası.
 */
function readTuples(body: string): RubricItem[] | string {
  const items: RubricItem[] = [];
  let i = 0;

  while (i < body.length) {
    const c = body[i];
    if (c === "," || /\s/.test(c)) {
      i++;
      continue;
    }
    if (c !== "(") {
      return `Beklenmeyen karakter: "${c}". Yalnız ([ölçüt], puan) demetleri okunabiliyor.`;
    }

    const closeIndex = findClosingBracket(body, i);
    if (closeIndex === -1) return "Ayraçlar kapanmamış.";

    const tuple = body.slice(i + 1, closeIndex);
    i = closeIndex + 1;

    // İçerik bloğu ile puanı ayır: [..] , sayı
    const contentStart = tuple.indexOf("[");
    if (contentStart === -1) return "Ölçüt metni [ ] içinde değil.";

    const contentEnd = findClosingBracket(tuple, contentStart);
    if (contentEnd === -1) return "Ölçüt metninin köşeli parantezi kapanmamış.";

    const criterion = tuple.slice(contentStart + 1, contentEnd).trim();
    if (criterion === "") return "Boş ölçüt metni.";

    const rest = tuple.slice(contentEnd + 1).replace(/^\s*,\s*/, "").trim();
    // Hesaplanmış puan (5 + 5, degisken) KABUL EDİLMEZ: doğru okuduğumuzu
    // ancak tam sayıda garanti edebiliriz.
    if (!/^\d+$/.test(rest)) {
      return `Puan tam sayı olmalı, okunan: "${rest}".`;
    }

    items.push({ criterion, points: Number(rest) });
  }

  return items.length > 0 ? items : "Hiç ölçüt bulunamadı.";
}

/**
 * Gövdedeki İLK `#rubrik(...)` bloğunu okur.
 *
 * Blok yoksa null döner — bu bir hata değil, olağan durum.
 */
export function importRubric(source: string): RubricImport | null {
  const callStart = source.indexOf(RUBRIC_CALL);
  if (callStart === -1) return null;

  const openIndex = callStart + RUBRIC_CALL.length - 1;
  const closeIndex = findClosingBracket(source, openIndex);
  if (closeIndex === -1) {
    return {
      ok: false,
      reason: "#rubrik( ayracı kapanmamış.",
      from: callStart,
      to: source.length,
    };
  }

  const to = closeIndex + 1;
  let inner = source.slice(openIndex + 1, closeIndex).trim();

  // `goster: true` bizim ürettiğimiz çağrıda var; içe aktarırken anlamı yok.
  inner = inner.replace(/,\s*goster\s*:\s*(true|false)\s*$/, "").trim();

  // Dış demet parantezi: ((..), (..)) → (..), (..)
  if (inner.startsWith("(") && findClosingBracket(inner, 0) === inner.length - 1) {
    inner = inner.slice(1, -1);
  }

  const result = readTuples(inner);
  if (typeof result === "string") {
    return { ok: false, reason: result, from: callStart, to };
  }
  return { ok: true, items: result, from: callStart, to };
}

/** Bloğu gövdeden çıkarır. Taşıma tek yönlü: panel artık sahibi. */
export function removeRange(source: string, from: number, to: number): string {
  return (source.slice(0, from) + source.slice(to)).replace(/\n{3,}/g, "\n\n").trim();
}
