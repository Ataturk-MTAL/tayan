use std::sync::Arc;
use sqlx::SqlitePool;
use tayan_db::{
    SqliteClassroomRepository, SqliteExamRepository,
    SqliteExamResultRepository, SqliteQuestionBankRepository,
    SqliteStudentRepository,
};

pub struct AppState {
    pub exams:    Arc<SqliteExamRepository>,
    pub bank:     Arc<SqliteQuestionBankRepository>,
    pub students: Arc<SqliteStudentRepository>,
    pub classes:  Arc<SqliteClassroomRepository>,
    pub results:  Arc<SqliteExamResultRepository>,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            exams:    Arc::new(SqliteExamRepository            { pool: pool.clone() }),
            bank:     Arc::new(SqliteQuestionBankRepository    { pool: pool.clone() }),
            students: Arc::new(SqliteStudentRepository         { pool: pool.clone() }),
            classes:  Arc::new(SqliteClassroomRepository       { pool: pool.clone() }),
            results:  Arc::new(SqliteExamResultRepository      { pool: pool.clone() }),
        }
    }
}
