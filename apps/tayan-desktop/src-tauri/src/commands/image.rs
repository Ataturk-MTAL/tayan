use base64::Engine;
use chrono::Local;
use uuid::Uuid;

/// Base64 kodlu bir görseli uygulama veri klasörüne yazar.
///
/// Dönen değer GÖRELİ yoldur: "images/soru_20260512_143022_a3f7b2c1.png".
/// Kaynağa mutlak yol yazmak kullanıcı adını gömer ve veri başka bir makineye
/// taşındığında kırılır — sınav görselsiz basılır ve bunu fark etmek zordur.
///
/// Klasör veritabanıyla AYNI yerdedir (dirs_next::data_local_dir()/tayan), yani
/// tek klasörü kopyalamak tam yedek demektir. Daha önce Tauri'nin
/// app_local_data_dir()'i kullanılıyordu ve görseller veritabanından ayrı bir
/// klasöre (com.tayan.app/images) düşüyordu; o klasörü kopyalamayan bir yedek
/// görselleri kaybediyordu.
/// Filename format: `{context}_{YYYYMMDD}_{HHMMSS}_{8hex}.{ext}`
/// Example: `soru_20260512_143022_a3f7b2c1.png`
#[tauri::command]
pub fn save_image(
    data: String,
    ext: String,
    context: String,
) -> Result<String, String> {
    // Sanitize context — only alphanumeric + underscore, max 32 chars
    let safe_context: String = context
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .take(32)
        .collect();
    let safe_context = if safe_context.is_empty() {
        "img".to_string()
    } else {
        safe_context
    };

    // Whitelist extension to prevent path traversal via extension
    let safe_ext = match ext.to_lowercase().trim_start_matches('.') {
        "jpg" | "jpeg" => "jpg",
        "png" => "png",
        "gif" => "gif",
        "webp" => "webp",
        // SVG vektörel kalır: baskıda her ölçekte keskin çıkar ve GeoGebra gibi
        // araçların doğal çıktı biçimidir. Typst onu resvg ile çizer, script
        // çalıştırmaz.
        "svg" => "svg",
        _ => "png",
    };

    // Timestamp
    let ts = Local::now().format("%Y%m%d_%H%M%S");

    // Short UUID (first 8 hex chars of UUID v4)
    let uid = Uuid::new_v4().to_string().replace('-', "");
    let short_id = &uid[..8];

    let filename = format!("{}_{}_{}.{}", safe_context, ts, short_id, safe_ext);

    let img_dir = tayan_compiler::world::app_data_root().join("images");
    std::fs::create_dir_all(&img_dir).map_err(|e| e.to_string())?;

    let path = img_dir.join(&filename);

    // Strip "data:image/...;base64," prefix if present
    let raw_b64 = if let Some(idx) = data.find(',') {
        &data[idx + 1..]
    } else {
        data.as_str()
    };

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(raw_b64.trim())
        .map_err(|e| format!("Base64 decode error: {e}"))?;

    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;

    // Göreli yol: Typst World bunu app_data_root() altından çözer.
    Ok(format!("images/{filename}"))
}
