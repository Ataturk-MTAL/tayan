//! tinymist dil sunucusunun isteğe bağlı kurulumu.
//!
//! İkili uygulamayla PAKETLENMİYOR. Gerekçe: platform başına 60 MB ve dondurulmuş
//! bir sürüm. tinymist zorunlu değil — o olmadan editör kendi sembol dökümüyle
//! (560 sembol) tam çalışır.
//!
//! Bu yüzden indirme AÇIK bir kullanıcı eylemidir, ilk açılışta sessizce
//! yapılmaz: "tamamen çevrimdışı" bir üründe kullanıcı sormadan ağa çıkmak
//! kabul edilemez.

use std::path::PathBuf;

use serde::Serialize;

const VERSION: &str = "v0.15.2";

#[derive(Debug, Serialize)]
pub struct LspStatus {
    pub installed: bool,
    pub path: Option<String>,
    pub version: String,
    /// Bu platform için indirilebilir bir yapı var mı.
    pub supported: bool,
}

/// Kurulum hedefi: uygulama veri klasörü. Veritabanı ve görsellerle aynı yer,
/// yani tek klasör kopyalamak yine tam yedek demek.
pub fn install_dir() -> PathBuf {
    tayan_compiler::world::app_data_root().join("lsp-server")
}

pub fn installed_binary() -> Option<PathBuf> {
    let p = install_dir().join("tinymist");
    p.is_file().then_some(p)
}

/// (varlık adı, indirme sonrası çalıştırılabilir dosya adı)
fn asset_for_platform() -> Option<&'static str> {
    Some(match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "tinymist-aarch64-apple-darwin.tar.gz",
        ("macos", "x86_64") => "tinymist-x86_64-apple-darwin.tar.gz",
        ("linux", "x86_64") => "tinymist-x86_64-unknown-linux-gnu.tar.gz",
        ("linux", "aarch64") => "tinymist-aarch64-unknown-linux-gnu.tar.gz",
        ("windows", "x86_64") => "tinymist-x86_64-pc-windows-msvc.zip",
        _ => return None,
    })
}

pub fn status() -> LspStatus {
    let path = installed_binary();
    LspStatus {
        installed: path.is_some(),
        path: path.map(|p| p.display().to_string()),
        version: VERSION.to_string(),
        supported: asset_for_platform().is_some(),
    }
}

/// İndirir, sha256 DOĞRULAR, açar ve kurar.
///
/// Doğrulama atlanamaz: dışarıdan gelip kullanıcının makinesinde çalışacak bir
/// ikili. Sağlama tutmazsa dosya kurulmaz.
pub fn install() -> Result<String, String> {
    let asset = asset_for_platform()
        .ok_or_else(|| format!(
            "Bu platform için hazır yapı yok: {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        ))?;

    let base = format!(
        "https://github.com/Myriad-Dreamin/tinymist/releases/download/{VERSION}"
    );

    let archive = http_get(&format!("{base}/{asset}"))?;
    let sums = http_get(&format!("{base}/{asset}.sha256"))?;

    let expected = String::from_utf8_lossy(&sums)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase();
    let actual = sha256_hex(&archive);

    if expected.is_empty() || expected != actual {
        return Err(format!(
            "sha256 uyuşmadı, kurulum iptal edildi.\nbeklenen: {expected}\ngerçek  : {actual}"
        ));
    }

    let dir = install_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let target = dir.join("tinymist");
    extract_tinymist(&archive, asset, &target)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&target).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&target, perms).map_err(|e| e.to_string())?;
    }

    Ok(target.display().to_string())
}

pub fn uninstall() -> Result<(), String> {
    let dir = install_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn http_get(url: &str) -> Result<Vec<u8>, String> {
    let resp = ureq::get(url)
        .call()
        .map_err(|e| format!("İndirilemedi: {e}"))?;

    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut resp.into_reader(), &mut buf)
        .map_err(|e| format!("Okunamadı: {e}"))?;
    Ok(buf)
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn extract_tinymist(archive: &[u8], asset: &str, target: &std::path::Path) -> Result<(), String> {
    if asset.ends_with(".zip") {
        return extract_from_zip(archive, target);
    }

    let decoder = flate2::read::GzDecoder::new(archive);
    let mut tar = tar::Archive::new(decoder);

    for entry in tar.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();

        if path.file_name().and_then(|n| n.to_str()) == Some("tinymist") {
            let mut out = std::fs::File::create(target).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    Err("Arşivde tinymist bulunamadı".to_string())
}

fn extract_from_zip(_archive: &[u8], _target: &std::path::Path) -> Result<(), String> {
    // Windows yapısı zip; zip okuyucusu henüz bağlanmadı.
    Err("Windows kurulumu henüz desteklenmiyor".to_string())
}

#[cfg(test)]
mod install_tests {
    use super::*;

    /// Gerçekten indirip kuruyor mu, VE sha256 doğrulaması işliyor mu?
    #[test]
    #[ignore = "ağ gerektirir: cargo test -- --ignored"]
    fn indirir_dogrular_kurar() {
        let _ = uninstall();
        assert!(installed_binary().is_none(), "önce temiz olmalı");

        let path = install().expect("kurulum başarısız");
        println!("kuruldu: {path}");

        let bin = installed_binary().expect("kurulduktan sonra bulunmalı");
        let out = std::process::Command::new(&bin)
            .arg("--version")
            .output()
            .expect("çalıştırılamadı");
        let text = String::from_utf8_lossy(&out.stdout);
        println!("{}", text.lines().take(3).collect::<Vec<_>>().join(" | "));
        assert!(text.contains("tinymist"), "sürüm çıktısı beklenmedik: {text}");
    }
}
