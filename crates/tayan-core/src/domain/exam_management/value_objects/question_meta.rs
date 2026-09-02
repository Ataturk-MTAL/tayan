use serde::{Deserialize, Serialize};

use crate::domain::shared::errors::DomainError;

/// Sorunun zorluk derecesi — öğretmenin KANAATİ.
///
/// `QuestionStats.difficulty_index` ise ÖLÇÜLEN güçlüktür; sınav sonuçlarından
/// hesaplanır. İkisi bilerek ayrı tutuluyor: öğretmenin "zor" dediği bir sorunun
/// aslında kolay olduğunu ölçüm gelince görmek değerli bilgidir. Birleştirmek o
/// farkı yok eder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Kolay,
    Orta,
    Zor,
}

impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Kolay => "Kolay",
            Difficulty::Orta => "Orta",
            Difficulty::Zor => "Zor",
        }
    }
}

/// MEB örgün öğretimde sınıf seviyesi aralığı.
pub const MIN_GRADE: u8 = 1;
pub const MAX_GRADE: u8 = 12;

/// Sorunun künyesi: hangi derse, hangi sınıf seviyesine ait, ne kadar zor.
///
/// Ders ve sınıf seviyesi ZORUNLUDUR. Sebebi ileriye dönük: kazanım kodları
/// (MAT.9.1.2) derse ve seviyeye bağlıdır; ikisi bilinmeden hangi kazanım
/// listesinin gösterileceği belirlenemez.
///
/// Zorluk isteğe bağlı. Öğretmen soruyu yazarken zorluğuna henüz karar vermemiş
/// olabilir ve ölçüm geldiğinde zaten gerçeği görecektir.
///
/// GERİYE DÖNÜK UYUM: bu alan eski kayıtlarda yok. `serde(default)` ile
/// `subject: ""`, `grade: 0` okunur — eski sorular YÜKLENİR, listelenir, basılır.
/// Yalnız yeniden KAYDEDİLİRKEN doğrulamaya takılırlar ve öğretmen o an alanları
/// doldurur. Eski veriyi uydurma değerlerle sessizce doldurmak, yanlış kazanım
/// eşleşmesi üretmekten daha kötü olurdu.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestionMeta {
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub grade: u8,
    #[serde(default)]
    pub difficulty: Option<Difficulty>,
}

impl QuestionMeta {
    pub fn new(subject: impl Into<String>, grade: u8, difficulty: Option<Difficulty>) -> Self {
        Self { subject: subject.into(), grade, difficulty }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.subject.trim().is_empty() {
            return Err(DomainError::Validation("Ders alanı boş olamaz.".into()));
        }

        if !(MIN_GRADE..=MAX_GRADE).contains(&self.grade) {
            return Err(DomainError::Validation(format!(
                "Sınıf seviyesi {MIN_GRADE} ile {MAX_GRADE} arasında olmalı (girilen: {}).",
                self.grade
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dolu_kunye_gecerli() {
        assert!(QuestionMeta::new("Matematik", 9, Some(Difficulty::Orta)).validate().is_ok());
    }

    #[test]
    fn zorluk_istege_bagli() {
        assert!(QuestionMeta::new("Fizik", 11, None).validate().is_ok());
    }

    #[test]
    fn bos_ders_reddedilir() {
        assert!(QuestionMeta::new("   ", 9, None).validate().is_err());
    }

    #[test]
    fn sifir_sinif_reddedilir() {
        // Eski kayıtların serde(default) ile aldığı değer tam olarak budur.
        assert!(QuestionMeta::new("Matematik", 0, None).validate().is_err());
    }

    #[test]
    fn on_ucuncu_sinif_reddedilir() {
        assert!(QuestionMeta::new("Matematik", 13, None).validate().is_err());
    }

    #[test]
    fn sinir_degerler_gecerli() {
        assert!(QuestionMeta::new("Hayat Bilgisi", MIN_GRADE, None).validate().is_ok());
        assert!(QuestionMeta::new("Matematik", MAX_GRADE, None).validate().is_ok());
    }

    #[test]
    fn alani_olmayan_json_varsayilana_duser_ve_dogrulamaya_takilir() {
        let m: QuestionMeta = serde_json::from_str("{}").unwrap();
        assert_eq!(m.subject, "");
        assert_eq!(m.grade, 0);
        assert_eq!(m.difficulty, None);
        assert!(m.validate().is_err());
    }

    #[test]
    fn zorluk_json_karsiligi() {
        let m = QuestionMeta::new("Kimya", 10, Some(Difficulty::Zor));
        let s = serde_json::to_string(&m).unwrap();
        assert!(s.contains("\"zor\""), "beklenen snake_case değer: {s}");
        let geri: QuestionMeta = serde_json::from_str(&s).unwrap();
        assert_eq!(geri, m);
    }
}
