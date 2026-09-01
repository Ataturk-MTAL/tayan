fn main() {
    let body = "#set text(size: 16pt)\n\nSatır içi: Denklemin kökü $x = 2$ olarak bulunur.\n\nBoşluklu: Denklemin kökü $ x = 2 $ olarak bulunur.\n\nKesir satır içi: oran $a/b$ kadar.\n\nKesir boşluklu: oran $ a/b $ kadar.\n";
    let src = tayan_compiler::typst_gen::TypstGenerator::preview_document(body);
    let pages = tayan_compiler::TayanWorld::compile_svg(src).expect("derlenmedi");
    std::fs::write("/private/tmp/claude-501/-Users-hakan-VSCodeProjects/c1f16c60-0f9e-4ced-bde6-cc57e12e5ea2/scratchpad/check.svg", &pages[0]).unwrap();
    println!("ok");
}
