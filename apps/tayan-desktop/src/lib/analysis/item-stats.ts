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

/**
 * Dağılım eğrisinin çizilebildiği en küçük sınıf.
 *
 * `MIN_DISCRIMINATION_N` ile AYNI İLKE: "Sayı uydurmaktansa yokluğunu
 * söylemek gerekiyor". Bu dosya n<10'da ayırt ediciliği hesaplamayı
 * reddediyordu ama aynı altı öğrenciden bütün bir dağılım şekli çiziliyordu —
 * kendi ölçütüne uymuyordu.
 *
 * EKRAN VE KÂĞIT AYNI KAPIYI KULLANIR. Eşik iki yerde ayrı yazılsaydı,
 * öğretmenin veliye gösterdiği PDF ekranda görmediği bir eğri taşıyabilirdi.
 */
export const MIN_CURVE_N = 30;

/**
 * Çarpıklık sayısının ve sözlü yorumunun okunabildiği en küçük sınıf.
 *
 * SE(G1) = √( 6n(n−1) / ((n−2)(n+1)(n+3)) ) — n=6'da 0.845, n=15'te 0.580,
 * n=30'da 0.427, n=50'de 0.337. `skewLabel`in ilk eşiği |0.5|; o eşiğin
 * altını okuyabilmek için SE'nin de altında kalması gerekiyor.
 */
export const MIN_SKEWNESS_N = 50;

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

/** Bir kazanımın SINIF düzeyindeki performansı. */
export type OutcomeStat = {
  outcome: string;
  /** Bu kazanımı taşıyan soru sayısı — yüzdenin ne kadar sağlam olduğunu söyler. */
  questionCount: number;
  pointsEarned: number;
  pointsAvailable: number;
  /** 0-100. */
  scorePct: number;
};

/**
 * Kazanım bazlı sınıf performansı.
 *
 * KAYNAK `itemStats`, SAKLANAN `outcome_performance` DEĞİL. İki gerekçe:
 *
 *   1. Saklanan alan YAZMA ANINDA hesaplanıyor. Puanlama düzeltilse bile eski
 *      sonuçlar eski değerlerini taşımaya devam ediyor; ekran onlardan
 *      beslenseydi öğretmen, düzeltilmiş bir hatayı hâlâ ekranında görürdü.
 *   2. Buradaki sayılar madde tablosunun sayılarıyla AYNI kaynaktan geliyor:
 *      `difficulty = kazanılan / (öğrenci × maxPoints)`. Yani kazanım yüzdesi
 *      ile soru güçlüğü hiçbir zaman birbiriyle çelişemez.
 *
 * Rust tarafındaki `ScoringService::compute_outcome_performance` ÖĞRENCİ
 * bazlı aynı ölçüyü hesaplıyor (karne için); ikisi de puan üzerinden çalışıyor.
 * Biri değişirse diğeri de gözden geçirilmeli.
 *
 * KODSUZ SORULAR AYRICA DÖNÜYOR. Kazanım kodu girilmemiş sorular hiçbir çubuğa
 * girmiyor; sayısı söylenmezse öğretmen grafiği "sınavın tamamı" sanır, oysa
 * yarısını kapsıyor olabilir.
 */
