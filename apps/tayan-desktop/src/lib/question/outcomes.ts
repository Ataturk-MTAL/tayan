import type { Question } from "$lib/types";

/**
 * Kazanım kodu — MEB biçimi: {ders}.{sınıf}.{ünite}.{kazanım}
 *
 * Bu kural Rust tarafındaki OutcomeCode::is_valid ile BİREBİR aynı olmak
 * zorunda. Ayrışırsa arayüz "geçerli" der, kaydetme reddedilir ve öğretmen
 * neyin yanlış olduğunu anlamaz.
 *
 * Ders kodu 1-5 harf; Türkçe harfler geçerli (FİZ, COĞ, TÜR). Kalan üç bölüm
 * 0-255 arası sayı — Rust tarafı u8 ayrıştırıyor, 256 orada sessizce reddedilir.
 */
export function isValidOutcome(code: string): boolean {
  const parts = code.split(".");
  if (parts.length !== 4) return false;

  const ders = parts[0];
  // [...ders].length: "FİZ" UTF-8'de 4 bayt ama 3 harftir.
  const harfSayisi = [...ders].length;
  if (harfSayisi < 1 || harfSayisi > 5) return false;
  // \p{L}: her dilde harf. \w Türkçe harfleri dışarıda bırakırdı.
  if (!/^\p{L}+$/u.test(ders)) return false;

  return parts.slice(1).every((p) => /^\d+$/.test(p) && Number(p) <= 255);
}

/** Serbest metni kazanım kodlarına ayırır, geçerli/geçersiz diye böler. */
export function splitOutcomes(text: string): { valid: string[]; invalid: string[] } {
  const parts = text
    .split(/[,\s]+/)
    .map((s) => s.trim())
    .filter(Boolean);

  return {
    valid: parts.filter(isValidOutcome),
    invalid: parts.filter((p) => !isValidOutcome(p)),
  };
}

/**
 * Ders adı → MEB ders kodu.
 *
 * Yaygın dersler için MEB'in kısaltması, diğerleri için ilk üç harf. Tahmin
 * yalnızca ÖNERİDİR: meslek derslerinin kodu okuldan okula değişir, doğrusunu
 * bilemem ve uydurduğumu dayatmam.
 */
const SUBJECT_CODES: Record<string, string> = {
  Matematik: "MAT",
  Fizik: "FİZ",
  Kimya: "KİM",
  Biyoloji: "BİY",
  "Türk Dili ve Edebiyatı": "TDE",
  Tarih: "TAR",
  "T.C. İnkılap Tarihi ve Atatürkçülük": "İNK",
  Coğrafya: "COĞ",
  İngilizce: "İNG",
  Felsefe: "FEL",
  "Din Kültürü ve Ahlak Bilgisi": "DKB",
  "Bilgisayar Bilimi": "BİL",
};

export function subjectCodeFor(subject: string): string {
  const trimmed = subject.trim();
  if (trimmed === "") return "";

  const known = SUBJECT_CODES[trimmed];
  if (known) return known;

  // İlk kelimenin ilk üç harfi. toLocaleUpperCase("tr") şart: "i" → "İ".
  const first = trimmed.split(/\s+/)[0];
  return [...first].slice(0, 3).join("").toLocaleUpperCase("tr");
}

/** Künyeden türeyen kod öneki — "MAT.9." gibi. Ders ya da seviye yoksa boş. */
export function outcomePrefix(subject: string, grade: number): string {
  const code = subjectCodeFor(subject);
  if (code === "" || !Number.isFinite(grade) || grade < 1 || grade > 12) return "";
  return `${code}.${grade}.`;
}

/**
 * Bankada AYNI ders ve seviyede daha önce kullanılmış kazanım kodları.
 *
 * Öğretmenin kendi kazanım listesi böyle birikir. MEB'in tam kazanım kataloğunu
 * uygulamaya gömmek ayrı ve büyük bir iş (her ders, her seviye, her yıl
 * güncelleniyor); gömülene kadar en doğru öneri kaynağı öğretmenin kendi
 * geçmişidir — uydurduğum bir liste değil.
 */
export function outcomeSuggestions(
  questions: Question[],
  subject: string,
  grade: number,
): string[] {
  const konu = subject.trim();
  if (konu === "") return [];

  const seen = new Set<string>();
  for (const q of questions) {
    if (q.meta?.subject?.trim() !== konu) continue;
    if (q.meta?.grade !== grade) continue;
    for (const o of q.outcomes) seen.add(o);
  }

  return [...seen].sort((a, b) => a.localeCompare(b, "tr"));
}
