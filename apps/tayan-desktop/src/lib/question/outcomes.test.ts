import { describe, expect, test } from "vitest";
import {
  isValidOutcome,
  outcomePrefix,
  outcomeSuggestions,
  splitOutcomes,
  subjectCodeFor,
} from "./outcomes";
import type { Question } from "$lib/types";

/**
 * Bu kural Rust tarafındaki OutcomeCode::is_valid ile İKİZ. Ayrışırsa arayüz
 * "geçerli" der, kaydetme reddedilir ve öğretmen sebebi anlamaz.
 *
 * Rust tarafında da aynı vakalar test ediliyor (learning_outcome.rs). İki
 * testin aynı beklentiyi tutması, ikizin ayrışmasını yakalar.
 */
describe("isValidOutcome", () => {
  test("arayüzün önerdiği kod kabul edilir", () => {
    // Bir zamanlar REDDEDİLİYORDU: doğrulayıcı tek harfli ders kodu istiyordu,
    // oysa yer tutucu ve yardım sayfası bu kodu örnek gösteriyordu.
    expect(isValidOutcome("MAT.9.1.2")).toBe(true);
  });

  test("Türkçe harfli ders kodu kabul edilir", () => {
    expect(isValidOutcome("FİZ.10.1.1")).toBe(true);
    expect(isValidOutcome("COĞ.11.2.3")).toBe(true);
  });

  test("tek harfli eski biçim hâlâ geçerli", () => {
    expect(isValidOutcome("M.7.2.3")).toBe(true);
  });

  test("biçimsiz kodlar reddedilir", () => {
    expect(isValidOutcome("MAT.9.1")).toBe(false);
    expect(isValidOutcome("MAT.9.1.2.3")).toBe(false);
    expect(isValidOutcome("9.9.1.2")).toBe(false);
    expect(isValidOutcome("MATEMA.9.1.2")).toBe(false);
    expect(isValidOutcome("MAT.a.1.2")).toBe(false);
    expect(isValidOutcome("")).toBe(false);
  });

  test("Rust u8 sınırı: 255 geçer, 256 geçmez", () => {
    expect(isValidOutcome("MAT.9.1.255")).toBe(true);
    expect(isValidOutcome("MAT.9.1.256")).toBe(false);
  });
});

describe("splitOutcomes", () => {
  test("boşluk ve virgülle ayrılır", () => {
    const r = splitOutcomes("MAT.9.1.2, MAT.9.1.3  MAT.9.2.1");
    expect(r.valid).toEqual(["MAT.9.1.2", "MAT.9.1.3", "MAT.9.2.1"]);
    expect(r.invalid).toEqual([]);
  });

  test("geçersizler ayrı toplanır", () => {
    const r = splitOutcomes("MAT.9.1.2 bozuk MAT.9.1");
    expect(r.valid).toEqual(["MAT.9.1.2"]);
    expect(r.invalid).toEqual(["bozuk", "MAT.9.1"]);
  });

  test("boş metin boş sonuç verir", () => {
    expect(splitOutcomes("   ")).toEqual({ valid: [], invalid: [] });
  });
});

describe("subjectCodeFor", () => {
  test("bilinen ders MEB kısaltmasını alır", () => {
    expect(subjectCodeFor("Matematik")).toBe("MAT");
    expect(subjectCodeFor("Fizik")).toBe("FİZ");
  });

  test("bilinmeyen ders ilk üç harfe düşer", () => {
    expect(subjectCodeFor("Sayısal Elektronik")).toBe("SAY");
  });

  test("Türkçe büyütme: i harfi İ olur, I olmaz", () => {
    // toUpperCase() "i" harfini "I" yapar ve Türkçede yanlıştır.
    expect(subjectCodeFor("içerik")).toBe("İÇE");
  });

  test("boş ders boş kod verir", () => {
    expect(subjectCodeFor("   ")).toBe("");
  });
});

describe("outcomePrefix", () => {
  test("ders ve seviyeden önek türer", () => {
    expect(outcomePrefix("Matematik", 9)).toBe("MAT.9.");
  });

  test("eksik ya da geçersiz künyede önek yok", () => {
    expect(outcomePrefix("", 9)).toBe("");
    expect(outcomePrefix("Matematik", 0)).toBe("");
    expect(outcomePrefix("Matematik", 13)).toBe("");
  });
});

/** Test verisi: yalnız bu testlerin okuduğu alanlar anlamlı doldurulmuş. */
function soru(subject: string, grade: number, outcomes: string[]): Question {
  return {
    question_type: "classic",
    meta: { subject, grade, difficulty: null, title: "" },
    id: `${subject}-${grade}-${outcomes.join("_")}`,
    points: 5,
    outcomes,
    body: [],
    sample_answer: null,
    rubric: [],
    answer_space: { Lines: 6 },
    stats: {
      times_used: 0,
      total_responses: 0,
      correct_responses: 0,
      difficulty_index: 0,
      discrimination_index: 0,
      avg_points_earned: 0,
      last_used_at: null,
      performance_score: 0,
    },
  };
}

describe("outcomeSuggestions", () => {
  const bank = [
    soru("Matematik", 9, ["MAT.9.1.2", "MAT.9.1.3"]),
    soru("Matematik", 10, ["MAT.10.1.1"]),
    soru("Fizik", 9, ["FİZ.9.1.1"]),
  ];

  test("yalnız aynı ders VE aynı seviyeden öneri gelir", () => {
    expect(outcomeSuggestions(bank, "Matematik", 9)).toEqual(["MAT.9.1.2", "MAT.9.1.3"]);
  });

  test("başka seviye sızmaz", () => {
    expect(outcomeSuggestions(bank, "Matematik", 10)).toEqual(["MAT.10.1.1"]);
  });

  test("eşleşme yoksa boş döner", () => {
    expect(outcomeSuggestions(bank, "Kimya", 9)).toEqual([]);
    expect(outcomeSuggestions(bank, "", 9)).toEqual([]);
  });

  test("aynı kazanım iki soruda geçse bir kez listelenir", () => {
    const iki = [...bank, soru("Matematik", 9, ["MAT.9.1.2"])];
    expect(outcomeSuggestions(iki, "Matematik", 9)).toEqual(["MAT.9.1.2", "MAT.9.1.3"]);
  });
});
