import type { Question } from "$lib/types";

/**
 * Soru kalıpları ve kaynaktan geri okuma.
 *
 * Tasarım kararı: sorunun yapısı — şıklar, doğru cevap, cevap alanı — ayrı bir
 * form panelinde değil, Typst kaynağının KENDİSİNDE durur. Öğretmen tek bir yere
 * bakar. Uygulama gereken alan modelini kaydederken kaynaktan geri okur.
 *
 * Bunun bedeli: kalıp çağrısı bozulursa kaydetme başarısız olur. Bilinçli bir
 * ödün — sessizce yanlış bir cevap anahtarı üretmektense açık bir hata iyidir,
 * çünkü yanlış cevap anahtarı yanlış not demektir.
 */

export type QuestionType = Question["question_type"];

export type Block = { label: string; hint: string; snippet: string };

/** Her soru tipinde işe yarayan genel parçalar. */
const COMMON: Block[] = [
  { label: "Matematik", hint: "$x^2 + y^2$ satır içi", snippet: "$  $" },
  { label: "Blok matematik", hint: "ortalanmış, kendi satırında", snippet: "\n$ \n$\n" },
  { label: "Kesir", hint: "a bölü b", snippet: "$ a/b $" },
  { label: "Kök", hint: "karekök", snippet: "$ sqrt(x) $" },
  { label: "Görsel", hint: "dosyadan resim", snippet: '#image("", width: 60%)' },
  { label: "Tablo", hint: "2 sütunlu", snippet: "#table(columns: 2, [], [])" },
  { label: "Kalın", hint: "*kalın*", snippet: "**" },
  { label: "Boşluk bırak", hint: "dikey aralık", snippet: "#v(0.5cm)" },
];

/** Soru tipine özel kalıplar. Doğru cevap kalıbın parametresinde taşınır. */
const BY_TYPE: Record<QuestionType, Block[]> = {
  multiple_choice: [
    {
      label: "Şıklar",
      hint: "5 şık, doğru cevap dogru: parametresinde",
      snippet:
        '\n#secenekler(dogru: "A",\n  [],\n  [],\n  [],\n  [],\n  [],\n)\n',
    },
  ],
  true_false: [
    {
      label: "Doğru / Yanlış",
      hint: "kutucuklar, doğru cevap dogru: parametresinde",
      snippet: "\n#dogru-yanlis(dogru: true)\n",
    },
  ],
  fill_in_blank: [
    {
      label: "Boşluk",
      hint: "cevap: içine kabul edilenleri | ile ayır",
      snippet: '#bosluk(cevap: "", width: 4cm)',
    },
  ],
  classic: [
    {
      label: "Cevap alanı",
      hint: "öğrencinin yazacağı çizgiler",
      snippet: "\n#cevap-alani(satir: 6)\n",
    },
  ],
};

export function blocksFor(type: QuestionType): { templates: Block[]; common: Block[] } {
  return { templates: BY_TYPE[type], common: COMMON };
}

// ── Kaynaktan geri okuma ──────────────────────────────────────────────────────

/**
 * `#ad(` çağrısının argüman metnini döndürür. Parantez ve köşeli parantez
 * derinliği sayılır, çünkü şıkların içinde de parantez olabilir.
 */
function callArguments(source: string, name: string): string | null {
  const marker = `#${name}(`;
  const start = source.indexOf(marker);
  if (start === -1) return null;

  let depth = 0;
  for (let i = start + marker.length - 1; i < source.length; i += 1) {
    const ch = source[i];
    if (ch === "(" || ch === "[") depth += 1;
    else if (ch === ")" || ch === "]") {
      depth -= 1;
      if (depth === 0) return source.slice(start + marker.length, i);
    }
  }
  return null;
}

/** Argüman metnindeki en üst düzey `[...]` bloklarını sırayla çıkarır. */
function bracketGroups(args: string): string[] {
  const groups: string[] = [];
  let depth = 0;
  let startIndex = -1;

  for (let i = 0; i < args.length; i += 1) {
    const ch = args[i];
    if (ch === "[") {
      if (depth === 0) startIndex = i + 1;
      depth += 1;
    } else if (ch === "]") {
      depth -= 1;
      if (depth === 0 && startIndex !== -1) {
        groups.push(args.slice(startIndex, i).trim());
        startIndex = -1;
      }
    }
  }
  return groups;
}

function namedString(args: string, key: string): string | null {
  const match = args.match(new RegExp(`${key}\\s*:\\s*"((?:[^"\\\\]|\\\\.)*)"`));
  return match ? match[1] : null;
}

function namedBool(args: string, key: string): boolean | null {
  const match = args.match(new RegExp(`${key}\\s*:\\s*(true|false)`));
  return match ? match[1] === "true" : null;
}

function namedNumber(args: string, key: string): number | null {
  const match = args.match(new RegExp(`${key}\\s*:\\s*(\\d+)`));
  return match ? Number(match[1]) : null;
}

export const OPTION_LETTERS = ["A", "B", "C", "D", "E", "F"];

export type ParsedOptions = {
  options: string[];
  correctIndex: number;
};

export function parseOptions(source: string): ParsedOptions | string {
  const args = callArguments(source, "secenekler");
  if (args === null) {
    return "Gövdede #secenekler(...) yok. Yukarıdaki Şıklar düğmesiyle ekle.";
  }

  const options = bracketGroups(args);
  if (options.length < 2) {
    return "En az iki şık gerekli.";
  }

  const letter = namedString(args, "dogru");
  if (letter === null) {
    return 'Doğru cevap belirtilmemiş. #secenekler(dogru: "A", ...) biçiminde yaz.';
  }

  const correctIndex = OPTION_LETTERS.indexOf(letter.trim().toUpperCase());
  if (correctIndex === -1 || correctIndex >= options.length) {
    return `Doğru cevap "${letter}" şıklarla eşleşmiyor.`;
  }

  if (options[correctIndex] === "") {
    return `Doğru işaretlenen ${letter} şıkkı boş.`;
  }

  return { options, correctIndex };
}

export function parseTrueFalse(source: string): boolean | string {
  const args = callArguments(source, "dogru-yanlis");
  if (args === null) {
    return "Gövdede #dogru-yanlis(...) yok. Yukarıdaki düğmeyle ekle.";
  }
  const value = namedBool(args, "dogru");
  if (value === null) {
    return "Doğru cevap belirtilmemiş. #dogru-yanlis(dogru: true) biçiminde yaz.";
  }
  return value;
}

export type ParsedBlank = { accepted: string[] };

export function parseBlanks(source: string): ParsedBlank[] | string {
  const blanks: ParsedBlank[] = [];
  let rest = source;

  while (true) {
    const args = callArguments(rest, "bosluk");
    if (args === null) break;

    const answer = namedString(args, "cevap");
    if (answer === null || answer.trim() === "") {
      return "Her #bosluk(...) için cevap: doldurulmalı.";
    }

    blanks.push({
      accepted: answer.split("|").map((a) => a.trim()).filter(Boolean),
    });

    const marker = "#bosluk(";
    rest = rest.slice(rest.indexOf(marker) + marker.length);
  }

  if (blanks.length === 0) {
    return "Gövdede #bosluk(...) yok. Yukarıdaki Boşluk düğmesiyle ekle.";
  }
  return blanks;
}

export function parseAnswerLines(source: string): number {
  const args = callArguments(source, "cevap-alani");
  if (args === null) return 6;
  return namedNumber(args, "satir") ?? 6;
}
