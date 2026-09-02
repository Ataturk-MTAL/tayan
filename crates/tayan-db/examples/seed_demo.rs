//! Deneme verisi ekler: sorular, sınıf, öğrenciler ve yayımlanmış bir sınav.
//!
//! Amaç, sonuç girişi → madde analizi zincirini elle kurmadan denenebilir hâle
//! getirmek. Elle JSON YAZMIYOR: alan tiplerini ve depoları kullanıyor, bu
//! yüzden kaydedilen şekiller uygulamanın yazdığıyla birebir aynı.
//!
//! Koşum: cargo run -p tayan-db --example seed_demo
//!
//! Mevcut kayıtları SİLMEZ, üzerine ekler.

use chrono::NaiveDate;
use tayan_core::application::ports::{
    ClassroomRepository, ExamRepository, QuestionBankRepository, StudentRepository,
};
use tayan_core::domain::exam_management::aggregates::{Exam, ExamMeta, ExamSigner};
use tayan_core::domain::exam_management::entities::classic::{AnswerSpace, ClassicQuestion};
use tayan_core::domain::exam_management::entities::multiple_choice::{
    MultipleChoiceQuestion, QuestionOption,
};
use tayan_core::domain::exam_management::entities::question::{Points, Question, QuestionId};
use tayan_core::domain::exam_management::entities::true_false::TrueFalseQuestion;
use tayan_core::domain::exam_management::value_objects::{
    ContentNode, Difficulty, OutcomeCode, QuestionBody, QuestionMeta,
};
use tayan_core::domain::student_management::aggregates::{Classroom, Student};

fn govde(src: &str) -> QuestionBody {
    QuestionBody(vec![ContentNode::typst_raw(src)])
}

fn kunye(zorluk: Difficulty) -> QuestionMeta {
    QuestionMeta::new("Matematik", 9, Some(zorluk))
}

