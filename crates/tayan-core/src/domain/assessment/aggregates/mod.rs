use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::exam_management::{
    aggregates::ExamId,
    entities::QuestionId,
    value_objects::OutcomeCode,
};
use crate::domain::student_management::StudentId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExamResultId(pub Uuid);
impl Default for ExamResultId {
    fn default() -> Self {
        Self::new()
    }
}

impl ExamResultId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

/// How a student answered a single question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnswer {
    pub question_id:       QuestionId,
    /// For MC/TF: the chosen option id ("A"/"B" or "true"/"false").
    /// For FillInBlank: JSON map of blank_id → given_value.
    /// For Classic: None (score is entered directly).
    pub given_answer:      Option<String>,
    pub points_earned:     f32,
    pub is_correct:        Option<bool>,  // None for Classic (manual)
}

/// Outcome-level performance for a single student on a single exam.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomePerformance {
    pub outcome:         OutcomeCode,
    pub total_questions: u32,
    pub correct:         u32,
    pub score_pct:       f32,
}

/// One student's full result for one exam.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExamResult {
    pub id:                  ExamResultId,
    pub exam_id:             ExamId,
    pub student_id:          StudentId,
    pub answers:             Vec<QuestionAnswer>,
    pub total_points_earned: f32,
    pub total_points_max:    f32,
    pub outcome_performance: Vec<OutcomePerformance>,
    /// Rank within the classroom (set after all results are entered).
    pub classroom_rank:      Option<u32>,
    pub recorded_at:         DateTime<Utc>,
    pub is_complete:         bool,
}

impl ExamResult {
    pub fn new(exam_id: ExamId, student_id: StudentId, total_max: f32) -> Self {
        Self {
            id:                  ExamResultId::new(),
            exam_id,
            student_id,
            answers:             vec![],
            total_points_earned: 0.0,
            total_points_max:    total_max,
            outcome_performance: vec![],
            classroom_rank:      None,
            recorded_at:         Utc::now(),
            is_complete:         false,
        }
    }

    pub fn score_percentage(&self) -> f32 {
        if self.total_points_max == 0.0 { return 0.0; }
        (self.total_points_earned / self.total_points_max * 100.0).clamp(0.0, 100.0)
    }
}
