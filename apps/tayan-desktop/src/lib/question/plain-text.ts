/**
 * Typst kaynağını LİSTELERDE okunacak düz metne çevirir.
 *
 * NEDEN VAR: yeni editör gövdeyi tek bir `typst_raw` düğümü olarak saklıyor.
 * Önizleme kodu olduğu gibi basınca sonuç girişi ve analiz ekranlarında
 * "[typst] Aşağıdaki denklemin köklerini bulunuz. $ x^2 - 5x + 6 = 0 $"
 * gibi satırlar çıkıyordu. Öğretmen puanlarken hangi soruya baktığını
 * göremiyorsa ölçme ekranı işe yaramaz.
 *
 * BU BİR DERLEYİCİ DEĞİL. Gerçek dizgi Typst'in kendi işi; burada amaç tek
 * satırlık, kayıpsız olmayan, GÖRÜLEBİLİR bir özet. Anlamadığı şeyi sessizce
 * atar — çünkü listede yanlış bir şey göstermektense az şey göstermek yeğdir.
 */

const SUPERSCRIPT_DIGITS: Record<string, string> = {
  "0": "⁰", "1": "¹", "2": "²", "3": "³", "4": "⁴",
  "5": "⁵", "6": "⁶", "7": "⁷", "8": "⁸", "9": "⁹",
};

const SUBSCRIPT_DIGITS: Record<string, string> = {
  "0": "₀", "1": "₁", "2": "₂", "3": "₃", "4": "₄",
  "5": "₅", "6": "₆", "7": "₇", "8": "₈", "9": "₉",
};

/** Matematik kipindeki sık simgeler. Uzun olan önce gelmeli: `<=` `<`ten önce. */
const MATH_REPLACEMENTS: ReadonlyArray<readonly [RegExp, string]> = [
  [/\bthin\b|\bquad\b|\bmed\b/g, " "],
  [/\bdot\b/g, "·"],
  [/\btimes\b/g, "×"],
  [/\bdiv\b/g, "÷"],
  [/\bapprox\b/g, "≈"],
  [/\bOmega\b/g, "Ω"],
  [/\balpha\b/g, "α"],
  [/\bbeta\b/g, "β"],
  [/\bDelta\b/g, "Δ"],
  [/\bpi\b/g, "π"],
  [/<=/g, "≤"],
  [/>=/g, "≥"],
  [/!=/g, "≠"],
  [/->/g, "→"],
];

/** Yalnız sarmalayıcı olan matematik işlevleri: içeriği kalır, adı gider. */
const MATH_WRAPPERS = /\b(?:bold|italic|upright|text|display|inline|cal|frak)\s*\(/g;

function toSuperscript(digits: string): string {
  return [...digits].map((d) => SUPERSCRIPT_DIGITS[d] ?? d).join("");
}

function toSubscript(digits: string): string {
  return [...digits].map((d) => SUBSCRIPT_DIGITS[d] ?? d).join("");
}

/**
 * Dengeli parantez tarayıcı. `konum` açılış ayracını gösterir; kapanışın
 * indisini döndürür, kapanmıyorsa metnin sonunu.
 */
function findClosingBracket(source: string, start: number, open: string, close: string): number {
  let depth = 0;
  for (let i = start; i < source.length; i++) {
    const c = source[i];
    if (c === "\\") {
      i++;
      continue;
    }
    if (c === open) depth++;
    else if (c === close) {
      depth--;
      if (depth === 0) return i;
    }
  }
  return source.length;
}

/** `$...$` içinin düz metin karşılığı. */
function mathPlain(content: string): string {
  let out = content;

  // Sarmalayıcıları aç: bold(300) → 300. Parantezi de düşürmek gerekiyor;
  // eşleşen kapanışı bulmak için tek tek tara.
  for (;;) {
    MATH_WRAPPERS.lastIndex = 0;
    const m = MATH_WRAPPERS.exec(out);
    if (!m) break;
    const openIndex = m.index + m[0].length - 1;
    const closeIndex = findClosingBracket(out, openIndex, "(", ")");
    out =
      out.slice(0, m.index) +
      out.slice(openIndex + 1, closeIndex) +
      out.slice(closeIndex + 1);
  }

  // Matematik kipinde tırnak "metin kipi" demek; okurken tırnak görünmemeli.
  out = out.replace(/"/g, "");

  for (const [pattern, replacement] of MATH_REPLACEMENTS) {
    out = out.replace(pattern, replacement);
  }

  // Üst/alt simge: (182)_10 → (182)₁₀ , x^2 → x²
  out = out.replace(/\^\(([0-9]+)\)/g, (_, d: string) => toSuperscript(d));
  out = out.replace(/\^([0-9]+)/g, (_, d: string) => toSuperscript(d));
  out = out.replace(/_\(([0-9]+)\)/g, (_, d: string) => toSubscript(d));
  out = out.replace(/_([0-9]+)/g, (_, d: string) => toSubscript(d));

  return out;
}

/** İşlev/değişken adı karakterleri: `#cevap-alani`, `#text`, `#h`. */
function isNameChar(c: string): boolean {
  return /[A-Za-z0-9_-]/.test(c);
}

/**
 * Typst kaynağı → tek satır düz metin.
 *
 * KURALLAR
 *   `#ad(...)`        → tamamen düşer (çizim/boşluk: `#cevap-alani(satir: 6)`)
 *   `#ad(...)[içerik]` → yalnız içerik kalır (`#text(8pt)[Not]` → `Not`)
 *   `$ ... $`         → matematikDuz()
 *   `*`, `_`          → vurgu imleri, düşer
 *   `\x`              → x olduğu gibi
 *   satır ve blok yorumlar → düşer
 */
export function typstPlain(source: string): string {
  let out = "";
  let i = 0;

  while (i < source.length) {
    const c = source[i];

    // Yorumlar
    if (c === "/" && source[i + 1] === "/") {
      const end = source.indexOf("\n", i);
      i = end === -1 ? source.length : end;
      continue;
    }
    if (c === "/" && source[i + 1] === "*") {
      const end = source.indexOf("*/", i + 2);
      i = end === -1 ? source.length : end + 2;
      continue;
    }

    if (c === "\\") {
      if (i + 1 < source.length) out += source[i + 1];
      i += 2;
      continue;
    }

    if (c === "$") {
      const closeIndex = source.indexOf("$", i + 1);
      const end = closeIndex === -1 ? source.length : closeIndex;
      out += " " + mathPlain(source.slice(i + 1, end)) + " ";
      i = end + 1;
      continue;
    }

    if (c === "#" && i + 1 < source.length && isNameChar(source[i + 1])) {
      let j = i + 1;
      while (j < source.length && isNameChar(source[j])) j++;

      if (source[j] === "(") {
        j = findClosingBracket(source, j, "(", ")") + 1;
      }

      if (source[j] === "[") {
        const closeIndex = findClosingBracket(source, j, "[", "]");
        // İçerik yine Typst; özyinele ki `#text(8pt)[$x^2$]` de çözülsün.
        out += typstPlain(source.slice(j + 1, closeIndex));
        i = closeIndex + 1;
      } else {
        // Gövdesiz çağrı: çizim, boşluk, ayraç. Okunacak bir şey yok.
        i = j;
      }
      continue;
    }

    // Vurgu imleri. Matematik `$` içinde ele alındığı için burada güvenli.
    if (c === "*" || c === "_") {
      i++;
      continue;
    }

    out += c;
    i++;
  }

  return out.replace(/\s+/g, " ").trim();
}
