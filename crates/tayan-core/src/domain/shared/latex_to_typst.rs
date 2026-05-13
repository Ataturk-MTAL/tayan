/// Convert a LaTeX math string to Typst math syntax.
///
/// Covers the symbols available in RichBodyEditor's equation palette plus
/// common structural commands (frac, sqrt, vec, mathbb).
pub fn convert(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len() + 8);
    let mut i = 0;
    while i < chars.len() {
        i = step(&chars, i, &mut out);
    }
    out
}

fn step(chars: &[char], mut i: usize, out: &mut String) -> usize {
    match chars[i] {
        '\\' => {
            i += 1;
            if i >= chars.len() { out.push('\\'); return i; }

            // Non-alpha escapes
            match chars[i] {
                ',' | ';' => { i += 1; out.push_str("space.thin"); return i; }
                '!' => { return i + 1; }
                '\\' => { i += 1; out.push_str("\\ "); return i; }
                ' ' => { i += 1; out.push(' '); return i; }
                _ => {}
            }

            // Read command name
            let start = i;
            while i < chars.len() && chars[i].is_alphabetic() { i += 1; }
            let cmd: &str = &chars[start..i].iter().collect::<String>();

            i = emit_cmd(cmd, chars, i, out);
        }

        // ^{...} and _{...}: convert braces to parens for multi-char groups
        '^' | '_' => {
            let mark = chars[i];
            out.push(mark);
            i += 1;
            if i < chars.len() && chars[i] == '{' {
                i += 1; // skip {
                let (inner, next) = read_brace_content(chars, i);
                let converted = convert(&inner);
                let char_count = converted.chars().count();
                if char_count <= 1 {
                    out.push_str(&converted);
                } else {
                    out.push('(');
                    out.push_str(&converted);
                    out.push(')');
                }
                i = next; // already past the closing }
            }
        }

        // Bare braces not following ^ or _ → just convert to parens
        '{' => {
            i += 1;
            let (inner, next) = read_brace_content(chars, i);
            out.push_str(&convert(&inner));
            i = next;
        }

        c => {
            out.push(c);
            i += 1;
            // Differential detection: bare `d` directly before a SINGLE letter
            // creates an unknown identifier in Typst ("dx", "dy" → error).
            // Require that the letter after `d` is NOT itself followed by
            // another letter, so multi-char names like "div", "dim", "det"
            // are left intact while "dx", "dy", "dz", "dt" are split.
            if c == 'd' && i < chars.len() && chars[i].is_alphabetic() {
                let after_next_alpha = chars.get(i + 1).map_or(false, |ch| ch.is_alphabetic());
                if !after_next_alpha {
                    let prev = out.chars().rev().nth(1); // char just before the 'd'
                    if !prev.map_or(false, |ch| ch.is_alphabetic()) {
                        // Space BEFORE `d` if not already separated
                        if prev.map_or(false, |ch| ch != ' ') {
                            out.pop();
                            out.push(' ');
                            out.push('d');
                        }
                        // Space AFTER `d`
                        out.push(' ');
                    }
                }
            }
        }
    }
    i
}

