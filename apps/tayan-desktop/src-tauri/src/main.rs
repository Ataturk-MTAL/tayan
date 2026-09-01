#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use tauri::Manager;
use tokio::sync::Mutex;
use state::AppState;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let db_path = resolve_db_path();
            let pool = tauri::async_runtime::block_on(async {
                let p = tayan_db::connect(&db_path).await
                    .expect("DB connection failed");
                tayan_db::run_migrations(&p).await
                    .expect("Migrations failed");
                p
            });
            app.manage(Mutex::new(AppState::new(pool)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Exam
            commands::exam::create_exam,
            commands::exam::get_exam,
            commands::exam::list_exams,
            commands::exam::add_question_to_exam,
            commands::exam::remove_question_from_exam,
            commands::exam::publish_exam,
            commands::exam::delete_exam,
            // Question bank
            commands::question::add_multiple_choice_question,
            commands::question::add_true_false_question,
            commands::question::add_fill_in_blank_question,
            commands::question::add_classic_question,
            commands::question::list_questions,
            commands::question::delete_question,
            commands::question::update_question,
            // Students
            commands::student::create_classroom,
            commands::student::list_classrooms,
            commands::student::add_student,
            commands::student::list_students_by_classroom,
            commands::student::delete_student,
            commands::student::delete_classroom,
            // Analysis / results
            commands::analysis::enter_exam_results,
            commands::analysis::get_exam_results,
            commands::analysis::generate_exam_pdf,
            commands::analysis::export_exam_pdf,
            commands::analysis::export_typst_file,
            commands::analysis::compile_typst_preview,
            commands::analysis::compile_typst_preview_svg,
            commands::analysis::compile_question_preview_svg,
            // Images
            commands::image::save_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn resolve_db_path() -> String {
    let base_dir = dirs_next::data_local_dir().unwrap_or_else(std::env::temp_dir);
    let app_dir = base_dir.join("tayan");
    let _ = std::fs::create_dir_all(&app_dir);

    let file_name = if cfg!(debug_assertions) {
        "tayan_dev.db"
    } else {
        "tayan.db"
    };

    format!("sqlite:{}", app_dir.join(file_name).display())
}
