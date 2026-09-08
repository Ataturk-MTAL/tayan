import type { Exam } from "$lib/types";

/**
 * Kaydetme diyaloğunun önerdiği dosya adı.
 *
 * BİÇİM:
 *   {sınıf}_{ders}_{başlık}_{YYYY-AA-GG}[_kitapcik-X][_cevap].{uzantı}
 *
 * Örnek:
 *   9-A_Matematik_1-Donem-1-Yazili_2026-11-12.pdf
 *   9-A_Matematik_1-Donem-1-Yazili_2026-11-12_kitapcik-B_cevap.pdf
 *
 * SIRALAMA BİLİNÇLİ. Sınıf ve ders başta olduğu için klasörde alfabetik
 * sıralama doğal gruplar üretir: bir sınıfın bütün kâğıtları yan yana durur.
 * Tarih sonda olduğu için aynı sınavın nüshaları da bitişik kalır.
 *
 * TÜRKÇE HARFLER ASCII'YE ÇEVRİLİR. macOS ve Linux Türkçe harfli adı sorunsuz
 * taşır, ama bu dosyalar e-postayla gönderiliyor, Windows'a kopyalanıyor ve
 * okul yazıcısına atılıyor. Oralarda "ı" ve "ğ" bozulabiliyor; bozuk ad
 * açılmayan dosya demek.
 */
const TR_ASCII: Record<string, string> = {
  "ç": "c",
  "Ç": "C",
  "ğ": "g",
  "Ğ": "G",
  "ı": "i",
  "İ": "I",
  "ö": "o",
  "Ö": "O",
  "ş": "s",
  "Ş": "S",
  "ü": "u",
  "Ü": "U",
};

function asciiye(s: string): string {
  return [...s].map((c) => TR_ASCII[c] ?? c).join("");
}

/**
 * Bir parçayı dosya adına uygun hâle getirir: ASCII'ye çevirir, harf/rakam
 * dışındaki her şeyi tek tireye indirir, baştaki ve sondaki tireleri atar.
 */
export function slugify(part: string): string {
  return asciiye(part)
    .replace(/[^A-Za-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

export type FilenameOptions = {
  answerKey: boolean;
  booklet: string | null;
  extension: string;
  /**
   * Sona eklenen ayırt edici ek — analiz raporu için "analiz".
   *
   * Aynı sınavın kâğıdı, cevap anahtarı ve analiz raporu aynı klasörde yan
   * yana durur; ek olmadan kâğıt ile rapor aynı adı isteyip birbirini ezerdi.
   */
  suffix?: string | null;
};

export function examFileName(exam: Exam, opts: FilenameOptions): string {
  const parcalar = [
    slugify(exam.meta.classroom),
    slugify(exam.meta.subject),
    slugify(exam.meta.title),
    // date zaten YYYY-AA-GG biçiminde; slugify tireleri koruyor.
    slugify(exam.meta.date),
  ].filter((p) => p !== "");

  // Bütün alanlar boş olsa bile kullanılabilir bir ad kalmalı.
  if (parcalar.length === 0) parcalar.push("sinav");

  if (opts.booklet) parcalar.push(`kitapcik-${slugify(opts.booklet)}`);
  // Cevap anahtarı EN SONDA: öğrenci nüshasıyla alfabetik olarak yan yana
  // dizilir ve ikisinin karıştırılması zorlaşır.
  if (opts.answerKey) parcalar.push("cevap");
  if (opts.suffix) parcalar.push(slugify(opts.suffix));

  return `${parcalar.join("_")}.${opts.extension}`;
}
