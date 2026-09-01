fn main() {
    let all = tayan_compiler::symbols::all_symbols();
    println!("toplam: {}", all.len());
    let f = all.iter().filter(|s| s.kind == "function").count();
    let sym = all.iter().filter(|s| s.kind == "symbol").count();
    let math = all.iter().filter(|s| s.math).count();
    println!("işlev: {f}  sembol: {sym}  matematik kipi: {math}");

    for name in ["image", "align", "figure", "table", "alpha", "integral", "sum", "sqrt"] {
        if let Some(s) = all.iter().find(|s| s.name == name) {
            println!("\n{} [{}{}]", s.name, s.kind, if s.math { ", math" } else { "" });
            if !s.params.is_empty() {
                println!("  parametreler: {}", s.params.join(", "));
            }
            if !s.summary.is_empty() {
                println!("  {}", s.summary);
            }
        } else {
            println!("\n{name}: BULUNAMADI");
        }
    }
}