fn emit_cmd(cmd: &str, chars: &[char], i: usize, out: &mut String) -> usize {
    // ── Single-arg function commands ─────────────────────────────────────────
    match cmd {
        "sqrt" => {
            // \sqrt[n]{x} → root(n, x)  |  \sqrt{x} → sqrt(x)
            if i < chars.len() && chars[i] == '[' {
                let (n, after_bracket) = read_bracket_content(chars, i + 1);
                let (x, next) = read_brace_arg(chars, after_bracket);
                out.push_str(&format!("root({}, {})", convert(&n), convert(&x)));
                return next;
            }
            let (x, next) = read_brace_arg(chars, i);
            out.push_str(&format!("sqrt({})", convert(&x)));
            return next;
        }
        "frac" => {
            let (a, i2) = read_brace_arg(chars, i);
            let (b, i3) = read_brace_arg(chars, i2);
            out.push_str(&format!("frac({}, {})", convert(&a), convert(&b)));
            return i3;
        }
        "vec" => {
            let (v, next) = read_brace_arg(chars, i);
            out.push_str(&format!("arrow({})", convert(&v)));
            return next;
        }
        "text" => {
            let (t, next) = read_brace_arg(chars, i);
            out.push_str(&format!("upright(\"{}\")", t));
            return next;
        }
        "mathbb" => {
            let (arg, next) = read_brace_arg(chars, i);
            let typst = match arg.trim() {
                "R" => "RR", "Z" => "ZZ", "N" => "NN",
                "Q" => "QQ", "C" => "CC", other => other,
            };
            out.push_str(typst);
            return next;
        }
        "mathbf" | "boldsymbol" => {
            let (arg, next) = read_brace_arg(chars, i);
            out.push_str(&format!("bold({})", convert(&arg)));
            return next;
        }
        "mathrm" | "operatorname" => {
            let (arg, next) = read_brace_arg(chars, i);
            out.push_str(&format!("upright({})", convert(&arg)));
            return next;
        }
        "left" | "right" => {
            // \left( \right) — just skip the keyword, emit the delimiter
            return i;
        }
        _ => {}
    }

    // ── Simple symbol renames ────────────────────────────────────────────────
    let typst = match cmd {
        // Greek (lowercase)
        "alpha"   => "alpha",   "beta"    => "beta",   "gamma"  => "gamma",
        "delta"   => "delta",   "epsilon" => "epsilon","varepsilon" => "epsilon.alt",
        "zeta"    => "zeta",    "eta"     => "eta",    "theta"  => "theta",
        "vartheta"=> "theta.alt","iota"   => "iota",   "kappa"  => "kappa",
        "lambda"  => "lambda",  "mu"      => "mu",     "nu"     => "nu",
        "xi"      => "xi",      "pi"      => "pi",     "varpi"  => "pi.alt",
        "rho"     => "rho",     "varsigma"=> "sigma.alt","sigma" => "sigma",
        "tau"     => "tau",     "upsilon" => "upsilon","phi"    => "phi",
        "varphi"  => "phi.alt", "chi"     => "chi",    "psi"    => "psi",
        "omega"   => "omega",
        // Greek (uppercase)
        "Gamma"   => "Gamma",   "Delta"   => "Delta",  "Theta"  => "Theta",
        "Lambda"  => "Lambda",  "Xi"      => "Xi",     "Pi"     => "Pi",
        "Sigma"   => "Sigma",   "Upsilon" => "Upsilon","Phi"    => "Phi",
        "Psi"     => "Psi",     "Omega"   => "Omega",
        // Arithmetic
        "times"   => "times",   "div"     => "div",
        "pm"      => "plus.minus", "mp"   => "minus.plus",
        "cdot"    => "dot.op",  "cdots"   => "dots.h",
        "ldots"   => "dots.l",  "vdots"   => "dots.v", "ddots" => "dots.down",
        // Relations
        "leq" | "le"  => "lt.eq",   "geq" | "ge" => "gt.eq",
        "neq" | "ne"  => "eq.not",  "approx"     => "approx",
        "sim"         => "tilde",   "cong"       => "equiv",
        "equiv"       => "equiv",   "propto"     => "prop",
        "ll"          => "lt.double","gg"         => "gt.double",
        // Arrows
        "to" | "rightarrow"       => "->",
        "leftarrow"               => "<-",
        "Rightarrow"              => "=>",
        "Leftarrow"               => "<=",
        "Leftrightarrow"          => "<=>",
        "leftrightarrow"          => "<->",
        "mapsto"                  => "arrow.r.bar",
        // Calculus / analysis
        "int"     => "integral",  "iint"  => "integral.double",
        "iiint"   => "integral.triple",
        "oint"    => "integral.cont",
        "sum"     => "sum",       "prod"  => "product",
        "lim"     => "lim",       "inf"   => "inf", "sup" => "sup",
        "max"     => "max",       "min"   => "min",
        "partial" => "partial",   "nabla" => "nabla",
        "infty"   => "infinity",
        // Trig / log
        "sin"     => "sin",   "cos"     => "cos",  "tan"    => "tan",
        "cot"     => "cot",   "sec"     => "sec",  "csc"    => "csc",
        "arcsin"  => "arcsin","arccos"  => "arccos","arctan" => "arctan",
        "sinh"    => "sinh",  "cosh"    => "cosh", "tanh"   => "tanh",
        "log"     => "log",   "ln"      => "ln",   "exp"    => "exp",
        // Set theory
        "in"        => "in",        "notin"   => "in.not",
        "subset"    => "subset",    "supset"  => "supset",
        "subseteq"  => "subset.eq", "supseteq"=> "supset.eq",
        "cup"       => "union",     "cap"     => "sect",
        "setminus"  => "without",   "emptyset"| "varnothing" => "emptyset",
        // Logic
        "forall"        => "forall",       "exists"       => "exists",
        "nexists"       => "exists.not",   "neg" | "lnot" => "not",
        "land" | "wedge"=> "and",          "lor" | "vee"  => "or",
        // Geometry
        "angle"    => "angle",     "parallel" => "parallel",
        "perp"     => "perp",      "triangle" => "triangle.stroked.t",
        "circ"     => "circle.small",
        // Misc
        "hbar"     => "planck.reduce",
        "ell"      => "ell",
        "Re"       => "Re",        "Im"     => "Im",
        "top"      => "top",       "bot"    => "bot",
        "dagger"   => "dagger",    "star"   => "star",
        "oplus"    => "plus.circle","otimes" => "times.circle",
        _ => {
            // Unknown: emit as-is (Typst might know it)
            out.push_str(cmd);
            // Prevent concatenation with following content
            if i < chars.len() && needs_space_after(chars[i]) {
                out.push(' ');
            }
            return i;
        }
    };

    out.push_str(typst);
    // Prevent symbol name from concatenating with following brace group or identifier
    // e.g. \int{x^2} → "integral " + "x^2"  (not "integralx^2")
    if i < chars.len() && needs_space_after(chars[i]) {
        out.push(' ');
    }
    i
}

