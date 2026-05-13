use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitStr};

// ── ToTypst derive ────────────────────────────────────────────────────────────

/// Automatically implements `ToTypst` for a struct by delegating to each
/// field's own `ToTypst` implementation. Fields tagged `#[typst(skip)]`
/// are excluded from the output.
#[proc_macro_derive(ToTypst, attributes(typst))]
pub fn derive_to_typst(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let expanded = quote! {
        impl crate::domain::shared::to_typst::ToTypst for #name {
            fn to_typst(&self, ctx: &crate::domain::shared::to_typst::TypstContext) -> String {
                self.emit_typst(ctx)
            }
        }
    };

    expanded.into()
}

// ── math! compile-time LaTeX brace validator ─────────────────────────────────

/// Validates LaTeX brace balance at compile time.
///
/// ```rust
/// let node = math!(r"\frac{a}{b}");        // ✅ compiles
/// let node = math!(r"\frac{a}{b");         // ❌ compile error: 1 unclosed brace
/// ```
#[proc_macro]
pub fn math(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let value = lit.value();

    if let Err(msg) = validate_latex_braces(&value) {
        return syn::Error::new(lit.span(), format!("Invalid LaTeX: {msg}"))
            .to_compile_error()
            .into();
    }

    if let Some(cmd) = find_dollar_sign(&value) {
        return syn::Error::new(
            lit.span(),
            format!("LaTeX string must not contain `$` delimiters (found near `{cmd}`). Use raw LaTeX content only."),
        )
        .to_compile_error()
        .into();
    }

    quote! {
        tayan_core::domain::exam_management::value_objects::content_node::MathNode {
            raw:     #value.into(),
            display: tayan_core::domain::exam_management::value_objects::content_node::MathDisplay::Inline,
        }
    }
    .into()
}

/// Block-display variant of `math!`.
#[proc_macro]
pub fn math_block(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let value = lit.value();

    if let Err(msg) = validate_latex_braces(&value) {
        return syn::Error::new(lit.span(), format!("Invalid LaTeX: {msg}"))
            .to_compile_error()
            .into();
    }

    quote! {
        tayan_core::domain::exam_management::value_objects::content_node::MathNode {
            raw:     #value.into(),
            display: tayan_core::domain::exam_management::value_objects::content_node::MathDisplay::Block,
        }
    }
    .into()
}

// ── typst_include! — embed a .typ file at compile time ───────────────────────

/// Embeds a Typst template file as a `&'static str` and validates it exists.
///
/// ```rust
/// static EXAM_TEMPLATE: &str = typst_include!("templates/exam.typ");
/// ```
#[proc_macro]
pub fn typst_include(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let path = lit.value();

    // Resolve relative to CARGO_MANIFEST_DIR at compile time via include_str!
    let span = lit.span();
    let expanded = quote::quote_spanned! { span =>
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #path))
    };

    expanded.into()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn validate_latex_braces(s: &str) -> Result<(), String> {
    let mut depth = 0i32;
    let mut col = 0usize;
    for ch in s.chars() {
        col += 1;
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth < 0 {
                    return Err(format!("Eşleşmeyen '}}' karakteri, konum: {col}"));
                }
            }
            _ => {}
        }
    }
    if depth > 0 {
        return Err(format!("{depth} adet kapatılmamış '{{{{' parantez"));
    }
    Ok(())
}

fn find_dollar_sign(s: &str) -> Option<&str> {
    if s.contains('$') { Some("$") } else { None }
}
