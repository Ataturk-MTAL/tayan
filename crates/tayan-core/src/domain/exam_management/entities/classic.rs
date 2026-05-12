use serde::{Deserialize, Serialize};
use crate::domain::exam_management::value_objects::{
    OutcomeCode, QuestionBody, QuestionStats,
};
use super::question::{Points, QuestionId};

/// Defines how much space to leave for the student's answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnswerSpace {
    Lines(u8),
    HeightCm(f32),
    Grid { rows: u8, cols: u8 },
}

impl Default for AnswerSpace {
    fn default() -> Self { AnswerSpace::Lines(6) }
}

/// A single scoring criterion for a classic (open-ended) question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricItem {
    pub criterion: String,
    pub points:    Points,
}

/// Classic / open-ended question. Scored manually by the teacher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicQuestion {
    pub id:           QuestionId,
    pub points:       Points,
    pub outcomes:     Vec<OutcomeCode>,
    pub body:         QuestionBody,
    /// Optional sample answer visible only in the answer-key PDF.
    pub sample_answer: Option<QuestionBody>,
    pub rubric:       Vec<RubricItem>,
    pub answer_space: AnswerSpace,
    pub stats:        QuestionStats,
}

impl ClassicQuestion {
    pub fn rubric_total(&self) -> u32 {
        self.rubric.iter().map(|r| r.points.value()).sum()
    }

    pub fn validate(&self) -> Result<(), crate::domain::shared::errors::DomainError> {
        use crate::domain::shared::errors::DomainError;
        if self.body.is_empty() {
            return Err(DomainError::Validation(
                "Classic question body must not be empty".into(),
            ));
        }
        if !self.rubric.is_empty() && self.rubric_total() != self.points.value() {
            return Err(DomainError::Validation(format!(
                "Rubric total ({}) does not match question points ({})",
                self.rubric_total(),
                self.points.value()
            )));
        }
        Ok(())
    }
}
