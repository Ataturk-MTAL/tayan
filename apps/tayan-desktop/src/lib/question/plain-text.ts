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

const UST: Record<string, string> = {
  "0": "⁰", "1": "¹", "2": "²", "3": "³", "4": "⁴",
  "5": "⁵", "6": "⁶", "7": "⁷", "8": "⁸", "9": "⁹",
};

const ALT: Record<string, string> = {
  "0": "₀", "1": "₁", "2": "₂", "3": "₃", "4": "₄",
  "5": "₅", "6": "₆", "7": "₇", "8": "₈", "9": "₉",
};

/** Matematik kipindeki sık simgeler. Uzun olan önce gelmeli: `<=` `<`ten önce. */
const MATEMATIK_SOZLUK: ReadonlyArray<readonly [RegExp, string]> = [
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
const SEFFAF_MATEMATIK = /\b(?:bold|italic|upright|text|display|inline|cal|frak)\s*\(/g;

function ustSimge(rakamlar: string): string {
  return [...rakamlar].map((d) => UST[d] ?? d).join("");
}

function altSimge(rakamlar: string): string {
  return [...rakamlar].map((d) => ALT[d] ?? d).join("");
}

/**
 * Dengeli parantez tarayıcı. `konum` açılış ayracını gösterir; kapanışın
 * indisini döndürür, kapanmıyorsa metnin sonunu.
 */
function ayraciKapat(kaynak: string, konum: number, ac: string, kapa: string): number {
  let derinlik = 0;
  for (let i = konum; i < kaynak.length; i++) {
    const c = kaynak[i];
    if (c === "\\") {
      i++;
      continue;
    }
    if (c === ac) derinlik++;
    else if (c === kapa) {
      derinlik--;
      if (derinlik === 0) return i;
    }
  }
  return kaynak.length;
}

/** `$...$` içinin düz metin karşılığı. */
function matematikDuz(icerik: string): string {
  let out = icerik;

  // Sarmalayıcıları aç: bold(300) → 300. Parantezi de düşürmek gerekiyor;
  // eşleşen kapanışı bulmak için tek tek tara.
  for (;;) {
    SEFFAF_MATEMATIK.lastIndex = 0;
    const m = SEFFAF_MATEMATIK.exec(out);
    if (!m) break;
    const acilis = m.index + m[0].length - 1;
    const kapanis = ayraciKapat(out, acilis, "(", ")");
    out =
      out.slice(0, m.index) +
      out.slice(acilis + 1, kapanis) +
      out.slice(kapanis + 1);
  }

  // Matematik kipinde tırnak "metin kipi" demek; okurken tırnak görünmemeli.
  out = out.replace(/"/g, "");

  for (const [kalip, karsilik] of MATEMATIK_SOZLUK) {
    out = out.replace(kalip, karsilik);
  }

  // Üst/alt simge: (182)_10 → (182)₁₀ , x^2 → x²
  out = out.replace(/\^\(([0-9]+)\)/g, (_, d: string) => ustSimge(d));
  out = out.replace(/\^([0-9]+)/g, (_, d: string) => ustSimge(d));
  out = out.replace(/_\(([0-9]+)\)/g, (_, d: string) => altSimge(d));
  out = out.replace(/_([0-9]+)/g, (_, d: string) => altSimge(d));

  return out;
}

/** İşlev/değişken adı karakterleri: `#cevap-alani`, `#text`, `#h`. */
function adKarakteri(c: string): boolean {
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
export function typstPlain(kaynak: string): string {
  let out = "";
  let i = 0;

  while (i < kaynak.length) {
    const c = kaynak[i];

    // Yorumlar
    if (c === "/" && kaynak[i + 1] === "/") {
      const son = kaynak.indexOf("\n", i);
      i = son === -1 ? kaynak.length : son;
      continue;
    }
    if (c === "/" && kaynak[i + 1] === "*") {
      const son = kaynak.indexOf("*/", i + 2);
      i = son === -1 ? kaynak.length : son + 2;
      continue;
    }

    if (c === "\\") {
      if (i + 1 < kaynak.length) out += kaynak[i + 1];
      i += 2;
      continue;
    }

    if (c === "$") {
      const kapanis = kaynak.indexOf("$", i + 1);
      const son = kapanis === -1 ? kaynak.length : kapanis;
      out += " " + matematikDuz(kaynak.slice(i + 1, son)) + " ";
      i = son + 1;
      continue;
    }

    if (c === "#" && i + 1 < kaynak.length && adKarakteri(kaynak[i + 1])) {
      let j = i + 1;
      while (j < kaynak.length && adKarakteri(kaynak[j])) j++;

      if (kaynak[j] === "(") {
        j = ayraciKapat(kaynak, j, "(", ")") + 1;
      }

      if (kaynak[j] === "[") {
        const kapanis = ayraciKapat(kaynak, j, "[", "]");
        // İçerik yine Typst; özyinele ki `#text(8pt)[$x^2$]` de çözülsün.
        out += typstPlain(kaynak.slice(j + 1, kapanis));
        i = kapanis + 1;
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
