import { describe, expect, test } from "vitest";
import {
  classifyAnswer,
  histogram,
  itemStats,
  needsReview,
  skewLabel,
  spread,
} from "./item-stats";
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
    // Tek öğrencide sapma sıfır ve çarpıklık tanımsız; mod, 42'nin düştüğü
    // 40–50 aralığının orta noktasıdır.
    expect(spread([42])).toEqual({
      n: 1,
      mean: 42,
      median: 42,
      mode: 45,
      sd: 0,
      skewness: null,
      min: 42,
      max: 42,
      q1: 42,
      q3: 42,
    });
  });

  test("boş dizi null", () => {
    expect(spread([])).toBeNull();
  });

  test("çift sayıda değerde ortanca ara değerlenir", () => {
    expect(spread([10, 20, 30, 40])?.median).toBe(25);
  });
});

describe("histogram", () => {
  test("10 puanlık aralıklar, 0-100", () => {
    const b = histogram([5, 15, 15, 95]);
    expect(b).toHaveLength(10);
    expect(b[0].count).toBe(1);
    expect(b[1].count).toBe(2);
    expect(b[9].count).toBe(1);
  });

  test("TAM PUAN KAYBOLMAZ", () => {
    // 100 üst sınıra tam otururdu; son aralık kapalı tutulmasaydı hiçbir
    // aralığa düşmez ve tam puan alan öğrenci grafikten silinirdi.
    expect(histogram([100])[9].count).toBe(1);
  });

  test("aralık orta noktaları", () => {
    expect(histogram([])[0].mid).toBe(5);
    expect(histogram([])[9].mid).toBe(95);
  });

  test("sınır dışı değerler kırpılır", () => {
    expect(histogram([-10])[0].count).toBe(1);
    expect(histogram([140])[9].count).toBe(1);
  });
});

describe("spread — merkezî eğilim ve çarpıklık", () => {
  test("mod en kalabalık aralığın orta noktası", () => {
    // 12, 15, 18 hepsi 10-20 aralığında; 55 tek başına. Tepe belirgin.
    const s = spread([12, 15, 18, 55]);
    expect(s?.mode).toBe(15);
  });

  test("TEPE YOKSA MOD YOK", () => {
    // Altı öğrenci altı ayrı aralığa düştüğünde "en kalabalık aralık" diye
    // bir şey kalmıyor. Beraberlikte ilkini seçmek, en düşük puanı mod diye
    // göstermek olurdu: medyan 55 iken mod 15 yazılıyordu ve öğretmen
    // "mod < medyan < ortalama, sağa çarpık" diye okuyordu.
    const s = spread([15, 30, 45, 65, 70, 90]);
    expect(s?.mode).toBeNull();
    expect(s?.median).toBe(55);
  });

  test("tek değerde mod o aralıktır", () => {
    expect(spread([42])?.mode).toBe(45);
  });

  test("standart sapma örneklem (n-1)", () => {
    // Excel STDEV.S([10,20,30,40]) = 12.909944
    expect(spread([10, 20, 30, 40])?.sd).toBeCloseTo(12.909944, 4);
  });

  test("çarpıklık Excel SKEW ile aynı formülden gelir", () => {
    // Elle: ortalama 22, s = sqrt(1902.5) = 43.61766,
    // Σ((xi-x̄)/s)³ = 5.35737, G1 = 5/(4·3) · 5.35737 = 2.23240
    // Excel SKEW() de aynı örneklem düzeltmeli formülü kullanır; öğretmen
    // kendi tablosuyla karşılaştırdığında aynı sayıyı görmeli.
    expect(spread([1, 2, 3, 4, 100])?.skewness).toBeCloseTo(2.2324, 3);
  });

  test("negatif çarpıklık: yığılma yüksek puanlarda", () => {
    const s = spread([10, 80, 85, 90, 95]);
    expect(s?.skewness).toBeLessThan(0);
  });

  test("üçten az veride çarpıklık YOK", () => {
    // İki noktadan bir dağılımın yönü çıkarılamaz; sayı uydurulmaz.
    expect(spread([40, 60])?.skewness).toBeNull();
  });

  test("herkes aynı puanı aldıysa çarpıklık tanımsız", () => {
    // Sapma sıfır; bölme tanımsız olurdu.
    const s = spread([50, 50, 50, 50]);
    expect(s?.sd).toBe(0);
    expect(s?.skewness).toBeNull();
  });
});

describe("skewLabel", () => {
  test("yön ve şiddet söylenir", () => {
    expect(skewLabel(-1.4)).toContain("sola çarpık");
    expect(skewLabel(1.4)).toContain("sağa çarpık");
    expect(skewLabel(-0.7)).toContain("Orta düzey");
    expect(skewLabel(-1.4)).toContain("Belirgin");
  });

  test("küçük değer simetrik sayılır", () => {
    expect(skewLabel(0.2)).toContain("Simetrik");
  });

  test("hesaplanamayan durum gizlenmez", () => {
    expect(skewLabel(null)).toBe("Hesaplanamadı");
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
