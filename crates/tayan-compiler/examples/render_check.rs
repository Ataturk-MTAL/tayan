fn main() {
    // dogru: "B" = YAZILDIĞI sıradaki ikinci şık ("İkinci").
    // sira: (2, 0, 1) o şıkkı yeni sırada üçüncüye, yani C harfine taşır.
    // Cevap anahtarı işareti C'nin yanında ve "İkinci" metninin karşısında
    // olmalı. Değilse anahtar kâğıttan sapmış demektir.
    let body = "#set text(size: 18pt)\n\n*Karıştırılmamış:*\n\n#secenekler(dogru: \"B\", anahtar: true,\n  [Birinci], [İkinci], [Üçüncü],\n)\n\n*Karıştırılmış sira: (2, 0, 1):*\n\n#secenekler(dogru: \"B\", sira: (2, 0, 1), anahtar: true,\n  [Birinci], [İkinci], [Üçüncü],\n)\n";
    let source = tayan_compiler::typst_gen::TypstGenerator::preview_document(body);
    let pages = tayan_compiler::TayanWorld::compile_svg(source).expect("derlenmedi");
    std::fs::write("/private/tmp/claude-501/-Users-hakan-VSCodeProjects/c1f16c60-0f9e-4ced-bde6-cc57e12e5ea2/scratchpad/check.svg", &pages[0]).unwrap();
    println!("yazildi");
}
