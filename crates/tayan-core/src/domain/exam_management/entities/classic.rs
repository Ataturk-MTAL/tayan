use serde::{Deserialize, Serialize};
use crate::domain::exam_management::value_objects::{
    QuestionMeta,
    OutcomeCode, QuestionBody, QuestionStats,
};
use super::question::{Points, QuestionId};

/// Defines how much space to leave for the student's answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnswerSpace {
    Lines(u8),
    HeightCm(f32),
    Grid { rows: u8, cols: u8 },
}

impl Default for AnswerSpace {
    fn default() -> Self { AnswerSpace::Lines(6) }
}

/// A single scoring criterion for a classic (open-ended) question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricItem {
    pub criterion: String,
    pub points:    Points,
}

/// Classic / open-ended question. Scored manually by the teacher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicQuestion {
    /// Ders, sınıf seviyesi ve zorluk. Eski kayıtlarda yok; serde(default)
    /// ile boş gelir ve yeniden kaydedilirken doğrulamaya takılır.
    #[serde(default)]
    pub meta:     QuestionMeta,
    pub id:           QuestionId,
    pub points:       Points,
    pub outcomes:     Vec<OutcomeCode>,
    pub body:         QuestionBody,
    /// Optional sample answer visible only in the answer-key PDF.
    pub sample_answer: Option<QuestionBody>,
    pub rubric:       Vec<RubricItem>,
    pub answer_space: AnswerSpace,
    pub stats:        QuestionStats,
}

impl ClassicQuestion {
    pub fn rubric_total(&self) -> u32 {
        self.rubric.iter().map(|r| r.points.value()).sum()
    }

    pub fn validate(&self) -> Result<(), crate::domain::shared::errors::DomainError> {
        use crate::domain::shared::errors::DomainError;
        if self.body.is_empty() {
            return Err(DomainError::Validation(
                "Klasik soru gövdesi boş olamaz".into(),
            ));
        }
        // GÖVDEYE YAZILMIŞ RUBRİK KAYDETMEYİ DURDURUR.
        //
        // Öğretmen gövdeye `#rubrik((...))` yazıp panele taşımazsa şu olurdu:
        // cevap anahtarında tablo YOK (önsözdeki güvenlik ağı `goster`
        // verilmediği için hiçbir şey basmaz), sonuç girişinde kutucuk YOK
        // (kutucuklar `rubric` alanından geliyor) ve toplam doğrulaması hiç
        // çalışmamış olurdu. Öğretmen ölçütleri yazdığını sanır, hiçbiri
        // işlemez. Sessiz kaybetmektense burada durdurmak gerekiyor.
        // Örnek cevap da denetlenir. Öğretmen eski cevap anahtarı dosyasını
        // büyük ihtimalle ÖRNEK CEVAP alanına yapıştırır ve o dosyalarda
        // ölçütler `#rubrik(...)` ile yazılı. Yalnız gövdeye bakmak, aynı
        // sessiz kaybı öbür kapıdan içeri alırdı.
        let ornek_cevapta = self
            .sample_answer
            .as_ref()
            .is_some_and(|sa| sa.raw_source().contains("#rubrik("));

        if self.body.raw_source().contains("#rubrik(") || ornek_cevapta {
            return Err(DomainError::Validation(
                "Soru gövdesinde ya da örnek cevapta #rubrik( var. Ölçütler \
                 kaynağa değil, panelin Puanlama ölçütleri bölümüne girilir; \
                 oradan girilmeyen rubrik cevap anahtarına da sonuç girişine de \
                 yansımaz."
                    .into(),
            ));
        }

        if !self.rubric.is_empty() && self.rubric_total() != self.points.value() {
            return Err(DomainError::Validation(format!(
                "Rubrik toplam puanı ({}) soru puanıyla ({}) eşleşmiyor",
                self.rubric_total(),
                self.points.value()
            )));
        }
        Ok(())
    }
}
