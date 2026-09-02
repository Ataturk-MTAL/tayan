use serde::{Deserialize, Serialize};
use crate::domain::exam_management::value_objects::{
    QuestionMeta,
    OutcomeCode, QuestionBody, QuestionStats,
};
use super::question::{Points, QuestionId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrueFalseQuestion {
    /// Ders, sınıf seviyesi ve zorluk. Eski kayıtlarda yok; serde(default)
    /// ile boş gelir ve yeniden kaydedilirken doğrulamaya takılır.
    #[serde(default)]
    pub meta:     QuestionMeta,
    pub id:             QuestionId,
    pub points:         Points,
    pub outcomes:       Vec<OutcomeCode>,
    pub body:           QuestionBody,
    /// The correct answer. `true` = statement is correct, `false` = incorrect.
    pub correct_answer: bool,
    /// Localizable labels — default "Doğru" / "Yanlış"
    pub label_true:     String,
    pub label_false:    String,
    pub stats:          QuestionStats,
}

impl TrueFalseQuestion {
    pub fn new(
        id: QuestionId,
        points: Points,
        outcomes: Vec<OutcomeCode>,
        body: QuestionBody,
        correct_answer: bool,
        meta: QuestionMeta,
    ) -> Self {
        Self {
            id,
            points,
            outcomes,
            body,
            meta,
            correct_answer,
            label_true:  "Doğru".into(),
            label_false: "Yanlış".into(),
            stats: QuestionStats::default(),
        }
    }

    pub fn score_answer(&self, given: bool) -> f32 {
        if given == self.correct_answer {
            self.points.value() as f32
        } else {
            0.0
        }
    }

    pub fn correct_label(&self) -> &str {
        if self.correct_answer { &self.label_true } else { &self.label_false }
    }

    pub fn validate(&self) -> Result<(), crate::domain::shared::errors::DomainError> {
        if self.body.is_empty() {
            return Err(crate::domain::shared::errors::DomainError::Validation(
                "Doğru/Yanlış soru gövdesi boş olamaz".into(),
            ));
        }
        Ok(())
    }
}
