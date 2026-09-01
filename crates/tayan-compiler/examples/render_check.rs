fn main() {
    let body = "#set text(size: 20pt)\n\nBoş #cb() · İşaretli #cb(checked: true) · Tik #tik()\n\n#dogru-yanlis(dogru: true)\n\n#secenekler(dogru: \"B\", anahtar: true,\n  [Birinci],\n  [İkinci],\n  [Üçüncü],\n)\n";
    let source = tayan_compiler::typst_gen::TypstGenerator::preview_document(body);
    let pages = tayan_compiler::TayanWorld::compile_svg(source).expect("derlenmedi");
    std::fs::write("/private/tmp/claude-501/-Users-hakan-VSCodeProjects/c1f16c60-0f9e-4ced-bde6-cc57e12e5ea2/scratchpad/check.svg", &pages[0]).unwrap();
    println!("yazıldı: {} bayt", pages[0].len());
}
