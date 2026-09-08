//! Analiz ekranını ÖLÇEKTE denemek için veri üretir.
//!
//! Koşum: cargo run -p tayan-db --example seed_analysis
//!
//! Ne üretir:
//!   • 20 soruluk AYRI bir sınav (16 çoktan seçmeli + 2 doğru-yanlış + 2 klasik,
//!     toplam 100 puan) — mevcut "1. Dönem 1. Yazılı"ya DOKUNMAZ.
//!   • Dört sınıf: 34, 8, 5 ve 220 öğrenci.
//!   • Her öğrenci için sonuç.
//!
//! ── NEDEN DÜZ RASTGELE DEĞİL ─────────────────────────────────────────────────
//!
//! `rand() * 100` ile puan üretmek analiz ekranını DENEYEMEZ. İki sebep:
//!
//!   1. Düz rastgele puan DÜZGÜN (uniform) dağılır. Hiçbir gerçek sınav öyle
//!      görünmez; dağılım panelini gerçekte karşılaşılmayacak bir şekle bakarak
//!      yargılamış oluruz.
//!
//!   2. Daha kötüsü: cevaplar bağımsız rastgele olursa her sorunun AYIRT
//!      EDİCİLİĞİ sıfıra yakınsar — iyi öğrenci ile zayıf öğrenci aynı
//!      olasılıkla doğru yapar. Madde analizinin bütün anlamı bu farktır.
//!      Güçlük × ayırt edicilik grafiğinde bütün noktalar D≈0 çizgisine dizilir
//!      ve grafiğin işe yarayıp yaramadığını ANLAYAMAYIZ.
//!
//! Bu yüzden cevaplar iki parametreli madde tepki modelinden (2PL) üretiliyor:
//!
//!     P(doğru | öğrenci, soru) = 1 / (1 + e^(−a · (θ − b)))
//!
//!   θ (teta) = öğrencinin yeteneği, N(0, 1)
//!   b        = sorunun güçlüğü — büyükse soru zor
//!   a        = sorunun ayırt ediciliği — büyükse iyi öğrenciyle zayıfı ayırır
//!
//! Rastgelelik duruyor; içine YAPI kondu. Dağılım çan benzeri çıkıyor, ayırt
//! edicilik kendiliğinden pozitif oluyor.
//!
//! ── BİLEREK BOZULMUŞ MADDELER ────────────────────────────────────────────────
//!
//! Grafiklerin bir şeyi GÖSTERİP göstermediğini, ancak gösterecek bir şey varsa
//! anlarız. Dört patolojik madde bilerek serpiştirildi:
//!
//!   • a ≈ 0       → hiç ayırmıyor; herkes aynı olasılıkla doğru yapıyor.
//!   • a < 0       → TERS ayırıyor: iyi öğrenci yanlış, zayıf öğrenci doğru
//!                   yapıyor. Klasik "cevap anahtarı yanlış" işareti ve bir
//!                   madde analizi ekranının yakalayabileceği EN DEĞERLİ şey.
//!   • b çok küçük → herkes doğru yapıyor (tavan); soru kimseyi ayırmıyor.
//!   • b çok büyük → kimse yapamıyor (taban).
//!
//! Ayrıca bir KAZANIM bilerek zayıf: MAT.10.4.x maddelerinin hepsi zor. Kazanım
//! bazlı bir grafik varsa bunu göstermeli.
//!
//! ── ÇELDİRİCİ ────────────────────────────────────────────────────────────────
//!
//! Öğrenci yanlış yaptığında hangi şıkkı işaretlediği rastgele DEĞİL: her soruda
//! bir "çekici çeldirici" var ve yanlışların yarısı oraya gidiyor.
//! `QuestionAnswer.given_answer` seçilen şıkkı sakladığı için çeldirici analizi
//! bu veriyle mümkün.
//!
//! ── TEKRAR ÇALIŞTIRMAK GÜVENLİ ───────────────────────────────────────────────
//!
//! Sınıf adı ve sınav başlığı zaten varsa YENİSİ AÇILMIYOR, mevcut kullanılıyor.
//! `seed_demo` bunu yapmadığı için veritabanında üç tane aynı adlı "9-A"
//! birikmişti. Sonuçlar da (exam_id, student_id) üzerinde ON CONFLICT DO UPDATE
//! ile yazılıyor, yani çiftlenmiyor.

