fn main() {
    let body = r#"#set text(size: 15pt)

Türev: $(dif y)/(dif x)$ · İkinci: $(dif^2 y)/(dif x^2)$ · Üs notasyonu: $f'(x)$, $f''(x)$

Kısmi: $(diff f)/(diff x)$ · İkinci kısmi: $(diff^2 f)/(diff x diff y)$

Blok:

$ (dif y)/(dif x) = 2x + 3 $

$ (diff^2 u)/(diff t^2) = c^2 (diff^2 u)/(diff x^2) $

İntegral: $integral f(x) dif x$ · Belirli: $integral_0^1 x^2 dif x$

Limit: $lim_(x -> 0) (sin x)/x = 1$

Nabla: $nabla f$ · Diverjans: $nabla dot bold(F)$ · Toplam diferansiyel: $dif f = (diff f)/(diff x) dif x + (diff f)/(diff y) dif y$
"#;
    let src = tayan_compiler::typst_gen::TypstGenerator::preview_document(body);
    match tayan_compiler::TayanWorld::compile_svg(src) {
        Ok(p) => {
            std::fs::write("/private/tmp/claude-501/-Users-hakan-VSCodeProjects/c1f16c60-0f9e-4ced-bde6-cc57e12e5ea2/scratchpad/check.svg", &p[0]).unwrap();
            println!("OK");
        }
        Err(e) => println!("HATA: {e}"),
    }
}
