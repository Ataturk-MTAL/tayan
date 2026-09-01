/// Convert a LaTeX math string to Typst math syntax.
///
/// Token-based recursive-descent transpiler. Handles fractions, roots,
/// matrices, cases, align, all standard symbols, accents, differentials.
pub fn convert(input: &str) -> String {
    let tokens = tokenize(input);
    let mut conv = Converter::new(&tokens);
    conv.run();
    conv.out
}

// ── Token ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Char(char),   // ordinary character (includes literal { } from \{ \})
    Cmd(String),  // \commandname
    Open,         // unescaped {
    Close,        // unescaped }
    Sub,          // _
    Sup,          // ^
    Amp,          // &
    NewRow,       // \\  (double backslash → row break in environments)
}

fn tokenize(input: &str) -> Vec<Token> {
    let chars: Vec<char> = input.chars().collect();
    let mut toks: Vec<Token> = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                i += 1;
                if i >= chars.len() { break; }
                match chars[i] {
                    // Double backslash = row break
                    '\\' => { toks.push(Token::NewRow); i += 1; }
                    // Escaped braces / delimiters become ordinary chars
                    '{' => { toks.push(Token::Char('{')); i += 1; }
                    '}' => { toks.push(Token::Char('}')); i += 1; }
                    '|' => { toks.push(Token::Char('|')); i += 1; }
                    '(' => { toks.push(Token::Char('(')); i += 1; }
                    ')' => { toks.push(Token::Char(')')); i += 1; }
                    '[' => { toks.push(Token::Char('[')); i += 1; }
                    ']' => { toks.push(Token::Char(']')); i += 1; }
                    // Spacing commands
                    ',' => { toks.push(Token::Cmd(",".into())); i += 1; }
                    ';' => { toks.push(Token::Cmd(";".into())); i += 1; }
                    ':' | '>' => { toks.push(Token::Cmd(":".into())); i += 1; }
                    '!' => { toks.push(Token::Cmd("!".into())); i += 1; }
                    ' ' => { toks.push(Token::Char(' ')); i += 1; }
                    // Alpha command name
                    c if c.is_alphabetic() => {
                        let start = i;
                        while i < chars.len() && chars[i].is_alphabetic() { i += 1; }
                        let name: String = chars[start..i].iter().collect();
                        toks.push(Token::Cmd(name));
                        // LaTeX convention: one trailing space terminates a command
                        if i < chars.len() && chars[i] == ' ' { i += 1; }
                    }
                    c => { toks.push(Token::Char(c)); i += 1; }
                }
            }
            '{' => { toks.push(Token::Open);  i += 1; }
            '}' => { toks.push(Token::Close); i += 1; }
            '_' => { toks.push(Token::Sub);   i += 1; }
            '^' => { toks.push(Token::Sup);   i += 1; }
            '&' => { toks.push(Token::Amp);   i += 1; }
            c   => { toks.push(Token::Char(c)); i += 1; }
        }
    }
    toks
}

// ── Converter ─────────────────────────────────────────────────────────────────

struct Converter<'a> {
    toks: &'a [Token],
    pos:  usize,
    pub out: String,
}

impl<'a> Converter<'a> {
    fn new(toks: &'a [Token]) -> Self {
        Converter { toks, pos: 0, out: String::new() }
    }

    // ── Lookahead helpers ────────────────────────────────────────────────────

    fn peek(&self) -> Option<&Token> { self.toks.get(self.pos) }

    fn peek2(&self) -> Option<&Token> { self.toks.get(self.pos + 1) }

    fn advance(&mut self) { self.pos += 1; }

    fn skip_spaces(&mut self) {
        while matches!(self.peek(), Some(Token::Char(' '))) { self.pos += 1; }
    }

    // ── Group readers ────────────────────────────────────────────────────────

    /// Read `{content}` (cursor must be ON the `{`); returns converted inner.
    fn read_group(&mut self) -> String {
        assert!(matches!(self.peek(), Some(Token::Open)));
        self.advance(); // skip {
        self.read_group_content()
    }

    /// Read until matching `}` (caller already consumed `{`); returns converted inner.
    fn read_group_content(&mut self) -> String {
        let start = self.pos;
        let mut depth = 1usize;
        while self.pos < self.toks.len() {
            match &self.toks[self.pos] {
                Token::Open  => { depth += 1; self.pos += 1; }
                Token::Close => {
                    depth -= 1;
                    self.pos += 1;
                    if depth == 0 { break; }
                }
                _ => { self.pos += 1; }
            }
        }
        // end is self.pos, but already advanced past }
        // Tokens from start to (self.pos - 1) were inside {}; the } is already consumed.
        let inner = &self.toks[start..self.pos - 1];
        sub_convert(inner)
    }

    /// Read a mandatory `{arg}` or single-token arg; returns converted.
    fn read_arg(&mut self) -> String {
        self.skip_spaces();
        match self.peek() {
            Some(Token::Open) => self.read_group(),
            Some(Token::Cmd(_)) => {
                if let Some(Token::Cmd(name)) = self.peek().cloned() {
                    self.advance();
                    if let Some(Token::Char(' ')) = self.peek() { self.advance(); }
                    symbol_map(&name).unwrap_or(&name).to_owned()
                } else { unreachable!() }
            }
            Some(_) => {
                let c = self.peek().cloned();
                self.advance();
                opt_token_to_str(&c)
            }
            None => String::new(),
        }
    }

    /// Read a mandatory `{arg}`; returns raw (unconverted) text content.
    fn read_arg_raw(&mut self) -> String {
        self.skip_spaces();
        if matches!(self.peek(), Some(Token::Open)) {
            self.advance(); // skip {
            let start = self.pos;
            let mut depth = 1usize;
            while self.pos < self.toks.len() {
                match &self.toks[self.pos] {
                    Token::Open  => { depth += 1; self.pos += 1; }
                    Token::Close => {
                        depth -= 1; self.pos += 1;
                        if depth == 0 { break; }
                    }
                    _ => { self.pos += 1; }
                }
            }
            let inner = &self.toks[start..self.pos - 1];
            inner.iter().map(token_to_str).collect::<String>()
        } else if let Some(c) = self.peek().cloned() {
            self.advance();
            token_to_str(&c)
        } else {
            String::new()
        }
    }

    /// Read optional `[arg]`; returns converted if present.
    fn read_opt(&mut self) -> Option<String> {
        self.skip_spaces();
        if !matches!(self.peek(), Some(Token::Char('['))) { return None; }
        self.advance(); // skip [
        let start = self.pos;
        let mut depth = 0usize;
        while self.pos < self.toks.len() {
            match &self.toks[self.pos] {
                Token::Char('[') => { depth += 1; self.pos += 1; }
                Token::Char(']') => {
                    if depth == 0 { self.pos += 1; break; }
                    depth -= 1; self.pos += 1;
                }
                _ => { self.pos += 1; }
            }
        }
        let inner = &self.toks[start..self.pos - 1];
        Some(sub_convert(inner))
    }

    /// Collect tokens until `\end{env_name}` (handling nested \begin/\end pairs).
    fn read_until_end(&mut self) -> Vec<Token> {
        let mut result = Vec::new();
        let mut depth: usize = 0;
        loop {
            match self.toks.get(self.pos).cloned() {
                None => break,
                Some(Token::Cmd(ref c)) if c == "begin" => {
                    depth += 1;
                    result.push(Token::Cmd("begin".into()));
                    self.pos += 1;
                    // Also absorb the {env} brace group into result
                    if matches!(self.peek(), Some(Token::Open)) {
                        result.push(Token::Open);
                        self.pos += 1;
                        let mut d = 1usize;
                        loop {
                            match self.toks.get(self.pos) {
                                None => break,
                                Some(Token::Open)  => { d += 1; result.push(Token::Open);  self.pos += 1; }
                                Some(Token::Close) => {
                                    d -= 1; result.push(Token::Close); self.pos += 1;
                                    if d == 0 { break; }
                                }
                                Some(t) => { result.push(t.clone()); self.pos += 1; }
                            }
                        }
                    }
                }
                Some(Token::Cmd(ref c)) if c == "end" => {
                    self.pos += 1;
                    self.skip_spaces();
                    // Consume {env_name}
                    let env_name_toks: Vec<Token> = if matches!(self.peek(), Some(Token::Open)) {
                        self.pos += 1; // skip {
                        let mut d = 1usize;
                        let mut v = Vec::new();
                        loop {
                            match self.toks.get(self.pos) {
                                None => break,
                                Some(Token::Open)  => { d += 1; v.push(Token::Open);  self.pos += 1; }
                                Some(Token::Close) => {
                                    d -= 1; self.pos += 1;
                                    if d == 0 { break; }
                                    v.push(Token::Close);
                                }
                                Some(t) => { v.push(t.clone()); self.pos += 1; }
                            }
                        }
                        v
                    } else { Vec::new() };

                    if depth == 0 {
                        break; // found our \end{...}
                    } else {
                        depth -= 1;
                        result.push(Token::Cmd("end".into()));
                        result.push(Token::Open);
                        result.extend(env_name_toks);
                        result.push(Token::Close);
                    }
                }
                Some(t) => { result.push(t); self.pos += 1; }
            }
        }
        result
    }

    // ── Main loop ────────────────────────────────────────────────────────────

    fn run(&mut self) {
        while self.pos < self.toks.len() {
            self.step();
        }
    }

    fn step(&mut self) {
        match self.toks.get(self.pos).cloned() {
            None => {}

            Some(Token::Cmd(cmd)) => {
                self.advance();
                self.emit_cmd(&cmd);
            }

            // Super/subscript
            Some(Token::Sup) | Some(Token::Sub) => {
                let mark = if matches!(self.toks[self.pos], Token::Sup) { '^' } else { '_' };
                self.advance();
                self.out.push(mark);
                self.skip_spaces();
                if matches!(self.peek(), Some(Token::Open)) {
                    let inner = self.read_group();
                    // Special case: ^{\circ} → degree  (angle notation)
                    if mark == '^' && inner == "circle.small" {
                        self.out.pop(); // remove the '^' already pushed
                        // Add separator space only if there's preceding content
                        if !self.out.is_empty() && !self.out.ends_with(' ') {
                            self.out.push(' ');
                        }
                        self.out.push_str("degree");
                    } else {
                        let n = inner.chars().count();
                        if n <= 1 {
                            self.out.push_str(&inner);
                        } else {
                            self.out.push('(');
                            self.out.push_str(&inner);
                            self.out.push(')');
                        }
                    }
                }
                // else: single-token script handled by next step()
            }

            // Bare group: {content} → strip braces
            Some(Token::Open) => {
                self.advance();
                let inner = self.read_group_content();
                self.out.push_str(&inner);
            }

            // & in alignment
            Some(Token::Amp) => { self.advance(); self.out.push('&'); }

            // \\ in top-level (outside environments) — math line break
            Some(Token::NewRow) => { self.advance(); self.out.push_str(" \\\n"); }

            Some(Token::Char(c)) => {
                self.advance();
                // ── Differential detection ───────────────────────────────
                // bare `d` + SINGLE alpha (not preceded by alpha in output)
                // → insert spaces: "dx" → "d x", "f(y)dy" → "f(y) d y"
                // Guard: if `d` is followed by 2 letters (div, det, dim…) → skip
                if c == 'd'
                    && let Some(Token::Char(next_c)) = self.peek()
                        && next_c.is_alphabetic() {
                            let after_is_alpha = matches!(self.peek2(), Some(Token::Char(a)) if a.is_alphabetic());
                            let prev_alpha = self.out.chars().last().is_some_and(|ch| ch.is_alphabetic());
                            if !after_is_alpha && !prev_alpha {
                                if !self.out.ends_with(' ') && !self.out.is_empty() {
                                    self.out.push(' ');
                                }
                                self.out.push('d');
                                self.out.push(' ');
                                return;
                            }
                        }
                self.out.push(c);
            }

            Some(_) => {
                let t = self.toks[self.pos].clone();
                self.advance();
                self.out.push_str(&token_to_str(&t));
            }
        }
    }

    // ── Command emitter ───────────────────────────────────────────────────────

