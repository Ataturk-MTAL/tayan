use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::exam_management::value_objects::{
    OutcomeCode, QuestionBody, QuestionStats,
};
use super::{
    multiple_choice::MultipleChoiceQuestion,
    true_false::TrueFalseQuestion,
    fill_in_blank::FillInBlankQuestion,
    classic::ClassicQuestion,
};

/// Typed question ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestionId(pub Uuid);

impl QuestionId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl Default for QuestionId {
    fn default() -> Self { Self::new() }
}

impl std::fmt::Display for QuestionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Scored points value — must be positive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Points(pub u32);

impl Points {
    pub fn new(v: u32) -> Self { Self(v) }
    pub fn value(self) -> u32 { self.0 }
}

impl std::fmt::Display for Points {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tagged union of all question variants.
/// Stored in `QuestionBank` — reusable across multiple exams.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "question_type", rename_all = "snake_case")]
pub enum Question {
    MultipleChoice(MultipleChoiceQuestion),
    TrueFalse(TrueFalseQuestion),
    FillInBlank(FillInBlankQuestion),
    Classic(ClassicQuestion),
}

impl Question {
    pub fn id(&self) -> &QuestionId {
        match self {
            Question::MultipleChoice(q) => &q.id,
            Question::TrueFalse(q)      => &q.id,
            Question::FillInBlank(q)    => &q.id,
            Question::Classic(q)        => &q.id,
        }
    }

    pub fn points(&self) -> Points {
        match self {
            Question::MultipleChoice(q) => q.points,
            Question::TrueFalse(q)      => q.points,
            Question::FillInBlank(q)    => q.total_points(),
            Question::Classic(q)        => q.points,
        }
    }

    pub fn outcomes(&self) -> &[OutcomeCode] {
        match self {
            Question::MultipleChoice(q) => &q.outcomes,
            Question::TrueFalse(q)      => &q.outcomes,
            Question::FillInBlank(q)    => &q.outcomes,
            Question::Classic(q)        => &q.outcomes,
        }
    }

    pub fn body(&self) -> &QuestionBody {
        match self {
            Question::MultipleChoice(q) => &q.body,
            Question::TrueFalse(q)      => &q.body,
            Question::FillInBlank(q)    => &q.body,
            Question::Classic(q)        => &q.body,
        }
    }

    pub fn stats(&self) -> &QuestionStats {
        match self {
            Question::MultipleChoice(q) => &q.stats,
            Question::TrueFalse(q)      => &q.stats,
            Question::FillInBlank(q)    => &q.stats,
            Question::Classic(q)        => &q.stats,
        }
    }

    pub fn stats_mut(&mut self) -> &mut QuestionStats {
        match self {
            Question::MultipleChoice(q) => &mut q.stats,
            Question::TrueFalse(q)      => &mut q.stats,
            Question::FillInBlank(q)    => &mut q.stats,
            Question::Classic(q)        => &mut q.stats,
        }
    }

    pub fn question_type_label(&self) -> &'static str {
        match self {
            Question::MultipleChoice(_) => "multiple_choice",
            Question::TrueFalse(_)      => "true_false",
            Question::FillInBlank(_)    => "fill_in_blank",
            Question::Classic(_)        => "classic",
        }
    }

    pub fn validate(&self) -> Result<(), crate::domain::shared::errors::DomainError> {
        match self {
            Question::MultipleChoice(q) => q.validate(),
            Question::TrueFalse(q)      => q.validate(),
            Question::FillInBlank(q)    => q.validate(),
            Question::Classic(q)        => q.validate(),
        }
    }
}
