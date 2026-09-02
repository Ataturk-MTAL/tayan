use serde::{Deserialize, Serialize};
use std::fmt;
use crate::domain::shared::errors::DomainError;

/// MEB kazanım kodu — örn. "MAT.9.1.2", "FİZ.10.1.1", "T.9.1.1"
/// Biçim: {ders_kodu}.{sınıf}.{ünite}.{kazanım}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OutcomeCode(String);

impl OutcomeCode {
    pub fn new(code: impl Into<String>) -> Result<Self, DomainError> {
        let code = code.into();
        if Self::is_valid(&code) {
            Ok(Self(code))
        } else {
            Err(DomainError::Validation(format!(
                "Geçersiz kazanım kodu '{code}'. Beklenen format: KONU.SINIF.ÜNİTE.KAZANIM (örn. M.7.2.3)"
            )))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn subject_code(&self) -> &str {
        self.0.split('.').next().unwrap_or("")
    }

    pub fn grade(&self) -> Option<u8> {
        self.0.split('.').nth(1)?.parse().ok()
    }

    /// Ders kodu 1-5 HARF, kalan üç bölüm sayı.
    ///
    /// Önceden tek harf şart koşuluyordu (`parts[0].len() == 1`). Bu, arayüzün
    /// her yerde örnek olarak gösterdiği MAT.9.1.2 kodunun REDDEDİLMESİ
    /// demekti: yer tutucu, ipucu ve yardım sayfası "MAT.9.1.2 yaz" diyor,
    /// doğrulayıcı ise "Geçersiz kazanım kodu" döndürüyordu.
    ///
    /// `len()` yerine `chars().count()`: "FİZ" UTF-8'de 4 bayt ama 3 harftir.
    /// Bayt saymak Türkçe ders kodlarını sessizce dışarıda bırakırdı.
    ///
    /// `is_ascii_alphabetic` yerine `is_alphabetic`: İ, Ö, Ü, Ş, Ğ, Ç geçerli
    /// harflerdir ve MEB kodlarında geçerler (FİZ, COĞ, TÜR).
    fn is_valid(code: &str) -> bool {
        let parts: Vec<&str> = code.split('.').collect();
        if parts.len() != 4 {
            return false;
        }

        let ders = parts[0];
        let harf_sayisi = ders.chars().count();

        (1..=5).contains(&harf_sayisi)
            && ders.chars().all(|c| c.is_alphabetic())
            && parts[1..].iter().all(|p| p.parse::<u8>().is_ok())
    }
}

impl fmt::Display for OutcomeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Full learning outcome — code + human-readable description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningOutcome {
    pub code:        OutcomeCode,
    pub description: String,
    pub subject:     String,
    pub grade:       u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arayuzun_onerdigi_kod_kabul_edilir() {
        // Yer tutucu, ipucu ve yardım sayfası bu kodu örnek gösteriyor.
        // Önceden REDDEDİLİYORDU; regresyon olmasın diye burada kilitli.
        assert!(OutcomeCode::new("MAT.9.1.2").is_ok());
    }

    #[test]
    fn turkce_harfli_ders_kodu_kabul_edilir() {
        assert!(OutcomeCode::new("FİZ.10.1.1").is_ok());
        assert!(OutcomeCode::new("COĞ.11.2.3").is_ok());
    }

    #[test]
    fn tek_harfli_eski_bicim_hala_gecerli() {
        // Kayıtlı veride bu biçim var; geriye dönük kırılma olmamalı.
        assert!(OutcomeCode::new("M.7.2.3").is_ok());
    }

    #[test]
    fn bicimsiz_kodlar_reddedilir() {
        assert!(OutcomeCode::new("MAT.9.1").is_err(), "üç bölüm yetmez");
        assert!(OutcomeCode::new("MAT.9.1.2.3").is_err(), "beş bölüm fazla");
        assert!(OutcomeCode::new("9.9.1.2").is_err(), "ders kodu harf olmalı");
        assert!(OutcomeCode::new("MATEMA.9.1.2").is_err(), "ders kodu 5 harfi geçemez");
        assert!(OutcomeCode::new("MAT.a.1.2").is_err(), "sınıf sayı olmalı");
        assert!(OutcomeCode::new("").is_err());
    }

    #[test]
    fn parcalar_okunabiliyor() {
        let c = OutcomeCode::new("MAT.9.1.2").unwrap();
        assert_eq!(c.subject_code(), "MAT");
        assert_eq!(c.grade(), Some(9));
        assert_eq!(c.as_str(), "MAT.9.1.2");
    }
}
