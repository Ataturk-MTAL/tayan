fn main() {
    let body = r#"#set text(size: 14pt)

Küçük: $alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu$

$nu xi omicron pi rho sigma tau upsilon phi chi psi omega$

Büyük: $Gamma Delta Theta Lambda Xi Pi Sigma Upsilon Phi Psi Omega$

Değişkeler: $epsilon.alt phi.alt theta.alt$

Kullanımda: $Delta x$, $sum_(i=1)^n$, $theta = pi/4$, $sigma^2$, $mu +- 2 sigma$
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
