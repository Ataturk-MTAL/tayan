use tayan_core::domain::exam_management::aggregates::Exam;
use tayan_core::domain::exam_management::entities::question::Question;
use tayan_core::domain::shared::to_typst::{ToTypst, TypstContext};

#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    #[error("Question not found: {0}")]
    QuestionNotFound(String),
    #[error("Render error: {0}")]
    Render(#[from] anyhow::Error),
}

/// Converts an `Exam` + resolved questions into a complete Typst source document.
pub struct TypstGenerator;

impl TypstGenerator {
    pub fn generate_exam(
        exam:      &Exam,
        questions: &[Question],
        ctx:       TypstContext,
    ) -> Result<String, CompilerError> {
        let mut out = String::new();

        out.push_str(PREAMBLE);
        out.push_str(&format!(
            "#show: exam-setup.with(\n  title: \"{}\",\n  subject: \"{}\",\n  duration: {},\n)\n\n",
            escape_typst(&exam.meta.title),
            escape_typst(&exam.meta.subject),
            exam.meta.duration_min,
        ));

        for (i, q) in questions.iter().enumerate() {
            let q_ctx = ctx.clone().with_number((i + 1) as u32);
            out.push_str(&q.to_typst(&q_ctx));
            out.push('\n');
        }

        Ok(out)
    }
}

fn escape_typst(s: &str) -> String {
    s.replace('"', "\\\"").replace('#', "\\#")
}

const PREAMBLE: &str = r#"#import "@preview/mitex:0.2.4": mi, mi-block
#import "@preview/chem-par:0.0.1": ce
#import "@preview/physica:0.9.4": *
#import "@preview/metro:0.3.0": *

"#;
