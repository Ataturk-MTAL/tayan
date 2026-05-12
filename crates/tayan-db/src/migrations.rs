pub const CREATE_TABLES: &str = r#"
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS exams (
    id          TEXT PRIMARY KEY,
    data        TEXT NOT NULL,          -- JSON-serialized Exam
    status      TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS question_bank (
    id          TEXT PRIMARY KEY DEFAULT 'singleton',
    data        TEXT NOT NULL           -- JSON-serialized QuestionBank
);

CREATE TABLE IF NOT EXISTS classrooms (
    id          TEXT PRIMARY KEY,
    data        TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS students (
    id           TEXT PRIMARY KEY,
    classroom_id TEXT NOT NULL REFERENCES classrooms(id),
    data         TEXT NOT NULL,
    created_at   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS exam_results (
    id           TEXT PRIMARY KEY,
    exam_id      TEXT NOT NULL REFERENCES exams(id),
    student_id   TEXT NOT NULL REFERENCES students(id),
    data         TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    UNIQUE (exam_id, student_id)
);

CREATE INDEX IF NOT EXISTS idx_students_classroom ON students(classroom_id);
CREATE INDEX IF NOT EXISTS idx_results_exam       ON exam_results(exam_id);
CREATE INDEX IF NOT EXISTS idx_results_student    ON exam_results(student_id);
"#;
