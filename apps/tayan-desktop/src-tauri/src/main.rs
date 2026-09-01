#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod migration;
mod state;

use tauri::Manager;
use tokio::sync::Mutex;
use state::AppState;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Font künye kaydını arka planda kur. Ölçüm: 510 aile için 2,7-4,4 s.
            // Tembel bırakılırsa bu süre öğretmenin ilk tuş vuruşuna biner ve
            // canlı önizleme donmuş görünür. Pencere açılırken bitirilir.
            std::thread::spawn(|| {
                let t0 = std::time::Instant::now();
                tayan_compiler::world::warm_font_registry();
                eprintln!("font kaydı hazır: {:?}", t0.elapsed());
            });

            let db_path = resolve_db_path();
            let pool = tauri::async_runtime::block_on(async {
                let p = tayan_db::connect(&db_path).await
                    .expect("DB connection failed");
                tayan_db::run_migrations(&p).await
                    .expect("Migrations failed");
                p
            });
            let app_state = AppState::new(pool);

            // Görsel depolama göçü. Etkisiz tekrarlanabilir: taşınacak dosya ve
            // düzeltilecek yol kalmadığında hiçbir şey yapmaz.
            //
            // Eski sürümler görselleri Tauri'nin app_local_data_dir()'ine
            // yazıyordu; veritabanı ise data_local_dir()/tayan altındaydı. İki
            // ayrı klasör, tek klasörü yedekleyen kullanıcının görsellerini
            // kaybetmesi demekti.
            let legacy_images = app.path().app_local_data_dir().ok().map(|d| d.join("images"));
            let bank = app_state.bank.clone();
            tauri::async_runtime::block_on(async move {
                match migration::migrate_image_storage(&bank, legacy_images).await {
                    Ok(r) if r.moved_files > 0 || r.rewritten_refs > 0 => eprintln!(
                        "görsel göçü: {} dosya taşındı, {} yol düzeltildi",
                        r.moved_files, r.rewritten_refs
                    ),
                    Ok(_) => {}
                    // Göç başarısız olsa da uygulama açılmalı: eski mutlak yollar
                    // hâlâ çözülüyor, yani veri kullanılabilir durumda.
                    Err(e) => eprintln!("görsel göçü başarısız: {e}"),
                }

                // Kullanılmayan görselleri topla. Yalnızca 24 saatten eski
                // dosyalar: açık editörde duran, henüz kaydedilmemiş sorunun
                // görseli hiçbir atıfta görünmez ve eşik olmadan silinirdi.
                match migration::collect_orphan_images(&bank).await {
                    Ok((n, bytes)) if n > 0 => {
                        eprintln!("kullanılmayan görsel silindi: {n} dosya, {} KB", bytes / 1024)
                    }
                    Ok(_) => {}
                    Err(e) => eprintln!("görsel temizliği başarısız: {e}"),
                }
            });

            app.manage(Mutex::new(app_state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Exam
            commands::exam::create_exam,
            commands::exam::get_exam,
            commands::exam::list_exams,
            commands::exam::add_question_to_exam,
            commands::exam::remove_question_from_exam,
            commands::exam::set_exam_question_points,
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
            commands::analysis::typst_symbols,
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
