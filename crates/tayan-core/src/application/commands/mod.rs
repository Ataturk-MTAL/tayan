use serde::{Deserialize, Serialize};
use crate::domain::exam_management::{
    aggregates::{ExamId, ExamMeta},
    entities::{AnswerSpace, Blank, QuestionId, QuestionOption, RubricItem},
    value_objects::{QuestionBody, QuestionMeta},
};
use crate::domain::student_management::{ClassroomId, StudentId};
use crate::domain::assessment::aggregates::QuestionAnswer;

// ── Exam commands ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateExam {
    pub meta: ExamMeta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddQuestionToExam {
    pub exam_id:     ExamId,
    pub question_id: QuestionId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishExam {
    pub exam_id: ExamId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReorderQuestion {
    pub exam_id:     ExamId,
    pub question_id: QuestionId,
    pub new_order:   u32,
}

// ── Question bank commands ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMultipleChoiceQuestion {
    /// Ders, sınıf seviyesi, zorluk. Ders ve seviye zorunlu.
    pub meta:     QuestionMeta,
    pub points:   u32,
    pub outcomes: Vec<String>,
    pub body:     QuestionBody,
    pub options:  Vec<QuestionOption>,
    pub shuffle:  bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddTrueFalseQuestion {
    /// Ders, sınıf seviyesi, zorluk. Ders ve seviye zorunlu.
    pub meta:     QuestionMeta,
    pub points:         u32,
    pub outcomes:       Vec<String>,
    pub body:           QuestionBody,
    pub correct_answer: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddFillInBlankQuestion {
    /// Ders, sınıf seviyesi, zorluk. Ders ve seviye zorunlu.
    pub meta:     QuestionMeta,
    pub outcomes: Vec<String>,
    pub body:     QuestionBody,
    pub blanks:   Vec<Blank>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddClassicQuestion {
    /// Ders, sınıf seviyesi, zorluk. Ders ve seviye zorunlu.
    pub meta:     QuestionMeta,
    pub points:       u32,
    pub outcomes:     Vec<String>,
    pub body:         QuestionBody,
    /// Yalnız cevap anahtarına basılan örnek çözüm.
    ///
    /// `serde(default)`: alan komut yapısında YOKTU ve işleyici sabit `None`
    /// yazıyordu — cevap anahtarı "Örnek cevap:" basmaya hazır olduğu hâlde
    /// öğretmen hiçbir yerden giremiyordu. Varsayılan, alanı göndermeyen eski
    /// çağrıların bozulmamasını sağlıyor.
    #[serde(default)]
    pub sample_answer: Option<QuestionBody>,
    pub rubric:       Vec<RubricItem>,
    pub answer_space: AnswerSpace,
}

// ── Assessment commands ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct EnterExamResults {
    pub exam_id:     ExamId,
    pub student_id:  StudentId,
    pub answers:     Vec<QuestionAnswer>,
    pub total_max:   f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GradeClassicAnswer {
    pub exam_id:     ExamId,
    pub student_id:  StudentId,
    pub question_id: QuestionId,
    pub points:      f32,
}

// ── Student commands ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateClassroom {
    pub name:   String,
    pub grade:  u8,
    pub branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddStudent {
    pub classroom_id: ClassroomId,
    pub number:       String,
    pub first_name:   String,
    pub last_name:    String,
}
