use serde::{Deserialize, Serialize};
use crate::domain::exam_management::{
    aggregates::ExamId,
    entities::QuestionId,
};
use crate::domain::student_management::{ClassroomId, StudentId};

// ── Exam queries ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct GetExam {
    pub exam_id: ExamId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListExams {
    pub page:     u32,
    pub per_page: u32,
}

// ── Question bank queries ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct GetQuestion {
    pub question_id: QuestionId,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SearchQuestions {
    pub outcome:       Option<String>,
    pub question_type: Option<String>,
    pub min_score:     Option<f32>,
    pub max_score:     Option<f32>,
    pub tag:           Option<String>,
    pub page:          u32,
    pub per_page:      u32,
}

// ── Assessment queries ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct GetClassAnalysis {
    pub exam_id:      ExamId,
    pub classroom_id: ClassroomId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStudentAnalysis {
    pub exam_id:    ExamId,
    pub student_id: StudentId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOutcomeAnalysis {
    pub classroom_id: ClassroomId,
    pub outcome:      String,
}

// ── Report queries ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateExamPdf {
    pub exam_id:    ExamId,
    pub answer_key: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateIndividualReport {
    pub exam_id:    ExamId,
    pub student_id: StudentId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateClassReport {
    pub exam_id:      ExamId,
    pub classroom_id: ClassroomId,
}