    fn emit_cmd(&mut self, cmd: &str) {
        // ── Environments ────────────────────────────────────────────────────
        if cmd == "begin" {
            self.skip_spaces();
            let env = if matches!(self.peek(), Some(Token::Open)) {
                self.advance(); // skip {
                let start = self.pos;
                let mut d = 1usize;
                while self.pos < self.toks.len() {
                    match &self.toks[self.pos] {
                        Token::Open  => { d += 1; self.pos += 1; }
                        Token::Close => { d -= 1; self.pos += 1; if d == 0 { break; } }
                        _ => { self.pos += 1; }
                    }
                }
                let inner = &self.toks[start..self.pos - 1];
                inner.iter().map(token_to_str).collect::<String>()
            } else { String::new() };
            self.emit_env(&env);
            return;
        }
        if cmd == "end" {
            // Stray \end — consume {env_name} and ignore
            self.skip_spaces();
            if matches!(self.peek(), Some(Token::Open)) { self.read_group(); }
            return;
        }

        // ── Commands with arguments ─────────────────────────────────────────
        match cmd {
            // Fractions
            "frac" | "dfrac" | "tfrac" | "cfrac" => {
                let a = self.read_arg();
                let b = self.read_arg();
                self.out.push_str(&format!("frac({a}, {b})"));
            }
            "binom" | "tbinom" | "dbinom" => {
                let a = self.read_arg();
                let b = self.read_arg();
                self.out.push_str(&format!("binom({a}, {b})"));
            }
            // Roots
            "sqrt" => {
                let opt = self.read_opt();
                let x = self.read_arg();
                match opt {
                    Some(n) => self.out.push_str(&format!("root({n}, {x})")),
                    None    => self.out.push_str(&format!("sqrt({x})")),
                }
            }
            // Accents
            "hat" | "widehat"   => { let x = self.read_arg(); self.out.push_str(&format!("hat({x})")); }
            "tilde" | "widetilde" => { let x = self.read_arg(); self.out.push_str(&format!("tilde({x})")); }
            "bar" | "overline"  => { let x = self.read_arg(); self.out.push_str(&format!("overline({x})")); }
            "underline"         => { let x = self.read_arg(); self.out.push_str(&format!("underline({x})")); }
            "vec"               => { let x = self.read_arg(); self.out.push_str(&format!("arrow({x})")); }
            "dot"               => { let x = self.read_arg(); self.out.push_str(&format!("dot({x})")); }
            "ddot"              => { let x = self.read_arg(); self.out.push_str(&format!("dot.double({x})")); }
            "breve"             => { let x = self.read_arg(); self.out.push_str(&format!("breve({x})")); }
            "acute"             => { let x = self.read_arg(); self.out.push_str(&format!("acute({x})")); }
            "grave"             => { let x = self.read_arg(); self.out.push_str(&format!("grave({x})")); }
            "check"             => { let x = self.read_arg(); self.out.push_str(&format!("caron({x})")); }
            "mathring"          => { let x = self.read_arg(); self.out.push_str(&format!("circle({x})")); }
            // Over/under structures (with optional annotation via ^ or _)
            "overbrace" => {
                let x = self.read_arg();
                self.skip_spaces();
                if matches!(self.peek(), Some(Token::Sup)) {
                    self.advance();
                    let ann = self.read_arg();
                    self.out.push_str(&format!("overbrace({x}, {ann})"));
                } else {
                    self.out.push_str(&format!("overbrace({x})"));
                }
            }
            "underbrace" => {
                let x = self.read_arg();
                self.skip_spaces();
                if matches!(self.peek(), Some(Token::Sub)) {
                    self.advance();
                    let ann = self.read_arg();
                    self.out.push_str(&format!("underbrace({x}, {ann})"));
                } else {
                    self.out.push_str(&format!("underbrace({x})"));
                }
            }
            "overbracket" => {
                let x = self.read_arg();
                self.skip_spaces();
                if matches!(self.peek(), Some(Token::Sup)) {
                    self.advance();
                    let ann = self.read_arg();
                    self.out.push_str(&format!("overbracket({x}, {ann})"));
                } else {
                    self.out.push_str(&format!("overbracket({x})"));
                }
            }
            "underbracket" => {
                let x = self.read_arg();
                self.skip_spaces();
                if matches!(self.peek(), Some(Token::Sub)) {
                    self.advance();
                    let ann = self.read_arg();
                    self.out.push_str(&format!("underbracket({x}, {ann})"));
                } else {
                    self.out.push_str(&format!("underbracket({x})"));
                }
            }
            "overset" | "stackrel" => {
                let top  = self.read_arg();
                let base = self.read_arg();
                self.out.push_str(&format!("overset({base}, {top})"));
            }
            "underset" => {
                let bot  = self.read_arg();
                let base = self.read_arg();
                self.out.push_str(&format!("underset({base}, {bot})"));
            }
            // Font / style
            "text" | "mbox" => {
                let t = self.read_arg_raw();
                self.out.push_str(&format!("upright(\"{t}\")"));
            }
            "textbf" => {
                let t = self.read_arg_raw();
                self.out.push_str(&format!("bold(upright(\"{t}\"))"));
            }
            "textit" => {
                let t = self.read_arg_raw();
                self.out.push_str(&format!("italic(upright(\"{t}\"))"));
            }
            "mathbf" | "boldsymbol" | "bm" => {
                let x = self.read_arg();
                self.out.push_str(&format!("bold({x})"));
            }
            "mathit" => {
                let x = self.read_arg();
                self.out.push_str(&format!("italic({x})"));
            }
            "mathrm" | "operatorname" | "operatorname*" => {
                let t = self.read_arg_raw();
                self.out.push_str(&format!("upright({t})"));
            }
            "mathsf" => { let x = self.read_arg(); self.out.push_str(&format!("sans({x})")); }
            "mathtt" => { let x = self.read_arg(); self.out.push_str(&format!("mono({x})")); }
            "mathcal" => { let x = self.read_arg(); self.out.push_str(&format!("cal({x})")); }
            "mathfrak" => { let x = self.read_arg(); self.out.push_str(&format!("frak({x})")); }
            "mathbb" => {
                let raw = self.read_arg_raw();
                let mapped = match raw.trim() {
                    "R" => "RR", "Z" => "ZZ", "N" => "NN",
                    "Q" => "QQ", "C" => "CC", "H" => "HH",
                    "F" => "FF", "k" => "kk",
                    other => other,
                };
                self.out.push_str(mapped);
            }
            // Phantom (consume arg, emit nothing)
            "phantom" | "vphantom" | "hphantom" => { self.read_arg(); }
            // Negation
            "not" => {
                // Try to negate the next command
                self.skip_spaces();
                if let Some(Token::Cmd(next_cmd)) = self.peek().cloned()
                    && let Some(negated) = negation_map(&next_cmd) {
                        self.advance();
                        // skip optional space after the negated command
                        if let Some(Token::Char(' ')) = self.peek() { self.advance(); }
                        self.out.push_str(negated);
                        return;
                    }
                // Fallback: just emit "not"
                self.out.push_str("not");
            }
            // Delimiter sizing — just pass through the following delimiter
            "left" | "right"
            | "bigl" | "bigr" | "Bigl" | "Bigr"
            | "biggl" | "biggr" | "Biggl" | "Biggr"
            | "big" | "Big" | "bigg" | "Bigg" => {
                // Invisible delimiter \left. \right.  → skip the dot
                if matches!(self.peek(), Some(Token::Char('.'))) { self.advance(); }
                // The actual delimiter is the next token — let step() handle it
            }
            // ── Spacing ─────────────────────────────────────────────────
            "," | ";" | ":" => { self.out.push_str("space.thin"); }
            "!" => { /* negative space — drop */ }
            "quad"  => { self.out.push_str("quad"); }
            "qquad" => { self.out.push_str("wide"); }
            // ── Limits / scripts forcing ─────────────────────────────────
            "limits" => {
                // Wrap the previous symbol in limits(...)
                // e.g. \sum\limits → limits(sum)
                self.wrap_last_symbol("limits");
            }
            "nolimits" => {
                self.wrap_last_symbol("scripts");
            }
            "displaylimits" => {} // default — ignore
            // ── Style / size modifiers (mostly no-op) ────────────────────
            "displaystyle" => { /* full display — Typst handles automatically */ }
            "textstyle" | "scriptstyle" | "scriptscriptstyle" => {}
            "normalsize" | "small" | "large" | "Large" | "LARGE" | "huge" | "Huge" => {}
            "rm" | "bf" | "it" | "sf" | "tt" | "cal" => {}
            "DeclareMathOperator" | "label" | "tag" | "notag" => {
                // Consume the argument (if any) but emit nothing
                if matches!(self.peek(), Some(Token::Open)) { self.read_group(); }
                if matches!(self.peek(), Some(Token::Open)) { self.read_group(); }
            }
            "hline" | "cline" => {} // table rules — skip in math context
            // ── Cancel ───────────────────────────────────────────────────
            "cancel"  => { let x = self.read_arg(); self.out.push_str(&format!("cancel({x})")); }
            "bcancel" => { let x = self.read_arg(); self.out.push_str(&format!("cancel({x}, inverted: true)")); }
            "xcancel" => { let x = self.read_arg(); self.out.push_str(&format!("cancel({x}, cross: true)")); }
            // ── Triple dot accent ────────────────────────────────────────
            "dddot" => { let x = self.read_arg(); self.out.push_str(&format!("dot.triple({x})")); }
            // ── Over/left arrows ─────────────────────────────────────────
            "overleftarrow"  => { let x = self.read_arg(); self.out.push_str(&format!("arrow.l({x})")); }
            "overrightarrow" => { let x = self.read_arg(); self.out.push_str(&format!("arrow({x})")); }
            "underleftarrow" | "underrightarrow" => {
                let x = self.read_arg(); self.out.push_str(&format!("underline({x})")); // approx.
            }
            // ── Extensible arrows with annotation ────────────────────────
            "xrightarrow" => {
                let _under = self.read_opt(); // optional under-label (skip)
                let over  = self.read_arg();
                self.out.push_str(&format!("overset(->, {over})"));
            }
            "xleftarrow" => {
                let _under = self.read_opt();
                let over  = self.read_arg();
                self.out.push_str(&format!("overset(<-, {over})"));
            }
            "xLeftarrow" => {
                let _under = self.read_opt();
                let over  = self.read_arg();
                self.out.push_str(&format!("overset(<=, {over})"));
            }
            "xRightarrow" => {
                let _under = self.read_opt();
                let over  = self.read_arg();
                self.out.push_str(&format!("overset(=>, {over})"));
            }
            // ── Modulo ───────────────────────────────────────────────────
            "pmod" => {
                let m = self.read_arg();
                self.out.push_str(&format!("quad (mod {m})"));
            }
            "bmod" | "imod" => { self.out.push_str("mod"); }
            // ── Substack ─────────────────────────────────────────────────
            "substack" => {
                let inner = self.read_arg();
                // inner may contain \\ as row separators — convert to mat
                let rows: Vec<&str> = inner.split("\\\\\n").collect();
                let joined = rows.iter().map(|r| r.trim()).collect::<Vec<_>>().join("; ");
                self.out.push_str(&format!("mat(delim: none, {joined})"));
            }
            // ── Prescript (left super/subscript) ─────────────────────────
            "prescript" => {
                // \prescript{tl}{bl}{base}
                let tl   = self.read_arg();
                let bl   = self.read_arg();
                let base = self.read_arg();
                self.out.push_str(&format!("attach({base}, tl: {tl}, bl: {bl})"));
            }
            // ── mathscr (distinct from mathcal) ──────────────────────────
            "mathscr" => {
                let x = self.read_arg();
                self.out.push_str(&format!("scr({x})"));
            }
            // ── Math upright / italic / bb shortcuts ──────────────────────
            "mathup" | "mathnormal" => {
                let x = self.read_arg();
                self.out.push_str(&format!("upright({x})"));
            }
            // ── Fraction variants ─────────────────────────────────────────
            "sfrac" | "nicefrac" => {
                let num = self.read_arg();
                let den = self.read_arg();
                self.out.push_str(&format!("frac({num}, {den})"));
            }
            "genfrac" => {
                // \genfrac{ldelim}{rdelim}{thickness}{style}{num}{den}
                // Approximate: ignore ldelim/rdelim/thickness/style
                let _ld = self.read_arg(); let _rd = self.read_arg();
                let _th = self.read_arg(); let _st = self.read_arg();
                let num = self.read_arg();
                let den = self.read_arg();
                self.out.push_str(&format!("frac({num}, {den})"));
            }
            // ── Delimiter shorthand functions ─────────────────────────────
            "abs" => { let x = self.read_arg(); self.out.push_str(&format!("abs({x})")); }
            "norm" => { let x = self.read_arg(); self.out.push_str(&format!("norm({x})")); }
            "floor" => { let x = self.read_arg(); self.out.push_str(&format!("floor({x})")); }
            "ceil" => { let x = self.read_arg(); self.out.push_str(&format!("ceil({x})")); }
            // ── Boxed (box around expression) ─────────────────────────────
            "boxed" => {
                let x = self.read_arg();
                self.out.push_str(&format!("cancel({x}, stroke: 0pt)"));
                // Note: there's no direct Typst math.boxed; emit content as-is
                // (We clear the cancel and just keep x instead)
                // Undo and just output x:
                let suffix = format!("cancel({x}, stroke: 0pt)");
                let end = self.out.len();
                self.out.truncate(end - suffix.len());
                self.out.push_str(&x);
            }
            // ── Smash (zero-height box) ───────────────────────────────────
            "smash" => {
                let x = self.read_arg();
                self.out.push_str(&x);
            }
            // ── Text in math ─────────────────────────────────────────────
            "textnormal" | "textrm" | "textmd" | "textup" | "textsf" | "textsc" | "textsl" => {
                let t = self.read_arg_raw();
                self.out.push_str(&format!("upright(\"{t}\")"));
            }
            // ── Spacing consuming arg ─────────────────────────────────────
            "hspace" | "hspace*" | "vspace" | "vspace*" | "mspace" => {
                // consume the dimension arg; emit a thin space approximation
                self.read_arg_raw();
                self.out.push_str("space.thin");
            }
            "kern" | "mkern" | "mskip" | "hskip" => {
                self.read_arg_raw();
                // negative kern → drop; positive → thin space
                self.out.push_str("space.thin");
            }
            // ── Phantom (already covered, add hphantom/vphantom variants) ─
            "mathstrut" => {} // zero-width strut
            // ── Intertext / noalign (consume arg) ─────────────────────────
            "intertext" | "shortintertext" => { self.read_arg_raw(); }
            // ── Sideset (left/right scripts on operator) ──────────────────
            "sideset" => {
                // \sideset{left_scripts}{right_scripts}{op}
                let left  = self.read_arg();
                let right = self.read_arg();
                let base  = self.read_arg();
                self.out.push_str(&format!("attach({base}, {left}, {right})"));
            }
            // ── Over/underparen / shell ───────────────────────────────────
            "overparen" => {
                let x = self.read_arg();
                self.skip_spaces();
                if matches!(self.peek(), Some(Token::Sup)) {
                    self.advance();
                    let ann = self.read_arg();
                    self.out.push_str(&format!("overparen({x}, {ann})"));
                } else {
                    self.out.push_str(&format!("overparen({x})"));
                }
            }
            "underparen" => {
                let x = self.read_arg();
                self.skip_spaces();
                if matches!(self.peek(), Some(Token::Sub)) {
                    self.advance();
                    let ann = self.read_arg();
                    self.out.push_str(&format!("underparen({x}, {ann})"));
                } else {
                    self.out.push_str(&format!("underparen({x})"));
                }
            }
            // ── Buildrel (like overset but different arg order) ───────────
            "buildrel" => {
                // \buildrel{top}\over{base}
                let top = self.read_arg();
                // expect \over
                self.skip_spaces();
                if let Some(Token::Cmd(c)) = self.peek()
                    && c == "over" { self.advance(); }
                let base = self.read_arg();
                self.out.push_str(&format!("overset({base}, {top})"));
            }
            // ── Scalebox / raisebox / makebox (TeX layout in math) ────────
            "scalebox" => { self.read_arg_raw(); let x = self.read_arg(); self.out.push_str(&x); }
            "raisebox" => { self.read_arg_raw(); let x = self.read_arg(); self.out.push_str(&x); }
            "makebox"  => { let x = self.read_arg(); self.out.push_str(&x); }

            // ═══════════════════════════════════════════════════════════════
            // ── Chemistry (mhchem \ce{...}) ──────────────────────────────
            // ═══════════════════════════════════════════════════════════════
            "ce" => {
                let raw = self.read_arg_raw();
                self.out.push_str(&ce_convert(&raw));
            }
            "chemfig" => {
                // chemfig structural formulas are too complex to convert;
                // consume arg and emit placeholder
                let _ = self.read_arg_raw();
                self.out.push_str("upright(\"[chemfig]\")");
            }

            // ═══════════════════════════════════════════════════════════════
            // ── Dirac / bra-ket notation (physics package) ───────────────
            // ═══════════════════════════════════════════════════════════════
            "bra" => {
                let x = self.read_arg();
                self.out.push_str(&format!("lr(angle.l {x} |)"));
            }
            "ket" => {
                let x = self.read_arg();
                self.out.push_str(&format!("lr(| {x} angle.r)"));
            }
            "braket" => {
                let a = self.read_arg();
                self.skip_spaces();
                if matches!(self.peek(), Some(Token::Open)) {
                    let b = self.read_arg();
                    self.out.push_str(&format!("lr(angle.l {a} | {b} angle.r)"));
                } else {
                    self.out.push_str(&format!("lr(angle.l {a} angle.r)"));
                }
            }
            "ketbra" | "op" => {
                let a = self.read_arg();
                let b = self.read_arg();
                self.out.push_str(&format!("lr(| {a} angle.r angle.l {b} |)"));
            }
            "ev" | "expval" | "expectationvalue" => {
                let a = self.read_arg();
                self.skip_spaces();
                if matches!(self.peek(), Some(Token::Open)) {
                    let x = self.read_arg();
                    self.out.push_str(&format!("lr(angle.l {x} | {a} | {x} angle.r)"));
                } else {
                    self.out.push_str(&format!("lr(angle.l {a} angle.r)"));
                }
            }
            "mel" | "matrixelement" => {
                let x = self.read_arg();
                let a = self.read_arg();
                let y = self.read_arg();
                self.out.push_str(&format!("lr(angle.l {x} | {a} | {y} angle.r)"));
            }
            "ip" | "innerproduct" => {
                let a = self.read_arg();
                let b = self.read_arg();
                self.out.push_str(&format!("lr(angle.l {a} | {b} angle.r)"));
            }

            // ═══════════════════════════════════════════════════════════════
            // ── Differential operators (physics package) ─────────────────
            // ═══════════════════════════════════════════════════════════════
            "dd" => {
                // \dd        → d
                // \dd{x}     → d x
                // \dd[n]{x}  → d^n x
                let order = self.read_opt();
                self.skip_spaces();
                if matches!(self.peek(), Some(Token::Open)) {
                    let x = self.read_arg();
                    match order {
                        Some(n) => self.out.push_str(&format!("d^({n}) {x}")),
                        None    => self.out.push_str(&format!("d {x}")),
                    }
                } else {
                    self.out.push('d');
                }
            }
            "dv" | "derivative" => {
                // \dv[n]{f}{x} → frac(d^n f, d x^n)
                let order = self.read_opt();
                let f = self.read_arg();
                let x = self.read_arg();
                match order {
                    Some(n) => self.out.push_str(&format!("frac(d^({n}) {f}, d {x}^({n}))")),
                    None    => self.out.push_str(&format!("frac(d {f}, d {x})")),
                }
            }
            "pdv" | "partialderivative" => {
                // \pdv[n]{f}{x} → frac(partial^n f, partial x^n)
                let order = self.read_opt();
                let f = self.read_arg();
                let x = self.read_arg();
                match order {
                    Some(n) => self.out.push_str(&format!("frac(partial^({n}) {f}, partial {x}^({n}))")),
                    None    => self.out.push_str(&format!("frac(partial {f}, partial {x})")),
                }
            }
            // Shorthand for common mixed/higher partials
            "flatfrac" => {
                let a = self.read_arg();
                let b = self.read_arg();
                self.out.push_str(&format!("{a}/{b}"));
            }

            // ═══════════════════════════════════════════════════════════════
            // ── Vector notation (physics package) ────────────────────────
            // ═══════════════════════════════════════════════════════════════
            "vb" | "vectorbold" => {
                let x = self.read_arg();
                self.out.push_str(&format!("bold({x})"));
            }
            "va" | "vectorarrow" => {
                let x = self.read_arg();
                self.out.push_str(&format!("arrow({x})"));
            }
            "vu" | "vectorunit" => {
                let x = self.read_arg();
                self.out.push_str(&format!("hat(bold({x}))"));
            }
            "cross" | "cp" => {
                // \cross or \cp → cross product symbol
                self.out.push_str("times");
            }

            // ═══════════════════════════════════════════════════════════════
            // ── Operators / named functions ───────────────────────────────
            // ═══════════════════════════════════════════════════════════════
            "Tr" | "Trace" => { self.out.push_str("upright(\"Tr\")"); }
            "tr" | "trace" => { self.out.push_str("upright(\"tr\")"); }
            "rank"  => { self.out.push_str("upright(\"rank\")"); }
            "erf"   => { self.out.push_str("upright(\"erf\")"); }
            "erfc"  => { self.out.push_str("upright(\"erfc\")"); }
            "Res"   => { self.out.push_str("upright(\"Res\")"); }
            "PV"    => { self.out.push_str("upright(\"P.V.\")"); }
            "sinc"  => { self.out.push_str("upright(\"sinc\")"); }
            "sgn"   => { self.out.push_str("upright(\"sgn\")"); }
            "sign"  => { self.out.push_str("upright(\"sign\")"); }
            "diag"  => { self.out.push_str("upright(\"diag\")"); }
            "spec"  => { self.out.push_str("upright(\"spec\")"); }
            "span"  => { self.out.push_str("upright(\"span\")"); }
            "lcm"   => { self.out.push_str("lcm"); } // already in symbol_map
            "Pr"    => { self.out.push_str("Pr"); }

            // ── Vector calculus operators ─────────────────────────────────
            "grad" | "gradient" => { self.out.push_str("nabla"); }
            "curl"   => { self.out.push_str("nabla times"); }
            "laplacian" | "laplace" => { self.out.push_str("nabla^2"); }

            // ═══════════════════════════════════════════════════════════════
            // ── Order notation / commutators ─────────────────────────────
            // ═══════════════════════════════════════════════════════════════
            "order" | "bigO" => {
                let x = self.read_arg();
                self.out.push_str(&format!("cal(O)({x})"));
            }
            "comm" | "commutator" => {
                let a = self.read_arg();
                let b = self.read_arg();
                self.out.push_str(&format!("[{a}, {b}]"));
            }
            "acomm" | "anticommutator" | "pb" | "poissonbracket" => {
                let a = self.read_arg();
                let b = self.read_arg();
                self.out.push_str(&format!("lr({{{a}, {b}}})"));
            }

            // ═══════════════════════════════════════════════════════════════
            // ── SI Units (siunitx package) ────────────────────────────────
            // ═══════════════════════════════════════════════════════════════
            "SI" => {
                let val  = self.read_arg_raw();   // numeric value
                let unit = self.read_arg_raw();   // unit string
                self.out.push_str(&format!("{val} upright(\"{unit}\")"));
            }
            "si" => {
                let unit = self.read_arg_raw();
                self.out.push_str(&format!("upright(\"{unit}\")"));
            }
            "num" => {
                let val = self.read_arg_raw();
                // Handle scientific notation: 1.23e4 → 1.23 × 10^4
                if let Some(pos) = val.find(['e', 'E']) {
                    let (mantissa, exp) = val.split_at(pos);
                    let exp = &exp[1..]; // skip 'e'/'E'
                    self.out.push_str(&format!("{mantissa} times 10^({exp})"));
                } else {
                    self.out.push_str(&val);
                }
            }
            "ang" => {
                // \ang{45} → 45°  or  \ang{45;30;00} → 45°30'00"
                let val = self.read_arg_raw();
                if val.contains(';') {
                    let parts: Vec<&str> = val.split(';').collect();
                    match parts.as_slice() {
                        [d, m]    => self.out.push_str(&format!("{d} degree {m}'")),
                        [d, m, s] => self.out.push_str(&format!("{d} degree {m}' {s}\"")),
                        _         => self.out.push_str(&format!("{val} degree")),
                    }
                } else {
                    self.out.push_str(&format!("{val} degree"));
                }
            }
            // Common siunitx unit macros
            "ohm"  => { self.out.push_str("Omega"); }
            "micro" => { self.out.push_str("mu"); }
            "kilo"  => { self.out.push_str("\"k\""); }
            "mega"  => { self.out.push_str("\"M\""); }
            "giga"  => { self.out.push_str("\"G\""); }
            "milli" => { self.out.push_str("\"m\""); }
            "nano"  => { self.out.push_str("\"n\""); }
            "pico"  => { self.out.push_str("\"p\""); }
            "tera"  => { self.out.push_str("\"T\""); }

            // ── Spacing: \, etc already caught above; bare \  is NewRow ──
            _ => {
                let typst = symbol_map(cmd).unwrap_or(cmd);
                // Space BEFORE: prevent "ipi", "xalpha" etc. from concatenating
                if self.out.chars().last().is_some_and(|c| c.is_alphanumeric()) {
                    self.out.push(' ');
                }
                self.out.push_str(typst);
                // \limits / \nolimits may follow immediately
                if let Some(Token::Cmd(next)) = self.peek() {
                    match next.as_str() {
                        "limits" => {
                            self.advance();
                            let len = typst.len();
                            let cut = self.out.len() - len;
                            let sym = self.out[cut..].to_string();
                            self.out.truncate(cut);
                            self.out.push_str(&format!("limits({sym})"));
                            return;
                        }
                        "nolimits" => {
                            self.advance();
                            let len = typst.len();
                            let cut = self.out.len() - len;
                            let sym = self.out[cut..].to_string();
                            self.out.truncate(cut);
                            self.out.push_str(&format!("scripts({sym})"));
                            return;
                        }
                        _ => {}
                    }
                }
                // Space AFTER: prevent the symbol running into following content
                if matches!(self.peek(), Some(t) if needs_space_after(t)) {
                    self.out.push(' ');
                }
            }
        }
    }

