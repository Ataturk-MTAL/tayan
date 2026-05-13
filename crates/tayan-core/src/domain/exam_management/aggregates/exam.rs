use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::shared::{aggregate_root::EventQueue, errors::DomainError};
use crate::domain::exam_management::{
    entities::QuestionId,
    events::ExamEvent,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExamId(pub Uuid);

impl ExamId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl Default for ExamId {
    fn default() -> Self { Self::new() }
}

impl std::fmt::Display for ExamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExamStatus {
    Draft,
    Published,
    Archived,
}

/// Ordered reference to a banked question within an exam.
/// The actual question content lives in QuestionBank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamQuestionRef {
    pub question_id:      QuestionId,
    pub display_order:    u32,
    /// Optional per-exam point override (otherwise uses question.points())
    pub points_override:  Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamMeta {
    pub title:        String,
    pub subject:      String,
    pub classroom:    String,
    pub teacher:      String,
    pub duration_min: u32,
    pub date:         NaiveDate,
    pub instructions: Option<String>,
}

/// An exam is a named, ordered collection of question references with metadata.
/// It does NOT own questions — it references them in QuestionBank.
#[derive(Debug, Serialize, Deserialize)]
pub struct Exam {
    pub id:         ExamId,
    pub meta:       ExamMeta,
    pub questions:  Vec<ExamQuestionRef>,
    pub status:     ExamStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip)]
    events: EventQueue<ExamEvent>,
}

impl Exam {
    pub fn new(meta: ExamMeta) -> Self {
        let now = Utc::now();
        Self {
            id:         ExamId::new(),
            meta,
            questions:  vec![],
            status:     ExamStatus::Draft,
            created_at: now,
            updated_at: now,
            events:     EventQueue::default(),
        }
    }

    pub fn id(&self) -> &ExamId { &self.id }

    pub fn add_question_ref(&mut self, question_id: QuestionId) {
        let order = self.questions.len() as u32 + 1;
        self.questions.push(ExamQuestionRef {
            question_id,
            display_order: order,
            points_override: None,
        });
        self.touch();
    }

    pub fn remove_question_ref(&mut self, question_id: &QuestionId) -> Result<(), DomainError> {
        let pos = self
            .questions
            .iter()
            .position(|r| &r.question_id == question_id)
            .ok_or_else(|| DomainError::NotFound {
                entity: "ExamQuestionRef",
                id:     question_id.to_string(),
            })?;
        self.questions.remove(pos);
        self.reorder();
        self.touch();
        Ok(())
    }

    pub fn reorder_question(&mut self, question_id: &QuestionId, new_order: u32) -> Result<(), DomainError> {
        let pos = self
            .questions
            .iter()
            .position(|r| &r.question_id == question_id)
            .ok_or_else(|| DomainError::NotFound {
                entity: "ExamQuestionRef",
                id:     question_id.to_string(),
            })?;
        let item = self.questions.remove(pos);
        let insert_at = ((new_order as usize).saturating_sub(1)).min(self.questions.len());
        self.questions.insert(insert_at, item);
        self.reorder();
        self.touch();
        Ok(())
    }

    pub fn publish(&mut self) -> Result<(), DomainError> {
        if self.questions.is_empty() {
            return Err(DomainError::InvariantViolation(
                "Sorusu olmayan sınav yayınlanamaz".into(),
            ));
        }
        self.status = ExamStatus::Published;
        self.events.push(ExamEvent::Published { exam_id: self.id.clone() });
        self.touch();
        Ok(())
    }

    pub fn total_points(&self, lookup: impl Fn(&QuestionId) -> Option<u32>) -> u32 {
        self.questions
            .iter()
            .map(|r| r.points_override.or_else(|| lookup(&r.question_id)).unwrap_or(0))
            .sum()
    }

    pub fn drain_events(&mut self) -> Vec<ExamEvent> {
        self.events.drain()
    }

    fn reorder(&mut self) {
        for (i, r) in self.questions.iter_mut().enumerate() {
            r.display_order = i as u32 + 1;
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}
