use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Classroom ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClassroomId(pub Uuid);

impl ClassroomId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl Default for ClassroomId {
    fn default() -> Self { Self::new() }
}

/// Academic year string, e.g. "2025-2026".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcademicYear(pub String);

impl AcademicYear {
    pub fn current() -> Self {
        let now   = Utc::now();
        let start = now.year();
        Self(format!("{start}-{}", start + 1))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Classroom {
    pub id:            ClassroomId,
    pub name:          String,
    pub grade:         u8,
    pub branch:        String,
    pub academic_year: AcademicYear,
    pub student_ids:   Vec<StudentId>,
    pub created_at:    DateTime<Utc>,
}

impl Classroom {
    pub fn new(name: impl Into<String>, grade: u8, branch: impl Into<String>) -> Self {
        Self {
            id:            ClassroomId::new(),
            name:          name.into(),
            grade,
            branch:        branch.into(),
            academic_year: AcademicYear::current(),
            student_ids:   vec![],
            created_at:    Utc::now(),
        }
    }

    pub fn id(&self) -> &ClassroomId { &self.id }

    pub fn add_student(&mut self, id: StudentId) {
        if !self.student_ids.contains(&id) {
            self.student_ids.push(id);
        }
    }
}

// ── Student ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StudentId(pub Uuid);

impl StudentId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl Default for StudentId {
    fn default() -> Self { Self::new() }
}

impl std::fmt::Display for StudentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// School-assigned student number (e.g. "2024001").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentNumber(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct Student {
    pub id:           StudentId,
    pub number:       StudentNumber,
    pub first_name:   String,
    pub last_name:    String,
    pub classroom_id: ClassroomId,
    pub created_at:   DateTime<Utc>,
}

impl Student {
    pub fn new(
        number:       impl Into<String>,
        first_name:   impl Into<String>,
        last_name:    impl Into<String>,
        classroom_id: ClassroomId,
    ) -> Self {
        Self {
            id:           StudentId::new(),
            number:       StudentNumber(number.into()),
            first_name:   first_name.into(),
            last_name:    last_name.into(),
            classroom_id,
            created_at:   Utc::now(),
        }
    }

    pub fn id(&self) -> &StudentId { &self.id }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}
