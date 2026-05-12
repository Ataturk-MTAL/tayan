// ── Content ───────────────────────────────────────────────────────────────────

export type TextStyle = {
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strikethrough: boolean;
};

export type ContentNode =
  | { type: 'text';    text: string; style: TextStyle }
  | { type: 'math';    raw: string;  display: 'inline' | 'block' }
  | { type: 'chem';    raw: string;  flavor: 'formula' | 'structural' }
  | { type: 'image';   src: string;  alt: string; width?: string }
  | { type: 'blank';   id: string;   width?: string }
  | { type: 'newline' };

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

export type ScoreBadge = 'excellent' | 'good' | 'fair' | 'poor' | 'untested';

export function scoreBadge(stats: QuestionStats): ScoreBadge {
  if (stats.times_used === 0) return 'untested';
  const s = stats.performance_score;
  if (s >= 80) return 'excellent';
  if (s >= 50) return 'good';
  if (s >= 20) return 'fair';
  return 'poor';
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
};

export type RubricCriterion = {
  description: string;
  max_points: number;
};

export type MultipleChoiceQuestion = {
  question_type: 'multiple_choice';
  id: string;
  points: number;
  outcomes: string[];
  body: ContentNode[];
  options: QuestionOption[];
  shuffle: boolean;
  stats: QuestionStats;
};

export type TrueFalseQuestion = {
  question_type: 'true_false';
  id: string;
  points: number;
  outcomes: string[];
  body: ContentNode[];
  correct_answer: boolean;
  stats: QuestionStats;
};

export type FillInBlankQuestion = {
  question_type: 'fill_in_blank';
  id: string;
  outcomes: string[];
  body: ContentNode[];
  blanks: Blank[];
  stats: QuestionStats;
};

export type ClassicQuestion = {
  question_type: 'classic';
  id: string;
  points: number;
  outcomes: string[];
  body: ContentNode[];
  sample_answer: ContentNode[] | null;
  rubric: RubricCriterion[];
  stats: QuestionStats;
};

export type Question =
  | MultipleChoiceQuestion
  | TrueFalseQuestion
  | FillInBlankQuestion
  | ClassicQuestion;

// ── Helpers ───────────────────────────────────────────────────────────────────

export function questionPoints(q: Question): number {
  if (q.question_type === 'fill_in_blank') {
    return q.blanks.reduce((sum, b) => sum + b.points, 0);
  }
  return (q as MultipleChoiceQuestion).points;
}

export function bodyPreview(body: ContentNode[], maxLen = 80): string {
  const text = body
    .map((n) => {
      if (n.type === 'text') return n.text;
      if (n.type === 'math') return `[${n.raw}]`;
      if (n.type === 'chem') return `[${n.raw}]`;
      return '';
    })
    .join('')
    .trim();
  return text.length > maxLen ? text.slice(0, maxLen) + '…' : text;
}

export const QUESTION_TYPE_LABELS: Record<Question['question_type'], string> = {
  multiple_choice: 'Çoktan Seçmeli',
  true_false:      'Doğru / Yanlış',
  fill_in_blank:   'Boşluk Doldurma',
  classic:         'Klasik',
};