fn kazanim(kod: &str) -> Vec<OutcomeCode> {
    vec![OutcomeCode::new(kod).expect("geçerli kazanım kodu")]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Uygulamanın kullandığı klasörün aynısı (world::app_data_root ile aynı
    // hesap). tayan-compiler'ı çekmek tüm typst'i bağımlılık yapardı.
    let base = dirs_next::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("tayan");
    std::fs::create_dir_all(&base)?;
    let url = format!("sqlite:{}", base.join("tayan_dev.db").display());
    println!("veritabanı: {url}");

    let pool = tayan_db::connect(&url).await?;
    tayan_db::run_migrations(&pool).await?;

    let bank_repo = tayan_db::repositories::SqliteQuestionBankRepository { pool: pool.clone() };
    let exam_repo = tayan_db::repositories::SqliteExamRepository { pool: pool.clone() };
    let class_repo = tayan_db::repositories::SqliteClassroomRepository { pool: pool.clone() };
    let student_repo = tayan_db::repositories::SqliteStudentRepository { pool: pool.clone() };

    // ── Sorular ───────────────────────────────────────────────────────────────
    let sorular = vec![
        Question::MultipleChoice(MultipleChoiceQuestion {
            id: QuestionId::new(),
            points: Points::new(25),
            outcomes: kazanim("MAT.9.1.2"),
            meta: kunye(Difficulty::Orta),
            body: govde("Aşağıdaki denklemin köklerini bulunuz.\n\n$ x^2 - 5x + 6 = 0 $"),
            options: vec![
                QuestionOption { id: "A".into(), body: govde("$x = 1$"), correct: false },
                QuestionOption { id: "B".into(), body: govde("$x = 2$"), correct: false },
                QuestionOption { id: "C".into(), body: govde("$x = 2$ ve $x = 3$"), correct: true },
                QuestionOption { id: "D".into(), body: govde("$x = 6$"), correct: false },
            ],
            shuffle: true,
            stats: Default::default(),
        }),
        Question::MultipleChoice(MultipleChoiceQuestion {
            id: QuestionId::new(),
            points: Points::new(25),
            outcomes: kazanim("MAT.9.1.3"),
            meta: kunye(Difficulty::Kolay),
            body: govde("$(1011 thin 0110)_2$ sayısının onluk tabandaki karşılığı kaçtır?"),
            options: vec![
                QuestionOption { id: "A".into(), body: govde("$180$"), correct: false },
                QuestionOption { id: "B".into(), body: govde("$182$"), correct: true },
                QuestionOption { id: "C".into(), body: govde("$184$"), correct: false },
                QuestionOption { id: "D".into(), body: govde("$186$"), correct: false },
            ],
            shuffle: false,
            stats: Default::default(),
        }),
        Question::TrueFalse(TrueFalseQuestion::new(
            QuestionId::new(),
            Points::new(25),
            kazanim("MAT.9.2.1"),
            govde("Bir üçgenin iç açılarının ölçüleri toplamı $180$ derecedir."),
            true,
            kunye(Difficulty::Kolay),
        )),
        Question::Classic(ClassicQuestion {
            id: QuestionId::new(),
            points: Points::new(25),
            outcomes: kazanim("MAT.9.5.1"),
            meta: kunye(Difficulty::Zor),
            body: govde(
                "$f(x) = 2x + 3$ fonksiyonunun grafiğini çiziniz ve eksenleri kestiği \
                 noktaları yazınız.\n\n#cevap-alani(satir: 10, bicim: \"kareli\")",
            ),
            sample_answer: None,
            rubric: vec![],
            answer_space: AnswerSpace::Lines(6),
            stats: Default::default(),
        }),
    ];

    // Etkisiz tekrar: aynı gövdeli soru zaten varsa onun kimliği kullanılır.
    // Aksi hâlde her koşuda banka kopyalarla dolardı.
    let mut bank = bank_repo.load().await?;
    let mut ids: Vec<QuestionId> = Vec::new();
    let mut yeni = 0usize;

    for q in sorular {
        q.validate()?;
        let govde_metni = format!("{:?}", q.body());
        let mevcut = bank
            .questions
            .iter()
            .find(|bq| format!("{:?}", bq.question.body()) == govde_metni)
            .map(|bq| bq.question.id().clone());

        match mevcut {
            Some(id) => ids.push(id),
            None => {
                ids.push(q.id().clone());
                bank.add_question(q)?;
                yeni += 1;
            }
        }
    }
    bank_repo.save(&bank).await?;
    println!("soru           : {yeni} yeni, {} toplam kullanılan", ids.len());

    // ── Sınıf ve öğrenciler ───────────────────────────────────────────────────
    // Altı öğrenci bilinçli: ayırt edicilik 6'dan az cevapta 0 döner, çünkü
    // üst ve alt %27 dilimleri o sayıda anlamlı olmaz. Altıncı sonuç girilince
    // değer ilk kez gerçek bir sayı olur.
    let mut sinif = Classroom::new("9-A", 9, "A");
    let ogrenciler = [
        ("101", "Ayşe", "YILMAZ"),
        ("102", "Berk", "DEMİR"),
        ("103", "Ceren", "KAYA"),
        ("104", "Deniz", "ŞAHİN"),
        ("105", "Emre", "ÇELİK"),
        ("106", "Fatma", "ÖZTÜRK"),
    ];
    // Sınıf ÖNCE kaydedilmeli: students tablosu classrooms(id)'ye yabancı
    // anahtarla bağlı, ters sırada "FOREIGN KEY constraint failed" döner.
    class_repo.save(&sinif).await?;
    let mut kayitli = Vec::new();
    for (no, ad, soyad) in ogrenciler {
        let s = Student::new(no, ad, soyad, sinif.id.clone());
        sinif.student_ids.push(s.id.clone());
        student_repo.save_student(&s).await?;
        kayitli.push(s.id.clone());
    }
    // Öğrenci kimlikleri eklendikten sonra sınıfı tekrar kaydet.
    class_repo.save(&sinif).await?;
    let _ = kayitli;
    println!("öğrenci eklendi: {}", ogrenciler.len());

    // ── Sınav ─────────────────────────────────────────────────────────────────
    let mut sinav = Exam::new(ExamMeta {
        title: "1. Dönem 1. Yazılı".into(),
        subject: "Matematik".into(),
        classroom: "9-A".into(),
        teacher: "Hakan GÜLEN".into(),
        duration_min: 40,
        date: NaiveDate::from_ymd_opt(2026, 11, 12).unwrap(),
        instructions: None,
        columns: 1,
        school: Some("Atatürk Mesleki ve Teknik Anadolu Lisesi".into()),
        department: Some("Elektrik-Elektronik Teknolojisi Alanı".into()),
        signers: vec![
            ExamSigner { name: "Ömer YİĞİT".into(), title: "Okul Müdürü".into() },
            ExamSigner { name: "Hakan GÜLEN".into(), title: "Ders Öğretmeni".into() },
        ],
    });
    for id in &ids {
        sinav.add_question_ref(id.clone());
    }
    // Yayımlanmış: analiz ekranı taslak sınavları ilk seçim olarak almıyor.
    sinav.publish()?;
    exam_repo.save(&sinav).await?;
    println!(
        "sınav eklendi  : {} ({} soru)",
        sinav.meta.title,
        sinav.questions.len()
    );

    println!("\nHazır. Analiz > Sonuç girişi ekranından 9-A sınıfına sonuç girebilirsin.");
    Ok(())
}
