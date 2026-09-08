use tauri::State;
use tokio::sync::Mutex;

use uuid::Uuid;
use tayan_core::{
    application::{
        commands::{AddStudent, CreateClassroom},
        ports::{ClassroomRepository, RepositoryError, StudentRepository},
    },
    domain::student_management::aggregates::{Classroom, ClassroomId, Student, StudentId},
};
use crate::state::AppState;

/// Depo hatasını öğretmenin okuyacağı Türkçeye çevirir.
///
/// `RepositoryError`in kendi `Display`i İngilizce bir önek basıyor
/// ("Conflict: …"); ham `to_string()` o öneki Türkçe arayüze sızdırırdı.
/// `Conflict` bilinçli olarak REDDEDİLMİŞ bir işlem ve gövdesi zaten tam bir
/// Türkçe cümle — olduğu gibi kullanılıyor. Diğer varyantlar beklenmeyen
/// durumlar; ayrıntıları teşhis için korunuyor.
fn student_error(action: &str, err: RepositoryError) -> String {
    match err {
        RepositoryError::Conflict(detail) => format!("{action}: {detail}."),
        other => format!("{action}: {other}"),
    }
}

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
    let cid = Uuid::parse_str(&classroom_id)
        .map(ClassroomId)
        .map_err(|e| e.to_string())?;
    st.students.list_by_classroom(&cid).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_student(
    state:      State<'_, Mutex<AppState>>,
    student_id: String,
) -> Result<(), String> {
    let st  = state.lock().await;
    let sid = Uuid::parse_str(&student_id)
        .map(StudentId)
        .map_err(|e| e.to_string())?;
    st.students
        .delete_student(&sid)
        .await
        .map_err(|e| student_error("Öğrenci silinemedi", e))
}

#[tauri::command]
pub async fn delete_classroom(
    state:        State<'_, Mutex<AppState>>,
    classroom_id: String,
) -> Result<(), String> {
    let st  = state.lock().await;
    let cid = Uuid::parse_str(&classroom_id)
        .map(ClassroomId)
        .map_err(|e| e.to_string())?;
    st.classes
        .delete(&cid)
        .await
        .map_err(|e| student_error("Sınıf silinemedi", e))
}

// ── Sınıf listesi içe aktarma ─────────────────────────────────────────────────

/// Bir çalışma sayfasının ham hâli: başlık satırı aranmadan, hücreler metne
/// çevrilmiş hâlde.
///
/// ANLAMLANDIRMA BURADA DEĞİL, ARAYÜZDE. Rust yalnız ikili biçimi çözüyor;
/// hangi sütunun "numara", hangisinin "ad" olduğuna öğretmen karar veriyor.
/// e-Okul çıktısının sütun düzeni elimizde yok ve tahmini koda gömmek, biçim
/// değiştiğinde sessizce yanlış veri üretirdi.
#[derive(serde::Serialize)]
pub struct RosterSheet {
    /// Sayfadaki tüm satırlar; her hücre metin, boş hücre boş dizge.
    pub rows: Vec<Vec<String>>,
}

/// Excel/OpenDocument sınıf listesini ayrıştırır.
///
/// Baytlar webview'in dosya seçicisinden IPC ile geliyor — yol değil, içerik.
/// Böylece `plugin-fs` ve `dialog:allow-open` izni gerekmiyor.
///
/// `.xlsx`, `.xls`, `.xlsb` ve `.ods` destekleniyor; biçim uzantıdan değil
/// içerikten anlaşılıyor, yani yanlış uzantıyla kaydedilmiş dosya da açılıyor.
#[tauri::command]
pub async fn parse_roster(bytes: Vec<u8>) -> Result<RosterSheet, String> {
    use calamine::{Data, Reader};
    use std::io::Cursor;

    let mut workbook = calamine::open_workbook_auto_from_rs(Cursor::new(bytes))
        .map_err(|e| format!("Dosya açılamadı: {e}"))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| "Dosyada çalışma sayfası yok.".to_string())?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("Sayfa okunamadı: {e}"))?;

    // Hücreler metne çevriliyor. Öğrenci numarası Excel'de sayı olarak
    // saklanmış olabilir ("101" yerine 101.0); `Data::Float` için ondalık
    // kısmı düşürmek gerekiyor, yoksa numara "101" değil "101.0" olurdu.
    let rows = range
        .rows()
        .map(|row| {
            row.iter()
                .map(|cell| match cell {
                    Data::Empty => String::new(),
                    Data::String(s) => s.trim().to_string(),
                    Data::Float(f) => {
                        if f.fract() == 0.0 {
                            format!("{}", *f as i64)
                        } else {
                            f.to_string()
                        }
                    }
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(d) => d.to_string(),
                    Data::DateTimeIso(s) => s.clone(),
                    Data::DurationIso(s) => s.clone(),
                    Data::Error(e) => format!("#{e:?}"),
                })
                .collect()
        })
        .collect();

    Ok(RosterSheet { rows })
}
