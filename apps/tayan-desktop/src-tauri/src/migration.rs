//! Tek seferlik veri göçleri.
//!
//! Her göç ETKİSİZ tekrarlanabilir olmak zorundadır: uygulama her açılışta
//! çalıştırır ve ikinci kez çalıştığında hiçbir şey değiştirmemelidir.

use std::path::{Path, PathBuf};

use tayan_core::application::ports::QuestionBankRepository;
use tayan_core::domain::exam_management::value_objects::content_node::ContentNode;
use tayan_db::SqliteQuestionBankRepository;

#[derive(Debug, Default)]
pub struct ImageMigrationReport {
    pub moved_files: usize,
    pub rewritten_refs: usize,
}

/// Görselleri veritabanıyla aynı klasöre taşır ve kaynaklardaki mutlak yolları
/// göreli hale getirir.
///
/// Neden gerekli: save_image bir dönem Tauri'nin app_local_data_dir()'ine
/// yazıyordu (com.tayan.app/images), veritabanı ise data_local_dir()/tayan
/// altındaydı. İki ayrı klasör, tek bir yedek alan kullanıcının görsellerini
/// kaybetmesi demekti. Ayrıca kaynağa MUTLAK yol yazılıyordu; o yol kullanıcı
/// adını içerir ve veri başka bir makineye taşındığında kırılır — sınav
/// görselsiz basılır ve bunu fark etmek zordur.
pub async fn migrate_image_storage(
    bank_repo: &SqliteQuestionBankRepository,
    legacy_dir: Option<PathBuf>,
) -> Result<ImageMigrationReport, String> {
    let mut report = ImageMigrationReport::default();

    let target_dir = tayan_compiler::world::app_data_root().join("images");
    std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    // 1) Dosyaları taşı. Aynı adlı dosya hedefte varsa DOKUNULMAZ: üzerine
    //    yazmak, adları çakışan iki farklı görselden birini yok etmek olur.
    if let Some(legacy) = legacy_dir {
        if legacy.exists() && legacy != target_dir {
            report.moved_files = move_images(&legacy, &target_dir)?;
        }
    }

    // 2) Kaynaklardaki mutlak yolları göreli yap.
    let mut bank = bank_repo.load().await.map_err(|e| e.to_string())?;
    let mut changed = 0usize;

    for bq in bank.questions.iter_mut() {
        for node in bq.question.body_mut().0.iter_mut() {
            match node {
                ContentNode::Image(n) => {
                    if let Some(rel) = relative_image_path(&n.src) {
                        n.src = rel;
                        changed += 1;
                    }
                }
                ContentNode::TypstRaw(n) => {
                    let (code, hits) = rewrite_image_calls(&n.code);
                    if hits > 0 {
                        n.code = code;
                        changed += hits;
                    }
                }
                _ => {}
            }
        }
    }

    if changed > 0 {
        bank_repo.save(&bank).await.map_err(|e| e.to_string())?;
    }
    report.rewritten_refs = changed;

    Ok(report)
}

fn move_images(from: &Path, to: &Path) -> Result<usize, String> {
    let entries = std::fs::read_dir(from).map_err(|e| e.to_string())?;
    let mut moved = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name() else { continue };
        let dest = to.join(name);

        if dest.exists() {
            continue; // çakışma: hedefteki dosya korunur
        }

        // Kopyala + sil, rename değil: iki klasör farklı birimlerde olabilir.
        std::fs::copy(&path, &dest).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&path);
        moved += 1;
    }

    Ok(moved)
}

/// Yüzde kodlamasını çözer.
///
/// Eski kayıtlar Tauri'nin asset protokolünü kullanıyor ve yol TAMAMEN
/// kodlanmış geliyor:
///   asset://localhost/%2FUsers%2F...%2Fimages%2Ffoo.png
/// Çözülmeden aranırsa "/images/" hiçbir zaman eşleşmez ve göç sessizce
/// hiçbir şey yapmaz.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(byte) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }

    String::from_utf8_lossy(&out).into_owned()
}

/// Mutlak bir görsel yolunu "images/<dosya>" biçimine çevirir.
/// Zaten göreliyse veya images/ içermiyorsa None döner.
fn relative_image_path(src: &str) -> Option<String> {
    if !src.starts_with('/') && !src.starts_with("asset:") {
        return None;
    }

    let decoded = percent_decode(src);
    let idx = decoded.rfind("/images/")?;
    let name = &decoded[idx + "/images/".len()..];

    if name.is_empty() || name.contains('/') {
        return None;
    }
    Some(format!("images/{name}"))
}

/// Typst kaynağındaki #image("...") çağrılarının yolunu göreli yapar.
fn rewrite_image_calls(code: &str) -> (String, usize) {
    const MARKER: &str = "#image(\"";

    let mut out = String::with_capacity(code.len());
    let mut rest = code;
    let mut hits = 0;

    while let Some(at) = rest.find(MARKER) {
        let head = at + MARKER.len();
        out.push_str(&rest[..head]);
        rest = &rest[head..];

        let Some(end) = rest.find('"') else { break };
        let path = &rest[..end];

        match relative_image_path(path) {
            Some(rel) => {
                out.push_str(&rel);
                hits += 1;
            }
            None => out.push_str(path),
        }
        rest = &rest[end..];
    }

    out.push_str(rest);
    (out, hits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutlak_yol_göreli_olur() {
        assert_eq!(
            relative_image_path("/Users/biri/Library/Application Support/com.tayan.app/images/a.png"),
            Some("images/a.png".to_string())
        );
    }

    #[test]
    fn asset_url_göreli_olur() {
        // Eski kayıtların GERÇEK biçimi: tamamen yüzde kodlanmış asset URL'i.
        let src = "asset://localhost/%2FUsers%2Fhakan%2FLibrary%2FApplication%20Support%2Fcom.tayan.app%2Fimages%2Fimg_20260512_211000_a24a65e8.png";
        assert_eq!(
            relative_image_path(src),
            Some("images/img_20260512_211000_a24a65e8.png".to_string())
        );
    }

    #[test]
    fn zaten_göreli_yol_değişmez() {
        assert_eq!(relative_image_path("images/a.png"), None);
    }

    #[test]
    fn images_içermeyen_mutlak_yol_değişmez() {
        assert_eq!(relative_image_path("/tmp/a.png"), None);
    }

    #[test]
    fn typst_çağrısı_yeniden_yazılır() {
        let (out, hits) = rewrite_image_calls(
            "Metin #image(\"/Users/x/images/a.png\", width: 60%) devam #image(\"images/b.png\")",
        );
        assert_eq!(hits, 1, "{out}");
        assert!(out.contains("#image(\"images/a.png\", width: 60%)"), "{out}");
        assert!(out.contains("#image(\"images/b.png\")"), "{out}");
    }

    #[test]
    fn görsel_yoksa_metin_bozulmaz() {
        let code = "Sadece metin, #v(1cm) ve $x^2$";
        let (out, hits) = rewrite_image_calls(code);
        assert_eq!(hits, 0);
        assert_eq!(out, code);
    }
}
