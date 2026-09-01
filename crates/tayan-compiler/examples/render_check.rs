fn main() {
    let p = "/private/tmp/claude-501/-Users-hakan-VSCodeProjects/c1f16c60-0f9e-4ced-bde6-cc57e12e5ea2/scratchpad/t.svg";
    let body = format!("#set text(size: 14pt)\n\nSVG görsel: #image(\"{p}\", width: 4cm)\n\nDaireni çiz.\n");
    let src = tayan_compiler::typst_gen::TypstGenerator::preview_document(&body);
    match tayan_compiler::TayanWorld::compile_svg(src) {
        Ok(pg) => { std::fs::write("/private/tmp/claude-501/-Users-hakan-VSCodeProjects/c1f16c60-0f9e-4ced-bde6-cc57e12e5ea2/scratchpad/check.svg", &pg[0]).unwrap(); println!("OK"); }
        Err(e) => println!("HATA: {e}"),
    }
}
