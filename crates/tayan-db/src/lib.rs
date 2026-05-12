pub mod repositories;
pub mod migrations;

pub use repositories::{
    SqliteClassroomRepository,
    SqliteExamRepository,
    SqliteExamResultRepository,
    SqliteQuestionBankRepository,
    SqliteStudentRepository,
};

use sqlx::{SqlitePool, sqlite::{SqliteConnectOptions, SqlitePoolOptions}};
use std::str::FromStr;

pub async fn connect(database_url: &str) -> anyhow::Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(migrations::CREATE_TABLES)
        .execute(pool)
        .await?;
    Ok(())
}
