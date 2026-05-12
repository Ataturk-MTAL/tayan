use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Entity not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("Invariant violated: {0}")]
    InvariantViolation(String),

    #[error("Operation not permitted: {0}")]
    NotPermitted(String),
}
