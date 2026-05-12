use anyhow::Context;
use sqlx::{Row, SqlitePool};
use tayan_core::{
    application::ports::{
        ClassroomRepository, ExamRepository, ExamResultRepository,
        QuestionBankRepository, RepositoryError, StudentRepository,
    },
    domain::{
        assessment::aggregates::ExamResult,
        exam_management::{
            aggregates::{Exam, ExamId, QuestionBank},
            entities::question::{Question, QuestionId},
        },
        student_management::aggregates::{Classroom, ClassroomId, Student, StudentId},
    },
};

// ── ExamRepository ────────────────────────────────────────────────────────────

pub struct SqliteExamRepository {
    pub pool: SqlitePool,
}

impl ExamRepository for SqliteExamRepository {
    async fn save(&self, exam: &Exam) -> Result<(), RepositoryError> {
        let id      = exam.id.0.to_string();
        let data    = serde_json::to_string(exam).context("serialize exam")?;
        let status  = format!("{:?}", exam.status);
        let created = exam.created_at.to_rfc3339();
        let updated = exam.updated_at.to_rfc3339();

        sqlx::query(
            r#"INSERT INTO exams (id, data, status, created_at, updated_at) VALUES (?1,?2,?3,?4,?5)
               ON CONFLICT(id) DO UPDATE SET
                 data=excluded.data, status=excluded.status, updated_at=excluded.updated_at"#,
        )
        .bind(id).bind(data).bind(status).bind(created).bind(updated)
        .execute(&self.pool)
        .await
        .context("save exam")?;
        Ok(())
    }

    async fn find_by_id(&self, id: &ExamId) -> Result<Exam, RepositoryError> {
        let id_str = id.0.to_string();
        let row = sqlx::query("SELECT data FROM exams WHERE id=?1")
            .bind(id_str)
            .fetch_optional(&self.pool)
            .await
            .context("query exam")?
            .ok_or_else(|| RepositoryError::NotFound(format!("Exam {id}")))?;

        let data: String = row.get("data");
        Ok(serde_json::from_str(&data).context("deserialize exam")?)
    }

    async fn list(&self, page: u32, per_page: u32) -> Result<Vec<Exam>, RepositoryError> {
        let offset = (page * per_page) as i64;
        let limit  = per_page as i64;
        let rows = sqlx::query(
            "SELECT data FROM exams ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
        )
        .bind(limit).bind(offset)
        .fetch_all(&self.pool)
        .await
        .context("list exams")?;

        rows.iter()
            .map(|r| {
                let data: String = r.get("data");
                serde_json::from_str(&data).context("deserialize exam")
            })
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(RepositoryError::from)
    }

    async fn delete(&self, id: &ExamId) -> Result<(), RepositoryError> {
        let id_str = id.0.to_string();
        sqlx::query("DELETE FROM exams WHERE id=?1")
            .bind(id_str)
            .execute(&self.pool)
            .await
            .context("delete exam")?;
        Ok(())
    }
}

// ── QuestionBankRepository ────────────────────────────────────────────────────

pub struct SqliteQuestionBankRepository {
    pub pool: SqlitePool,
}

impl QuestionBankRepository for SqliteQuestionBankRepository {
    async fn save(&self, bank: &QuestionBank) -> Result<(), RepositoryError> {
        let data = serde_json::to_string(bank).context("serialize bank")?;
        sqlx::query(
            r#"INSERT INTO question_bank (id, data) VALUES ('singleton',?1)
               ON CONFLICT(id) DO UPDATE SET data=excluded.data"#,
        )
        .bind(data)
        .execute(&self.pool)
        .await
        .context("save bank")?;
        Ok(())
    }

    async fn load(&self) -> Result<QuestionBank, RepositoryError> {
        let row = sqlx::query("SELECT data FROM question_bank WHERE id='singleton'")
            .fetch_optional(&self.pool)
            .await
            .context("load bank")?;

        match row {
            Some(r) => {
                let data: String = r.get("data");
                Ok(serde_json::from_str(&data).context("deserialize bank")?)
            }
            None => Ok(QuestionBank::new("Soru Bankası")),
        }
    }

    async fn find_question(&self, id: &QuestionId) -> Result<Option<Question>, RepositoryError> {
        let bank = self.load().await?;
        Ok(bank.find(id).map(|bq| bq.question.clone()))
    }
}

// ── StudentRepository ─────────────────────────────────────────────────────────

pub struct SqliteStudentRepository {
    pub pool: SqlitePool,
}

impl StudentRepository for SqliteStudentRepository {
    async fn save_student(&self, student: &Student) -> Result<(), RepositoryError> {
        let id           = student.id.0.to_string();
        let classroom_id = student.classroom_id.0.to_string();
        let data         = serde_json::to_string(student).context("serialize student")?;
        let created      = student.created_at.to_rfc3339();

        sqlx::query(
            r#"INSERT INTO students (id, classroom_id, data, created_at) VALUES (?1,?2,?3,?4)
               ON CONFLICT(id) DO UPDATE SET data=excluded.data"#,
        )
        .bind(id).bind(classroom_id).bind(data).bind(created)
        .execute(&self.pool)
        .await
        .context("save student")?;
        Ok(())
    }

