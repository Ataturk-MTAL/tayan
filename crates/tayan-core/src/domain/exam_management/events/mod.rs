use crate::domain::exam_management::{
    aggregates::{ExamId, QuestionBankId},
    entities::QuestionId,
};

/// Events emitted by the `Exam` aggregate.
#[derive(Debug, Clone)]
pub enum ExamEvent {
    Published { exam_id: ExamId },
}

/// Events emitted by the `QuestionBank` aggregate.
#[derive(Debug, Clone)]
pub enum BankEvent {
    QuestionAdded {
        bank_id:       QuestionBankId,
        question_id:   QuestionId,
        question_type: &'static str,
    },
    QuestionRemoved {
        bank_id:     QuestionBankId,
        question_id: QuestionId,
    },
    StatsUpdated {
        question_id:      QuestionId,
        new_score:        f32,
        difficulty_index: f32,
    },
}
