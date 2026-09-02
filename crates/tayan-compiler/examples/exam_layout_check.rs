//! Tek sütunlu ve çift sütunlu sınav kâğıdının GERÇEKTEN dizildiğini doğrular.
//!
//! `cargo check` yalnızca Rust'ın derlendiğini söyler; üretilen Typst'in geçerli
//! olup olmadığını söylemez. Bu örnek iki yerleşimi de derleyip sayfa sayısını
//! ve SVG boyutunu basar.
//!
//! Koşum: cargo run -p tayan-compiler --example exam_layout_check

use chrono::NaiveDate;
use tayan_core::domain::exam_management::aggregates::{Exam, ExamMeta, ExamSigner};
use tayan_core::domain::exam_management::entities::classic::{AnswerSpace, ClassicQuestion};
use tayan_core::domain::exam_management::entities::multiple_choice::{
    MultipleChoiceQuestion, QuestionOption,
};
use tayan_core::domain::exam_management::entities::question::{Points, Question, QuestionId};
use tayan_core::domain::exam_management::entities::true_false::TrueFalseQuestion;
use tayan_core::domain::exam_management::value_objects::{ContentNode, QuestionBody};
use tayan_core::domain::shared::to_typst::TypstContext;

const OUT_DIR: &str =
    "/private/tmp/claude-501/-Users-hakan-VSCodeProjects/c1f16c60-0f9e-4ced-bde6-cc57e12e5ea2/scratchpad";

fn typst(src: &str) -> QuestionBody {
    QuestionBody(vec![ContentNode::typst_raw(src)])
}

fn meta(columns: u8) -> ExamMeta {
    ExamMeta {
        title:        "2025-2026 Haziran Dönemi Sorumluluk Sınavı".into(),
        subject:      "Mikrodenetleyici ve Güvenlik Atölyesi".into(),
        classroom:    "11. Sınıf · Anadolu Teknik Programı".into(),
        teacher:      "Hakan GÜLEN".into(),
        duration_min: 40,
        date:         NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        instructions: Some(
            "Sınav 4 sorudan oluşur, her soru 25 puandır (toplam 100). \
             İşlem gerektiren sorularda işlemleri gösteriniz."
                .into(),
        ),
        columns,
        school:     Some("Atatürk Mesleki ve Teknik Anadolu Lisesi".into()),
        department: Some("Elektrik-Elektronik Teknolojisi Alanı".into()),
        signers: vec![
            ExamSigner { name: "Ömer YİĞİT".into(),  title: "Okul Müdürü".into() },
            ExamSigner { name: "Sedat İLYAS".into(), title: "Ders Öğretmeni".into() },
            ExamSigner { name: "Hakan GÜLEN".into(), title: "Ders Öğretmeni".into() },
        ],
    }
}

fn questions() -> Vec<Question> {
    vec![
        Question::MultipleChoice(MultipleChoiceQuestion {
            id:       QuestionId::new(),
            points:   Points::new(25),
            outcomes: vec![],
            body:     typst(
                "$(1011 0110)_2$ sayısının onluk tabandaki karşılığı aşağıdakilerden hangisidir?",
            ),
            options: vec![
                QuestionOption { id: "A".into(), body: typst("$180$"), correct: false },
                QuestionOption { id: "B".into(), body: typst("$182$"), correct: true },
                QuestionOption { id: "C".into(), body: typst("$184$"), correct: false },
                QuestionOption { id: "D".into(), body: typst("$186$"), correct: false },
            ],
            shuffle: true,
            stats:   Default::default(),
        }),
        Question::TrueFalse(TrueFalseQuestion::new(
            QuestionId::new(),
            Points::new(25),
            vec![],
            typst("10 bitlik bir ADC toplam 1024 farklı değer üretebilir."),
            true,
        )),
        Question::Classic(ClassicQuestion {
            id:            QuestionId::new(),
            points:        Points::new(25),
            outcomes:      vec![],
            body:          typst(
                "Pin çıkış gerilimi $5 \"V\"$, LED ileri gerilimi $V_F = 2 \"V\"$ ve \
                 LED akımı $I = 10 \"mA\"$ ise seri direnç $R$ değerini hesaplayınız. \
                 İşlem adımlarını gösteriniz.",
            ),
            sample_answer: None,
            rubric:        vec![],
            answer_space:  AnswerSpace::Lines(5),
            stats:         Default::default(),
        }),
        Question::Classic(ClassicQuestion {
            id:            QuestionId::new(),
            points:        Points::new(25),
            outcomes:      vec![],
            body:          typst(
                "$f(x) = 2x + 3$ fonksiyonunun grafiğini çiziniz.\n\n\
                 #cevap-alani(satir: 10, bicim: \"kareli\")",
            ),
            sample_answer: None,
            rubric:        vec![],
            answer_space:  AnswerSpace::Lines(6),
            stats:         Default::default(),
        }),
    ]
}

fn render(columns: u8, repeat: usize, label: &str) {
    let exam = Exam::new(meta(columns));
    let qs: Vec<Question> = (0..repeat).flat_map(|_| questions()).collect();

    let src = match tayan_compiler::typst_gen::TypstGenerator::generate_exam(
        &exam,
        &qs,
        TypstContext::default(),
    ) {
        Ok(s) => s,
        Err(e) => {
            println!("{label}: ÜRETİM HATASI: {e}");
            return;
        }
    };

    std::fs::write(format!("{OUT_DIR}/exam-{columns}col-x{repeat}.typ"), &src).ok();

    match tayan_compiler::TayanWorld::compile_svg(src) {
        Ok(pages) => {
            for (i, p) in pages.iter().enumerate() {
                std::fs::write(format!("{OUT_DIR}/exam-{columns}col-x{repeat}-p{}.svg", i + 1), p).ok();
            }
            let bytes: usize = pages.iter().map(|p| p.len()).sum();
            println!("{label}: OK — {} sayfa, {} KB SVG", pages.len(), bytes / 1024);
        }
        Err(e) => println!("{label}: DERLEME HATASI: {e}"),
    }
}

fn main() {
    render(1, 1, "tek sütun  x1");
    render(2, 1, "çift sütun x1");
    render(2, 3, "çift sütun x3");
}
