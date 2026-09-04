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
  /**
   * Mod: en çok öğrencinin düştüğü puan aralığının orta noktası.
   *
   * null = TEPE NOKTASI YOK. Altı öğrencinin altısı ayrı aralığa düşerse her
   * aralıkta bir kişi olur ve "en kalabalık aralık" diye bir şey kalmaz;
   * beraberlikte ilkini seçmek, en düşük puanı mod diye göstermek olurdu.
   * Medyan 55 iken mod 15 yazmak öğretmene "mod < medyan < ortalama, sağa
   * çarpık" dedirtir — oysa çarpıklık −0.06, dağılım simetrik.
   */
  mode: number | null;
  /** Standart sapma (örneklem, n-1). */
  sd: number;
  /**
   * Çarpıklık katsayısı (Fisher–Pearson, örneklem düzeltmeli).
   *
   * Excel'in SKEW() işleviyle aynı hesap; öğretmen kendi tablosuyla
   * karşılaştırdığında aynı sayıyı görmeli. n < 3 ya da sapma sıfırsa
   * tanımsız (null).
   *
   * NEGATİF = sola çarpık: kuyruk düşük puanlarda, yığılma yüksekte —
   * sınıf başarılı. POZİTİF = sağa çarpık: yığılma düşük puanlarda.
   */
  skewness: number | null;
  min: number;
  max: number;
  /** Alt ve üst çeyrek: kutunun kenarları. */
  q1: number;
  q3: number;
};

/** Frekans dağılımındaki puan aralığı genişliği. */
export const BIN_WIDTH = 10;

export type Bin = { from: number; to: number; count: number; mid: number };

/**
 * Frekans dağılımı: 0-100 arası, varsayılan 10 puanlık aralıklar.
 *
 * SON ARALIK KAPALI. 100 alan öğrenci aksi hâlde hiçbir aralığa düşmez ve
 * grafikten kaybolurdu; tam puan alan öğrenciyi yok saymak kabul edilemez.
 */
export function histogram(values: number[], width = BIN_WIDTH): Bin[] {
  const sayi = Math.ceil(100 / width);
  const bins: Bin[] = Array.from({ length: sayi }, (_, i) => ({
    from: i * width,
    to: (i + 1) * width,
    count: 0,
    mid: i * width + width / 2,
  }));

  for (const v of values) {
    const kirpik = Math.min(Math.max(v, 0), 100);
    const i = Math.min(Math.floor(kirpik / width), sayi - 1);
    bins[i].count += 1;
  }
  return bins;
}

export type CurvePoint = { x: number; y: number };

/**
 * Dağılım eğrisi — ölçme-değerlendirme kitaplarındaki çan.
 *
 * Çekirdek yoğunluk kestirimi (Gauss çekirdeği). Histogram aralık sınırına
 * duyarlıdır: sınır bir puan kaysa şekil değişir. Eğri bunu yapmaz — her puan
 * kendi etrafına bir tümsek koyar, tümsekler toplanır.
 *
 * Y EKSENİ FREKANS KALIR. Ham yoğunluk 0-1 arası soyut bir sayıdır ve
 * öğretmene bir şey söylemez; `n × aralık genişliği` ile çarpılıp "bu
 * genişlikte beklenen öğrenci sayısı"na çevriliyor. Böylece eğri, altındaki
 * nokta şeridiyle aynı ölçekte okunuyor.
 *
 * Bant genişliği Silverman kuralı: h = 1.06 · s · n^(-1/5). Elle bir sayı
 * seçmek, eğrinin şeklini veriye değil o seçime bağlardı.
 *
 * n < 3 ya da sapma sıfırsa boş dizi: iki noktadan çan çizmek veriyi değil
 * çekirdek genişliğini göstermek olur.
 */
