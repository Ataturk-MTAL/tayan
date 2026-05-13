use serde::{Deserialize, Serialize};

// ── Math ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MathDisplay {
    Inline,
    Block,
}

/// LaTeX math fragment — stored as raw LaTeX, rendered via KaTeX in UI
/// and via MiTeX (#mi) in the Typst PDF pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MathNode {
    pub raw:     String,
    pub display: MathDisplay,
}

// ── Chemistry ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChemFlavor {
    /// Simple formula — H₂SO₄, C₆H₁₂O₆ etc.
    Formula,
    /// Skeletal / structural diagram
    Structural,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChemNode {
    pub raw:    String,
    pub flavor: ChemFlavor,
}

// ── Image ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageNode {
    /// Path relative to the exam's asset directory.
    pub src:   String,
    pub alt:   String,
    /// Optional width override, e.g. "40%", "200pt".
    pub width: Option<String>,
}

// ── Fill-in-blank placeholder ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlankNode {
    /// Must match a `Blank.id` in the parent `FillInBlankQuestion`.
    pub id:    String,
    pub width: Option<String>,
}

// ── Text ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TextStyle {
    pub bold:          bool,
    pub italic:        bool,
    pub underline:     bool,
    pub strikethrough: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextNode {
    pub text:  String,
    pub style: TextStyle,
}

impl TextNode {
    pub fn plain(text: impl Into<String>) -> Self {
        Self { text: text.into(), style: TextStyle::default() }
    }
}

// ── Union ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentNode {
    Text(TextNode),
    Math(MathNode),
    Chem(ChemNode),
    Image(ImageNode),
    Blank(BlankNode),
    Newline,
}

impl ContentNode {
    pub fn text(s: impl Into<String>) -> Self {
        ContentNode::Text(TextNode::plain(s))
    }

    pub fn math_inline(raw: impl Into<String>) -> Self {
        ContentNode::Math(MathNode { raw: raw.into(), display: MathDisplay::Inline })
    }

    pub fn math_block(raw: impl Into<String>) -> Self {
        ContentNode::Math(MathNode { raw: raw.into(), display: MathDisplay::Block })
    }

    pub fn chem_formula(raw: impl Into<String>) -> Self {
        ContentNode::Chem(ChemNode { raw: raw.into(), flavor: ChemFlavor::Formula })
    }

    pub fn chem_structural(raw: impl Into<String>) -> Self {
        ContentNode::Chem(ChemNode { raw: raw.into(), flavor: ChemFlavor::Structural })
    }

    pub fn image(src: impl Into<String>, alt: impl Into<String>) -> Self {
        ContentNode::Image(ImageNode { src: src.into(), alt: alt.into(), width: None })
    }

    pub fn blank(id: impl Into<String>) -> Self {
        ContentNode::Blank(BlankNode { id: id.into(), width: None })
    }
}

/// Newtype for a sequence of content nodes — the body of any question.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QuestionBody(pub Vec<ContentNode>);

impl QuestionBody {
    pub fn new(nodes: Vec<ContentNode>) -> Self {
        Self(nodes)
    }

    pub fn push(&mut self, node: ContentNode) {
        self.0.push(node);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