/// Returns true when a space must be emitted after a symbol to prevent the
/// symbol name from running into the following character.
/// We add a space before `{`, `\`, and alphanumeric chars.
/// We do NOT add one before `^`, `_`, `(`, `)`, operators, or whitespace.
#[inline]
fn needs_space_after(c: char) -> bool {
    matches!(c, '{' | '\\') || c.is_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::convert;

    #[test]
    fn integral_brace_no_concat() {
        // \int{x^2}dx must NOT produce "integralx^2dx"
        assert_eq!(convert(r"\int{x^2}dx"), "integral x^2 d x");
    }

    #[test]
    fn integral_space() {
        assert_eq!(convert(r"\int x"), "integral x");
    }

    #[test]
    fn frac() {
        assert_eq!(convert(r"\frac{a}{b}"), "frac(a, b)");
    }

    #[test]
    fn sqrt_simple() {
        assert_eq!(convert(r"\sqrt{x}"), "sqrt(x)");
    }

    #[test]
    fn sqrt_nth() {
        assert_eq!(convert(r"\sqrt[3]{x}"), "root(3, x)");
    }

    #[test]
    fn superscript_multi() {
        assert_eq!(convert(r"x^{n+1}"), "x^(n+1)");
    }

    #[test]
    fn greek_space_preserved() {
        // Space between \alpha and x must survive
        assert_eq!(convert(r"\alpha x"), "alpha x");
    }

    #[test]
    fn sum_subscript_no_space() {
        // single-char subscript: no parens needed in Typst
        assert_eq!(convert(r"\sum_{i}"), "sum_i");
        // multi-char subscript: parens required
        assert_eq!(convert(r"\sum_{i=1}"), "sum_(i=1)");
    }

    #[test]
    fn mathbb_r() {
        assert_eq!(convert(r"\mathbb{R}"), "RR");
    }

    #[test]
    fn mixed_formula() {
        // The formula from the bug report
        let input = r"f(x) = x^2 + 5x";
        assert_eq!(convert(input), "f(x) = x^2 + 5x");
    }

    #[test]
    fn differential_dx() {
        // dx after a brace group must become "d x"
        assert_eq!(convert(r"\int{x^2}dx"), "integral x^2 d x");
    }

    #[test]
    fn differential_dy() {
        assert_eq!(convert(r"\int f(y)dy"), "integral f(y) d y");
    }

    #[test]
    fn differential_not_split_in_word() {
        // "add" must stay "add" — d preceded by a letter → no split
        assert_eq!(convert("add"), "add");
    }

    #[test]
    fn differential_frac_derivative() {
        // \frac{d}{dx} → frac(d, d x)
        assert_eq!(convert(r"\frac{d}{dx}"), "frac(d, d x)");
    }

    // ── Greek letters ─────────────────────────────────────────────────────────

    #[test]
    fn greek_lowercase() {
        assert_eq!(convert(r"\alpha"),   "alpha");
        assert_eq!(convert(r"\beta"),    "beta");
        assert_eq!(convert(r"\gamma"),   "gamma");
        assert_eq!(convert(r"\delta"),   "delta");
        assert_eq!(convert(r"\epsilon"), "epsilon");
        assert_eq!(convert(r"\varepsilon"), "epsilon.alt");
        assert_eq!(convert(r"\theta"),   "theta");
        assert_eq!(convert(r"\vartheta"),"theta.alt");
        assert_eq!(convert(r"\lambda"),  "lambda");
        assert_eq!(convert(r"\mu"),      "mu");
        assert_eq!(convert(r"\pi"),      "pi");
        assert_eq!(convert(r"\varpi"),   "pi.alt");
        assert_eq!(convert(r"\sigma"),   "sigma");
        assert_eq!(convert(r"\varsigma"),"sigma.alt");
        assert_eq!(convert(r"\phi"),     "phi");
        assert_eq!(convert(r"\varphi"),  "phi.alt");
        assert_eq!(convert(r"\omega"),   "omega");
    }

    #[test]
    fn greek_uppercase() {
        assert_eq!(convert(r"\Gamma"),  "Gamma");
        assert_eq!(convert(r"\Delta"),  "Delta");
        assert_eq!(convert(r"\Sigma"),  "Sigma");
        assert_eq!(convert(r"\Omega"),  "Omega");
        assert_eq!(convert(r"\Lambda"), "Lambda");
        assert_eq!(convert(r"\Phi"),    "Phi");
        assert_eq!(convert(r"\Pi"),     "Pi");
    }

    // ── Arithmetic operators ──────────────────────────────────────────────────

    #[test]
    fn arithmetic_operators() {
        assert_eq!(convert(r"\times"),  "times");
        assert_eq!(convert(r"\div"),    "div");
        assert_eq!(convert(r"\pm"),     "plus.minus");
        assert_eq!(convert(r"\mp"),     "minus.plus");
        assert_eq!(convert(r"\cdot"),   "dot.op");
        assert_eq!(convert(r"\cdots"),  "dots.h");
        assert_eq!(convert(r"\ldots"),  "dots.l");
    }

    // ── Relations ─────────────────────────────────────────────────────────────

    #[test]
    fn relations() {
        assert_eq!(convert(r"\leq"),   "lt.eq");
        assert_eq!(convert(r"\le"),    "lt.eq");
        assert_eq!(convert(r"\geq"),   "gt.eq");
        assert_eq!(convert(r"\ge"),    "gt.eq");
        assert_eq!(convert(r"\neq"),   "eq.not");
        assert_eq!(convert(r"\ne"),    "eq.not");
        assert_eq!(convert(r"\approx"),"approx");
        assert_eq!(convert(r"\sim"),   "tilde");
        assert_eq!(convert(r"\equiv"), "equiv");
        assert_eq!(convert(r"\propto"),"prop");
        assert_eq!(convert(r"\ll"),    "lt.double");
        assert_eq!(convert(r"\gg"),    "gt.double");
    }

    // ── Arrows ────────────────────────────────────────────────────────────────

    #[test]
    fn arrows() {
        assert_eq!(convert(r"\to"),              "->");
        assert_eq!(convert(r"\rightarrow"),      "->");
        assert_eq!(convert(r"\leftarrow"),       "<-");
        assert_eq!(convert(r"\Rightarrow"),      "=>");
        assert_eq!(convert(r"\Leftarrow"),       "<=");
        assert_eq!(convert(r"\Leftrightarrow"),  "<=>");
        assert_eq!(convert(r"\leftrightarrow"),  "<->");
        assert_eq!(convert(r"\mapsto"),          "arrow.r.bar");
    }

    // ── Calculus ──────────────────────────────────────────────────────────────

    #[test]
    fn calculus_symbols() {
        assert_eq!(convert(r"\int"),     "integral");
        assert_eq!(convert(r"\iint"),    "integral.double");
        assert_eq!(convert(r"\iiint"),   "integral.triple");
        assert_eq!(convert(r"\oint"),    "integral.cont");
        assert_eq!(convert(r"\sum"),     "sum");
        assert_eq!(convert(r"\prod"),    "product");
        assert_eq!(convert(r"\partial"), "partial");
        assert_eq!(convert(r"\nabla"),   "nabla");
        assert_eq!(convert(r"\infty"),   "infinity");
    }

    #[test]
    fn limit_expression() {
        // \lim_{x \to \infty} f(x)
        assert_eq!(
            convert(r"\lim_{x \to \infty} f(x)"),
            "lim_(x -> infinity ) f(x)"
        );
    }

    #[test]
    fn sum_with_bounds() {
        // \sum_{i=0}^{n} i
        assert_eq!(convert(r"\sum_{i=0}^{n} i"), "sum_(i=0)^n i");
    }

    #[test]
    fn integral_with_bounds() {
        // \int_{0}^{\infty} e^{-x} dx
        assert_eq!(
            convert(r"\int_{0}^{\infty} e^{-x} dx"),
            "integral_(0)^(infinity ) e^(-x) d x"
        );
    }

    // ── Trig / log ────────────────────────────────────────────────────────────

    #[test]
    fn trig_functions() {
        assert_eq!(convert(r"\sin"),    "sin");
        assert_eq!(convert(r"\cos"),    "cos");
        assert_eq!(convert(r"\tan"),    "tan");
        assert_eq!(convert(r"\arcsin"), "arcsin");
        assert_eq!(convert(r"\arctan"), "arctan");
        assert_eq!(convert(r"\ln"),     "ln");
        assert_eq!(convert(r"\log"),    "log");
        assert_eq!(convert(r"\exp"),    "exp");
    }

    // ── Set theory ────────────────────────────────────────────────────────────

    #[test]
    fn set_theory() {
        assert_eq!(convert(r"\in"),        "in");
        assert_eq!(convert(r"\notin"),     "in.not");
        assert_eq!(convert(r"\subset"),    "subset");
        assert_eq!(convert(r"\subseteq"),  "subset.eq");
        assert_eq!(convert(r"\cup"),       "union");
        assert_eq!(convert(r"\cap"),       "sect");
        assert_eq!(convert(r"\setminus"),  "without");
        assert_eq!(convert(r"\emptyset"),  "emptyset");
        assert_eq!(convert(r"\varnothing"),"emptyset");
    }

    // ── Logic ─────────────────────────────────────────────────────────────────

    #[test]
    fn logic_operators() {
        assert_eq!(convert(r"\forall"),  "forall");
        assert_eq!(convert(r"\exists"), "exists");
        assert_eq!(convert(r"\nexists"),"exists.not");
        assert_eq!(convert(r"\neg"),    "not");
        assert_eq!(convert(r"\lnot"),   "not");
        assert_eq!(convert(r"\land"),   "and");
        assert_eq!(convert(r"\wedge"),  "and");
        assert_eq!(convert(r"\lor"),    "or");
        assert_eq!(convert(r"\vee"),    "or");
    }

    // ── mathbb ────────────────────────────────────────────────────────────────

    #[test]
    fn mathbb_all() {
        assert_eq!(convert(r"\mathbb{R}"), "RR");
        assert_eq!(convert(r"\mathbb{Z}"), "ZZ");
        assert_eq!(convert(r"\mathbb{N}"), "NN");
        assert_eq!(convert(r"\mathbb{Q}"), "QQ");
        assert_eq!(convert(r"\mathbb{C}"), "CC");
    }

    // ── Text / font commands ──────────────────────────────────────────────────

    #[test]
    fn text_command() {
        assert_eq!(convert(r"\text{hello}"),       "upright(\"hello\")");
        assert_eq!(convert(r"\mathbf{v}"),         "bold(v)");
        assert_eq!(convert(r"\boldsymbol{F}"),     "bold(F)");
        assert_eq!(convert(r"\mathrm{Re}"),        "upright(Re)");
        assert_eq!(convert(r"\operatorname{div}"), "upright(div)");
    }

    // ── vec ───────────────────────────────────────────────────────────────────

    #[test]
    fn vec_command() {
        assert_eq!(convert(r"\vec{v}"), "arrow(v)");
        assert_eq!(convert(r"\vec{AB}"), "arrow(AB)");
    }

    // ── \left / \right ───────────────────────────────────────────────────────

    #[test]
    fn left_right_stripped() {
        // \left and \right keywords are dropped, delimiter passes through
        assert_eq!(convert(r"\left(x\right)"), "(x)");
        assert_eq!(convert(r"\left[x\right]"), "[x]");
    }

    // ── Spacing escapes ───────────────────────────────────────────────────────

    #[test]
    fn spacing_escapes() {
        assert_eq!(convert(r"\,"),  "space.thin");
        assert_eq!(convert(r"\;"),  "space.thin");
        // \! is negative space — should be dropped
        assert_eq!(convert(r"\!"),  "");
        // \\ is line break
        assert_eq!(convert(r"\\"), "\\ ");
    }

    // ── Nested structures ─────────────────────────────────────────────────────

    #[test]
    fn nested_frac() {
        // \frac{\frac{a}{b}}{c} → frac(frac(a, b), c)
        assert_eq!(convert(r"\frac{\frac{a}{b}}{c}"), "frac(frac(a, b), c)");
    }

    #[test]
    fn frac_with_greek() {
        assert_eq!(convert(r"\frac{\alpha}{\beta}"), "frac(alpha, beta)");
    }

    #[test]
    fn quadratic_formula() {
        // x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
        assert_eq!(
            convert(r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}"),
            "x = frac(-b plus.minus sqrt(b^2 - 4ac), 2a)"
        );
    }

    // ── Misc symbols ─────────────────────────────────────────────────────────

    #[test]
    fn misc_symbols() {
        assert_eq!(convert(r"\hbar"),   "planck.reduce");
        assert_eq!(convert(r"\ell"),    "ell");
        assert_eq!(convert(r"\infty"),  "infinity");
        assert_eq!(convert(r"\dagger"), "dagger");
        assert_eq!(convert(r"\oplus"),  "plus.circle");
        assert_eq!(convert(r"\otimes"), "times.circle");
        assert_eq!(convert(r"\top"),    "top");
        assert_eq!(convert(r"\bot"),    "bot");
    }

    // ── Unknown command passthrough ───────────────────────────────────────────

    #[test]
    fn unknown_command_passthrough() {
        // Unknown commands should pass through as-is for Typst to handle
        assert_eq!(convert(r"\foo"), "foo");
        assert_eq!(convert(r"\ker"), "ker");
    }
}

