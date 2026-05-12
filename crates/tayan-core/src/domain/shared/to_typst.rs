/// Context passed during Typst code generation — controls output mode.
#[derive(Debug, Clone)]
pub struct TypstContext {
    /// Generate answer key version (shows correct answers).
    pub answer_key: bool,
    /// Shuffle multiple-choice options.
    pub shuffle: bool,
    /// 1-based question number to prefix in output.
    pub question_number: Option<u32>,
}

impl Default for TypstContext {
    fn default() -> Self {
        Self { answer_key: false, shuffle: false, question_number: None }
    }
}

impl TypstContext {
    pub fn answer_key() -> Self {
        Self { answer_key: true, shuffle: false, question_number: None }
    }

    pub fn with_number(mut self, n: u32) -> Self {
        self.question_number = Some(n);
        self
    }
}

/// Every domain type that can emit Typst source code implements this.
pub trait ToTypst {
    fn to_typst(&self, ctx: &TypstContext) -> String;
}
