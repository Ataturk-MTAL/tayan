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
        out.push_str(&exam_header(exam));

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

fn exam_header(exam: &Exam) -> String {
    let title   = escape_typst(&exam.meta.title);
    let subject = escape_typst(&exam.meta.subject);
    let class   = escape_typst(&exam.meta.classroom);
    let teacher = escape_typst(&exam.meta.teacher);
    let dur     = exam.meta.duration_min;
    format!(
        "#align(center)[
  #text(weight: \"bold\", size: 14pt)[{title}]
  #linebreak()
  #text(size: 10pt)[{subject} #h(1em) | #h(1em) {class} #h(1em) | #h(1em) {teacher}]
  #linebreak()
  #text(size: 9pt, fill: gray)[Süre: {dur} dk]
]
#line(length: 100%)
#v(0.4cm)
#grid(columns: (1fr, 1fr, 1fr),
  [Ad Soyad: #underline(offset: 2pt)[#h(5cm)]],
  [No: #underline(offset: 2pt)[#h(2cm)]],
  [Puan: #underline(offset: 2pt)[#h(2cm)]],
)
#v(0.6cm)

"
    )
}

const PREAMBLE: &str = "#set page(paper: \"a4\", margin: (x: 2cm, y: 2.5cm))
#set text(lang: \"tr\", size: 11pt, font: \"Linux Libertine\")
#set par(leading: 0.75em, justify: false)
#set list(marker: ([--], [•]))

#let blank(width: 4cm) = box(width: width, baseline: 20%, stroke: (bottom: 0.5pt + black), height: 1.1em)
#let cb(checked: false) = if checked [ ☑ ] else [ ☐ ]

";
