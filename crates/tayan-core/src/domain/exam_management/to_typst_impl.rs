use crate::domain::shared::to_typst::{ToTypst, TypstContext};
use crate::domain::exam_management::value_objects::content_node::{
    ContentNode, MathDisplay, QuestionBody,
};
use crate::domain::exam_management::entities::{
    classic::{AnswerSpace, ClassicQuestion},
    fill_in_blank::FillInBlankQuestion,
    multiple_choice::MultipleChoiceQuestion,
    question::Question,
    true_false::TrueFalseQuestion,
};

// ── Helper ────────────────────────────────────────────────────────────────────

/// Escape Typst markup special characters in plain text fragments.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for ch in s.chars() {
        match ch {
            '#' | '@' | '$' | '*' | '_' | '`' | '~' | '<' | '>' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            ch => out.push(ch),
        }
    }
    out
}

// ── ContentNode ───────────────────────────────────────────────────────────────

impl ToTypst for ContentNode {
    fn to_typst(&self, _ctx: &TypstContext) -> String {
        match self {
            ContentNode::Text(n) => {
                let t = esc(&n.text);
                match (n.style.bold, n.style.italic, n.style.underline, n.style.strikethrough) {
                    (true, true, _, _)  => format!("*_{t}_*"),
                    (true, false, _, _) => format!("*{t}*"),
                    (false, true, _, _) => format!("_{t}_"),
                    (_, _, true, _)     => format!("#underline[{t}]"),
                    (_, _, _, true)     => format!("#strike[{t}]"),
                    _                   => t,
                }
            }
            ContentNode::Math(n) => {
                // Typst native math: $...$ inline, $ ... $ (with spaces) for block
                let raw = &n.raw;
                match n.display {
                    MathDisplay::Inline => format!("${raw}$"),
                    MathDisplay::Block  => format!("\n$ {raw} $\n"),
                }
            }
            ContentNode::Chem(n) => {
                // Emit as monospace; chem rendering handled in future macro layer
                let raw = esc(&n.raw);
                format!("`{raw}`")
            }
            ContentNode::Image(n) => {
                let w = n.width.as_deref().unwrap_or("80%");
                format!("#image(\"{}\", width: {})", n.src, w)
            }
            ContentNode::Blank(n) => {
                let w = n.width.as_deref().unwrap_or("4cm");
                format!("#box(width: {w}, stroke: (bottom: 0.5pt))[#h(1fr)]")
            }
            ContentNode::Newline => "\n\n".into(),
        }
    }
}

impl ToTypst for QuestionBody {
    fn to_typst(&self, ctx: &TypstContext) -> String {
        self.0.iter().map(|n| n.to_typst(ctx)).collect()
    }
}

// ── MultipleChoiceQuestion ────────────────────────────────────────────────────

impl ToTypst for MultipleChoiceQuestion {
    fn to_typst(&self, ctx: &TypstContext) -> String {
        let num = ctx.question_number.map(|n| format!("*{n}.* ")).unwrap_or_default();
        let body = self.body.to_typst(ctx);
        let pts  = self.points.value();

        let options: Vec<String> = self.options.iter().map(|opt| {
            let opt_body = opt.body.to_typst(ctx);
            if ctx.answer_key && opt.correct {
                format!("  [*{}\\) {}* ✓]", opt.id, opt_body)
            } else {
                format!("  [{}\\) {}]", opt.id, opt_body)
            }
        }).collect();

        let cols = if self.options.len() <= 2 { "1fr, 1fr" } else { "1fr, 1fr, 1fr, 1fr" };
        let grid_rows = options.join(",\n");

        format!(
            "#block(width: 100%, breakable: false)[\n  {num}({pts} puan) {body}\n\n  \
             #grid(columns: ({cols}),\n{grid_rows}\n  )\n]\n"
        )
    }
}

// ── TrueFalseQuestion ─────────────────────────────────────────────────────────

