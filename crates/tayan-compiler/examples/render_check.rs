fn main() {
    let p = "/private/tmp/claude-501/-Users-hakan-VSCodeProjects/c1f16c60-0f9e-4ced-bde6-cc57e12e5ea2/scratchpad/t.svg";
    let body = format!(r#"#set text(size: 13pt)

*Çizilmiş işaretler:* Boş #cb() · İşaretli #cb(checked: true) · Tik #tik()

*Doğru/yanlış:* #dogru-yanlis(dogru: true)

*Karıştırılmış + anahtar* (dogru: "B", sira: (2, 0, 1)):

#secenekler(dogru: "B", sira: (2, 0, 1), anahtar: true,
  [Birinci], [İkinci], [Üçüncü],
)

*Matematik:* satır içi $x^2 + 1$ · türev $(dif y)/(dif x)$ · kısmi $(partial f)/(partial x)$

*Yunan:* $alpha beta gamma Delta Sigma Omega$ · artı-eksi $plus.minus$

*Boşluk:* İç açılar toplamı #bosluk(cevap: "180", width: 2cm) derecedir.

*SVG:* #align(center)[#image("{p}", width: 3cm)]
"#);
    let src = tayan_compiler::typst_gen::TypstGenerator::preview_document(&body);
    match tayan_compiler::TayanWorld::compile_svg(src) {
        Ok(pg) => { std::fs::write("/private/tmp/claude-501/-Users-hakan-VSCodeProjects/c1f16c60-0f9e-4ced-bde6-cc57e12e5ea2/scratchpad/check.svg", &pg[0]).unwrap(); println!("OK {} sayfa", pg.len()); }
        Err(e) => println!("HATA: {e}"),
    }
}
