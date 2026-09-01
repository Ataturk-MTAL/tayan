//! Bellek ölçümü: art arda derleme sürecin belleğini şişiriyor mu?
//!
//! Uygulamayı açmadan çalıştırılır. Kullanım:
//!   /usr/bin/time -l cargo run -p tayan-compiler --example mem_probe -- <tur>
//! "maximum resident set size" satırı okunur.

use std::time::Instant;
use tayan_compiler::TayanWorld;

const DOC: &str = r#"#set page(paper: "a4", margin: (x: 2cm, y: 2.5cm))
#set text(lang: "tr", size: 11pt, font: "Libertinus Serif")

= Deneme

Aşağıdaki denklemin köklerini bulunuz.

$ x^2 - 5x + 6 = 0 $

Şöyle bir metin: ğüşıöç İĞÜŞÖÇ
"#;

fn main() {
    let rounds: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(2);

    let t_warm = Instant::now();
    tayan_compiler::world::warm_font_registry();
    println!("font kaydı kurulumu: {:?}", t_warm.elapsed());

    for i in 1..=rounds {
        // Her turda kaynağı değiştir; comemo önbelleğini gerçekten zorlar.
        let source = format!("{DOC}\n\nTur {i}\n");
        let t0 = Instant::now();
        match TayanWorld::compile_svg(source) {
            Ok(pages) => println!("tur {i}: {} sayfa, {:?}", pages.len(), t0.elapsed()),
            Err(e) => {
                eprintln!("tur {i} HATA: {e}");
                std::process::exit(1);
            }
        }
    }

    let families = tayan_compiler::world::available_font_families();
    println!("{rounds} tur bitti");
    println!("kayıtlı aile: {}", families.len());
    println!("ilk 8: {:?}", &families[..families.len().min(8)]);
}
