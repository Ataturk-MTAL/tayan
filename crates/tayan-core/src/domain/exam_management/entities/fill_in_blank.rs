use serde::{Deserialize, Serialize};
use crate::domain::exam_management::value_objects::{
    OutcomeCode, QuestionBody, QuestionStats,
};
use super::question::{Points, QuestionId};

/// A single blank slot inside a FillInBlankQuestion body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blank {
    /// Must match a `BlankNode.id` in the question body.
    pub id:               String,
    /// All accepted correct values (case-normalised at scoring time).
    pub accepted_answers: Vec<String>,
    pub points:           Points,
    pub case_sensitive:   bool,
}

impl Blank {
    pub fn score_answer(&self, given: &str) -> f32 {
        let normalised = if self.case_sensitive {
            given.trim().to_owned()
        } else {
            given.trim().to_lowercase()
        };

        let matches = self.accepted_answers.iter().any(|a| {
            if self.case_sensitive {
                a.trim() == normalised
            } else {
                a.trim().to_lowercase() == normalised
            }
        });

        if matches { self.points.value() as f32 } else { 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillInBlankQuestion {
    pub id:       QuestionId,
    pub outcomes: Vec<OutcomeCode>,
    /// The body contains `ContentNode::Blank` nodes whose `id` fields reference
    /// entries in `blanks`.
    pub body:     QuestionBody,
    pub blanks:   Vec<Blank>,
    pub stats:    QuestionStats,
}

impl FillInBlankQuestion {
    pub fn total_points(&self) -> Points {
        Points(self.blanks.iter().map(|b| b.points.value()).sum())
    }

    pub fn blank_by_id(&self, id: &str) -> Option<&Blank> {
        self.blanks.iter().find(|b| b.id == id)
    }

    pub fn validate(&self) -> Result<(), crate::domain::shared::errors::DomainError> {
        use crate::domain::shared::errors::DomainError;
        if self.blanks.is_empty() {
            return Err(DomainError::Validation(
                "Fill-in-blank question must have at least one blank".into(),
            ));
        }
        // Every blank referenced in the body must exist
        let blank_ids: std::collections::HashSet<&str> =
            self.blanks.iter().map(|b| b.id.as_str()).collect();
        for node in &self.body.0 {
            if let crate::domain::exam_management::value_objects::ContentNode::Blank(b) = node {
                if !blank_ids.contains(b.id.as_str()) {
                    return Err(DomainError::Validation(format!(
                        "Body references undefined blank id `{}`",
                        b.id
                    )));
                }
            }
        }
        Ok(())
    }
}
