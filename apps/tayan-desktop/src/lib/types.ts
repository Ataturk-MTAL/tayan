import { typstPlain } from "$lib/question/plain-text";
// ── Content ───────────────────────────────────────────────────────────────────

export type TextStyle = {
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strikethrough: boolean;
};

export type ContentNode =
  | { type: "text"; text: string; style: TextStyle }
  | { type: "math"; raw: string; display: "inline" | "block" }
  | { type: "typst_raw"; code: string }
  | { type: "chem"; raw: string; flavor: "formula" | "structural" }
  | {
      type: "image";
      src: string;
      alt: string;
      align?: "left" | "center" | "right";
      width?: string;
      wrap?: boolean;
    }
  | { type: "blank"; id: string; width?: string }
  | { type: "newline" };

// ── Stats ─────────────────────────────────────────────────────────────────────

export type QuestionStats = {
  times_used: number;
  total_responses: number;
  correct_responses: number;
  difficulty_index: number;
  discrimination_index: number;
  avg_points_earned: number;
  last_used_at: string | null;
  performance_score: number;
};

export type ScoreBadge = "excellent" | "good" | "fair" | "poor" | "untested";

export function scoreBadge(stats: QuestionStats): ScoreBadge {
  if (stats.times_used === 0) return "untested";
  const s = stats.performance_score;
  if (s >= 80) return "excellent";
  if (s >= 50) return "good";
  if (s >= 20) return "fair";
  return "poor";
}

// ── Questions ─────────────────────────────────────────────────────────────────

export type QuestionOption = {
  id: string;
  body: ContentNode[];
  correct: boolean;
};

export type Blank = {
  id: string;
  accepted_answers: string[];
  points: number;
  case_sensitive: boolean;
};

export type RubricItem = {
  criterion: string;
  points: number;
};

// Rust enum: Lines(u8) | HeightCm(f32) | Grid { rows, cols }
export type AnswerSpace =
  | { Lines: number }
  | { HeightCm: number }
  | { Grid: { rows: number; cols: number } };

/** Öğretmenin kanaati. Ölçülen güçlük ayrı: QuestionStats.difficulty_index. */
export type Difficulty = "kolay" | "orta" | "zor";

export const DIFFICULTY_LABELS: Record<Difficulty, string> = {
  kolay: "Kolay",
  orta: "Orta",
  zor: "Zor",
};

export const MIN_GRADE = 1;
export const MAX_GRADE = 12;

/**
 * Sorunun künyesi. Ders ve sınıf seviyesi ZORUNLU — kazanım kodu (MAT.9.1.2)
 * ikisine bağlı, bilinmeden hangi kazanım listesinin gösterileceği belirlenemez.
 *
 * Eski kayıtlarda bu alan yok; Rust tarafı serde(default) ile subject: "",
 * grade: 0 döndürür. Böyle bir soru yüklenir ve basılır ama yeniden
 * kaydedilirken doğrulamaya takılır.
 */
export type QuestionMeta = {
  subject: string;
  grade: number;
  difficulty: Difficulty | null;
  /**
   * Kısa başlık — "Dijital Çıkış — LED Sürme". Yalnız cevap anahtarına basılır;
   * öğrenci nüshasında başlık konuyu ele verir. Boş dize başlıksız demektir.
   */
  title: string;
};

export type MultipleChoiceQuestion = {
  question_type: "multiple_choice";
  meta: QuestionMeta;
  id: string;
  points: number;
  outcomes: string[];
  body: ContentNode[];
  options: QuestionOption[];
  shuffle: boolean;
  stats: QuestionStats;
};

export type TrueFalseQuestion = {
  question_type: "true_false";
  meta: QuestionMeta;
  id: string;
  points: number;
  outcomes: string[];
  body: ContentNode[];
  correct_answer: boolean;
  stats: QuestionStats;
};

export type FillInBlankQuestion = {
  question_type: "fill_in_blank";
  meta: QuestionMeta;
  id: string;
  outcomes: string[];
  body: ContentNode[];
  blanks: Blank[];
  stats: QuestionStats;
};

export type ClassicQuestion = {
  question_type: "classic";
  meta: QuestionMeta;
  id: string;
  points: number;
  outcomes: string[];
  body: ContentNode[];
  sample_answer: ContentNode[] | null;
  rubric: RubricItem[];
  answer_space: AnswerSpace;
  stats: QuestionStats;
};

export type Question =
  | MultipleChoiceQuestion
  | TrueFalseQuestion
  | FillInBlankQuestion
  | ClassicQuestion;

// ── Helpers ───────────────────────────────────────────────────────────────────