impl ToTypst for TrueFalseQuestion {
    fn to_typst(&self, ctx: &TypstContext) -> String {
        let num  = ctx.question_number.map(|n| format!("*{n}.* ")).unwrap_or_default();
        let body = self.body.to_typst(ctx);
        let pts  = self.points.value();

        let (t_mark, f_mark) = if ctx.answer_key {
            if self.correct_answer {
                ("☑", "☐")
            } else {
                ("☐", "☑")
            }
        } else {
            ("☐", "☐")
        };

        format!(
            "#block(width: 100%, breakable: false)[\n  {num}({pts} puan) {body}\n\n  \
             #h(2em) {t_mark} {} #h(2em) {f_mark} {}\n]\n",
            esc(&self.label_true),
            esc(&self.label_false),
        )
    }
}

// ── FillInBlankQuestion ───────────────────────────────────────────────────────

impl ToTypst for FillInBlankQuestion {
    fn to_typst(&self, ctx: &TypstContext) -> String {
        let num  = ctx.question_number.map(|n| format!("*{n}.* ")).unwrap_or_default();
        let body = self.body.to_typst(ctx);

        let answer_key_note = if ctx.answer_key {
            let answers: Vec<String> = self.blanks.iter().map(|b| {
                let ans = b.accepted_answers.first().map(|s| s.as_str()).unwrap_or("—");
                format!("[{}]: {}", b.id, esc(ans))
            }).collect();
            format!("\n  #text(fill: red.darken(20%))[(Cevaplar: {})]", answers.join(", "))
        } else {
            String::new()
        };

        format!(
            "#block(width: 100%, breakable: false)[\n  {num}{body}{answer_key_note}\n]\n"
        )
    }
}

// ── ClassicQuestion ───────────────────────────────────────────────────────────

impl ToTypst for ClassicQuestion {
    fn to_typst(&self, ctx: &TypstContext) -> String {
        let num  = ctx.question_number.map(|n| format!("*{n}.* ")).unwrap_or_default();
        let body = self.body.to_typst(ctx);
        let pts  = self.points.value();

        let answer_space = match &self.answer_space {
            AnswerSpace::Lines(n) => {
                let line_height = 0.8_f32 * (*n as f32);
                format!("#rect(width: 100%, height: {:.1}cm, stroke: 0.5pt)[]", line_height)
            }
            AnswerSpace::HeightCm(h) => {
                format!("#rect(width: 100%, height: {:.1}cm, stroke: 0.5pt)[]", h)
            }
            AnswerSpace::Grid { rows, cols } => {
                format!(
                    "#grid(columns: {}, rows: {}, ..range({}).map(_ => rect(width: 100%, height: 0.8cm, stroke: 0.3pt)[]))",
                    cols, rows, rows * cols
                )
            }
        };

        let rubric_block = if ctx.answer_key && !self.rubric.is_empty() {
            let items: Vec<String> = self.rubric.iter().map(|r| {
                format!("  - {} ({} puan)", esc(&r.criterion), r.points.value())
            }).collect();
            format!("\n\n  #text(fill: blue.darken(20%))[*Dereceli puanlama:*\n{}]", items.join("\n"))
        } else {
            String::new()
        };

        let sample = if ctx.answer_key {
            if let Some(sa) = &self.sample_answer {
                format!("\n\n  #text(fill: green.darken(20%))[*Örnek cevap:* {}]", sa.to_typst(ctx))
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        format!(
            "#block(width: 100%, breakable: false)[\n  {num}({pts} puan) {body}{rubric_block}{sample}\n\n  {answer_space}\n]\n"
        )
    }
}

// ── Question (dispatch) ───────────────────────────────────────────────────────

impl ToTypst for Question {
    fn to_typst(&self, ctx: &TypstContext) -> String {
        match self {
            Question::MultipleChoice(q) => q.to_typst(ctx),
            Question::TrueFalse(q)      => q.to_typst(ctx),
            Question::FillInBlank(q)    => q.to_typst(ctx),
            Question::Classic(q)        => q.to_typst(ctx),
        }
    }
}
