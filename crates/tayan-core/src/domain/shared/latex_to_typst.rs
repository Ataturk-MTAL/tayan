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

            // Skip trailing space (LaTeX \alpha x convention)
            if i < chars.len() && chars[i] == ' ' { i += 1; }

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

        c => { out.push(c); i += 1; }
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
            return i;
        }
    };

    out.push_str(typst);
    i
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