export function outcomeStats(
  items: ItemStat[],
  bank: Question[],
  studentCount: number,
): { outcomes: OutcomeStat[]; uncodedQuestions: number } {
  const toplam = new Map<string, { questionCount: number; earned: number; available: number }>();
  let uncodedQuestions = 0;

  for (const item of items) {
    const q = bank.find((b) => b.id === item.questionId);
    if (!q) continue;

    if (q.outcomes.length === 0) {
      uncodedQuestions += 1;
      continue;
    }

    const available = item.maxPoints * studentCount;
    // `difficulty` zaten kazanılan/alınabilir oranı; çarpınca ham puana dönüyor.
    const earned = item.difficulty * available;

    /*
      ÇOK KAZANIMLI SORU HER KAZANIMA TAM PUANIYLA GİRİYOR, bölünerek değil.
      Soru iki kazanımı birden ölçüyorsa, o sorudan alınan puan ikisi için de
      kanıttır; yarıya bölmek "bu kazanımdan 2 puan alınabilirdi" gibi
      uydurma bir payda üretirdi. Yüzdeler bu yüzden kazanımlar arasında
      toplanabilir değil — her biri kendi içinde okunur.
    */
    for (const outcome of q.outcomes) {
      const e = toplam.get(outcome) ?? { questionCount: 0, earned: 0, available: 0 };
      toplam.set(outcome, {
        questionCount: e.questionCount + 1,
        earned: e.earned + earned,
        available: e.available + available,
      });
    }
  }

  const outcomes = [...toplam.entries()]
    .map(([outcome, v]) => ({
      outcome,
      questionCount: v.questionCount,
      pointsEarned: v.earned,
      pointsAvailable: v.available,
      scorePct: v.available > 0 ? (v.earned / v.available) * 100 : 0,
    }))
    // EN KÖTÜ ÜSTTE: grafiğin üst kısmı doğrudan telafi planı oluyor.
    .sort((a, b) => a.scorePct - b.scorePct);

  return { outcomes, uncodedQuestions };
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

  /*
    BANDWIDTH VERİDEN GELİR, BİR GÖRÜNTÜ SABİTİNDEN DEĞİL.

    Bir ara burada `h = min(max(silverman, binWidth/2), binWidth)` vardı; amaç
    Silverman'ın küçük örneklemde aşırı düzleştirmesini telafi etmekti. Sonuç
    UYDURMA BİR BULGU oldu: 15/30/45/65/70/90 puanlarında h=10 ile eğri iki
    tepeli çıkıyor — x=30'da 0.659, x=50'de 0.591 çukur, x=67.5'te 0.837 —
    ve o çukur TAM geçme eşiğinin altına düşüyor. Grafik "sınıf geçme notu
    etrafında ikiye ayrılıyor" diyordu; veride böyle bir şey yok. Aynı altı
    puan Silverman'ın kendi h=20.52'siyle çukursuz tek bir kemer veriyor.

    Düzleştirme parametresini `BIN_WIDTH` gibi bir ekran sabitine eşitlemek
    yoğunluk kestirimi değil, hoşa giden şekli aramaktır. Kural veriden
    gelir; eğri az örneklemde düz çıkıyorsa doğru cevap eğriyi ZORLAMAK değil,
    o boyutta EĞRİ ÇİZMEMEKTİR (ekran tarafındaki `MIN_CURVE_N` kapısı).
  */
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
  /*
    "Simetrik — puanlar ortada TOPLANMIŞ" yazıyordu ve bu YANLIŞTI. Çarpıklık
    bakışımı ölçer, toplanmayı değil; düzgün (uniform) dağılım da simetriktir,
    uçlara yığılmış çift tepeli bir dağılım da. 15/30/45/65/70/90 puanlı bir
    sınıf (sd 27.7, ölçeğin 75 puanına yayılmış) "ortada toplanmış" diye
    tanıtılıyordu — toplanmanın tam tersi. Yayılımı sd ve ranj söyler.
  */
  if (Math.abs(skew) < 0.5) return "Bakışımlı — iki kuyruk dengeli";
  const siddet = Math.abs(skew) < 1 ? "Orta düzey" : "Belirgin";
  /*
    NEDEN İKİ OKUMA BİRDEN. Sola çarpıklık tek başına "sınıf başarılı"
    demiyor: yüzde puanları 100'de sınırlı olduğu için KOLAY bir test de
    mekanik olarak sola çarpıklık üretir (tavan etkisi). İki yorum zıt
    öğretimsel sonuçlara götürüyor; yalnız birini yazmak öğretmeni yanıltır.
  */
  return skew < 0
    ? `${siddet} sola çarpık — yığılma yüksek puanlarda (sınıf başarılı ya da test kolay gelmiş)`
    : `${siddet} sağa çarpık — yığılma düşük puanlarda (sınıf zorlanmış ya da test zor gelmiş)`;
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
/**
 * Ayırt ediciliğin "ters" sayıldığı eşik.
 *
 * SIFIR DEĞİL, çünkü küçük sınıfta D'nin örneklem gürültüsü tek başına eksiye
 * geçebiliyor. Üretilmiş veriyle ölçüldü (bkz. `seed_analysis`): gerçek ayırt
 * ediciliği sıfır olan bir madde 34 kişilik sınıfta D = −0.11 veriyor (üst/alt
 * dilim 9 kişi), 220 kişide ise D = +0.19'a oturuyor. Gerçekten ters olan madde
 * aynı iki ölçekte −0.44 ve −0.41 veriyor. Sıfır eşiği tipik sınıfta yanlış
 * alarm üretirdi; −0.20 ikisini de doğru ayırıyor.
 */
export const REVERSE_DISCRIMINATION = -0.2;

export function needsReview(item: ItemStat): string | null {
  if (item.answered === 0) return "Kimse cevaplamamış";
  if (item.difficulty < 0.2) return "Çok zor — kimse yapamamış";
  if (item.difficulty > 0.9) return "Çok kolay — herkes yapmış";

  /*
    TERS AYIRAN MADDE, "AYIRT ETMİYOR"DAN AYRI BİR TEŞHİSTİR.

    Eskiden tek dal vardı: `discrimination < 0.2` ⇒ "Ayırt etmiyor". İşarete
    bakılmıyordu, dolayısıyla D = −0.44 olan madde ile D = 0.05 olan madde aynı
    cümleyi alıyordu. Oysa öğretmenin yapacağı iş bambaşka:

      D ≈ 0  → soru kimseyi ayırmıyor; belirsiz ya da uç güçlükte.
               YAPILACAK: soruyu gözden geçir.
      D < 0  → soruyu İYİ öğrenciler yanlış, ZAYIF öğrenciler doğru yapmış.
               Tesadüf değil, sistematik ters ilişki; ölçme kitaplarında
               birincil açıklaması CEVAP ANAHTARININ YANLIŞ olmasıdır.
               YAPILACAK: önce anahtarı kontrol et.
  */
  if (item.discrimination !== null && item.discrimination <= REVERSE_DISCRIMINATION) {
    return "Ters ayırıyor — cevap anahtarını kontrol et";
  }
  if (item.discrimination !== null && item.discrimination < 0.2) {
    return "Ayırt etmiyor";
  }
  return null;
}
