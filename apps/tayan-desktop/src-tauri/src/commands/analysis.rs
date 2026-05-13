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
) -> Result<String, String> {
    let st  = state.lock().await;
    let eid = uuid::Uuid::parse_str(&exam_id).map(ExamId).map_err(|e| e.to_string())?;

    let exam = st.exams.find_by_id(&eid).await.map_err(|e| e.to_string())?;
    let bank = st.bank.load().await.map_err(|e| e.to_string())?;

    let ctx = tayan_core::domain::shared::to_typst::TypstContext {
        answer_key,
        shuffle: false,
        question_number: None,
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
) -> Result<String, String> {
    let st  = state.lock().await;
    let eid = uuid::Uuid::parse_str(&exam_id).map(ExamId).map_err(|e| e.to_string())?;

    let exam = st.exams.find_by_id(&eid).await.map_err(|e| e.to_string())?;
    let bank = st.bank.load().await.map_err(|e| e.to_string())?;

    let ctx = tayan_core::domain::shared::to_typst::TypstContext {
        answer_key,
        shuffle: false,
        question_number: None,
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
) -> Result<String, String> {
    let st  = state.lock().await;
    let eid = uuid::Uuid::parse_str(&exam_id).map(ExamId).map_err(|e| e.to_string())?;

    let exam = st.exams.find_by_id(&eid).await.map_err(|e| e.to_string())?;
    let bank = st.bank.load().await.map_err(|e| e.to_string())?;

    let ctx = tayan_core::domain::shared::to_typst::TypstContext {
        answer_key,
        shuffle: false,
        question_number: None,
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
