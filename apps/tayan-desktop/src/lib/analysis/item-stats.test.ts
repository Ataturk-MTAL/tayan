import { describe, expect, test } from "vitest";
import { classifyAnswer, itemStats, needsReview, spread } from "./item-stats";
import type { ExamResult, Question } from "$lib/types";

function cevap(qid: string, puan: number, dogru: boolean | null) {
  return {
    question_id: qid,
    given_answer: null,
    points_earned: puan,
    is_correct: dogru,
    rubric_met: [],
  };
}

function sonuc(id: string, toplam: number, cevaplar: ReturnType<typeof cevap>[]): ExamResult {
  return {
    id,
    exam_id: "e1",
    student_id: id,
    answers: cevaplar,
    total_points_earned: toplam,
    total_points_max: 20,
    outcome_performance: [],
    classroom_rank: null,
    recorded_at: "2026-01-01T00:00:00Z",
    is_complete: true,
  } as unknown as ExamResult;
}

const SORU: Question = {
  question_type: "true_false",
  meta: { subject: "Matematik", grade: 9, difficulty: null, title: "" },
  id: "q1",
  points: 10,
  outcomes: [],
  body: [],
  correct_answer: true,
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
} as unknown as Question;

describe("classifyAnswer", () => {
  test("cevap yoksa boş", () => {
    expect(classifyAnswer(undefined)).toBe("blank");
  });

  test("doğru ve yanlış olduğu gibi", () => {
    expect(classifyAnswer(cevap("q1", 10, true))).toBe("correct");
    expect(classifyAnswer(cevap("q1", 0, false))).toBe("wrong");
  });

  test("klasik soruda puan varsa kısmi", () => {
    expect(classifyAnswer(cevap("q1", 6, null))).toBe("partial");
  });

  test("PUANSIZ KLASİK CEVAP 'boş' DEĞİL", () => {
    // Veri, cevabın hiç yazılmadığını söylemiyor: is_correct yok, given_answer
    // zaten null. "cevapsız" demek olmayan bir bilgiyi iddia etmek olurdu.
    expect(classifyAnswer(cevap("q1", 0, null))).toBe("wrong");
  });
});

describe("itemStats", () => {
  const sonuclar = [
    sonuc("s1", 10, [cevap("q1", 10, true)]),
    sonuc("s2", 0, [cevap("q1", 0, false)]),
  ];

  test("sayımlar ve güçlük", () => {
    const [i] = itemStats(sonuclar, ["q1"], [SORU]);
    expect(i.correct).toBe(1);
    expect(i.wrong).toBe(1);
    expect(i.difficulty).toBeCloseTo(0.5);
    expect(i.maxPoints).toBe(10);
  });

  test("küçük sınıfta ayırt edicilik null — sayı uydurulmaz", () => {
    // Altı kişilik sınıfta üst dilim tek öğrenci; çıkan sayı o öğrencinin o
    // günkü hâlini ölçer, sorunun niteliğini değil.
    const [i] = itemStats(sonuclar, ["q1"], [SORU]);
    expect(i.discrimination).toBeNull();
  });

  test("yeterli sınıfta ayırt edicilik hesaplanır", () => {
    // 10 öğrenci: üst %27 = 2 kişi, alt %27 = 2 kişi.
    const on = Array.from({ length: 10 }, (_, k) =>
      sonuc(`s${k}`, k < 5 ? 10 : 0, [cevap("q1", k < 5 ? 10 : 0, k < 5)]),
    );
    const [i] = itemStats(on, ["q1"], [SORU]);
    expect(i.discrimination).toBeCloseTo(1);
  });

  test("bankada olmayan soru çökmez", () => {
    const [i] = itemStats(sonuclar, ["yok"], [SORU]);
    expect(i.maxPoints).toBe(0);
    expect(i.difficulty).toBe(0);
  });

  test("kaynak dizi sıralanmaz", () => {
    const kopya = [...sonuclar];
    itemStats(sonuclar, ["q1"], [SORU]);
    expect(sonuclar).toEqual(kopya);
  });
});

describe("spread", () => {
  test("temel ölçüler", () => {
    const s = spread([10, 20, 30, 40, 50]);
    expect(s).not.toBeNull();
    if (!s) return;
    expect(s.n).toBe(5);
    expect(s.mean).toBe(30);
    expect(s.median).toBe(30);
    expect(s.min).toBe(10);
    expect(s.max).toBe(50);
    expect(s.q1).toBe(20);
    expect(s.q3).toBe(40);
  });

  test("tek değer", () => {
    expect(spread([42])).toEqual({
      n: 1, mean: 42, median: 42, min: 42, max: 42, q1: 42, q3: 42,
    });
  });

  test("boş dizi null", () => {
    expect(spread([])).toBeNull();
  });

  test("çift sayıda değerde ortanca ara değerlenir", () => {
    expect(spread([10, 20, 30, 40])?.median).toBe(25);
  });
});

describe("needsReview", () => {
  const temel = {
    questionId: "q1", order: 1, correct: 5, partial: 0, wrong: 5,
    blank: 0, answered: 10, difficulty: 0.5, discrimination: 0.4, maxPoints: 10,
  };

  test("iyi madde işaretlenmez", () => {
    expect(needsReview(temel)).toBeNull();
  });

  test("çok zor, çok kolay ve ayırt etmeyen", () => {
    expect(needsReview({ ...temel, difficulty: 0.1 })).toContain("zor");
    expect(needsReview({ ...temel, difficulty: 0.95 })).toContain("kolay");
    expect(needsReview({ ...temel, discrimination: 0.1 })).toContain("Ayırt");
  });

  test("ayırt edicilik yoksa o ölçüt uygulanmaz", () => {
    // null "kötü" demek değil, "bilmiyoruz" demek.
    expect(needsReview({ ...temel, discrimination: null })).toBeNull();
  });

  test("hiç cevaplanmamış madde", () => {
    expect(needsReview({ ...temel, answered: 0 })).toContain("Kimse");
  });
});