// ── Argument parsing helpers ─────────────────────────────────────────────────

/// Read the content of a `{...}` block starting just AFTER the opening `{`.
/// Returns (content_string, index_after_closing_brace).
fn read_brace_content(chars: &[char], start: usize) -> (String, usize) {
    let mut depth = 1usize;
    let mut i = start;
    let mut content = String::new();
    while i < chars.len() {
        match chars[i] {
            '{' => { depth += 1; content.push('{'); i += 1; }
            '}' => {
                depth -= 1;
                if depth == 0 { i += 1; break; }
                content.push('}'); i += 1;
            }
            c => { content.push(c); i += 1; }
        }
    }
    (content, i)
}

/// Skip optional whitespace then read a `{...}` argument.
fn read_brace_arg(chars: &[char], mut i: usize) -> (String, usize) {
    while i < chars.len() && chars[i] == ' ' { i += 1; }
    if i < chars.len() && chars[i] == '{' {
        read_brace_content(chars, i + 1)
    } else if i < chars.len() {
        // Single-char arg (e.g. \sqrt x)
        let c = chars[i].to_string();
        (c, i + 1)
    } else {
        (String::new(), i)
    }
}

/// Read the content of a `[...]` optional arg starting just AFTER the opening `[`.
fn read_bracket_content(chars: &[char], mut i: usize) -> (String, usize) {
    let mut content = String::new();
    while i < chars.len() && chars[i] != ']' {
        content.push(chars[i]);
        i += 1;
    }
    if i < chars.len() { i += 1; } // skip ]
    (content, i)
}
