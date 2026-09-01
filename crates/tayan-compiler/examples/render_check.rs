fn main() {
    let p = "/private/tmp/claude-501/-Users-hakan-VSCodeProjects/c1f16c60-0f9e-4ced-bde6-cc57e12e5ea2/scratchpad/t.svg";
    let body = format!(r#"#set text(size: 12pt)

*1. Varsayılan (blok, sola dayalı):*

#image("{p}", width: 3cm)

*2. Ortalı:*

#align(center)[#image("{p}", width: 3cm)]

*3. Sağa dayalı:*

#align(right)[#image("{p}", width: 3cm)]

*4. Şekil numarası ve başlıkla:*

#figure(image("{p}", width: 3cm), caption: [Bir çember])

*5. Metnin yanında:*

#grid(columns: (1fr, auto), gutter: 0.5cm,
  [Yandaki çemberin yarıçapı $r$ olduğuna göre alanını bulunuz.],
  image("{p}", width: 2.5cm),
)
"#);
    let src = tayan_compiler::typst_gen::TypstGenerator::preview_document(&body);
    match tayan_compiler::TayanWorld::compile_svg(src) {
        Ok(pg) => { std::fs::write("/private/tmp/claude-501/-Users-hakan-VSCodeProjects/c1f16c60-0f9e-4ced-bde6-cc57e12e5ea2/scratchpad/check.svg", &pg[0]).unwrap(); println!("OK"); }
        Err(e) => println!("HATA: {e}"),
    }
}