export function questionPoints(q: Question): number {
  if (q.question_type === "fill_in_blank") {
    return q.blanks.reduce((sum, b) => sum + b.points, 0);
  }
  return (q as MultipleChoiceQuestion).points;
}

/**
 * Listelerde gösterilecek tek satırlık özet.
 *
 * Gövde tek bir `typst_raw` düğümü olarak saklandığı için burası bir zamanlar
 * ham kodu "[typst] ..." diye basıyordu; sonuç girişi ve analiz ekranlarında
 * öğretmen hangi soruyu puanladığını göremiyordu. Artık kaynak önce Typst
 * kurallarına göre düz metne çevriliyor.
 */
export function bodyPreview(body: ContentNode[], maxLen = 80): string {
  const text = body
    .map((n) => {
      if (n.type === "text") return n.text;
      if (n.type === "math") return typstPlain(`$${n.raw}$`);
      if (n.type === "typst_raw") return typstPlain(n.code);
      if (n.type === "chem") return typstPlain(`$${n.raw}$`);
      return "";
    })
    .join(" ")
    .replace(/\s+/g, " ")
    .trim();
  return text.length > maxLen ? text.slice(0, maxLen) + "…" : text;
}

// ── Exam ─────────────────────────────────────────────────────────────────────

export type ExamStatus = "Draft" | "Published" | "Archived";

/** Kâğıdı imzalayan kişi — "Ömer YİĞİT" / "Okul Müdürü". */
export type ExamSigner = {
  name: string;
  title: string;
};

export type ExamMeta = {
  title: string;
  subject: string;
  classroom: string;
  teacher: string;
  duration_min: number;
  date: string; // NaiveDate → "YYYY-MM-DD"
  instructions: string | null;

  /**
   * Kâğıdın sütun sayısı: 1 veya 2.
   *
   * Çift sütun kısa sorularda kâğıt tasarrufu sağlar. Sorular `breakable: false`
   * blok içinde dizildiği için hiçbir soru sütun sonunda ikiye bölünmez.
   *
   * Typst sütunları DENGELEMEZ: birinci sütunu sayfa yüksekliğine kadar
   * doldurup ikinciye geçer. Az sayıda kısa soruda ikinci sütun boş kalır —
   * bu doğru davranıştır, hata değil.
   */
  columns: number;

  /** Kâğıdın başındaki kurum satırları. Boşsa basılmaz. */
  school: string | null;
  department: string | null;

  /** Kâğıdın altındaki imza bloğu. Boşsa blok hiç basılmaz. */
  signers: ExamSigner[];
};

export type ExamQuestionRef = {
  question_id: string;
  display_order: number;
  points_override: number | null;
};

export type Exam = {
  id: string;
  meta: ExamMeta;
  questions: ExamQuestionRef[];
  status: ExamStatus;
  created_at: string;
  updated_at: string;
};

export const EXAM_STATUS_LABELS: Record<ExamStatus, string> = {
  Draft: "Taslak",
  Published: "Yayında",
  Archived: "Arşiv",
};

// ── Question Type Labels ──────────────────────────────────────────────────────

export const QUESTION_TYPE_LABELS: Record<Question["question_type"], string> = {
  multiple_choice: "Çoktan Seçmeli",
  true_false: "Doğru / Yanlış",
  fill_in_blank: "Boşluk Doldurma",
  classic: "Klasik",
};

// ── Classroom / Student ───────────────────────────────────────────────────────

export type Classroom = {
  id: string;
  name: string;
  grade: number;
  branch: string;
  academic_year: string;
  student_ids: string[];
  created_at: string;
};

export type Student = {
  id: string;
  number: string;
  first_name: string;
  last_name: string;
  classroom_id: string;
  created_at: string;
};

// ── Assessment ────────────────────────────────────────────────────────────────

export type QuestionAnswerInput = {
  question_id: string;
  given_answer: string | null;
  points_earned: number;
  is_correct: boolean | null;
  /**
   * Klasik soruda karşılanan rubrik ölçütlerinin sırası (0'dan başlar).
   *
   * KANIT, KAYNAK DEĞİL: puan `points_earned` alanında durur ve giriş anında
   * donar. Rubrik sonradan düzenlenirse verilmiş notlar değişmez.
   */
  rubric_met: number[];
};

export type OutcomePerformance = {
  outcome: string;
  total_questions: number;
  correct: number;
  score_pct: number;
};

export type ExamResult = {
  id: string;
  exam_id: string;
  student_id: string;
  answers: QuestionAnswerInput[];
  total_points_earned: number;
  total_points_max: number;
  outcome_performance: OutcomePerformance[];
  classroom_rank: number | null;
  recorded_at: string;
  is_complete: boolean;
};
