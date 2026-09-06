use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::exam_management::{
    aggregates::ExamId,
    entities::QuestionId,
    value_objects::OutcomeCode,
};
use crate::domain::student_management::StudentId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExamResultId(pub Uuid);
impl Default for ExamResultId {
    fn default() -> Self {
        Self::new()
    }
}

impl ExamResultId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

/// How a student answered a single question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnswer {
    pub question_id:       QuestionId,
    /// For MC/TF: the chosen option id ("A"/"B" or "true"/"false").
    /// For FillInBlank: JSON map of blank_id → given_value.
    /// For Classic: None (score is entered directly).
    pub given_answer:      Option<String>,
    pub points_earned:     f32,
    pub is_correct:        Option<bool>,  // None for Classic (manual)
    /// Klasik soruda karşılanan rubrik ölçütlerinin sırası (0'dan başlar).
    ///
    /// KANIT, KAYNAK DEĞİL. Puanın kendisi `points_earned` alanında durur ve
    /// giriş anında donar. Öğretmen sonradan rubriği düzenlerse verilmiş
    /// notlar DEĞİŞMEZ — yalnız bu kırılım eski rubriğe işaret ediyor olabilir.
    /// Notun kendiliğinden kayması, kırılımın eskimesinden çok daha kötüdür.
    ///
    /// Boş: rubrik yok, ya da puan doğrudan elle girilmiş.
    #[serde(default)]
    pub rubric_met:        Vec<u16>,
}

/// Outcome-level performance for a single student on a single exam.
///
/// PUAN DA SAKLANIYOR, YALNIZ SAYI DEĞİL.
///
/// `score_pct` eskiden `correct / total_questions` idi ve bu SESSİZCE YANLIŞ
/// sonuç veriyordu: `is_correct` klasik soruda her zaman `None` (puan elle
/// giriliyor, "doğru/yanlış" diye bir şey yok) ve boşluk doldurmada kısmi
/// doğru `false` sayılıyor. Yani açık uçlu bir sorunun kazanımı, sınıf o
/// sorudan tam puan alsa bile %0 görünüyordu.
///
/// `points_earned` / `points_available` AYRICA saklanıyor, çünkü SINIF
/// düzeyinde toplama ancak böyle doğru yapılabiliyor: öğrenci yüzdelerinin
/// ortalaması, sorular farklı puanlardaysa yanlış sonuç verir. Puanlar
/// toplanıp bölününce sonuç kesin.
///
/// `correct` ve `total_questions` KORUNUYOR: karne / öğrenci bazlı raporda
/// "8 sorunun 5'i doğru" ifadesi puandan farklı ve gerekli bir bilgi.
/// `correct` yalnız otomatik puanlanan sorularda TAM doğruyu sayar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomePerformance {
    pub outcome:         OutcomeCode,
    pub total_questions: u32,
    pub correct:         u32,
    /// Bu kazanımın sorularından alınan toplam puan.
    ///
    /// `#[serde(default)]`: alan sonradan eklendi ve veritabanındaki eski
    /// sonuçlarda YOK. Varsayılansız olsaydı eski kayıtların tamamı okunamaz
    /// hâle gelir, öğretmenin girdiği bütün sonuçlar ekrandan kaybolurdu.
    /// Eski satırlar 0.0 ile geliyor; sonuç yeniden kaydedildiğinde doğru
    /// değerle doluyor.
    #[serde(default)]
    pub points_earned:    f32,
    /// Bu kazanımın sorularından alınabilecek toplam puan.
    #[serde(default)]
    pub points_available: f32,
    pub score_pct:       f32,
}

/// One student's full result for one exam.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExamResult {
    pub id:                  ExamResultId,
    pub exam_id:             ExamId,
    pub student_id:          StudentId,
    pub answers:             Vec<QuestionAnswer>,
    pub total_points_earned: f32,
    pub total_points_max:    f32,
    pub outcome_performance: Vec<OutcomePerformance>,
    /// Rank within the classroom (set after all results are entered).
    pub classroom_rank:      Option<u32>,
    pub recorded_at:         DateTime<Utc>,
    pub is_complete:         bool,
}

impl ExamResult {
    pub fn new(exam_id: ExamId, student_id: StudentId, total_max: f32) -> Self {
        Self {
            id:                  ExamResultId::new(),
            exam_id,
            student_id,
            answers:             vec![],
            total_points_earned: 0.0,
            total_points_max:    total_max,
            outcome_performance: vec![],
            classroom_rank:      None,
            recorded_at:         Utc::now(),
            is_complete:         false,
        }
    }

    pub fn score_percentage(&self) -> f32 {
        if self.total_points_max == 0.0 { return 0.0; }
        (self.total_points_earned / self.total_points_max * 100.0).clamp(0.0, 100.0)
    }
}