export function densityCurve(
  values: number[],
  binWidth = BIN_WIDTH,
  step = 2,
): CurvePoint[] {
  const n = values.length;
  if (n < 3) return [];

  const mean = values.reduce((a, b) => a + b, 0) / n;
  const sd = Math.sqrt(
    values.reduce((acc, v) => acc + (v - mean) ** 2, 0) / (n - 1),
  );
  if (sd === 0) return [];

  const h = 1.06 * sd * Math.pow(n, -1 / 5);
  const olcek = (n * binWidth) / (h * Math.sqrt(2 * Math.PI));

  const noktalar: CurvePoint[] = [];
  for (let x = 0; x <= 100; x += step) {
    let toplam = 0;
    for (const v of values) {
      const z = (x - v) / h;
      toplam += Math.exp(-0.5 * z * z);
    }
    noktalar.push({ x, y: (toplam / n) * olcek });
  }
  return noktalar;
}

/**
 * Çarpıklığın sözle karşılığı.
 *
 * Eşikler ölçme-değerlendirmede yerleşik: |0.5| altı simetrik sayılır,
 * |1| üstü belirgin çarpıklıktır. Sayının kendisi öğretmene bir şey
 * söylemiyor; yönü ve şiddeti söylüyor.
 */
export function skewLabel(skew: number | null): string {
  if (skew === null) return "Hesaplanamadı";
  if (Math.abs(skew) < 0.5) return "Simetrik — puanlar ortada toplanmış";
  const siddet = Math.abs(skew) < 1 ? "Orta düzey" : "Belirgin";
  return skew < 0
    ? `${siddet} sola çarpık — yığılma yüksek puanlarda, sınıf başarılı`
    : `${siddet} sağa çarpık — yığılma düşük puanlarda, sınıf zorlanmış`;
}

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

/**
 * Çarpıklık katsayısı — Fisher–Pearson, örneklem düzeltmeli (Excel SKEW).
 *
 *   G1 = n / ((n-1)(n-2)) · Σ((xi − x̄) / s)³
 *
 * n < 3'te tanımsız: üç noktadan az veriyle bir dağılımın yönü hakkında
 * konuşulamaz. Sapma sıfırsa (herkes aynı puanı almış) bölme tanımsız.
 */
function skew(values: number[], mean: number, sd: number): number | null {
  const n = values.length;
  if (n < 3 || sd === 0) return null;
  const toplam = values.reduce((acc, x) => acc + ((x - mean) / sd) ** 3, 0);
  return (n / ((n - 1) * (n - 2))) * toplam;
}

/**
 * En kalabalık aralığın orta noktası; TEPE YOKSA null.
 *
 * Eşitlikte ilk aralığı seçmek en sinsi hataydı: altı öğrenci altı ayrı
 * aralığa düştüğünde "mod" en düşük puan çıkıyordu ve öğretmen mod < medyan
 * < ortalama sırasına bakıp dağılımı sağa çarpık sanıyordu.
 *
 * Birden çok aralık aynı en yüksek sayıyı paylaşıyorsa tek bir tepe yok
 * demektir; sayı uydurmak yerine yokluğu söyleniyor.
 */
function modeOf(values: number[]): number | null {
  const bins = histogram(values);
  const enYuksek = Math.max(...bins.map((b) => b.count));
  if (enYuksek === 0) return null;

  const tepeler = bins.filter((b) => b.count === enYuksek);
  return tepeler.length === 1 ? tepeler[0].mid : null;
}

export function spread(values: number[]): Spread | null {
  if (values.length === 0) return null;
  const s = [...values].sort((a, b) => a - b);
  const n = s.length;
  const mean = s.reduce((a, b) => a + b, 0) / n;

  // Örneklem sapması (n-1): sınıf, tüm öğrencilerin evreni değil, o sınavdaki
  // bir ölçümdür. Excel'in STDEV.S'iyle aynı sayıyı verir.
  const varyans =
    n < 2 ? 0 : s.reduce((acc, x) => acc + (x - mean) ** 2, 0) / (n - 1);
  const sd = Math.sqrt(varyans);

  return {
    n,
    mean,
    median: quantile(s, 0.5),
    mode: modeOf(s),
    sd,
    skewness: skew(s, mean, sd),
    min: s[0],
    max: s[n - 1],
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
