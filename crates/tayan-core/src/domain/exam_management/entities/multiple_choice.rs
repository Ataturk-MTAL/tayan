use serde::{Deserialize, Serialize};
use crate::domain::exam_management::value_objects::{
    OutcomeCode, QuestionBody, QuestionStats,
};
use super::question::{Points, QuestionId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    /// "A", "B", "C", "D", "E"
    pub id:      String,
    pub body:    QuestionBody,
    pub correct: bool,
}

impl QuestionOption {
    pub fn new(id: impl Into<String>, body: QuestionBody, correct: bool) -> Self {
        Self { id: id.into(), body, correct }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleChoiceQuestion {
    pub id:       QuestionId,
    pub points:   Points,
    pub outcomes: Vec<OutcomeCode>,
    pub body:     QuestionBody,
    pub options:  Vec<QuestionOption>,
    /// When true, options are shuffled in the generated PDF.
    pub shuffle:  bool,
    pub stats:    QuestionStats,
}

impl MultipleChoiceQuestion {
    pub fn correct_option_id(&self) -> Option<&str> {
        self.options
            .iter()
            .find(|o| o.correct)
            .map(|o| o.id.as_str())
    }

    pub fn score_answer(&self, given_id: &str) -> f32 {
        if self.options.iter().any(|o| o.correct && o.id == given_id) {
            self.points.value() as f32
        } else {
            0.0
        }
    }

    pub fn validate(&self) -> Result<(), crate::domain::shared::errors::DomainError> {
        use crate::domain::shared::errors::DomainError;
        if self.options.len() < 2 {
            return Err(DomainError::Validation(
                "Çok seçenekli soruda en az 2 seçenek olmalıdır".into(),
            ));
        }
        let correct_count = self.options.iter().filter(|o| o.correct).count();
        if correct_count != 1 {
            return Err(DomainError::Validation(format!(
                "Çok seçenekli soruda tam olarak 1 doğru seçenek olmalıdır, {correct_count} bulundu"
            )));
        }
        if self.body.is_empty() {
            return Err(DomainError::Validation(
                "Soru gövdesi boş olamaz".into(),
            ));
        }
        Ok(())
    }
}
