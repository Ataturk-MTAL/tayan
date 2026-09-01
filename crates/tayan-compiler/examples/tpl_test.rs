fn main() {
    let cases: Vec<(&str, &str)> = vec![
        ("karıştırılmış + anahtar", "Kökleri bulunuz.\n\n#secenekler(dogru: \"C\", sira: (3, 0, 2, 4, 1), anahtar: true,\n  [$x = 1$],\n  [$x = 2$],\n  [$x = 2$ ve $x = 3$],\n  [$x = 6$],\n  [Hiçbiri],\n)\n"),
        ("çoktan seçmeli", "Kökleri bulunuz.\n\n$ x^2 - 5x + 6 = 0 $\n\n#secenekler(dogru: \"C\",\n  [$x = 1$],\n  [$x = 2$],\n  [$x = 2$ ve $x = 3$],\n  [$x = 6$],\n  [Hiçbiri],\n)\n"),
        ("çizilmiş işaretler", "Boş: #cb() İşaretli: #cb(checked: true) Tik: #tik()\n"),
        ("doğru yanlış", "Her kare bir dikdörtgendir.\n\n#dogru-yanlis(dogru: true)\n"),
        ("klasik", "İkinci dereceden denklemin çözümünü açıklayınız.\n\n#cevap-alani(satir: 5)\n"),
        ("boşluk", "Bir üçgenin iç açıları toplamı #bosluk(cevap: \"180|180 derece\", width: 2cm) derecedir.\n"),
    ];

    let mut failed = 0;
    for (ad, body) in cases {
        let source = tayan_compiler::typst_gen::TypstGenerator::preview_document(body);
        match tayan_compiler::TayanWorld::compile_svg(source) {
            Ok(pages) => println!("OK   {ad}: {} sayfa", pages.len()),
            Err(e) => { println!("HATA {ad}:\n{e}"); failed += 1; }
        }
    }
    std::process::exit(if failed > 0 { 1 } else { 0 });
}
