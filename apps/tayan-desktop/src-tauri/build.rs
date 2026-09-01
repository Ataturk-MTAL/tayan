fn main() {
    // Sidecar ikilisinin adı hedef üçlüsünü taşır (Tauri kuralı).
    // Çalışma zamanında doğru dosyayı bulabilmek için üçlüyü derlemeye gömüyoruz.
    let triple = std::env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=TINYMIST_TARGET_TRIPLE={triple}");

    tauri_build::build()
}
