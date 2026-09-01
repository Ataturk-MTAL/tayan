use tauri::State;
use tokio::sync::Mutex;

use tayan_core::{
    application::ports::{ExamRepository, ExamResultRepository, QuestionBankRepository},
    domain::{
        assessment::aggregates::{ExamResult, QuestionAnswer},
        exam_management::aggregates::ExamId,
        student_management::aggregates::StudentId,
    },
};
use crate::state::AppState;

#[tauri::command]
pub async fn enter_exam_results(
    state:      State<'_, Mutex<AppState>>,
    exam_id:    String,
    student_id: String,
    answers:    Vec<QuestionAnswer>,
    total_max:  f32,
) -> Result<(), String> {
    let st  = state.lock().await;
    let eid = uuid::Uuid::parse_str(&exam_id).map(ExamId).map_err(|e| e.to_string())?;
    let sid = uuid::Uuid::parse_str(&student_id).map(StudentId).map_err(|e| e.to_string())?;

    let bank = st.bank.load().await.map_err(|e| e.to_string())?;
    let mut result = ExamResult::new(eid, sid, total_max);

    tayan_core::domain::assessment::services::ScoringService::auto_score(
        &mut result, answers, &bank,
    );

    result.is_complete = true;
    st.results.save(&result).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_exam_results(
    state:   State<'_, Mutex<AppState>>,
    exam_id: String,
) -> Result<Vec<ExamResult>, String> {
    let st  = state.lock().await;
    let eid = uuid::Uuid::parse_str(&exam_id).map(ExamId).map_err(|e| e.to_string())?;
    st.results.list_by_exam(&eid).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_exam_pdf(
    state:      State<'_, Mutex<AppState>>,
    exam_id:    String,
    answer_key: bool,
    booklet:    Option<String>,
) -> Result<String, String> {
    let st  = state.lock().await;
    let eid = uuid::Uuid::parse_str(&exam_id).map(ExamId).map_err(|e| e.to_string())?;

    let exam = st.exams.find_by_id(&eid).await.map_err(|e| e.to_string())?;
    let bank = st.bank.load().await.map_err(|e| e.to_string())?;

    let ctx = tayan_core::domain::shared::to_typst::TypstContext {
        answer_key,
        booklet,
        ..Default::default()
    };

    let questions: Vec<_> = exam.questions.iter()
        .filter_map(|r| bank.find(&r.question_id).map(|bq| bq.question.clone()))
        .collect();

    tayan_compiler::typst_gen::TypstGenerator::generate_exam(&exam, &questions, ctx)
        .map_err(|e| e.to_string())
}

/// Compiles the exam to PDF via Typst and returns the raw bytes as a Base64 string.
#[tauri::command]
pub async fn export_exam_pdf(
    state:      State<'_, Mutex<AppState>>,
    exam_id:    String,
    answer_key: bool,
    booklet:    Option<String>,
) -> Result<String, String> {
    let st  = state.lock().await;
    let eid = uuid::Uuid::parse_str(&exam_id).map(ExamId).map_err(|e| e.to_string())?;

    let exam = st.exams.find_by_id(&eid).await.map_err(|e| e.to_string())?;
    let bank = st.bank.load().await.map_err(|e| e.to_string())?;

    let ctx = tayan_core::domain::shared::to_typst::TypstContext {
        answer_key,
        booklet,
        ..Default::default()
    };

    let questions: Vec<_> = exam.questions.iter()
        .filter_map(|r| bank.find(&r.question_id).map(|bq| bq.question.clone()))
        .collect();

    let source = tayan_compiler::typst_gen::TypstGenerator::generate_exam(&exam, &questions, ctx)
        .map_err(|e| e.to_string())?;

    let pdf_bytes = tayan_compiler::TayanWorld::compile_pdf(source)
        .map_err(|e| e.to_string())?;

    use base64::Engine as _;
    Ok(base64::engine::general_purpose::STANDARD.encode(&pdf_bytes))
}

/// Generates a Typst file and saves it to the Downloads (or Desktop) folder.
/// Returns the absolute path to the saved file.
#[tauri::command]
pub async fn export_typst_file(
    state:      State<'_, Mutex<AppState>>,
    exam_id:    String,
    answer_key: bool,
    booklet:    Option<String>,
) -> Result<String, String> {
    let st  = state.lock().await;
    let eid = uuid::Uuid::parse_str(&exam_id).map(ExamId).map_err(|e| e.to_string())?;

    let exam = st.exams.find_by_id(&eid).await.map_err(|e| e.to_string())?;
    let bank = st.bank.load().await.map_err(|e| e.to_string())?;

    let ctx = tayan_core::domain::shared::to_typst::TypstContext {
        answer_key,
        booklet,
        ..Default::default()
    };

    let questions: Vec<_> = exam.questions.iter()
        .filter_map(|r| bank.find(&r.question_id).map(|bq| bq.question.clone()))
        .collect();

    let source = tayan_compiler::typst_gen::TypstGenerator::generate_exam(&exam, &questions, ctx)
        .map_err(|e| e.to_string())?;

    let base_dir = dirs_next::download_dir()
        .or_else(dirs_next::desktop_dir)
        .ok_or_else(|| "İndirme klasörü bulunamadı".to_string())?;

    let slug: String = exam.meta.title
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    let slug = if slug.is_empty() { "sinav".to_string() } else { slug };

    let ts     = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let suffix = if answer_key { "_cevap" } else { "" };
    let filename = format!("{}{}__{}.typ", slug, suffix, ts);
    let path = base_dir.join(&filename);

    std::fs::write(&path, source).map_err(|e| e.to_string())?;
    Ok(path.display().to_string())
}

/// Compiles an arbitrary Typst source to PDF and returns Base64 bytes.
/// Used by the Typst body editor for live preview.
///
/// CPU-yoğun iş spawn_blocking ile ayrı thread'e alınır —
/// async runtime'ı bloklamaz, UI akışkan kalır.
#[tauri::command]
pub async fn compile_typst_preview(source: String) -> Result<String, String> {
    let pdf_bytes = tokio::task::spawn_blocking(move || {
        tayan_compiler::TayanWorld::compile_pdf(source)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    use base64::Engine as _;
    Ok(base64::engine::general_purpose::STANDARD.encode(&pdf_bytes))
}

/// Canlı önizleme: rastgele Typst kaynağını sayfa başına bir SVG dizesine derler.
///
/// PDF yolundan (compile_typst_preview) ayrıdır. PDF her derlemede iframe'i
/// baştan yükletir; kaydırma konumu sıfırlanır ve ekran titrer. SVG doğrudan
/// DOM'a girer, yalnızca değişen sayfa değişir.
///
/// CPU-yoğun iş spawn_blocking ile ayrı thread'e alınır —
/// async runtime'ı bloklamaz, UI akışkan kalır.
#[tauri::command]
pub async fn compile_typst_preview_svg(source: String) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || tayan_compiler::TayanWorld::compile_svg(source))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// Soru editörünün canlı önizlemesi: gövdeyi sınavın gerçek önsözüyle sarmalar,
/// sayfa başına bir SVG döndürür.
///
/// Önsöz Rust tarafında kalır (TypstGenerator::preview_document). Ön yüz onu
/// bilmez, dolayısıyla kopyalayıp sürükleyemez.
#[tauri::command]
pub async fn compile_question_preview_svg(body: String) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        let source = tayan_compiler::typst_gen::TypstGenerator::preview_document(&body);
        tayan_compiler::TayanWorld::compile_svg(source)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| {
        let offset = tayan_compiler::typst_gen::TypstGenerator::preview_line_offset();
        shift_diagnostic_lines(&e.to_string(), offset)
    })
}

/// Tanılamalardaki satır numaralarını öğretmenin gördüğü metne çevirir.
///
/// Typst birleşik belgeye (önsöz + gövde) göre satır verir; editörde ise
/// yalnızca gövde vardır. Kaydırılmazsa 5. satırdaki hata "satır 98" diye
/// raporlanır: kenardaki kırmızı kalem işareti hiç çıkmaz, çünkü 98. satır
/// yoktur, ve mesaj öğretmene hiçbir şey söylemez.
///
/// Önsözün kendi içindeki bir hata (satır <= offset) kaydırılmaz; o bizim
/// kusurumuzdur ve olduğu gibi görünmelidir.
fn shift_diagnostic_lines(message: &str, offset: usize) -> String {
    const PREFIX: &str = "(satır ";

    let mut out = String::with_capacity(message.len());
    let mut rest = message;

    while let Some(at) = rest.find(PREFIX) {
        out.push_str(&rest[..at]);
        rest = &rest[at..];

        let after = &rest[PREFIX.len()..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();

        match digits.parse::<usize>() {
            Ok(line) if line > offset => {
                out.push_str(PREFIX);
                out.push_str(&(line - offset).to_string());
                rest = &after[digits.len()..];
            }
            _ => {
                out.push_str(PREFIX);
                rest = after;
            }
        }
    }

    out.push_str(rest);
    out
}


#[cfg(test)]
mod diagnostic_shift_tests {
    use super::shift_diagnostic_lines;

    #[test]
    fn gövde_hatası_editör_satırına_çevrilir() {
        let msg = "Typst derleme hatası:\nexpected comma (satır 98, sütun 39)";
        let out = shift_diagnostic_lines(msg, 93);
        assert!(out.contains("(satır 5, sütun 39)"), "{out}");
    }

    #[test]
    fn önsöz_hatası_kaydırılmaz() {
        // Önsözün kendi içindeki hata bizim kusurumuz; olduğu gibi görünmeli.
        let msg = "unknown variable (satır 40, sütun 3)";
        let out = shift_diagnostic_lines(msg, 93);
        assert!(out.contains("(satır 40, sütun 3)"), "{out}");
    }

    #[test]
    fn birden_çok_tanılama_hepsi_kaydırılır() {
        let msg = "a (satır 100, sütun 1)\nb (satır 110, sütun 2)";
        let out = shift_diagnostic_lines(msg, 93);
        assert!(out.contains("(satır 7, sütun 1)"), "{out}");
        assert!(out.contains("(satır 17, sütun 2)"), "{out}");
    }

    #[test]
    fn konumsuz_mesaj_bozulmaz() {
        let msg = "Typst derleme hatası:\nbir şey oldu";
        assert_eq!(shift_diagnostic_lines(msg, 93), msg);
    }
}

/// Typst standart kütüphanesinin ve şablon yardımcılarının tam dökümü.
///
/// Editörün otomatik tamamlaması bunu kullanır. Elle yazılmış bir liste iki
/// yönden bozulurdu: Typst sürümü değişince eskir, ve baştan eksik kalır.
/// Ölçülen döküm: 554 sembol (133 işlev, 300 sembol, 397'si matematik kipi).
#[tauri::command]
pub fn typst_symbols() -> Vec<tayan_compiler::symbols::TypstSymbol> {
    tayan_compiler::symbols::all_symbols()
}

/// tinymist'ten tamamlama ister.
///
/// ASYNC ve spawn_blocking şart. Tauri'de senkron komutlar ana iş parçacığında
/// koşar; bu komut tinymist'e tam belgeyi gönderip yanıtı BLOKLAYARAK bekliyor.
/// Senkron bırakıldığında her tuş vuruşunda — silme dahil — ana iş parçacığı
/// duruyor ve imleç yazının gerisinde kalıyordu.
///
/// Gövde, önsözle SARMALANARAK gönderilir: tinymist #secenekler gibi şablon
/// yardımcılarını ancak #let tanımlarını görürse önerebilir. Bu yüzden satır
/// numarası da önsöz kadar kaydırılır.
///
/// Hata durumunda Err döner ve ön yüz kendi sembol dökümüne düşer. tinymist
/// yoksa, çökmüşse veya yavaşsa editör yazmaya devam eder.
#[tauri::command]
pub async fn lsp_complete(
    tinymist: tauri::State<'_, std::sync::Arc<crate::lsp::Tinymist>>,
    app: tauri::AppHandle,
    body: String,
    line: u32,
    character: u32,
) -> Result<Vec<crate::lsp::LspCompletion>, String> {
    let bin = crate::lsp::binary_path(&app)?;
    let root = tayan_compiler::world::app_data_root().join("lsp");
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;

    let client = tinymist.inner().clone();

    tokio::task::spawn_blocking(move || {
        let source = tayan_compiler::typst_gen::TypstGenerator::preview_document(&body);
        let offset = tayan_compiler::typst_gen::TypstGenerator::preview_line_offset() as u32;
        client.complete(&bin, &root, &source, line + offset, character)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Dil sunucusunun kurulu olup olmadığı.
#[tauri::command]
pub fn lsp_status() -> crate::lsp_install::LspStatus {
    crate::lsp_install::status()
}

/// Dil sunucusunu indirir ve kurar.
///
/// AÇIK bir kullanıcı eylemi olarak çağrılır, ilk açılışta kendiliğinden değil:
/// "tamamen çevrimdışı" bir üründe kullanıcı sormadan ağa çıkmak kabul edilemez.
/// İndirme sha256 ile doğrulanır.
#[tauri::command]
pub async fn lsp_install() -> Result<String, String> {
    tokio::task::spawn_blocking(crate::lsp_install::install)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn lsp_uninstall(
    tinymist: tauri::State<'_, std::sync::Arc<crate::lsp::Tinymist>>,
) -> Result<(), String> {
    // Çalışan süreci önce durdur: kullanılan dosyayı silmek bazı sistemlerde
    // başarısız olur, kalanı da yetim süreç bırakır.
    tinymist.shutdown();
    crate::lsp_install::uninstall()
}