    // ── wrap_last_symbol ─────────────────────────────────────────────────────
    /// Wraps the last emitted identifier token in `wrapper(...)`.
    /// Used for `\limits` → `limits(sum)`, `\nolimits` → `scripts(integral)`, etc.
    fn wrap_last_symbol(&mut self, wrapper: &str) {
        // Find the start of the last "word" (run of alphanumeric / dot chars)
        let bytes = self.out.as_bytes();
        let mut end = bytes.len();
        // Trim trailing space
        while end > 0 && bytes[end - 1] == b' ' { end -= 1; }
        let mut start = end;
        while start > 0 {
            let c = bytes[start - 1] as char;
            if c.is_alphanumeric() || c == '.' || c == '_' { start -= 1; }
            else { break; }
        }
        if start < end {
            let sym = self.out[start..end].to_string();
            self.out.truncate(start);
            self.out.push_str(&format!("{wrapper}({sym})"));
        }
    }

    // ── Environment emitter ──────────────────────────────────────────────────

    fn emit_env(&mut self, env: &str) {
        match env {
            "matrix"      => self.emit_matrix("none"),
            "pmatrix"     => self.emit_matrix("("),
            "bmatrix"     => self.emit_matrix("["),
            "Bmatrix"     => self.emit_matrix("{"),
            "vmatrix"     => self.emit_matrix("|"),
            "Vmatrix"     => self.emit_matrix("‖"),
            "smallmatrix" => self.emit_matrix("none"),
            "cases"       => self.emit_cases(false),
            "cases*"      => self.emit_cases(true),
            "rcases"      => self.emit_cases(false),
            "align" | "align*" | "aligned"
            | "alignat" | "alignat*"
            | "split" => self.emit_aligned(),
            "gather" | "gather*" | "gathered" => self.emit_aligned(),
            "array" => {
                // Consume optional [pos] and {cols} spec then treat as matrix
                self.read_opt();
                if matches!(self.peek(), Some(Token::Open)) { self.read_group(); }
                self.emit_matrix("none");
            }
            _ => {
                // Unknown env: just convert content
                let inner_toks = self.read_until_end();
                let converted = sub_convert(&inner_toks);
                self.out.push_str(&converted);
            }
        }
    }

