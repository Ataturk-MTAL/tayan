import type { ExamResult, Question } from "$lib/types";

/**
 * Sınav analizinin hesapları. Çizim yok, karar var.
 *
 * NEDEN AYRI: aynı sayılar hem ekranda hem PDF raporunda görünecek. İki yerde
 * ayrı hesaplamak, kâğıtla ekranın ayrışması demekti — öğretmen veliye
 * gösterdiği rapordan başka bir şey görürdü.
 *
 * BU SINAVIN VERİSİ. Bankadaki QuestionStats bütün sınavların toplamı; buradaki
 * hesaplar yalnız seçili sınav ve sınıf için. İkisi farklı sorulara cevap verir.
 */

export type CellState = "correct" | "wrong" | "partial" | "blank";

/** Ayırt ediciliğin anlamlı sayılabildiği en küçük sınıf. */
export const MIN_DISCRIMINATION_N = 10;

/** Üst ve alt dilim oranı — madde analizinde yerleşik değer. */
const UPPER_LOWER_FRACTION = 0.27;

type Answer = ExamResult["answers"][number];

/**
 * Bir cevabın durumu.
 *
 * KLASİK SORUDA "boş" DİYEMİYORUZ. `is_correct` yok ve `given_answer` her
 * hâlükârda null; 0 puan almış bir cevabın hiç yazılmadığı mı yoksa yanlış mı
 * olduğu veriden ANLAŞILMIYOR. "cevapsız" demek, olmayan bir bilgiyi iddia
 * etmek olurdu; bu yüzden puansız klasik cevap "wrong" sayılıyor ve arayüz
 * bunu "puan yok" diye adlandırıyor.
 */
export function classifyAnswer(answer: Answer | undefined): CellState {
  if (!answer) return "blank";
  if (answer.is_correct === true) return "correct";
  if (answer.is_correct === false) return "wrong";
  return answer.points_earned > 0 ? "partial" : "wrong";
}

export type ItemStat = {
  questionId: string;
  /** Sınav kâğıdındaki sıra, 1'den başlar. */
  order: number;
  correct: number;
  partial: number;
  wrong: number;
  blank: number;
  answered: number;
  /** Güçlük (p): kazanılan puanın alınabilecek puana oranı ∈ [0, 1]. */
  difficulty: number;
  /**
   * Ayırt edicilik (D): üst %27 ile alt %27 arasındaki güçlük farkı.
   *
   * null = güvenilir hesaplanamıyor. Altı kişilik bir sınıfta üst dilim tek
   * öğrenci demek; çıkan sayı o öğrencinin o gün nasıl olduğunu ölçer, sorunun
   * niteliğini değil. Sayı uydurmaktansa yokluğunu söylemek gerekiyor.
   */
  discrimination: number | null;
  maxPoints: number;
};

function answerOf(result: ExamResult, questionId: string): Answer | undefined {
  return result.answers.find((a) => a.question_id === questionId);
}

function maxPointsOf(q: Question | undefined): number {
  if (!q) return 0;
  if (q.question_type === "fill_in_blank") {
    return q.blanks.reduce((sum, b) => sum + b.points, 0);
  }
  return q.points;
}

/** Bir dilimin ortalama güçlüğü: kazanılan / alınabilecek. */
function sliceDifficulty(
  results: ExamResult[],
  questionId: string,
  maxPoints: number,
): number {
  if (results.length === 0 || maxPoints === 0) return 0;
  const kazanilan = results.reduce(
    (sum, r) => sum + (answerOf(r, questionId)?.points_earned ?? 0),
    0,
  );
  return kazanilan / (results.length * maxPoints);
}

export function itemStats(
  results: ExamResult[],
  questionIds: string[],
  bank: Question[],
): ItemStat[] {
  // Ayırt edicilik için toplam puana göre sıralı kopya. Kaynak dizi
  // DEĞİŞTİRİLMEZ: çağıran taraf kendi sırasını kaybetmemeli.
  const sirali = [...results].sort(
    (a, b) => b.total_points_earned - a.total_points_earned,
  );
  const dilim = Math.floor(sirali.length * UPPER_LOWER_FRACTION);
  const yeterli = results.length >= MIN_DISCRIMINATION_N && dilim > 0;
  const ust = sirali.slice(0, dilim);
  const alt = sirali.slice(sirali.length - dilim);

  return questionIds.map((qid, i) => {
    const q = bank.find((b) => b.id === qid);
    const maxPoints = maxPointsOf(q);

    let correct = 0;
    let partial = 0;
    let wrong = 0;
    let blank = 0;

    for (const r of results) {
      switch (classifyAnswer(answerOf(r, qid))) {
        case "correct":
          correct += 1;
          break;
        case "partial":
          partial += 1;
          break;
        case "wrong":
          wrong += 1;
          break;
        default:
          blank += 1;
      }
    }

    return {
      questionId: qid,
      order: i + 1,
      correct,
      partial,
      wrong,
      blank,
      answered: results.length - blank,
      difficulty: sliceDifficulty(results, qid, maxPoints),
      discrimination: yeterli
        ? sliceDifficulty(ust, qid, maxPoints) - sliceDifficulty(alt, qid, maxPoints)
        : null,
      maxPoints,
    };
  });
}

export type Spread = {
  n: number;
  mean: number;
  median: number;
  min: number;
  max: number;
  /** Alt ve üst çeyrek: kutunun kenarları. */
  q1: number;
  q3: number;
};

/** Sıralı dizide oransal konum — doğrusal ara değerleme. */
function quantile(sorted: number[], p: number): number {
  if (sorted.length === 0) return 0;
  if (sorted.length === 1) return sorted[0];
  const pos = (sorted.length - 1) * p;
  const alt = Math.floor(pos);
  const kalan = pos - alt;
  const ustDeger = sorted[alt + 1] ?? sorted[alt];
  return sorted[alt] + kalan * (ustDeger - sorted[alt]);
}

export function spread(values: number[]): Spread | null {
  if (values.length === 0) return null;
  const s = [...values].sort((a, b) => a - b);
  return {
    n: s.length,
    mean: s.reduce((a, b) => a + b, 0) / s.length,
    median: quantile(s, 0.5),
    min: s[0],
    max: s[s.length - 1],
    q1: quantile(s, 0.25),
    q3: quantile(s, 0.75),
  };
}

/**
 * Gözden geçirilmesi gereken maddeler.
 *
 * Ölçütler madde analizinin yerleşik eşikleri: güçlük 0.20'nin altı ya da
 * 0.90'ın üstü (soru ya kimseye ya herkese göre), ayırt edicilik 0.20'nin
 * altı (iyi ve zayıf öğrenciyi ayırmıyor).
 */
export function needsReview(item: ItemStat): string | null {
  if (item.answered === 0) return "Kimse cevaplamamış";
  if (item.difficulty < 0.2) return "Çok zor — kimse yapamamış";
  if (item.difficulty > 0.9) return "Çok kolay — herkes yapmış";
  if (item.discrimination !== null && item.discrimination < 0.2) {
    return "Ayırt etmiyor";
  }
  return null;
}
