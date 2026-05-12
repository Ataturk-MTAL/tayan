//! Ports (output interfaces) — implemented by tayan-db and tayan-compiler.
#![allow(async_fn_in_trait)]

use crate::domain::{
    assessment::aggregates::ExamResult,
    exam_management::{
        aggregates::{Exam, ExamId, QuestionBank},
        entities::QuestionId,
    },
    student_management::{Classroom, ClassroomId, Student, StudentId},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Storage error: {0}")]
    Storage(#[from] anyhow::Error),
}

// Each trait is async-trait style — implementors use async fn freely (Rust 2024+)

pub trait ExamRepository: Send + Sync {
    async fn save(&self, exam: &Exam) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: &ExamId) -> Result<Exam, RepositoryError>;
    async fn list(&self, page: u32, per_page: u32) -> Result<Vec<Exam>, RepositoryError>;
    async fn delete(&self, id: &ExamId) -> Result<(), RepositoryError>;
}

pub trait QuestionBankRepository: Send + Sync {
    async fn save(&self, bank: &QuestionBank) -> Result<(), RepositoryError>;
    async fn load(&self) -> Result<QuestionBank, RepositoryError>;
    async fn find_question(&self, id: &QuestionId)
        -> Result<Option<crate::domain::exam_management::entities::Question>, RepositoryError>;
}

pub trait StudentRepository: Send + Sync {
    async fn save_student(&self, student: &Student) -> Result<(), RepositoryError>;
    async fn find_student(&self, id: &StudentId) -> Result<Student, RepositoryError>;
    async fn list_by_classroom(&self, classroom_id: &ClassroomId)
        -> Result<Vec<Student>, RepositoryError>;
}

pub trait ClassroomRepository: Send + Sync {
    async fn save(&self, classroom: &Classroom) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: &ClassroomId) -> Result<Classroom, RepositoryError>;
    async fn list(&self) -> Result<Vec<Classroom>, RepositoryError>;
}

pub trait ExamResultRepository: Send + Sync {
    async fn save(&self, result: &ExamResult) -> Result<(), RepositoryError>;
    async fn find_by_exam_and_student(
        &self, exam_id: &ExamId, student_id: &StudentId,
    ) -> Result<Option<ExamResult>, RepositoryError>;
    async fn list_by_exam(&self, exam_id: &ExamId) -> Result<Vec<ExamResult>, RepositoryError>;
}

pub trait PdfCompiler: Send + Sync {
    async fn compile_exam(&self, typst_source: &str) -> Result<Vec<u8>, anyhow::Error>;
}
