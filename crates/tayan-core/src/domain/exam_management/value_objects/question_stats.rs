use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Psychometric statistics accumulated every time a question is used in an exam.
/// Updated by `QuestionStatsUpdater` in the assessment domain service.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestionStats {
    /// How many exams this question has appeared in.
    pub times_used: u32,
    /// Sum of all student responses across all uses.
    pub total_responses: u32,
    /// Sum of correct responses across all uses.
    pub correct_responses: u32,
    /// p-value: correct_responses / total_responses ∈ [0, 1]
    /// 0 = extremely difficult, 1 = trivially easy.
    /// Optimal range for a good exam item: [0.3, 0.7]
    pub difficulty_index: f32,
    /// Discrimination index (top 27% – bottom 27%) ∈ [0, 1]
    /// Values below 0.2 indicate poor discriminating power.
    pub discrimination_index: f32,
    /// Average points earned (relevant for ClassicQuestion partial credit).
    pub avg_points_earned: f32,
    pub last_used_at: Option<DateTime<Utc>>,
    /// Composite quality score in [0, 100]. Higher is better.
    /// Computed from discrimination, difficulty range optimality, and usage count.
    pub performance_score: f32,
}

impl QuestionStats {
    pub fn record_usage(&mut self, correct: u32, total: u32, avg_pts: f32, disc: f32) {
        if total == 0 { return; }
        self.times_used           += 1;
        self.correct_responses    += correct;
        self.total_responses      += total;
        self.difficulty_index      = self.correct_responses as f32 / self.total_responses as f32;
        self.discrimination_index  = disc;
        self.avg_points_earned     = avg_pts;
        self.last_used_at          = Some(Utc::now());
        self.performance_score     = self.compute_score();
    }

    pub fn is_too_easy(&self) -> bool {
        self.total_responses > 0 && self.difficulty_index > 0.8
    }

    pub fn is_too_hard(&self) -> bool {
        self.total_responses > 0 && self.difficulty_index < 0.2
    }

    pub fn has_poor_discrimination(&self) -> bool {
        self.total_responses > 0 && self.discrimination_index < 0.2
    }

    pub fn is_untested(&self) -> bool {
        self.times_used == 0
    }

    fn compute_score(&self) -> f32 {
        if self.total_responses == 0 { return 0.0; }

        // Reward items near p = 0.5 (ideal difficulty)
        let difficulty_quality = 1.0 - (self.difficulty_index - 0.5).abs() * 2.0;
        // Reward high discrimination
        let discrimination_score = self.discrimination_index.clamp(0.0, 1.0);
        // Small bonus for reuse — diminishing returns via natural log
        let usage_bonus = (self.times_used as f32 + 1.0).ln() / 5.0_f32.ln();

        ((discrimination_score  * 0.50
          + difficulty_quality  * 0.35
          + usage_bonus         * 0.15)
         * 100.0)
            .clamp(0.0, 100.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreBadge {
    /// 80–100: high quality, recommended
    Excellent,
    /// 50–79: usable
    Good,
    /// 20–49: needs review
    Fair,
    /// 0–19: revise or discard
    Poor,
    /// No data yet
    Untested,
}

impl From<&QuestionStats> for ScoreBadge {
    fn from(s: &QuestionStats) -> Self {
        if s.is_untested() { return ScoreBadge::Untested; }
        match s.performance_score as u32 {
            80..=100 => ScoreBadge::Excellent,
            50..=79  => ScoreBadge::Good,
            20..=49  => ScoreBadge::Fair,
            _        => ScoreBadge::Poor,
        }
    }
}
