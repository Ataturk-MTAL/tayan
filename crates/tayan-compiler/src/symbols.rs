//! Typst standart kütüphanesinin sembol dökümü.
//!
//! Elle yazılmış bir liste yerine kütüphanenin KENDİSİ taranır. Elle liste iki
//! yönden bozulur: Typst sürümü değişince eskir, ve baştan eksiktir — kimse
//! yüzlerce sembolü tek tek yazmaz. Buradaki döküm her zaman gerçekten
//! derlenebilecek olanı listeler.

use serde::Serialize;
use typst::foundations::{Module, Value};

use crate::world::standard_library_ref;

#[derive(Debug, Clone, Serialize)]
pub struct TypstSymbol {
    /// Sembolün adı: "image", "align", "alpha", "integral" …
    pub name: String,
    /// "function" | "symbol" | "type" | "module" | "value"
    pub kind: String,
    /// Yalnızca matematik kipinde geçerli mi ($ … $ içinde).
    pub math: bool,
    /// İşlevin parametre adları, Typst'in bildirdiği sırada.
    pub params: Vec<String>,
    /// Tek satırlık açıklama, Typst'in kendi belgelerinden.
    pub summary: String,
}

/// Global ve matematik kapsamlarındaki her şeyi döker.
pub fn all_symbols() -> Vec<TypstSymbol> {
    let library = standard_library_ref();
    let mut out = Vec::new();

    collect(&library.global, false, &mut out);
    collect(&library.math, true, &mut out);

    // Şablonun kendi yardımcıları (#secenekler, #cb, #tik …). Bunlar
    // preview_document tarafından her belgeye ekleniyor, yani gerçekten
    // çağrılabilir durumdalar; listede olmamaları eksiklik olurdu.
    out.extend(preamble_symbols());

    out.sort_by(|a, b| a.name.cmp(&b.name).then(a.math.cmp(&b.math)));
    out.dedup_by(|a, b| a.name == b.name && a.math == b.math);
    out
}

fn collect(module: &Module, math: bool, out: &mut Vec<TypstSymbol>) {
    for (name, binding) in module.scope().iter() {
        let value = binding.read();

        let (kind, params, summary) = match value {
            Value::Func(f) => {
                let params = f
                    .params()
                    .map(|ps| ps.iter().map(|p| p.name.to_string()).collect())
                    .unwrap_or_default();
                ("function", params, first_line(f.docs().unwrap_or("")))
            }
            Value::Symbol(_) => ("symbol", Vec::new(), String::new()),
            Value::Type(t) => ("type", Vec::new(), first_line(t.docs())),
            Value::Module(_) => ("module", Vec::new(), String::new()),
            _ => ("value", Vec::new(), String::new()),
        };

        out.push(TypstSymbol {
            name: name.to_string(),
            kind: kind.to_string(),
            math,
            params,
            summary,
        });
    }
}

/// Önsözdeki `#let ad(param: …)` tanımlarını ayrıştırır.
///
/// Elle listelemek yerine ayrıştırmak bilinçli: önsöze yeni bir yardımcı
/// eklendiğinde tamamlama listesi kendiliğinden öğrenir, kimsenin iki yeri
/// güncel tutması gerekmez.
fn preamble_symbols() -> Vec<TypstSymbol> {
    const MARKER: &str = "#let ";
    let source = crate::typst_gen::TypstGenerator::preamble();

    let mut out = Vec::new();
    let mut rest = source;

    while let Some(at) = rest.find(MARKER) {
        rest = &rest[at + MARKER.len()..];

        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }

        let after = &rest[name.len()..];
        let params = if after.starts_with('(') {
            parse_param_names(after)
        } else {
            Vec::new()
        };

        out.push(TypstSymbol {
            name,
            kind: "tayan".to_string(),
            math: false,
            params,
            summary: "TAYAN şablon yardımcısı".to_string(),
        });
    }

    out
}

/// `(a: 1, b, ..c)` biçimindeki listeden adları çıkarır. Parantez derinliği
/// sayılır, çünkü varsayılan değerlerin içinde de parantez olabilir.
fn parse_param_names(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut depth = 0usize;
    let mut current = String::new();

    for ch in text.chars() {
        match ch {
            '(' | '[' | '{' => {
                depth += 1;
                if depth == 1 {
                    continue;
                }
            }
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    push_param(&current, &mut names);
                    break;
                }
            }
            ',' if depth == 1 => {
                push_param(&current, &mut names);
                current.clear();
                continue;
            }
            _ => {}
        }
        if depth >= 1 {
            current.push(ch);
        }
    }

    names
}

fn push_param(raw: &str, out: &mut Vec<String>) {
    let name: String = raw
        .trim()
        .trim_start_matches("..")
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if !name.is_empty() {
        out.push(name);
    }
}

/// Typst belgeleri Markdown; tamamlama listesine yalnızca ilk cümle sığar.
fn first_line(docs: &str) -> String {
    docs.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .chars()
        .take(120)
        .collect()
}
