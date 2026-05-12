use tauri::State;
use tokio::sync::Mutex;

use tayan_core::{
    application::{
        commands::{AddStudent, CreateClassroom},
        ports::{ClassroomRepository, StudentRepository},
    },
    domain::student_management::aggregates::{Classroom, ClassroomId, Student},
};
use crate::state::AppState;

#[tauri::command]
pub async fn create_classroom(
    state:   State<'_, Mutex<AppState>>,
    payload: CreateClassroom,
) -> Result<String, String> {
    let st        = state.lock().await;
    let classroom = Classroom::new(payload.name, payload.grade, payload.branch);
    let id        = classroom.id.0.to_string();
    st.classes.save(&classroom).await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn list_classrooms(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<Classroom>, String> {
    let st = state.lock().await;
    st.classes.list().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_student(
    state:   State<'_, Mutex<AppState>>,
    payload: AddStudent,
) -> Result<String, String> {
    let st      = state.lock().await;
    let student = Student::new(
        payload.number,
        payload.first_name,
        payload.last_name,
        payload.classroom_id,
    );
    let id = student.id.0.to_string();
    st.students.save_student(&student).await.map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn list_students_by_classroom(
    state:        State<'_, Mutex<AppState>>,
    classroom_id: String,
) -> Result<Vec<Student>, String> {
    let st  = state.lock().await;
    let cid = uuid::Uuid::parse_str(&classroom_id)
        .map(ClassroomId)
        .map_err(|e| e.to_string())?;
    st.students.list_by_classroom(&cid).await.map_err(|e| e.to_string())
}
