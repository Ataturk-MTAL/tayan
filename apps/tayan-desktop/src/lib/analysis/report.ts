import type { Exam, ExamResult, Question, Student } from "$lib/types";
import {
  BIN_WIDTH,
  histogram,
  needsReview,
  skewLabel,
  spread,
  type ItemStat,
} from "./item-stats";

/**
 * Rust'a gönderilen rapor yükü. Alan adları `analysis_report.rs` ile BİREBİR;
 * serde onları böyle bekliyor.
 *
 * ÖLÇÜLER BURADA HESAPLANIR, RUST'TA DEĞİL. İkinci bir hesap, öğretmenin
 * veliye gösterdiği kâğıtla ekranda gördüğünün ayrışması demekti.
 */
export type ReportItem = {
  order: number;
  title: string;
  correct: number;
  partial: number;
  wrong: number;
  blank: number;
  difficulty: number;
  discrimination: number | null;
  review_note: string | null;
};

export type ReportStudent = {
  number: string;
  name: string;
  percentage: number;
};

export type AnalysisReport = {
  exam_title: string;
  subject: string;
  classroom: string;
  teacher: string;
  date: string;
  school: string | null;
  department: string | null;
  mean: number;
  median: number;
  mode: number;
  sd: number;
  skewness: number | null;
  /**
   * Çarpıklığın sözle karşılığı — EKRANDA üretilir.
   *
   * Eşikleri Rust'ta bir daha yazmak, kâğıtla ekranın aynı sayı için farklı
   * yorum yazması demekti.
   */
  skew_label: string;
  /** Frekans dağılımı: her aralıktaki öğrenci sayısı. */
  bins: number[];
  bin_width: number;
  min: number;
  max: number;
  q1: number;
  q3: number;
  threshold: number;
  below_threshold: number;
  students: ReportStudent[];
  items: ReportItem[];
};

/** Sorunun rapordaki adı: başlık varsa o, yoksa numarası. */
function itemTitle(item: ItemStat, bank: Question[]): string {
  const q = bank.find((b) => b.id === item.questionId);
  const t = q?.meta.title.trim() ?? "";
  return t !== "" ? t : `${item.order}. soru`;
}

export function buildReport(args: {
  exam: Exam;
  items: ItemStat[];
  bank: Question[];
  results: ExamResult[];
  students: Student[];
  threshold: number;
}): AnalysisReport | null {
  const { exam, items, bank, results, students, threshold } = args;

  const satirlar = results
    .map((r) => {
      const s = students.find((x) => x.id === r.student_id);
      const yuzde =
        r.total_points_max > 0 ? (r.total_points_earned / r.total_points_max) * 100 : 0;
      return {
        number: s?.number ?? "",
        name: s ? `${s.first_name} ${s.last_name}` : "—",
        percentage: yuzde,
      };
    })
    // Numaraya göre: yoklama defteriyle aynı sıra, öğretmen kaybolmaz.
    .sort((a, b) => a.number.localeCompare(b.number, "tr", { numeric: true }));

  const dagilim = spread(satirlar.map((s) => s.percentage));
  if (dagilim === null) return null;

  return {
    exam_title: exam.meta.title,
    subject: exam.meta.subject,
    classroom: exam.meta.classroom,
    teacher: exam.meta.teacher,
    date: exam.meta.date,
    school: exam.meta.school ?? null,
    department: exam.meta.department ?? null,
    mean: dagilim.mean,
    median: dagilim.median,
    mode: dagilim.mode,
    sd: dagilim.sd,
    skewness: dagilim.skewness,
    skew_label: skewLabel(dagilim.skewness),
    bins: histogram(satirlar.map((s) => s.percentage)).map((b) => b.count),
    bin_width: BIN_WIDTH,
    min: dagilim.min,
    max: dagilim.max,
    q1: dagilim.q1,
    q3: dagilim.q3,
    threshold,
    below_threshold: satirlar.filter((s) => s.percentage < threshold).length,
    students: satirlar,
    items: items.map((i) => ({
      order: i.order,
      title: itemTitle(i, bank),
      correct: i.correct,
      partial: i.partial,
      wrong: i.wrong,
      blank: i.blank,
      difficulty: i.difficulty,
      discrimination: i.discrimination,
      review_note: needsReview(i),
    })),
  };
}
