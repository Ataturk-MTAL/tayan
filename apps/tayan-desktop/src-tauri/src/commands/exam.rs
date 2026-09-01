use tauri::State;
use tokio::sync::Mutex;

use tayan_core::{
    application::{
        commands::{AddQuestionToExam, CreateExam},
        ports::ExamRepository,
    },
    domain::exam_management::aggregates::{Exam, ExamId},
    domain::exam_management::entities::question::QuestionId,
};
use crate::state::AppState;

#[tauri::command]
pub async fn create_exam(
    state:   State<'_, Mutex<AppState>>,
    payload: CreateExam,
) -> Result<String, String> {
    let st   = state.lock().await;
    let exam = Exam::new(payload.meta);
    let id   = exam.id.0.to_string();
    st.exams.save(&exam).await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn get_exam(
    state:   State<'_, Mutex<AppState>>,
    exam_id: String,
) -> Result<Exam, String> {
    let st = state.lock().await;
    let id = parse_exam_id(&exam_id)?;
    st.exams.find_by_id(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_exams(
    state:    State<'_, Mutex<AppState>>,
    page:     u32,
    per_page: u32,
) -> Result<Vec<Exam>, String> {
    let st = state.lock().await;
    st.exams.list(page, per_page).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_question_to_exam(
    state:   State<'_, Mutex<AppState>>,
    payload: AddQuestionToExam,
) -> Result<(), String> {
    let st = state.lock().await;
    let mut exam = st.exams.find_by_id(&payload.exam_id).await.map_err(|e| e.to_string())?;
    exam.add_question_ref(payload.question_id);
    st.exams.save(&exam).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_question_from_exam(
    state:       State<'_, Mutex<AppState>>,
    exam_id:     String,
    question_id: String,
) -> Result<(), String> {
    let st  = state.lock().await;
    let eid = parse_exam_id(&exam_id)?;
    let qid = uuid::Uuid::parse_str(&question_id).map(QuestionId).map_err(|e| e.to_string())?;
    let mut exam = st.exams.find_by_id(&eid).await.map_err(|e| e.to_string())?;
    exam.remove_question_ref(&qid).map_err(|e| e.to_string())?;
    st.exams.save(&exam).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn publish_exam(
    state:   State<'_, Mutex<AppState>>,
    exam_id: String,
) -> Result<(), String> {
    let st = state.lock().await;
    let id = parse_exam_id(&exam_id)?;
    let mut exam = st.exams.find_by_id(&id).await.map_err(|e| e.to_string())?;
    exam.publish().map_err(|e| e.to_string())?;
    st.exams.save(&exam).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_exam(
    state:   State<'_, Mutex<AppState>>,
    exam_id: String,
) -> Result<(), String> {
    let st = state.lock().await;
    let id = parse_exam_id(&exam_id)?;
    st.exams.delete(&id).await.map_err(|e| e.to_string())
}

fn parse_exam_id(s: &str) -> Result<ExamId, String> {
    uuid::Uuid::parse_str(s)
        .map(ExamId)
        .map_err(|e| format!("Geçersiz sınav kimliği: {e}"))
}

/// Bir sorunun BU SINAVDAKİ puanını belirler.
///
/// Puan soruya değil, sorunun sınavdaki kullanımına aittir: aynı soru bir
/// yazılıda 5, başkasında 10 puan edebilir. points None ise sorunun kendi
/// puanına dönülür.
#[tauri::command]
pub async fn set_exam_question_points(
    state:       State<'_, Mutex<AppState>>,
    exam_id:     String,
    question_id: String,
    points:      Option<u32>,
) -> Result<(), String> {
    let st  = state.lock().await;
    let eid = uuid::Uuid::parse_str(&exam_id).map(ExamId).map_err(|e| e.to_string())?;
    let qid = uuid::Uuid::parse_str(&question_id)
        .map(tayan_core::domain::exam_management::entities::question::QuestionId)
        .map_err(|e| e.to_string())?;

    let mut exam = st.exams.find_by_id(&eid).await.map_err(|e| e.to_string())?;

    exam.set_question_points(&qid, points).map_err(|e| e.to_string())?;
    st.exams.save(&exam).await.map_err(|e| e.to_string())?;
    Ok(())
}
