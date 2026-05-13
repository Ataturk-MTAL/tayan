use base64::Engine;
use chrono::Local;
use tauri::Manager;
use uuid::Uuid;

/// Saves a base64-encoded image to the app-local data directory.
///
/// Returns the absolute path to the saved file.
/// Filename format: `{context}_{YYYYMMDD}_{HHMMSS}_{8hex}.{ext}`
/// Example: `soru_20260512_143022_a3f7b2c1.png`
#[tauri::command]
pub fn save_image(
    app: tauri::AppHandle,
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
        _ => "png",
    };

    // Timestamp
    let ts = Local::now().format("%Y%m%d_%H%M%S");

    // Short UUID (first 8 hex chars of UUID v4)
    let uid = Uuid::new_v4().to_string().replace('-', "");
    let short_id = &uid[..8];

    let filename = format!("{}_{}_{}.{}", safe_context, ts, short_id, safe_ext);

    // Target directory — use Tauri's app_local_data_dir() so $APPLOCALDATA scope matches
    let img_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("images");
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

    Ok(path.to_string_lossy().to_string())
}