    async fn find_student(&self, id: &StudentId) -> Result<Student, RepositoryError> {
        let id_str = id.0.to_string();
        let row = sqlx::query("SELECT data FROM students WHERE id=?1")
            .bind(id_str)
            .fetch_optional(&self.pool)
            .await
            .context("query student")?
            .ok_or_else(|| RepositoryError::NotFound(format!("Student {id}")))?;

        let data: String = row.get("data");
        Ok(serde_json::from_str(&data).context("deserialize student")?)
    }

    async fn list_by_classroom(
        &self,
        classroom_id: &ClassroomId,
    ) -> Result<Vec<Student>, RepositoryError> {
        let cid = classroom_id.0.to_string();
        let rows = sqlx::query(
            "SELECT data FROM students WHERE classroom_id=?1 ORDER BY created_at",
        )
        .bind(cid)
        .fetch_all(&self.pool)
        .await
        .context("list students")?;

        rows.iter()
            .map(|r| {
                let data: String = r.get("data");
                serde_json::from_str(&data).context("deserialize student")
            })
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(RepositoryError::from)
    }
}

// ── ClassroomRepository ───────────────────────────────────────────────────────

pub struct SqliteClassroomRepository {
    pub pool: SqlitePool,
}

impl ClassroomRepository for SqliteClassroomRepository {
    async fn save(&self, classroom: &Classroom) -> Result<(), RepositoryError> {
        let id      = classroom.id.0.to_string();
        let data    = serde_json::to_string(classroom).context("serialize classroom")?;
        let created = classroom.created_at.to_rfc3339();

        sqlx::query(
            r#"INSERT INTO classrooms (id, data, created_at) VALUES (?1,?2,?3)
               ON CONFLICT(id) DO UPDATE SET data=excluded.data"#,
        )
        .bind(id).bind(data).bind(created)
        .execute(&self.pool)
        .await
        .context("save classroom")?;
        Ok(())
    }

    async fn find_by_id(&self, id: &ClassroomId) -> Result<Classroom, RepositoryError> {
        let id_str = id.0.to_string();
        let row = sqlx::query("SELECT data FROM classrooms WHERE id=?1")
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .context("query classroom")?
            .ok_or_else(|| RepositoryError::NotFound(format!("Classroom {id_str}")))?;

        let data: String = row.get("data");
        Ok(serde_json::from_str(&data).context("deserialize classroom")?)
    }

    async fn list(&self) -> Result<Vec<Classroom>, RepositoryError> {
        let rows = sqlx::query("SELECT data FROM classrooms ORDER BY created_at")
            .fetch_all(&self.pool)
            .await
            .context("list classrooms")?;

        rows.iter()
            .map(|r| {
                let data: String = r.get("data");
                serde_json::from_str(&data).context("deserialize classroom")
            })
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(RepositoryError::from)
    }
}

// ── ExamResultRepository ──────────────────────────────────────────────────────

pub struct SqliteExamResultRepository {
    pub pool: SqlitePool,
}

impl ExamResultRepository for SqliteExamResultRepository {
    async fn save(&self, result: &ExamResult) -> Result<(), RepositoryError> {
        let id         = result.id.0.to_string();
        let exam_id    = result.exam_id.0.to_string();
        let student_id = result.student_id.0.to_string();
        let data       = serde_json::to_string(result).context("serialize result")?;
        let created    = result.recorded_at.to_rfc3339();

        sqlx::query(
            r#"INSERT INTO exam_results (id, exam_id, student_id, data, created_at)
               VALUES (?1,?2,?3,?4,?5)
               ON CONFLICT(exam_id, student_id) DO UPDATE SET data=excluded.data"#,
        )
        .bind(id).bind(exam_id).bind(student_id).bind(data).bind(created)
        .execute(&self.pool)
        .await
        .context("save result")?;
        Ok(())
    }

    async fn find_by_exam_and_student(
        &self,
        exam_id:    &ExamId,
        student_id: &StudentId,
    ) -> Result<Option<ExamResult>, RepositoryError> {
        let eid = exam_id.0.to_string();
        let sid = student_id.0.to_string();

        let row = sqlx::query(
            "SELECT data FROM exam_results WHERE exam_id=?1 AND student_id=?2",
        )
        .bind(eid).bind(sid)
        .fetch_optional(&self.pool)
        .await
        .context("query result")?;

        match row {
            Some(r) => {
                let data: String = r.get("data");
                Ok(Some(serde_json::from_str(&data).context("deserialize result")?))
            }
            None => Ok(None),
        }
    }

    async fn list_by_exam(&self, exam_id: &ExamId) -> Result<Vec<ExamResult>, RepositoryError> {
        let eid = exam_id.0.to_string();
        let rows = sqlx::query(
            "SELECT data FROM exam_results WHERE exam_id=?1 ORDER BY created_at",
        )
        .bind(eid)
        .fetch_all(&self.pool)
        .await
        .context("list results")?;

        rows.iter()
            .map(|r| {
                let data: String = r.get("data");
                serde_json::from_str(&data).context("deserialize result")
            })
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(RepositoryError::from)
    }
}
