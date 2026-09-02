use crate::domain::shared::to_typst::{seed_from, ToTypst, TypstContext};
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

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert a Tauri `asset://localhost/%2F...` URL to an absolute fs path.
/// If `src` is already a plain path it is returned unchanged.
fn asset_url_to_fs_path(src: &str) -> String {
    let encoded = src
        .strip_prefix("asset://localhost/")
        .or_else(|| src.strip_prefix("asset:/localhost/"))
        .unwrap_or(src);
    percent_decode(encoded)
}

fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut iter = s.chars();
    while let Some(c) = iter.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let h1 = iter.next();
        let h2 = iter.next();
        match (h1, h2) {
            (Some(h1), Some(h2)) => {
                let hex = format!("{h1}{h2}");
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    out.push(byte as char);
                } else {
                    out.push('%');
                    out.push(h1);
                    out.push(h2);
                }
            }
            _ => out.push('%'),
        }
    }
    out
}

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
                let typst = crate::domain::shared::latex_to_typst::convert(&n.raw);
                match n.display {
                    MathDisplay::Inline => format!("${typst}$"),
                    MathDisplay::Block  => format!("\n$ {typst} $\n"),
                }
            }
            ContentNode::Chem(n) => {
                // Emit as monospace; chem rendering handled in future macro layer
                let raw = esc(&n.raw);
                format!("`{raw}`")
            }
            ContentNode::Image(n) => {
                let w    = n.width.as_deref().unwrap_or("80%");
                let path = asset_url_to_fs_path(&n.src);
                format!("#image(\"{}\", width: {w})", path.replace('"', "\\\""))
            }
            ContentNode::TypstRaw(n) => n.code.clone(),
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

/// Deterministik karışık sıra.
///
/// Rastgelelik kullanılmaz: aynı tohum her zaman aynı permütasyonu verir.
/// Sebebi, yeniden basılan kâğıdın öncekiyle birebir aynı çıkması gerekmesidir;
/// aksi halde öğrenciye dağıtılan kâğıt ile elindeki cevap anahtarı ayrı düşer.
fn permutation(count: usize, seed: u64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..count).collect();
    let mut state = seed | 1; // xorshift sıfır tohumda kilitlenir

    // Fisher-Yates, xorshift64 ile
    let mut i = count;
    while i > 1 {
        i -= 1;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let j = (state % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    order
}

impl ToTypst for MultipleChoiceQuestion {
    fn to_typst(&self, ctx: &TypstContext) -> String {
        let num = ctx.question_number.map(|n| format!("*{n}.* ")).unwrap_or_default();
        let body = self.body.to_typst(ctx);
        let pts  = self.points.value();

        // Gövde kendi şık kalıbını taşıyorsa şıkları BİR KEZ o dizer.
        //
        // Bu ayrım olmadan şıklar iki kez basılıyordu: önce gövdedeki
        // #secenekler çağrısı, sonra buradaki #grid. Kâğıtta aynı beş şık alt
        // alta iki kez çıkıyordu.
        if body.contains("#secenekler(") {
            let body = inject_option_args(&body, self, ctx);
            return format!(
                "#block(width: 100%, breakable: false)[\n  {num}({pts} puan) {body}\n]\n"
            );
        }

        // Eski gövdeler (zengin metin editöründen kalanlar) şıkları taşımaz;
        // onlar için ızgara burada üretilir.
        let options: Vec<String> = self.options.iter().map(|opt| {
            let opt_body = opt.body.to_typst(ctx);
            if ctx.answer_key && opt.correct {
                format!("  [*{}\\) {}* #tik()]", opt.id, opt_body)
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

/// `#secenekler(` çağrısına dizgi anına ait argümanları enjekte eder.
///
/// Kaynak DEĞİŞTİRİLMEZ; yalnızca bu baskı için üretilen metne eklenir.
/// Öğretmenin yazdığı dosya olduğu gibi kalır.
fn inject_option_args(body: &str, q: &MultipleChoiceQuestion, ctx: &TypstContext) -> String {
    const MARKER: &str = "#secenekler(";

    let Some(at) = body.find(MARKER) else { return body.to_string() };
    let insert_at = at + MARKER.len();
    let args_tail = &body[insert_at..];

    let mut injected = String::new();

    // Öğretmen elle yazmışsa üstüne yazma: Typst'te aynı adlı argümanın iki kez
    // verilmesi hatadır.
    if q.shuffle && !args_tail.starts_with("sira:") && !args_tail.contains("sira:") {
        let seed = ctx.shuffle_seed ^ seed_from(&q.id.0.to_string());
        let order = permutation(q.options.len(), seed);
        let list: Vec<String> = order.iter().map(|i| i.to_string()).collect();
        injected.push_str(&format!("sira: ({},), ", list.join(", ")));
    }

    if ctx.answer_key && !args_tail.contains("anahtar:") {
        injected.push_str("anahtar: true, ");
    }

    if injected.is_empty() {
        return body.to_string();
    }

    format!("{}{}{}", &body[..insert_at], injected, args_tail)
}

// ── TrueFalseQuestion ─────────────────────────────────────────────────────────

impl ToTypst for TrueFalseQuestion {
    fn to_typst(&self, ctx: &TypstContext) -> String {
        let num  = ctx.question_number.map(|n| format!("*{n}.* ")).unwrap_or_default();
        let body = self.body.to_typst(ctx);
        let pts  = self.points.value();

        let (t_mark, f_mark) = if ctx.answer_key {
            if self.correct_answer {
                ("#cb(checked: true)", "#cb()")
            } else {
                ("#cb()", "#cb(checked: true)")
            }
        } else {
            ("#cb()", "#cb()")
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

        // Gövde kendi cevap alanını taşıyorsa alan BİR KEZ o çizer.
        //
        // Şıklardaki `#secenekler(` korumasının aynısı: öğretmen gövdeye
        // `#cevap-alani(satir: 10, bicim: "kareli")` yazdığında hem kareli
        // alan hem de buradaki çizgiler basılıyordu — kâğıtta iki cevap alanı.
        // Gövdedeki çağrı biçimi de (kareli/çizgili/boş) seçtiği için ondan
        // vazgeçip domain varsayılanını basmak, öğretmenin kararını silmek olur.
        let body_has_answer_space = body.contains("#cevap-alani(");

        let answer_space = if body_has_answer_space {
            String::new()
        } else {
            match &self.answer_space {
            // Varyantın adı `Lines` ama önceden boş bir dikdörtgen basıyordu.
            // Öğretmen "6 satır" seçtiğinde kâğıtta çizgi bekler, boş kutu
            // değil; önsözdeki #cevap-alani zaten doğrusunu çiziyor.
            AnswerSpace::Lines(n) => format!("#cevap-alani(satir: {n})"),
            AnswerSpace::HeightCm(h) => {
                format!("#rect(width: 100%, height: {:.1}cm, stroke: 0.5pt)[]", h)
            }
            AnswerSpace::Grid { rows, cols } => {
                format!(
                    "#grid(columns: {}, rows: {}, ..range({}).map(_ => rect(width: 100%, height: 0.8cm, stroke: 0.3pt)[]))",
                    cols, rows, rows * cols
                )
            }
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