    fn emit_matrix(&mut self, delim: &str) {
        let inner = self.read_until_end();
        let rows = split_rows_cols(&inner);

        let mat_content: Vec<String> = rows.iter().map(|row| {
            row.iter()
                .map(|cell| sub_convert(cell).trim().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        }).collect();

        let body = mat_content.join("; ");
        if delim == "none" {
            self.out.push_str(&format!("mat(delim: none, {body})"));
        } else if delim == "(" {
            self.out.push_str(&format!("mat({body})"));
        } else {
            self.out.push_str(&format!("mat(delim: \"{delim}\", {body})"));
        }
    }

    fn emit_cases(&mut self, _reverse: bool) {
        let inner = self.read_until_end();
        let rows = split_rows_cols(&inner);

        let lines: Vec<String> = rows.iter().map(|row| {
            row.iter()
                .map(|cell| sub_convert(cell).trim().to_string())
                .collect::<Vec<_>>()
                .join("  ")   // columns within a case line just spaced
        }).collect();

        let body = lines.join(",\n  ");
        self.out.push_str(&format!("cases(\n  {body}\n)"));
    }

    fn emit_aligned(&mut self) {
        let inner = self.read_until_end();
        // In aligned environments, \\ becomes a line break and & stays
        // Typst math: use `\` (single backslash via NewRow) and `&`
        let mut sub = Converter::new(&inner);
        while sub.pos < sub.toks.len() {
            match sub.toks[sub.pos].clone() {
                Token::NewRow => {
                    sub.advance();
                    sub.out.push_str(" \\\n");
                }
                _ => sub.step(),
            }
        }
        self.out.push_str(&sub.out);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert a slice of tokens as a sub-expression.
fn sub_convert(toks: &[Token]) -> String {
    let mut c = Converter::new(toks);
    c.run();
    c.out
}

fn token_to_str(tok: &Token) -> String {
    match tok {
        Token::Char(c)  => c.to_string(),
        Token::Cmd(s)   => format!("\\{s}"),
        Token::Open     => "{".into(),
        Token::Close    => "}".into(),
        Token::Sub      => "_".into(),
        Token::Sup      => "^".into(),
        Token::Amp      => "&".into(),
        Token::NewRow   => "\\\\".into(),
    }
}

fn opt_token_to_str(tok: &Option<Token>) -> String {
    tok.as_ref().map(token_to_str).unwrap_or_default()
}

/// Split a flat token slice by NewRow (rows) then by Amp (columns).
/// Returns Vec<row> where each row is Vec<cell> of token slices (owned).
fn split_rows_cols(toks: &[Token]) -> Vec<Vec<Vec<Token>>> {
    let mut rows: Vec<Vec<Vec<Token>>>  = Vec::new();
    let mut cur_row: Vec<Vec<Token>>    = vec![Vec::new()];
    let mut depth = 0usize;

    for tok in toks {
        match tok {
            Token::Open  => { depth += 1; cur_row.last_mut().unwrap().push(tok.clone()); }
            Token::Close => { depth -= 1; cur_row.last_mut().unwrap().push(tok.clone()); }
            Token::Amp if depth == 0 => {
                cur_row.push(Vec::new());
            }
            Token::NewRow if depth == 0 => {
                rows.push(cur_row);
                cur_row = vec![Vec::new()];
            }
            _ => { cur_row.last_mut().unwrap().push(tok.clone()); }
        }
    }

    // Push the last row if it has any non-empty cells
    if cur_row.iter().any(|c| !c.is_empty()) {
        rows.push(cur_row);
    }
    rows
}

fn needs_space_after(tok: &Token) -> bool {
    matches!(tok, Token::Char(c) if c.is_alphanumeric() || *c == '{')
        || matches!(tok, Token::Open | Token::Cmd(_))
}

// ── mhchem \ce{} converter ────────────────────────────────────────────────────
//
// Converts a raw mhchem chemical-formula string into Typst math notation.
// Handles: elements, subscripts, superscripts (charges), reaction arrows,
// state symbols (s/l/g/aq), grouping brackets, coefficients, LaTeX commands.
//
// Examples:
//   "H2O"              → upright("H")_2 upright("O")
//   "Na+"              → upright("Na")^+
//   "Ca^2+"            → upright("Ca")^(2+)
//   "H2 + Cl2 -> 2HCl" → ... arrow.r ...
//   "A <=> B"          → ... harpoons.rtlb ...
fn ce_convert(raw: &str) -> String {
    let chars: Vec<char> = raw.trim().chars().collect();
    let n = chars.len();
    let mut out = String::new();
    let mut i = 0;

    while i < n {
        // ── Multi-char reaction arrows (longest match first) ──────────────
        if i + 2 < n && chars[i] == '<' && chars[i+1] == '=' && chars[i+2] == '>' {
            out.push_str(" harpoons.rtlb "); i += 3; continue;
        }
        if i + 2 < n && chars[i] == '<' && chars[i+1] == '-' && chars[i+2] == '>' {
            out.push_str(" harpoons.rtlb "); i += 3; continue;
        }
        if i + 1 < n && chars[i] == '-' && chars[i+1] == '>' {
            out.push_str(" arrow.r "); i += 2; continue;
        }
        if i + 1 < n && chars[i] == '<' && chars[i+1] == '-' {
            out.push_str(" arrow.l "); i += 2; continue;
        }
        if i + 1 < n && chars[i] == '=' && chars[i+1] == '>' {
            out.push_str(" arrow.r.double "); i += 2; continue;
        }

        match chars[i] {
            // ── LaTeX command ─────────────────────────────────────────────
            '\\' => {
                i += 1;
                if i >= n { break; }
                if chars[i].is_alphabetic() {
                    let mut cmd = String::new();
                    while i < n && chars[i].is_alphabetic() { cmd.push(chars[i]); i += 1; }
                    if i < n && chars[i] == ' ' { i += 1; } // consume trailing space
                    let mapped = match cmd.as_str() {
                        "Delta" => "Delta",
                        "delta" => "delta",
                        "rightarrow" | "to"            => " arrow.r ",
                        "leftarrow"                    => " arrow.l ",
                        "Rightarrow"                   => " arrow.r.double ",
                        "rightleftharpoons"            => " harpoons.rtlb ",
                        "leftrightharpoons"            => " harpoons.ltrb ",
                        "cdot"                         => " dot.op ",
                        "cdots"                        => " dots.c ",
                        "ldots"                        => " dots.h ",
                        "quad"                         => " quad ",
                        "text" | "mathrm" | "mbox"     => "",  // handled by brace content below
                        "ce"                           => "",
                        other                          => other,
                    };
                    out.push_str(mapped);
                } else {
                    match chars[i] {
                        ',' | ';' => { out.push(' '); i += 1; }
                        c         => { out.push(c); i += 1; }
                    }
                }
            }
            // ── Chemical element symbol ───────────────────────────────────
            c if c.is_uppercase() => {
                let mut elem = c.to_string();
                i += 1;
                // Consume lowercase continuation (e.g. 'Na', 'Ca', 'Cl')
                while i < n && chars[i].is_lowercase() { elem.push(chars[i]); i += 1; }
                // Add separator space between consecutive symbols
                if !out.is_empty() && !out.ends_with(' ') { out.push(' '); }
                out.push_str(&format!("upright(\"{elem}\")"));
                // Subscript: digits immediately following element
                if i < n && chars[i].is_ascii_digit() {
                    let mut sub = String::new();
                    while i < n && chars[i].is_ascii_digit() { sub.push(chars[i]); i += 1; }
                    if sub.len() == 1 { out.push_str(&format!("_{sub}")); }
                    else              { out.push_str(&format!("_({sub})")); }
                }
            }
            // ── Explicit superscript ──────────────────────────────────────
            '^' => {
                i += 1;
                if i < n && chars[i] == '{' {
                    i += 1;
                    let mut sup = String::new();
                    while i < n && chars[i] != '}' { sup.push(chars[i]); i += 1; }
                    if i < n { i += 1; } // consume '}'
                    if sup.len() <= 1 { out.push_str(&format!("^{sup}")); }
                    else              { out.push_str(&format!("^({sup})")); }
                } else {
                    // Collect digits + charges until non-superscript char
                    let mut sup = String::new();
                    while i < n && (chars[i].is_ascii_digit() || chars[i] == '+' || chars[i] == '-') {
                        sup.push(chars[i]); i += 1;
                    }
                    if sup.is_empty() {}
                    else if sup.len() == 1 { out.push('^'); out.push_str(&sup); }
                    else                   { out.push_str(&format!("^({sup})")); }
                }
            }
            // ── Explicit subscript ────────────────────────────────────────
            '_' => {
                i += 1;
                if i < n && chars[i] == '{' {
                    i += 1;
                    let mut sub = String::new();
                    while i < n && chars[i] != '}' { sub.push(chars[i]); i += 1; }
                    if i < n { i += 1; }
                    if sub.len() == 1 { out.push_str(&format!("_{sub}")); }
                    else              { out.push_str(&format!("_({sub})")); }
                } else if i < n {
                    out.push('_'); out.push(chars[i]); i += 1;
                }
            }
            // ── Parenthesised state symbol or ligand group ────────────────
            '(' => {
                i += 1;
                let mut content = String::new();
                let mut depth = 1usize;
                while i < n {
                    if chars[i] == '(' { depth += 1; }
                    if chars[i] == ')' { depth -= 1; if depth == 0 { i += 1; break; } }
                    content.push(chars[i]); i += 1;
                }
                match content.trim() {
                    "s" | "l" | "g" | "aq" | "cr" | "cd" | "am" => {
                        // State symbol → upright small text
                        out.push_str(&format!("upright(\"({})\")", content.trim()));
                    }
                    other => {
                        // Ligand group or generic group → recurse
                        out.push('(');
                        out.push_str(&ce_convert(other));
                        out.push(')');
                    }
                }
                // Subscript after closing paren: e.g. (NH3)4
                if i < n && chars[i].is_ascii_digit() {
                    let mut sub = String::new();
                    while i < n && chars[i].is_ascii_digit() { sub.push(chars[i]); i += 1; }
                    if sub.len() == 1 { out.push_str(&format!("_{sub}")); }
                    else              { out.push_str(&format!("_({sub})")); }
                }
            }
            // ── Square brackets (complex ions) ────────────────────────────
            '[' => { out.push('['); i += 1; }
            ']' => {
                out.push(']'); i += 1;
                // Subscript/superscript after ] — e.g. [CuCl4]^2-
            }
            // ── Coefficient / standalone digits ───────────────────────────
            c if c.is_ascii_digit() => {
                let mut num = String::new();
                while i < n && chars[i].is_ascii_digit() { num.push(chars[i]); i += 1; }
                if i < n && chars[i] == '/' {
                    // Fractional coefficient: 1/2
                    i += 1;
                    let mut den = String::new();
                    while i < n && chars[i].is_ascii_digit() { den.push(chars[i]); i += 1; }
                    out.push_str(&format!("frac({num}, {den})"));
                } else {
                    out.push_str(&num);
                }
            }
            // ── Plus: compound operator ' + ' or ionic charge 'Na+' ───────
            '+' => {
                // Rule: if the PREVIOUS output ends with a space → compound separator.
                // Otherwise (no preceding space, e.g. "Na+") → ionic charge ^+.
                let prev_sp = out.ends_with(' ') || out.is_empty();
                if prev_sp {
                    // " + "  style (there was already a space before)
                    out.push_str("+ ");
                } else {
                    // Trailing charge: Na+ or 2+
                    out.push_str("^+");
                }
                i += 1;
            }
            // ── Minus: usually trailing charge (Cl-), rarely subtraction ──
            '-' => {
                let prev_sp = out.ends_with(' ') || out.is_empty();
                if prev_sp {
                    out.push_str(" - ");
                } else {
                    out.push_str("^-");
                }
                i += 1;
            }
            // ── Dot: bond or multiplication ───────────────────────────────
            '.' => {
                out.push_str(" dot.c "); i += 1;
            }
            // ── Space ─────────────────────────────────────────────────────
            ' ' => {
                if !out.ends_with(' ') { out.push(' '); }
                i += 1;
            }
            // ── Other characters ──────────────────────────────────────────
            c => { out.push(c); i += 1; }
        }
    }

    // Clean up multiple consecutive spaces
    let result = out.trim().to_string();
    // Collapse " +" / "+ " sequences (shouldn't be needed but just in case)
    result
}

// ── Symbol map ────────────────────────────────────────────────────────────────

fn symbol_map(cmd: &str) -> Option<&'static str> {
    Some(match cmd {
        // ── Greek lowercase ──────────────────────────────────────────────────
        "alpha"      => "alpha",       "beta"       => "beta",
        "gamma"      => "gamma",       "delta"      => "delta",
        "epsilon"    => "epsilon",     "varepsilon" => "epsilon.alt",
        "zeta"       => "zeta",        "eta"        => "eta",
        "theta"      => "theta",       "vartheta"   => "theta.alt",
        "iota"       => "iota",        "kappa"      => "kappa",
        "lambda"     => "lambda",      "mu"         => "mu",
        "nu"         => "nu",          "xi"         => "xi",
        "pi"         => "pi",          "varpi"      => "pi.alt",
        "rho"        => "rho",         "varrho"     => "rho.alt",
        "sigma"      => "sigma",       "varsigma"   => "sigma.alt",
        "tau"        => "tau",         "upsilon"    => "upsilon",
        "phi"        => "phi",         "varphi"     => "phi.alt",
        "chi"        => "chi",         "psi"        => "psi",
        "omega"      => "omega",
        // ── Greek uppercase ──────────────────────────────────────────────────
        "Gamma"   => "Gamma",   "Delta"   => "Delta",
        "Theta"   => "Theta",   "Lambda"  => "Lambda",
        "Xi"      => "Xi",      "Pi"      => "Pi",
        "Sigma"   => "Sigma",   "Upsilon" => "Upsilon",
        "Phi"     => "Phi",     "Psi"     => "Psi",
        "Omega"   => "Omega",
        // ── Arithmetic ──────────────────────────────────────────────────────
        "times"  => "times",      "div"    => "div",
        "pm"     => "plus.minus", "mp"     => "minus.plus",
        "cdot"   => "dot.op",     "cdots"  => "dots.c",
        "ldots"  => "dots.h",     "vdots"  => "dots.v",
        "ddots"  => "dots.d",     "iddots" => "dots.up",
        // ── Relations ───────────────────────────────────────────────────────
        "leq" | "le"       => "lt.eq",       "geq" | "ge" => "gt.eq",
        "neq" | "ne"       => "eq.not",      "approx"     => "approx",
        "sim"              => "tilde",        "simeq"      => "tilde.eq",
        "cong"             => "tilde.equiv",  "equiv"      => "equiv",
        "propto"           => "prop",         "asymp"      => "asymp",
        "ll"               => "lt.double",    "gg"         => "gt.double",
        "leqslant"         => "lt.eq.slant",  "geqslant"   => "gt.eq.slant",
        "prec"             => "prec",         "succ"       => "succ",
        "preceq"           => "prec.eq",      "succeq"     => "succ.eq",
        "perp"             => "perp",         "mid"        => "divides",
        "nmid"             => "divides.not",  "parallel"   => "parallel",
        "doteq"            => "dot.eq",
        "lesssim"          => "lt.tilde",     "gtrsim"     => "gt.tilde",
        "lessapprox"       => "lt.approx",    "gtrapprox"  => "gt.approx",
        "nleq"             => "lt.eq.not",    "ngeq"       => "gt.eq.not",
        "nleqslant"        => "lt.eq.not",    "ngeqslant"  => "gt.eq.not",
        "nless"            => "lt.not",       "ngtr"       => "gt.not",
        "nprec"            => "prec.not",     "nsucc"      => "succ.not",
        "lhd" | "vartriangleleft"  => "triangle.stroked.l.small",
        "rhd" | "vartriangleright" => "triangle.stroked.r.small",
        "unlhd" | "trianglelefteq"  => "triangle.stroked.l.eq",
        "unrhd" | "trianglerighteq" => "triangle.stroked.r.eq",
        // ── Proof / turnstile ───────────────────────────────────────────────
        "vdash"       => "tack.r",      "dashv"      => "tack.l",
        "Vdash"       => "tack.r.double","vDash"     => "tack.r.double.not",
        "models"      => "models",      "nvdash"     => "tack.r.not",
        "nVdash"      => "tack.r.nequiv","nvDash"    => "tack.r.not",
        "therefore"   => "therefore",   "because"    => "because",
        // ── Defined-as / colonrelations ──────────────────────────────────────
        "coloneq" | "coloneqq" => "colon.eq",
        "eqcolon"              => "eq.colon",
        "colonsim"             => "colon.tilde",
        "Coloneqq"             => "colon.double.eq",
        // ── Arrows ──────────────────────────────────────────────────────────
        "to" | "rightarrow"          => "->",
        "leftarrow"                  => "<-",
        "Rightarrow"                 => "=>",
        "Leftarrow"                  => "<=",
        "Leftrightarrow"             => "<=>",
        "leftrightarrow"             => "<->",
        "mapsto"                     => "arrow.r.bar",
        "longmapsto"                 => "arrow.r.long.bar",
        "longrightarrow"             => "arrow.r.long",
        "longleftarrow"              => "arrow.l.long",
        "Longrightarrow"             => "arrow.r.double.long",
        "Longleftarrow"              => "arrow.l.double.long",
        "Longleftrightarrow"         => "arrow.l.r.double.long",
        "longleftrightarrow"         => "arrow.l.r.long",
        "uparrow"                    => "arrow.t",
        "downarrow"                  => "arrow.b",
        "updownarrow"                => "arrow.t.b",
        "Uparrow"                    => "arrow.t.double",
        "Downarrow"                  => "arrow.b.double",
        "Updownarrow"                => "arrow.t.b.double",
        "nearrow"                    => "arrow.tr",
        "nwarrow"                    => "arrow.tl",
        "searrow"                    => "arrow.br",
        "swarrow"                    => "arrow.bl",
        "hookrightarrow"             => "arrow.r.hook",
        "hookleftarrow"              => "arrow.l.hook",
        "rightharpoonup"             => "harpoon.rt",
        "rightharpoondown"           => "harpoon.rb",
        "leftharpoonup"              => "harpoon.lt",
        "leftharpoondown"            => "harpoon.lb",
        "rightleftharpoons"          => "harpoons.rtlb",
        "leftrightharpoons"          => "harpoons.ltrb",
        "nrightarrow"                => "arrow.r.not",
        "nleftarrow"                 => "arrow.l.not",
        "nRightarrow"                => "arrow.r.double.not",
        "nLeftarrow"                 => "arrow.l.double.not",
        "dashrightarrow"             => "arrow.r.dashed",
        "dashleftarrow"              => "arrow.l.dashed",
        "twoheadrightarrow"          => "arrow.r.twohead",
        "twoheadleftarrow"           => "arrow.l.twohead",
        "rightarrowtail"             => "arrow.r.tail",
        "leftarrowtail"              => "arrow.l.tail",
        // ── Calculus / analysis ──────────────────────────────────────────────
        "int"     => "integral",          "iint"    => "integral.double",
        "iiint"   => "integral.triple",   "iiiint"  => "integral.quad",
        "oint"    => "integral.cont",     "oiint"   => "integral.surf",
        "oiiint"  => "integral.vol",
        "sum"     => "sum",               "prod"    => "product",
        "coprod"  => "product.co",
        "lim"     => "lim",               "inf"     => "inf",
        "sup"     => "sup",               "liminf"  => "liminf",
        "limsup"  => "limsup",            "varliminf" => "liminf",
        "varlimsup" => "limsup",
        "max"     => "max",               "min"     => "min",
        "partial" => "partial",           "nabla"   => "nabla",
        "infty"   => "infinity",          "hbar"    => "planck.reduce",
        "hslash"  => "planck.reduce",
        // ── Trig / log / standard functions ─────────────────────────────────
        "sin"    => "sin",    "cos"    => "cos",    "tan"    => "tan",
        "cot"    => "cot",    "sec"    => "sec",    "csc"    => "csc",
        "arcsin" => "arcsin", "arccos" => "arccos", "arctan" => "arctan",
        "sinh"   => "sinh",   "cosh"   => "cosh",   "tanh"   => "tanh",
        "coth"   => "coth",
        "log"    => "log",    "ln"     => "ln",     "exp"    => "exp",
        "det"    => "det",    "dim"    => "dim",    "deg"    => "deg",
        "ker"    => "ker",    "hom"    => "hom",    "arg"    => "arg",
        "gcd"    => "gcd",    "lcm"    => "lcm",    "Pr"     => "Pr",
        "Re"     => "Re",     "Im"     => "Im",
        "mod"    => "mod",
        // ── Set theory ──────────────────────────────────────────────────────
        "in"          => "in",          "notin"     => "in.not",
        "ni"          => "in.rev",
        "subset"      => "subset",      "supset"    => "supset",
        "subseteq"    => "subset.eq",   "supseteq"  => "supset.eq",
        "subsetneq"   => "subset.neq",  "supsetneq" => "supset.neq",
        "sqsubset"    => "subset.sq",   "sqsupset"  => "supset.sq",
        "sqsubseteq"  => "subset.sq.eq","sqsupseteq"=> "supset.sq.eq",
        "cup"         => "union",       "cap"       => "sect",
        "sqcup"       => "union.sq",    "sqcap"     => "sect.sq",
        "uplus"       => "union.plus",
        "bigcup"      => "union.big",   "bigcap"    => "sect.big",
        "bigsqcup"    => "union.sq.big","biguplus"  => "union.plus.big",
        "setminus"    => "without",     "emptyset"  => "emptyset",
        "varnothing"  => "emptyset",    "complement"=> "complement",
        // ── Logic ────────────────────────────────────────────────────────────
        "forall"          => "forall",      "exists"   => "exists",
        "nexists"         => "exists.not",
        "neg" | "lnot"    => "not",
        "land" | "wedge"  => "and",
        "lor"  | "vee"    => "or",
        "veebar"          => "xor",
        "barwedge"        => "and.not",      "barvee"  => "or.not",   // NAND ⊼, NOR ⊽
        "curlyvee"        => "or.curly",     "curlywedge" => "and.curly",
        "oplus"           => "plus.circle",  "ominus"  => "minus.circle",
        "otimes"          => "times.circle", "oslash"  => "div.circle",
        "odot"            => "dot.circle",
        "bigoplus"        => "plus.circle.big",
        "bigotimes"       => "times.circle.big",
        "bigodot"         => "dot.circle.big",
        "bigvee"          => "or.big",       "bigwedge" => "and.big",
        // ── Geometry ────────────────────────────────────────────────────────
        "angle"           => "angle",
        "measuredangle"   => "angle.arc",
        "sphericalangle"  => "angle.spheric",
        "triangle"        => "triangle.stroked.t",
        "triangledown"    => "triangle.stroked.b",
        "triangleleft"    => "triangle.stroked.l",
        "triangleright"   => "triangle.stroked.r",
        "circ"            => "circle.small",
        "bullet"          => "circle.filled.small",
        "square"          => "square.stroked",
        "blacksquare"     => "square.filled",
        "lozenge"         => "lozenge.stroked",
        "Diamond"         => "diamond.stroked",
        // ── Delimiters ───────────────────────────────────────────────────────
        "langle"       => "angle.l",    "rangle"      => "angle.r",
        "lfloor"       => "floor.l",    "rfloor"      => "floor.r",
        "lceil"        => "ceil.l",     "rceil"       => "ceil.r",
        "vert"         => "|",          "Vert"        => "||",
        "lVert"        => "||",         "rVert"       => "||",
        "lvert"        => "|",          "rvert"       => "|",
        // ── Misc ─────────────────────────────────────────────────────────────
        "ell"          => "ell",
        "top"          => "top",        "bot"      => "bot",
        "dagger"       => "dagger",     "dag"      => "dagger",
        "ddagger"      => "dagger.double", "ddag"  => "dagger.double",
        "star"         => "star",       "ast"      => "ast",
        "aleph"        => "aleph",      "beth"     => "beth",
        "gimel"        => "gimel",
        "wp"           => "weierstrass",
        "imath"        => "dotless.i",  "jmath"    => "dotless.j",
        "prime"        => "prime",      "backprime"=> "prime.rev",
        "flat"         => "music.flat", "sharp"    => "music.sharp",
        "natural"      => "music.natural",
        "clubsuit"     => "suit.club",  "diamondsuit" => "suit.diamond",
        "heartsuit"    => "suit.heart", "spadesuit"   => "suit.spade",
        "pounds"       => "pound",      "yen"      => "yen",
        "euro"         => "euro",
        "degree"       => "degree",
        "triangleq"    => "eq.delta",
        "circeq"       => "eq.circle",
        "bowtie"       => "bowtie",
        "ltimes"       => "times.l",    "rtimes"   => "times.r",
        "divideontimes"=> "times.div",
        "wr"           => "wreath",
        "amalg"        => "product.co",
        "intercal"     => "top.big",
        "checkmark"    => "checkmark",  "maltese"  => "maltese",
        "mho"          => "ohm.inv",
        "Finv"         => "finv",       "Game"     => "game",
        "P"            => "pilcrow",    "S"        => "section",
        "copyright"    => "copyright",  "ae"       => "ae",
        "oe"           => "oe",
        _ => return None,
    })
}

/// Map `\not\cmd` to negated Typst symbol.
fn negation_map(cmd: &str) -> Option<&'static str> {
    Some(match cmd {
        "in"       => "in.not",      "ni"      => "in.rev.not",
        "subset"   => "subset.not",  "supset"  => "supset.not",
        "subseteq" => "subset.eq.not","supseteq"=> "supset.eq.not",
        "leq" | "le" => "lt.eq.not", "geq" | "ge" => "gt.eq.not",
        "prec"     => "prec.not",    "succ"    => "succ.not",
        "sim"      => "tilde.not",   "cong"    => "tilde.equiv.not",
        "equiv"    => "equiv.not",   "parallel"=> "parallel.not",
        "perp"     => "perp.not",
        _          => return None,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::convert;

    // ── Fractions ──────────────────────────────────────────────────────────

    #[test]
    fn frac() { assert_eq!(convert(r"\frac{a}{b}"), "frac(a, b)"); }

    #[test]
    fn dfrac() { assert_eq!(convert(r"\dfrac{1}{2}"), "frac(1, 2)"); }

    #[test]
    fn nested_frac() {
        assert_eq!(convert(r"\frac{\frac{a}{b}}{c}"), "frac(frac(a, b), c)");
    }

    #[test]
    fn binom() { assert_eq!(convert(r"\binom{n}{k}"), "binom(n, k)"); }

    // ── Roots ─────────────────────────────────────────────────────────────

    #[test]
    fn sqrt_simple() { assert_eq!(convert(r"\sqrt{x}"), "sqrt(x)"); }

    #[test]
    fn sqrt_nth() { assert_eq!(convert(r"\sqrt[3]{x}"), "root(3, x)"); }

    #[test]
    fn sqrt_single_char() { assert_eq!(convert(r"\sqrt x"), "sqrt(x)"); }

    // ── Super/subscript ────────────────────────────────────────────────────

    #[test]
    fn superscript_single() { assert_eq!(convert(r"x^2"), "x^2"); }

    #[test]
    fn superscript_multi() { assert_eq!(convert(r"x^{n+1}"), "x^(n+1)"); }

    #[test]
    fn subscript_single() { assert_eq!(convert(r"x_i"), "x_i"); }

    #[test]
    fn subscript_multi() { assert_eq!(convert(r"x_{i+1}"), "x_(i+1)"); }

    #[test]
    fn sum_with_bounds() {
        assert_eq!(convert(r"\sum_{i=0}^{n} i"), "sum_(i=0)^n i");
    }

    // ── Greek ─────────────────────────────────────────────────────────────

    #[test]
    fn greek_lower() {
        assert_eq!(convert(r"\alpha"), "alpha");
        assert_eq!(convert(r"\varepsilon"), "epsilon.alt");
        assert_eq!(convert(r"\varphi"), "phi.alt");
        assert_eq!(convert(r"\omega"), "omega");
    }

    #[test]
    fn greek_upper() {
        assert_eq!(convert(r"\Sigma"), "Sigma");
        assert_eq!(convert(r"\Omega"), "Omega");
    }

    // ── Operators ─────────────────────────────────────────────────────────

    #[test]
    fn operators() {
        assert_eq!(convert(r"\times"), "times");
        assert_eq!(convert(r"\div"), "div");
        assert_eq!(convert(r"\pm"), "plus.minus");
        assert_eq!(convert(r"\cdot"), "dot.op");
    }

    #[test]
    fn relations() {
        assert_eq!(convert(r"\leq"), "lt.eq");
        assert_eq!(convert(r"\geq"), "gt.eq");
        assert_eq!(convert(r"\neq"), "eq.not");
        assert_eq!(convert(r"\approx"), "approx");
        assert_eq!(convert(r"\cong"), "tilde.equiv");
        assert_eq!(convert(r"\equiv"), "equiv");
    }

    #[test]
    fn arrows() {
        assert_eq!(convert(r"\to"), "->");
        assert_eq!(convert(r"\Rightarrow"), "=>");
        assert_eq!(convert(r"\leftrightarrow"), "<->");
        assert_eq!(convert(r"\mapsto"), "arrow.r.bar");
        assert_eq!(convert(r"\uparrow"), "arrow.t");
        assert_eq!(convert(r"\hookrightarrow"), "arrow.r.hook");
    }

    // ── Calculus ──────────────────────────────────────────────────────────

    #[test]
    fn integral_plain() {
        assert_eq!(convert(r"\int x"), "integral x");
    }

    #[test]
    fn integral_brace() {
        assert_eq!(convert(r"\int{x^2}dx"), "integral x^2 d x");
    }

    #[test]
    fn integral_with_bounds() {
        assert_eq!(
            convert(r"\int_0^{\infty} e^{-x} dx"),
            "integral_0^(infinity) e^(-x) d x"
        );
    }

    #[test]
    fn limit_expr() {
        assert_eq!(
            convert(r"\lim_{x \to 0} \frac{\sin x}{x}"),
            "lim_(x -> 0) frac(sin x, x)"
        );
    }

    // ── Differentials ─────────────────────────────────────────────────────

    #[test]
    fn diff_dx() { assert_eq!(convert(r"\int f(x)dx"), "integral f(x) d x"); }

    #[test]
    fn diff_dy() { assert_eq!(convert(r"\int f(y)dy"), "integral f(y) d y"); }

    #[test]
    fn diff_no_split_word() { assert_eq!(convert("add"), "add"); }

    #[test]
    fn diff_no_split_word_div() {
        // \operatorname{div} must stay "div", not "d iv"
        assert_eq!(convert(r"\operatorname{div}"), "upright(div)");
    }

    // ── Accents ───────────────────────────────────────────────────────────

    #[test]
    fn accents() {
        assert_eq!(convert(r"\hat{x}"), "hat(x)");
        assert_eq!(convert(r"\tilde{x}"), "tilde(x)");
        assert_eq!(convert(r"\bar{x}"), "overline(x)");
        assert_eq!(convert(r"\vec{v}"), "arrow(v)");
        assert_eq!(convert(r"\dot{x}"), "dot(x)");
        assert_eq!(convert(r"\ddot{x}"), "dot.double(x)");
        assert_eq!(convert(r"\breve{x}"), "breve(x)");
        assert_eq!(convert(r"\check{x}"), "caron(x)");
    }

    // ── Over/under ────────────────────────────────────────────────────────

    #[test]
    fn overline_underline() {
        assert_eq!(convert(r"\overline{AB}"), "overline(AB)");
        assert_eq!(convert(r"\underline{x}"), "underline(x)");
    }

    #[test]
    fn overset() {
        assert_eq!(convert(r"\overset{!}{=}"), "overset(=, !)");
    }

    #[test]
    fn underset() {
        assert_eq!(convert(r"\underset{n \to \infty}{\lim}"), "underset(lim, n -> infinity)");
    }

    // ── Font commands ─────────────────────────────────────────────────────

    #[test]
    fn mathbb() {
        assert_eq!(convert(r"\mathbb{R}"), "RR");
        assert_eq!(convert(r"\mathbb{Z}"), "ZZ");
        assert_eq!(convert(r"\mathbb{N}"), "NN");
        assert_eq!(convert(r"\mathbb{C}"), "CC");
    }

    #[test]
    fn mathbf() { assert_eq!(convert(r"\mathbf{v}"), "bold(v)"); }

    #[test]
    fn mathcal() { assert_eq!(convert(r"\mathcal{L}"), "cal(L)"); }

    #[test]
    fn text_cmd() {
        assert_eq!(convert(r"\text{hello}"), "upright(\"hello\")");
    }

    #[test]
    fn operatorname() {
        assert_eq!(convert(r"\operatorname{div}"), "upright(div)");
        assert_eq!(convert(r"\mathrm{Re}"), "upright(Re)");
    }

    // ── Set / Logic ───────────────────────────────────────────────────────

    #[test]
    fn sets() {
        assert_eq!(convert(r"\in"), "in");
        assert_eq!(convert(r"\notin"), "in.not");
        assert_eq!(convert(r"\cup"), "union");
        assert_eq!(convert(r"\cap"), "sect");
        assert_eq!(convert(r"\subset"), "subset");
        assert_eq!(convert(r"\subseteq"), "subset.eq");
        assert_eq!(convert(r"\emptyset"), "emptyset");
        assert_eq!(convert(r"\varnothing"), "emptyset");
    }

    #[test]
    fn logic() {
        assert_eq!(convert(r"\forall"), "forall");
        assert_eq!(convert(r"\exists"), "exists");
        assert_eq!(convert(r"\neg"), "not");
        assert_eq!(convert(r"\land"), "and");
        assert_eq!(convert(r"\lor"), "or");
    }

    #[test]
    fn not_negation() {
        assert_eq!(convert(r"\not\in"), "in.not");
        assert_eq!(convert(r"\not\subset"), "subset.not");
    }

    // ── Spacing ───────────────────────────────────────────────────────────

    #[test]
    fn spacing() {
        assert_eq!(convert(r"\,"), "space.thin");
        assert_eq!(convert(r"\;"), "space.thin");
        assert_eq!(convert(r"\!"), "");
        assert_eq!(convert(r"\quad"), "quad");
    }

    // ── \left \right ──────────────────────────────────────────────────────

    #[test]
    fn left_right() {
        assert_eq!(convert(r"\left(x\right)"), "(x)");
        assert_eq!(convert(r"\left[x\right]"), "[x]");
    }

    #[test]
    fn left_right_invisible() {
        // \left. and \right. are invisible delimiters — just stripped
        assert_eq!(convert(r"\left. x \right|"), " x |");
    }

    // ── Matrix environments ───────────────────────────────────────────────

    #[test]
    fn pmatrix_2x2() {
        let input = r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}";
        assert_eq!(convert(input), "mat(a, b; c, d)");
    }

    #[test]
    fn bmatrix_2x2() {
        let input = r"\begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix}";
        assert_eq!(convert(input), "mat(delim: \"[\", 1, 0; 0, 1)");
    }

    #[test]
    fn matrix_3x3() {
        let input = r"\begin{pmatrix} a & b & c \\ d & e & f \\ g & h & i \end{pmatrix}";
        assert_eq!(convert(input), "mat(a, b, c; d, e, f; g, h, i)");
    }

    // ── Cases environment ─────────────────────────────────────────────────

    #[test]
    fn cases_env() {
        let input = r"\begin{cases} x & x > 0 \\ -x & x \leq 0 \end{cases}";
        let out = convert(input);
        assert!(out.starts_with("cases("), "got: {out}");
        assert!(out.contains("x > 0"), "got: {out}");
        assert!(out.contains("lt.eq"), "got: {out}");
    }

    // ── Align environment ─────────────────────────────────────────────────

    #[test]
    fn align_env() {
        let input = r"\begin{align} a &= b \\ c &= d \end{align}";
        let out = convert(input);
        assert!(out.contains('&'), "got: {out}");
        assert!(out.contains('\n'), "got: {out}");
    }

    // ── Complex formulas ──────────────────────────────────────────────────

    #[test]
    fn quadratic_formula() {
        assert_eq!(
            convert(r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}"),
            "x = frac(-b plus.minus sqrt(b^2 - 4ac), 2a)"
        );
    }

    #[test]
    fn euler_identity() {
        assert_eq!(
            convert(r"e^{i\pi} + 1 = 0"),
            "e^(i pi) + 1 = 0"
        );
    }

    #[test]
    fn gaussian_integral() {
        assert_eq!(
            convert(r"\int_{-\infty}^{\infty} e^{-x^2} dx"),
            "integral_(-infinity)^(infinity) e^(-x^2) d x"
        );
    }

    // ── Dots ──────────────────────────────────────────────────────────────

    #[test]
    fn dots() {
        assert_eq!(convert(r"\ldots"), "dots.h");
        assert_eq!(convert(r"\cdots"), "dots.c");
        assert_eq!(convert(r"\vdots"), "dots.v");
        assert_eq!(convert(r"\ddots"), "dots.d");
    }

    // ── Delimiters ────────────────────────────────────────────────────────

    #[test]
    fn delimiters() {
        assert_eq!(convert(r"\langle x \rangle"), "angle.l x angle.r");
        assert_eq!(convert(r"\lfloor x \rfloor"), "floor.l x floor.r");
        assert_eq!(convert(r"\lceil x \rceil"), "ceil.l x ceil.r");
    }

    // ── Proof symbols ─────────────────────────────────────────────────────

    #[test]
    fn proof_symbols() {
        assert_eq!(convert(r"\therefore"), "therefore");
        assert_eq!(convert(r"\because"), "because");
        assert_eq!(convert(r"\vdash"), "tack.r");
        assert_eq!(convert(r"\models"), "models");
    }

    // ── Negated relations ─────────────────────────────────────────────────

    #[test]
    fn negated_relations() {
        assert_eq!(convert(r"\nleq"), "lt.eq.not");
        assert_eq!(convert(r"\ngeq"), "gt.eq.not");
        assert_eq!(convert(r"\nless"), "lt.not");
        assert_eq!(convert(r"\ngtr"), "gt.not");
    }

    // ── Cancel ───────────────────────────────────────────────────────────

    #[test]
    fn cancel_cmds() {
        assert_eq!(convert(r"\cancel{x}"), "cancel(x)");
        assert_eq!(convert(r"\bcancel{x}"), "cancel(x, inverted: true)");
        assert_eq!(convert(r"\xcancel{x}"), "cancel(x, cross: true)");
    }

    // ── mathscr ───────────────────────────────────────────────────────────

    #[test]
    fn mathscr() {
        assert_eq!(convert(r"\mathscr{L}"), "scr(L)");
        assert_eq!(convert(r"\mathcal{L}"), "cal(L)");  // still cal, not scr
    }

    // ── underbrace with annotation ────────────────────────────────────────

    #[test]
    fn underbrace_annotated() {
        assert_eq!(convert(r"\underbrace{x+y}_{s}"), "underbrace(x+y, s)");
        assert_eq!(convert(r"\overbrace{x+y}^{s}"), "overbrace(x+y, s)");
    }

    // ── pmod / bmod ───────────────────────────────────────────────────────

    #[test]
    fn modulo_cmds() {
        assert_eq!(convert(r"a \equiv b \pmod{n}"), "a equiv b quad (mod n)");
    }

    // ── limits / nolimits ─────────────────────────────────────────────────

    #[test]
    fn limits_cmd() {
        assert_eq!(convert(r"\sum\limits_{i=0}"), "limits(sum)_(i=0)");
        assert_eq!(convert(r"\int\nolimits_0^1"), "scripts(integral)_0^1");
    }

    // ── Chemistry ─────────────────────────────────────────────────────────

    #[test]
    fn ce_element_subscripts() {
        assert_eq!(convert(r"\ce{H2O}"),   "upright(\"H\")_2 upright(\"O\")");
        assert_eq!(convert(r"\ce{CO2}"),   "upright(\"C\") upright(\"O\")_2");
        assert_eq!(convert(r"\ce{NaCl}"),  "upright(\"Na\") upright(\"Cl\")");
        assert_eq!(convert(r"\ce{H2SO4}"), "upright(\"H\")_2 upright(\"S\") upright(\"O\")_4");
    }

    #[test]
    fn ce_ions() {
        assert_eq!(convert(r"\ce{Na+}"),     "upright(\"Na\")^+");
        assert_eq!(convert(r"\ce{Cl-}"),     "upright(\"Cl\")^-");
        assert_eq!(convert(r"\ce{Ca^2+}"),   "upright(\"Ca\")^(2+)");
        assert_eq!(convert(r"\ce{SO4^2-}"),  "upright(\"S\") upright(\"O\")_4^(2-)");
    }

    #[test]
    fn ce_reaction_arrow() {
        let out = convert(r"\ce{H2 + Cl2 -> 2HCl}");
        assert!(out.contains("arrow.r"), "got: {out}");
        assert!(out.contains("upright(\"H\")_2"), "got: {out}");
        assert!(out.contains("upright(\"Cl\")_2"), "got: {out}");
    }

    #[test]
    fn ce_equilibrium() {
        let out = convert(r"\ce{N2 + 3H2 <=> 2NH3}");
        assert!(out.contains("harpoons.rtlb"), "got: {out}");
        assert!(out.contains("upright(\"N\")_2"), "got: {out}");
    }

    #[test]
    fn ce_state_symbols() {
        let out = convert(r"\ce{NaCl(aq) + AgNO3(aq) -> AgCl(s) + NaNO3(aq)}");
        assert!(out.contains("upright(\"(aq)\")"), "got: {out}");
        assert!(out.contains("upright(\"(s)\")"),  "got: {out}");
    }

    // ── Physics / Dirac notation ──────────────────────────────────────────

    #[test]
    fn bra_ket() {
        assert_eq!(convert(r"\bra{\psi}"),       "lr(angle.l psi |)");
        assert_eq!(convert(r"\ket{\phi}"),        "lr(| phi angle.r)");
        assert_eq!(convert(r"\braket{\psi}{\phi}"), "lr(angle.l psi | phi angle.r)");
    }

    #[test]
    fn mel_test() {
        assert_eq!(
            convert(r"\mel{\psi}{H}{\phi}"),
            "lr(angle.l psi | H | phi angle.r)"
        );
    }

    // ── Physics derivatives ───────────────────────────────────────────────

    #[test]
    fn dv_test() {
        assert_eq!(convert(r"\dv{f}{x}"), "frac(d f, d x)");
        assert_eq!(convert(r"\dv[2]{f}{x}"), "frac(d^(2) f, d x^(2))");
    }

    #[test]
    fn pdv_test() {
        assert_eq!(convert(r"\pdv{f}{x}"), "frac(partial f, partial x)");
    }

    #[test]
    fn dd_test() {
        assert_eq!(convert(r"\dd{x}"), "d x");
        assert_eq!(convert(r"\dd[2]{x}"), "d^(2) x");
    }

    // ── Commutator ────────────────────────────────────────────────────────

    #[test]
    fn commutator_test() {
        assert_eq!(convert(r"\comm{A}{B}"),  "[A, B]");
        assert_eq!(convert(r"\acomm{A}{B}"), "lr({A, B})");
    }

    // ── Logic NAND / NOR ─────────────────────────────────────────────────

    #[test]
    fn logic_gates() {
        assert_eq!(convert(r"\barwedge"), "and.not");
        assert_eq!(convert(r"\barvee"),   "or.not");
        assert_eq!(convert(r"\veebar"),   "xor");
    }

    // ── SI units ─────────────────────────────────────────────────────────

    #[test]
    fn si_units() {
        assert_eq!(convert(r"\si{m/s^2}"),   "upright(\"m/s^2\")");
        assert_eq!(convert(r"\SI{9.8}{m/s}"), "9.8 upright(\"m/s\")");
        assert_eq!(convert(r"\num{1.5e3}"),   "1.5 times 10^(3)");
    }
}

/// ─────────────────────────────────────────────────────────────────────────────
/// TipTap çıktısına dayalı entegrasyon testleri
///
/// Bu testler, RichBodyEditor.svelte'deki EqItem paletinden ve kullanıcıların
/// yazacağı tipik Türk lise/üniversite formüllerinden oluşur.
/// Her test, TipTap'ın `node.attrs.latex` olarak sakladığı ham LaTeX stringini
/// alıp doğru Typst çıktısı ürettiğini doğrular.
/// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tiptap_tests {
    use super::convert;

    // ── Sembol Paleti: Yunan harfleri ─────────────────────────────────────────

    #[test]
    fn palette_greek() {
        assert_eq!(convert(r"\alpha"),   "alpha");
        assert_eq!(convert(r"\beta"),    "beta");
        assert_eq!(convert(r"\gamma"),   "gamma");
        assert_eq!(convert(r"\delta"),   "delta");
        assert_eq!(convert(r"\epsilon"), "epsilon");
        assert_eq!(convert(r"\theta"),   "theta");
        assert_eq!(convert(r"\lambda"),  "lambda");
        assert_eq!(convert(r"\mu"),      "mu");
        assert_eq!(convert(r"\pi"),      "pi");
        assert_eq!(convert(r"\sigma"),   "sigma");
        assert_eq!(convert(r"\phi"),     "phi");
        assert_eq!(convert(r"\Delta"),   "Delta");
        assert_eq!(convert(r"\Sigma"),   "Sigma");
        assert_eq!(convert(r"\Omega"),   "Omega");
    }

    // ── Sembol Paleti: Aritmetik / İşlemler ──────────────────────────────────

    #[test]
    fn palette_arithmetic() {
        assert_eq!(convert(r"\times"),      "times");
        assert_eq!(convert(r"\div"),        "div");
        assert_eq!(convert(r"\pm"),         "plus.minus");
        assert_eq!(convert(r"\cdot"),       "dot.op");
        assert_eq!(convert(r"\frac{a}{b}"), "frac(a, b)");
        assert_eq!(convert(r"\sqrt{x}"),    "sqrt(x)");
        assert_eq!(convert(r"\sqrt[n]{x}"), "root(n, x)");
    }

    // ── Sembol Paleti: Üs / Alt indis (TipTap braces ile saklar) ─────────────

    #[test]
    fn palette_scripts() {
        assert_eq!(convert(r"x^{n}"),  "x^n");
        assert_eq!(convert(r"x_{n}"),  "x_n");
        assert_eq!(convert(r"x^{2}"),  "x^2");
        assert_eq!(convert(r"x_{i}"),  "x_i");
        assert_eq!(convert(r"x^{n+1}"), "x^(n+1)");
        assert_eq!(convert(r"x_{i+1}"), "x_(i+1)");
    }

    // ── Sembol Paleti: Derece (°) ─────────────────────────────────────────────

    #[test]
    fn palette_degree() {
        // TipTap'ta derece: ^{\circ} → Typst: degree
        assert_eq!(convert(r"^{\circ}"),    "degree");
        assert_eq!(convert(r"45^{\circ}"),  "45 degree");
        assert_eq!(convert(r"90^{\circ}"),  "90 degree");
        assert_eq!(convert(r"360^{\circ}"), "360 degree");
    }

    // ── Sembol Paleti: İlişkiler ──────────────────────────────────────────────

    #[test]
    fn palette_relations() {
        assert_eq!(convert(r"\leq"),   "lt.eq");
        assert_eq!(convert(r"\geq"),   "gt.eq");
        assert_eq!(convert(r"\neq"),   "eq.not");
        assert_eq!(convert(r"\approx"),"approx");
        assert_eq!(convert(r"\infty"), "infinity");
    }

    // ── Sembol Paleti: Analiz / Kalkülüs ─────────────────────────────────────

    #[test]
    fn palette_calculus() {
        assert_eq!(convert(r"\int"),          "integral");
        assert_eq!(convert(r"\int_a^b"),      "integral_a^b");
        assert_eq!(convert(r"\sum_{i=1}^{n}"), "sum_(i=1)^n");
        assert_eq!(convert(r"\lim_{x \to 0}"), "lim_(x -> 0)");
        assert_eq!(convert(r"\frac{d}{dx}"),   "frac(d, d x)");
        assert_eq!(convert(r"\partial"),       "partial");
    }

    // ── Sembol Paleti: Kümeler ────────────────────────────────────────────────

    #[test]
    fn palette_sets() {
        assert_eq!(convert(r"\in"),         "in");
        assert_eq!(convert(r"\subset"),     "subset");
        assert_eq!(convert(r"\cup"),        "union");
        assert_eq!(convert(r"\cap"),        "sect");
        assert_eq!(convert(r"\emptyset"),   "emptyset");
        assert_eq!(convert(r"\mathbb{R}"),  "RR");
    }

    // ── Sembol Paleti: Mantık ─────────────────────────────────────────────────

    #[test]
    fn palette_logic() {
        assert_eq!(convert(r"\forall"),         "forall");
        assert_eq!(convert(r"\exists"),         "exists");
        assert_eq!(convert(r"\neg"),            "not");
        assert_eq!(convert(r"\land"),           "and");
        assert_eq!(convert(r"\lor"),            "or");
        assert_eq!(convert(r"\Rightarrow"),     "=>");
        assert_eq!(convert(r"\Leftrightarrow"), "<=>");
    }

    // ── Sembol Paleti: Geometri ───────────────────────────────────────────────

    #[test]
    fn palette_geometry() {
        assert_eq!(convert(r"\angle"),    "angle");
        assert_eq!(convert(r"\parallel"), "parallel");
        assert_eq!(convert(r"\perp"),     "perp");
        assert_eq!(convert(r"\triangle"), "triangle.stroked.t");
        assert_eq!(convert(r"\vec{v}"),   "arrow(v)");
    }

    // ── Bileşik Formüller: Türk Lise Müfredatı ────────────────────────────────

    #[test]
    fn formula_polynomial() {
        // Kullanıcı parantez kutusu + superscript ile girer
        assert_eq!(
            convert(r"f(x) = x^{2} + 3x - 1"),
            "f(x) = x^2 + 3x - 1"
        );
    }

    #[test]
    fn formula_quadratic() {
        assert_eq!(
            convert(r"\frac{-b \pm \sqrt{b^{2}-4ac}}{2a}"),
            "frac(-b plus.minus sqrt(b^2-4ac), 2a)"
        );
    }

    #[test]
    fn formula_definite_integral() {
        // \, (ince boşluk) + dx ayrımı
        assert_eq!(
            convert(r"\int_{0}^{1} x^{2} \, dx"),
            "integral_0^1 x^2 space.thin d x"
        );
    }

    #[test]
    fn formula_sum_to_n() {
        assert_eq!(
            convert(r"\sum_{k=1}^{n} k = \frac{n(n+1)}{2}"),
            "sum_(k=1)^n k = frac(n(n+1), 2)"
        );
    }

    #[test]
    fn formula_set_equation() {
        assert_eq!(
            convert(r"A \cap B = \emptyset"),
            "A sect B = emptyset"
        );
    }

    #[test]
    fn formula_derivative_limit() {
        // f'(x) ve lim tanımı
        assert_eq!(
            convert(r"f'(x) = \lim_{h \to 0} \frac{f(x+h)-f(x)}{h}"),
            "f'(x) = lim_(h -> 0) frac(f(x+h)-f(x), h)"
        );
    }

    #[test]
    fn formula_geometry_angle() {
        // Açı gösterimi: 30 derece
        assert_eq!(convert(r"m(\angle ABC) = 60^{\circ}"), "m(angle ABC) = 60 degree");
    }

    #[test]
    fn formula_vector_dot() {
        assert_eq!(
            convert(r"\vec{u} \cdot \vec{v} = |\vec{u}||\vec{v}|\cos\theta"),
            "arrow(u) dot.op arrow(v) = |arrow(u)||arrow(v)|cos theta"
        );
    }

    #[test]
    fn formula_logarithm() {
        assert_eq!(convert(r"\log_{2} 8 = 3"),   "log_2 8 = 3");
        assert_eq!(convert(r"\ln(e^{x}) = x"),   "ln(e^x) = x");
    }

    #[test]
    fn formula_trig_identity() {
        // Not: \theta ardından tokenizer trailing space'i yutar; Typst binary-op
        // spacing'i otomatik yapar, kaynak boşluğu zorunlu değil.
        assert_eq!(
            convert(r"\sin^{2}\theta + \cos^{2}\theta = 1"),
            "sin^2 theta+ cos^2 theta= 1"
        );
    }

    #[test]
    fn formula_binomial() {
        assert_eq!(convert(r"\binom{n}{k}"),           "binom(n, k)");
        assert_eq!(convert(r"\binom{n}{k} = \binom{n}{n-k}"), "binom(n, k) = binom(n, n-k)");
    }

    #[test]
    fn formula_matrix_2x2() {
        assert_eq!(
            convert(r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}"),
            "mat(a, b; c, d)"
        );
    }

    #[test]
    fn formula_abs_value() {
        // |x| geçiyor: düz karakter olarak
        assert_eq!(convert(r"|x|"),          "|x|");
        assert_eq!(convert(r"|x + y|"),      "|x + y|");
        assert_eq!(convert(r"\left|x\right|"), "|x|");
    }

    #[test]
    fn formula_physics_kinematics() {
        // Türk lise fiziği: kinematik denklemler
        assert_eq!(
            convert(r"v = v_{0} + at"),
            "v = v_0 + at"
        );
        // v_{0}t → v_0t  (t harfi direkt subscript sonrasına yapışır;
        // Typst math mode'da bu ayrı semboller olarak render edilir)
        assert_eq!(
            convert(r"x = v_{0}t + \frac{1}{2}at^{2}"),
            "x = v_0t + frac(1, 2)at^2"
        );
    }

    #[test]
    fn formula_chemistry_ce() {
        // mhchem ile kimya formülleri
        let out = convert(r"\ce{H2SO4 + 2NaOH -> Na2SO4 + 2H2O}");
        assert!(out.contains("upright(\"H\")_2 upright(\"S\") upright(\"O\")_4"), "got: {out}");
        assert!(out.contains("arrow.r"), "got: {out}");
        assert!(out.contains("upright(\"H\")_2 upright(\"O\")"), "got: {out}");
    }

    #[test]
    fn formula_probability() {
        // Olasılık
        assert_eq!(convert(r"P(A \cup B) = P(A) + P(B) - P(A \cap B)"),
                   "P(A union B) = P(A) + P(B) - P(A sect B)");
    }

    #[test]
    fn formula_complex_numbers() {
        // Karmaşık sayılar: i ve |z|
        assert_eq!(convert(r"z = a + bi"),            "z = a + bi");
        assert_eq!(convert(r"|z| = \sqrt{a^{2}+b^{2}}"), "|z| = sqrt(a^2+b^2)");
    }

    #[test]
    fn formula_limits_infinity() {
        assert_eq!(
            convert(r"\lim_{x \to \infty} \frac{1}{x} = 0"),
            "lim_(x -> infinity) frac(1, x) = 0"
        );
    }
}
