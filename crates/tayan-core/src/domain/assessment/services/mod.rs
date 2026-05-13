use crate::domain::assessment::aggregates::{ExamResult, OutcomePerformance, QuestionAnswer};
use crate::domain::exam_management::{
    aggregates::QuestionBank,
    entities::{FillInBlankQuestion, Question, QuestionId},
    value_objects::OutcomeCode,
};

// ── Auto-scoring ──────────────────────────────────────────────────────────────

pub struct ScoringService;

impl ScoringService {
    /// Scores all auto-scorable answers. Classic questions must be scored separately.
    pub fn auto_score(
        result:  &mut ExamResult,
        answers: Vec<QuestionAnswer>,
        bank:    &QuestionBank,
    ) {
        let mut total = 0.0f32;
        let mut scored = vec![];

        for mut ans in answers {
            if let Some(bq) = bank.find(&ans.question_id) {
                let (pts, correct) = Self::score_answer(&bq.question, &ans);
                ans.points_earned = pts;
                ans.is_correct    = correct;
                total            += pts;
            }
            scored.push(ans);
        }

        result.answers             = scored;
        result.total_points_earned = total;
        result.outcome_performance = Self::compute_outcome_performance(result, bank);
    }

    fn score_answer(question: &Question, ans: &QuestionAnswer) -> (f32, Option<bool>) {
        match question {
            Question::MultipleChoice(q) => {
                let given = ans.given_answer.as_deref().unwrap_or("");
                let pts   = q.score_answer(given);
                (pts, Some(pts > 0.0))
            }
            Question::TrueFalse(q) => {
                let given = ans.given_answer
                    .as_deref()
                    .and_then(|s| s.parse::<bool>().ok())
                    .unwrap_or(false);
                let pts = q.score_answer(given);
                (pts, Some(pts > 0.0))
            }
            Question::FillInBlank(q) => {
                let pts = Self::score_fill_in_blank(q, &ans.given_answer);
                (pts, Some(pts == q.total_points().value() as f32))
            }
            Question::Classic(_) => (ans.points_earned, None), // manual — keep passed value
        }
    }

    fn score_fill_in_blank(q: &FillInBlankQuestion, given: &Option<String>) -> f32 {
        let map: std::collections::HashMap<String, String> = given
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        q.blanks
            .iter()
            .map(|b| b.score_answer(map.get(&b.id).map(|s| s.as_str()).unwrap_or("")))
            .sum()
    }

    fn compute_outcome_performance(
        result: &ExamResult,
        bank:   &QuestionBank,
    ) -> Vec<OutcomePerformance> {
        let mut map: std::collections::HashMap<&OutcomeCode, (u32, u32)> =
            std::collections::HashMap::new();

        for ans in &result.answers {
            if let Some(bq) = bank.find(&ans.question_id) {
                for outcome in bq.question.outcomes() {
                    let entry = map.entry(outcome).or_insert((0, 0));
                    entry.0 += 1;
                    if ans.is_correct == Some(true) {
                        entry.1 += 1;
                    }
                }
            }
        }

        map.into_iter()
            .map(|(outcome, (total, correct))| OutcomePerformance {
                outcome:         outcome.clone(),
                total_questions: total,
                correct,
                score_pct:       if total > 0 { correct as f32 / total as f32 * 100.0 } else { 0.0 },
            })
            .collect()
    }
}

// ── QuestionStats updater ─────────────────────────────────────────────────────

pub struct QuestionStatsUpdater;

impl QuestionStatsUpdater {
    /// After all results for an exam are entered, update bank statistics.
    pub fn update_from_results(bank: &mut QuestionBank, results: &[ExamResult]) {
        use std::collections::HashMap;

        struct Accum {
            correct: u32,
            total:   u32,
            pts_sum: f32,
            scores:  Vec<(f32, f32)>, // (exam_total, is_correct as 0/1)
        }

        let mut acc: HashMap<QuestionId, Accum> = HashMap::new();

        for result in results {
            for ans in &result.answers {
                let entry = acc.entry(ans.question_id.clone()).or_insert(Accum {
                    correct: 0, total: 0, pts_sum: 0.0, scores: vec![],
                });
                entry.total   += 1;
                entry.pts_sum += ans.points_earned;
                entry.scores.push((
                    result.total_points_earned,
                    if ans.is_correct == Some(true) { 1.0 } else { 0.0 },
                ));
                if ans.is_correct == Some(true) {
                    entry.correct += 1;
                }
            }
        }

        for (qid, a) in acc {
            if let Some(bq) = bank.find_mut(&qid) {
                let disc  = Self::discrimination_index(&a.scores);
                let avg   = if a.total > 0 { a.pts_sum / a.total as f32 } else { 0.0 };
                bq.question.stats_mut().record_usage(a.correct, a.total, avg, disc);
            }
        }
    }

    fn discrimination_index(scores: &[(f32, f32)]) -> f32 {
        let n = scores.len();
        if n < 6 { return 0.0; }

        let cutoff = ((n as f32 * 0.27).ceil() as usize).max(1);
        let mut sorted = scores.to_vec();
        sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let top_correct: f32 = sorted[..cutoff].iter().map(|(_, c)| c).sum();
        let bot_correct: f32 = sorted[n - cutoff..].iter().map(|(_, c)| c).sum();

        (top_correct - bot_correct) / cutoff as f32
    }
}