use chrono::NaiveDate;
use tayan_core::application::ports::{
    ClassroomRepository, ExamRepository, ExamResultRepository, QuestionBankRepository,
    StudentRepository,
};
use tayan_core::domain::assessment::aggregates::{ExamResult, QuestionAnswer};
use tayan_core::domain::assessment::services::{QuestionStatsUpdater, ScoringService};
use tayan_core::domain::exam_management::aggregates::{Exam, ExamMeta, ExamSigner};
use tayan_core::domain::exam_management::entities::classic::{
    AnswerSpace, ClassicQuestion, RubricItem,
};
use tayan_core::domain::exam_management::entities::multiple_choice::{
    MultipleChoiceQuestion, QuestionOption,
};
use tayan_core::domain::exam_management::entities::question::{Points, Question, QuestionId};
use tayan_core::domain::exam_management::entities::true_false::TrueFalseQuestion;
use tayan_core::domain::exam_management::value_objects::{
    ContentNode, Difficulty, OutcomeCode, QuestionBody, QuestionMeta,
};
use tayan_core::domain::student_management::aggregates::{Classroom, Student};

// ── Yeniden üretilebilir rastgelelik ─────────────────────────────────────────

/// xorshift64* — küçük, hızlı ve TOHUMLU.
///
/// `rand` bağımlılığı EKLENMEDİ: bu bir örnek dosyası ve tek ihtiyacı yeniden
/// üretilebilir sayı. Sabit tohum sayesinde herkes aynı veriyi görüyor;
/// "bende farklı çıkıyor" tartışması olmuyor.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// [0, 1) aralığında düzgün.
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Standart normal — Box-Muller.
    fn normal(&mut self) -> f64 {
        let u1 = self.unit().max(1e-12);
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// 2PL madde tepki eğrisi.
fn p_correct(theta: f64, a: f64, b: f64) -> f64 {
    1.0 / (1.0 + (-a * (theta - b)).exp())
}

// ── Madde parametreleri ──────────────────────────────────────────────────────

/// Bir maddenin üretim parametreleri.
struct Madde {
    /// Ayırt edicilik. Negatif = ters ayıran (bozuk) madde.
    a: f64,
    /// Güçlük. Büyük = zor.
    b: f64,
    kazanim: &'static str,
    /// Neden bu parametreler — çıktıda ne görmeyi beklediğimiz.
    not: &'static str,
}

/// Yirmi maddenin parametreleri.
///
/// SIRA ÖNEMLİ: 0-15 çoktan seçmeli, 16-17 doğru-yanlış, 18-19 klasik.
/// Aşağıdaki soru üretimi bu sırayı varsayıyor.
const MADDELER: [Madde; 20] = [
    Madde { a: 1.2, b: -1.0, kazanim: "MAT.10.1.1", not: "sağlıklı, kolay" },
    Madde { a: 1.5, b: -0.4, kazanim: "MAT.10.1.2", not: "sağlıklı" },
    Madde { a: 0.9, b: 0.2, kazanim: "MAT.10.1.3", not: "sağlıklı" },
    Madde { a: 1.1, b: -2.8, kazanim: "MAT.10.1.4", not: "TAVAN: neredeyse herkes doğru yapıyor" },
    Madde { a: 1.7, b: 0.6, kazanim: "MAT.10.2.1", not: "sağlıklı, iyi ayırıyor" },
    Madde { a: 1.3, b: -0.2, kazanim: "MAT.10.2.2", not: "sağlıklı" },
    Madde { a: 1.0, b: 1.1, kazanim: "MAT.10.2.3", not: "sağlıklı, zorca" },
    Madde { a: 0.05, b: 0.0, kazanim: "MAT.10.2.4", not: "AYIRMIYOR: a yaklaşık sıfır" },
    Madde { a: 1.4, b: -0.7, kazanim: "MAT.10.3.1", not: "sağlıklı" },
    Madde { a: 1.6, b: 0.3, kazanim: "MAT.10.3.2", not: "sağlıklı, iyi ayırıyor" },
    Madde { a: 0.8, b: 0.9, kazanim: "MAT.10.3.3", not: "sağlıklı" },
    Madde { a: 1.2, b: -0.5, kazanim: "MAT.10.3.4", not: "sağlıklı" },
    Madde { a: -0.9, b: 0.1, kazanim: "MAT.10.3.5", not: "TERS AYIRIYOR: anahtar şüpheli" },
    Madde { a: 1.5, b: 0.4, kazanim: "MAT.10.4.1", not: "zayıf kazanım" },
    Madde { a: 1.3, b: 1.4, kazanim: "MAT.10.4.2", not: "zayıf kazanım, zor" },
    Madde { a: 1.1, b: 1.6, kazanim: "MAT.10.4.3", not: "zayıf kazanım, zor" },
    Madde { a: 1.4, b: -0.9, kazanim: "MAT.10.5.1", not: "doğru-yanlış, sağlıklı" },
    Madde { a: 1.0, b: 0.5, kazanim: "MAT.10.5.2", not: "doğru-yanlış" },
    Madde { a: 1.2, b: 2.8, kazanim: "MAT.10.4.4", not: "TABAN: kimse yapamıyor (klasik)" },
    Madde { a: 1.3, b: 0.0, kazanim: "MAT.10.6.1", not: "klasik, rubrikli" },
];

/// Sorunun puanı — sıraya bağlı. `MADDELER` sırasıyla birlikte okunmalı.
fn puan(index: usize) -> u32 {
    if index < 16 {
        4
    } else if index < 18 {
        3
    } else {
        15
    }
}

// ── Yardımcılar ──────────────────────────────────────────────────────────────

fn govde(src: &str) -> QuestionBody {
    QuestionBody(vec![ContentNode::typst_raw(src)])
}

fn kunye(zorluk: Difficulty) -> QuestionMeta {
    QuestionMeta::new("Matematik", 10, Some(zorluk))
}

fn kazanim(kod: &str) -> Vec<OutcomeCode> {
    vec![OutcomeCode::new(kod).expect("geçerli kazanım kodu")]
}

/// `b` değerinden okunabilir zorluk etiketi. Yalnız künye için; hesaba girmez.
fn zorluk_etiketi(b: f64) -> Difficulty {
    if b < -0.7 {
        Difficulty::Kolay
    } else if b < 0.8 {
        Difficulty::Orta
    } else {
        Difficulty::Zor
    }
}

const ADLAR: [&str; 24] = [
    "Ayşe", "Berk", "Ceren", "Deniz", "Emre", "Fatma", "Gökhan", "Hale", "İsmail", "Jale",
    "Kerem", "Leyla", "Mert", "Nazlı", "Onur", "Pelin", "Rüya", "Serkan", "Tuğçe", "Ufuk",
    "Vildan", "Yasin", "Zeynep", "Çağla",
];

const SOYADLAR: [&str; 25] = [
    "YILMAZ", "DEMİR", "KAYA", "ŞAHİN", "ÇELİK", "ÖZTÜRK", "ARSLAN", "DOĞAN", "KILIÇ",
    "ASLAN", "ÇETİN", "KURT", "ÖZDEMİR", "ŞİMŞEK", "POLAT", "KOÇ", "GÜNEŞ", "AYDIN",
    "ERDOĞAN", "KARA", "TAŞ", "BULUT", "GÜL", "ÜNAL", "AKSU",
];

/// (ad, şube, öğrenci sayısı, ne sınıyor)
const SINIFLAR: [(&str, &str, usize, &str); 4] = [
    ("10-A", "A", 34, "tipik sınıf — asıl hedef"),
    ("10-B", "B", 8, "küçük grup, ayırt edicilik eşiğinin altı"),
    ("10-C", "C", 5, "en küçük — hiçbir dağılım iddiası tutmuyor"),
    ("10-D", "D", 220, "kademe geneli benzetimi"),
];

const SINAV_BASLIGI: &str = "Deneme Analizi — 20 Soru";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base = dirs_next::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("tayan");
    std::fs::create_dir_all(&base)?;
    let url = format!("sqlite:{}", base.join("tayan_dev.db").display());
    println!("veritabanı: {url}\n");

    let pool = tayan_db::connect(&url).await?;
    tayan_db::run_migrations(&pool).await?;

    let bank_repo = tayan_db::repositories::SqliteQuestionBankRepository { pool: pool.clone() };
    let exam_repo = tayan_db::repositories::SqliteExamRepository { pool: pool.clone() };
    let class_repo = tayan_db::repositories::SqliteClassroomRepository { pool: pool.clone() };
    let student_repo = tayan_db::repositories::SqliteStudentRepository { pool: pool.clone() };
    let result_repo = tayan_db::repositories::SqliteExamResultRepository { pool: pool.clone() };

    // ── Sorular ──────────────────────────────────────────────────────────────
    let mut sorular: Vec<Question> = Vec::with_capacity(20);

    // 0-15: çoktan seçmeli. Doğru şık madde sırasına göre dönüyor ki hepsi "A"
    // olmasın — çeldirici tablosuna bakarken bu fark ediliyor.
    for (i, m) in MADDELER.iter().enumerate().take(16) {
        let dogru_index = i % 4;
        let options: Vec<QuestionOption> = (0..4)
            .map(|k| QuestionOption {
                id: ["A", "B", "C", "D"][k].into(),
                body: govde(&format!("${}$ seçeneği", k + 1)),
                correct: k == dogru_index,
            })
            .collect();

        sorular.push(Question::MultipleChoice(MultipleChoiceQuestion {
            id: QuestionId::new(),
            points: Points::new(puan(i)),
            outcomes: kazanim(m.kazanim),
            meta: kunye(zorluk_etiketi(m.b)),
            body: govde(&format!(
                "Benzetim sorusu {}. ({}) Gövde yalnız analiz ekranını denemek \
                 için üretilmiştir.",
                i + 1,
                m.not
            )),
            options,
            shuffle: false,
            stats: Default::default(),
        }));
    }

    // 16-17: doğru-yanlış.
    for (i, m) in MADDELER.iter().enumerate().skip(16).take(2) {
        sorular.push(Question::TrueFalse(TrueFalseQuestion::new(
            QuestionId::new(),
            Points::new(puan(i)),
            kazanim(m.kazanim),
            govde(&format!(
                "Benzetim önermesi {}. ({}) Bu önerme denemek içindir.",
                i + 1,
                m.not
            )),
            i % 2 == 0,
            kunye(zorluk_etiketi(m.b)),
        )));
    }

    // 18-19: klasik, rubrikli. Rubrik toplamı soru puanına EŞİT olmak zorunda
    // (ClassicQuestion::validate); 6 + 5 + 4 = 15.
    for (i, m) in MADDELER.iter().enumerate().skip(18).take(2) {
        sorular.push(Question::Classic(ClassicQuestion {
            id: QuestionId::new(),
            points: Points::new(puan(i)),
            outcomes: kazanim(m.kazanim),
            meta: kunye(zorluk_etiketi(m.b))
                .with_title(format!("Benzetim Klasik Soru {}", i - 17)),
            body: govde(&format!(
                "Benzetim sorusu {}. ({}) Çözümü adım adım yazınız.\n\n\
                 #cevap-alani(satir: 8, bicim: \"kareli\")",
                i + 1,
                m.not
            )),
            sample_answer: None,
            rubric: vec![
                RubricItem { criterion: "Yöntem doğru seçilmiş".into(), points: Points::new(6) },
                RubricItem { criterion: "İşlem adımları doğru".into(), points: Points::new(5) },
                RubricItem { criterion: "Sonuç ve birim doğru".into(), points: Points::new(4) },
            ],
            answer_space: AnswerSpace::Lines(8),
            stats: Default::default(),
        }));
    }

    // Bankaya ekle — gövde eşleşiyorsa mevcut kimlik korunur (seed_demo deseni).
    let mut bank = bank_repo.load().await?;
    let mut ids: Vec<QuestionId> = Vec::with_capacity(20);
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
    println!("soru      : {yeni} yeni, {} atıf", ids.len());

    // ── Sınav ────────────────────────────────────────────────────────────────
    //
    // BAŞLIĞA GÖRE TEKİL: aynı adlı sınav varsa yenisi açılmıyor, yoksa her
    // koşuda analiz ekranındaki sınav listesi kopyalarla dolardı.
    let mevcut_sinavlar = exam_repo.list(0, 500).await?;
    let sinav = match mevcut_sinavlar
        .into_iter()
        .find(|e| e.meta.title == SINAV_BASLIGI)
    {
        Some(e) => {
            println!("sınav     : zaten var, yeniden kullanılıyor");
            e
        }
        None => {
            let mut e = Exam::new(ExamMeta {
                title: SINAV_BASLIGI.into(),
                subject: "Matematik".into(),
                classroom: "10. sınıflar".into(),
                teacher: "Hakan GÜLEN".into(),
                duration_min: 60,
                date: NaiveDate::from_ymd_opt(2026, 12, 10).unwrap(),
                instructions: None,
                columns: 1,
                school: Some("Atatürk Mesleki ve Teknik Anadolu Lisesi".into()),
                department: Some("Elektrik-Elektronik Teknolojisi Alanı".into()),
                signers: vec![ExamSigner {
                    name: "Hakan GÜLEN".into(),
                    title: "Ders Öğretmeni".into(),
                }],
            });
            for id in &ids {
                e.add_question_ref(id.clone());
            }
            e.publish()?;
            exam_repo.save(&e).await?;
            println!("sınav     : oluşturuldu ({} soru)", e.questions.len());
            e
        }
    };

    let toplam_puan: f32 = (0..20).map(|i| puan(i) as f32).sum();

    // ── Sınıflar, öğrenciler, sonuçlar ───────────────────────────────────────
    let mevcut_siniflar = class_repo.list().await?;
    let mut rng = Rng::new(0x5EED_A11A_1751);
    let mut tum_sonuclar: Vec<ExamResult> = Vec::new();

    for (ad, sube, kisi, aciklama) in SINIFLAR {
        // ADA GÖRE TEKİL — üç tane "9-A" birikmesinin sebebi bu kontrolün
        // seed_demo'da olmamasıydı.
        let sinif = match mevcut_siniflar.iter().find(|c| c.name == ad) {
            Some(c) => {
                println!("\nsınıf {ad} : zaten var, yeniden kullanılıyor ({aciklama})");
                class_repo.find_by_id(&c.id).await?
            }
            None => {
                let c = Classroom::new(ad, 10, sube);
                class_repo.save(&c).await?;
                println!("\nsınıf {ad} : oluşturuldu, hedef {kisi} öğrenci ({aciklama})");
                c
            }
        };

        // Zaten yeterli öğrencisi varsa yenilerini EKLEMİYORUZ; mevcutları
        // kullanıp yalnız sonuçları tazeliyoruz.
        let mevcut_ogrenciler = student_repo.list_by_classroom(&sinif.id).await?;
        let ogrenciler: Vec<Student> = if mevcut_ogrenciler.len() >= kisi {
            println!("  öğrenci : {} zaten kayıtlı, yeniden kullanılıyor", mevcut_ogrenciler.len());
            mevcut_ogrenciler
        } else {
            let mut v = Vec::with_capacity(kisi);
            for i in 0..kisi {
                let s = Student::new(
                    format!("{}{:03}", sube, i + 1),
                    ADLAR[i % ADLAR.len()],
                    SOYADLAR[(i * 7 + 3) % SOYADLAR.len()],
                    sinif.id.clone(),
                );
                student_repo.save_student(&s).await?;
                v.push(s);
            }
            println!("  öğrenci : {} eklendi", v.len());
            v
        };

        for ogrenci in &ogrenciler {
            let theta = rng.normal();
            let mut cevaplar: Vec<QuestionAnswer> = Vec::with_capacity(20);

            for (i, m) in MADDELER.iter().enumerate() {
                let qid = ids[i].clone();
                let dogru = rng.unit() < p_correct(theta, m.a, m.b);

                if i < 16 {
                    // Çoktan seçmeli. Yanlışsa hangi şık?
                    //
                    // ÇEKİCİ ÇELDİRİCİ: yanlışların yarısı tek bir şıkka
                    // gidiyor. Düz rastgele dağıtsaydık çeldirici analizi "üç
                    // şık da eşit" derdi ve o tabloyu denemek imkânsız olurdu.
                    let dogru_index = i % 4;
                    let secilen = if dogru {
                        dogru_index
                    } else {
                        let yanlislar: Vec<usize> =
                            (0..4).filter(|k| *k != dogru_index).collect();
                        if rng.unit() < 0.5 {
                            yanlislar[i % 3]
                        } else {
                            yanlislar[(rng.next_u64() % 3) as usize]
                        }
                    };
                    cevaplar.push(QuestionAnswer {
                        question_id: qid,
                        given_answer: Some(["A", "B", "C", "D"][secilen].to_string()),
                        points_earned: 0.0, // ScoringService dolduracak
                        is_correct: None,
                        rubric_met: vec![],
                    });
                } else if i < 18 {
                    // Doğru-yanlış. Sorunun cevabı yukarıda (i % 2 == 0) idi.
                    let sorunun_cevabi = i % 2 == 0;
                    let verilen = if dogru { sorunun_cevabi } else { !sorunun_cevabi };
                    cevaplar.push(QuestionAnswer {
                        question_id: qid,
                        given_answer: Some(verilen.to_string()),
                        points_earned: 0.0,
                        is_correct: None,
                        rubric_met: vec![],
                    });
                } else {
                    // Klasik: ScoringService puanı HESAPLAMAZ, verileni korur.
                    // Rubrik ölçütleri yetenekten türetiliyor, puan onların
                    // toplamı — arayüzün yaptığının aynısı.
                    let olcut_puanlari = [6.0f32, 5.0, 4.0];
                    let mut karsilanan: Vec<u16> = Vec::new();
                    let mut kazanilan = 0.0f32;
                    for (k, deger) in olcut_puanlari.iter().enumerate() {
                        // Sonraki ölçüt bir öncekinden biraz daha zor.
                        if rng.unit() < p_correct(theta, m.a, m.b + k as f64 * 0.4) {
                            karsilanan.push(k as u16);
                            kazanilan += deger;
                        }
                    }
                    cevaplar.push(QuestionAnswer {
                        question_id: qid,
                        given_answer: None,
                        points_earned: kazanilan,
                        is_correct: None,
                        rubric_met: karsilanan,
                    });
                }
            }

            let mut sonuc =
                ExamResult::new(sinav.id.clone(), ogrenci.id.clone(), toplam_puan);
            ScoringService::auto_score(&mut sonuc, cevaplar, &bank);
            sonuc.is_complete = true;
            result_repo.save(&sonuc).await?;
            tum_sonuclar.push(sonuc);
        }

        println!("  sonuç   : {} öğrenci puanlandı", ogrenciler.len());
    }

    // ── Madde istatistikleri ─────────────────────────────────────────────────
    //
    // Arayüz her sonuç kaydından sonra bunu çağırıyor (commands/analysis.rs).
    // Burada bir kez, hepsi yazıldıktan sonra — aksi hâlde banka istatistikleri
    // boş kalır ve soru listesi "Denenmemiş" gösterir.
    QuestionStatsUpdater::update_from_results(&mut bank, &tum_sonuclar);
    bank_repo.save(&bank).await?;

    println!("\n─────────────────────────────────────────────────────────────");
    println!("Toplam {} sonuç yazıldı.", tum_sonuclar.len());
    println!("Analiz ekranında sınav olarak \"{SINAV_BASLIGI}\" seç.\n");
    println!("Grafiklerin GÖSTERMESİ gereken bulgular:");
    for (i, m) in MADDELER.iter().enumerate() {
        if m.not.contains("TERS")
            || m.not.contains("AYIRMIYOR")
            || m.not.contains("TAVAN")
            || m.not.contains("TABAN")
        {
            println!("  • Soru {:2}: {}", i + 1, m.not);
        }
    }
    println!("  • MAT.10.4.x kazanımı bilerek zayıf (4 madde, hepsi zor).");
    println!("  • 10-A (34) ile 10-D (220): aynı sınav, farklı ölçek.");

    Ok(())
}
