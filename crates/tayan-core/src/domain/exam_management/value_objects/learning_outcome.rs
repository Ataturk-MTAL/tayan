use serde::{Deserialize, Serialize};
use std::fmt;
use crate::domain::shared::errors::DomainError;

/// MEB kazanım kodu — örn. "M.7.2.3", "P.9.1.2", "C.10.3.1"
/// Format: {subject_code}.{grade}.{unit}.{outcome}
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
                "Invalid outcome code `{code}`. Expected format: SUBJECT.GRADE.UNIT.OUTCOME (e.g. M.7.2.3)"
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

    fn is_valid(code: &str) -> bool {
        let parts: Vec<&str> = code.split('.').collect();
        parts.len() == 4
            && parts[0].len() == 1
            && parts[0].chars().all(|c| c.is_ascii_alphabetic())
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
