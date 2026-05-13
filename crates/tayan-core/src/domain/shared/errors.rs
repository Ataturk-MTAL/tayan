use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Doğrulama hatası: {0}")]
    Validation(String),

    #[error("{entity} bulunamadı (id: {id})")]
    NotFound { entity: &'static str, id: String },

    #[error("Kısıtlama ihlali: {0}")]
    InvariantViolation(String),

    #[error("İşlem izni yok: {0}")]
    NotPermitted(String),
}
