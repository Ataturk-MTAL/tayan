import { invoke } from "@tauri-apps/api/core";
import type {
  Classroom,
  ContentNode,
  Exam,
  ExamMeta,
  ExamResult,
  Question,
  QuestionAnswerInput,
  QuestionOption,
  Student,
} from "./types";

// ── Helpers ───────────────────────────────────────────────────────────────────

function textBody(text: string): ContentNode[] {
  return [
    {
      type: "text",
      text,
      style: {
        bold: false,
        italic: false,
        underline: false,
        strikethrough: false,
      },
    },
  ];
}

// ── API ───────────────────────────────────────────────────────────────────────

export const api = {
  questions: {
    list: () => invoke<Question[]>("list_questions"),

    addMultipleChoice: (p: {
      points: number;
      outcomes: string[];
      body: ContentNode[];
      options: QuestionOption[];
      shuffle: boolean;
    }) => invoke<string>("add_multiple_choice_question", { payload: p }),

    addTrueFalse: (p: {
      points: number;
      outcomes: string[];
      body: ContentNode[];
      correct_answer: boolean;
    }) => invoke<string>("add_true_false_question", { payload: p }),

    addFillInBlank: (p: {
      outcomes: string[];
      body: ContentNode[];
      blanks: Array<{ id: string; accepted_answers: string[]; points: number }>;
    }) => invoke<string>("add_fill_in_blank_question", { payload: p }),

    addClassic: (p: {
      points: number;
      outcomes: string[];
      body: ContentNode[];
      sample_answer?: ContentNode[] | null;
      rubric: Array<{ criterion: string; points: number }>;
      answer_space:
        | { Lines: number }
        | { HeightCm: number }
        | { Grid: { rows: number; cols: number } };
    }) => invoke<string>("add_classic_question", { payload: p }),

    delete: (questionId: string) =>
      invoke<void>("delete_question", { questionId }),

    update: (q: Question) => invoke<void>("update_question", { payload: q }),
  },

  exams: {
    list: (page = 0, perPage = 100) =>
      invoke<Exam[]>("list_exams", { page, perPage }),

    get: (examId: string) => invoke<Exam>("get_exam", { examId }),

    create: (meta: ExamMeta) =>
      invoke<string>("create_exam", { payload: { meta } }),

    addQuestion: (examId: string, questionId: string) =>
      invoke<void>("add_question_to_exam", {
        payload: { exam_id: examId, question_id: questionId },
      }),

    removeQuestion: (examId: string, questionId: string) =>
      invoke<void>("remove_question_from_exam", { examId, questionId }),

    /**
     * Sorunun BU SINAVDAKİ puanı. null geçilirse sorunun kendi puanına dönülür.
     * Puan soruya değil, sorunun sınavdaki kullanımına aittir.
     */
    setQuestionPoints: (examId: string, questionId: string, points: number | null) =>
      invoke<void>("set_exam_question_points", { examId, questionId, points }),

    publish: (examId: string) => invoke<void>("publish_exam", { examId }),

    delete: (examId: string) => invoke<void>("delete_exam", { examId }),
  },

  students: {
    listClassrooms: () => invoke<Classroom[]>("list_classrooms"),

    createClassroom: (p: { name: string; grade: number; branch: string }) =>
      invoke<string>("create_classroom", { payload: p }),

    listByClassroom: (classroomId: string) =>
      invoke<Student[]>("list_students_by_classroom", { classroomId }),

    addStudent: (p: {
      classroom_id: string;
      number: string;
      first_name: string;
      last_name: string;
    }) => invoke<string>("add_student", { payload: p }),

    deleteStudent: (studentId: string) =>
      invoke<void>("delete_student", { studentId }),

    deleteClassroom: (classroomId: string) =>
      invoke<void>("delete_classroom", { classroomId }),
  },

  results: {
    getByExam: (examId: string) =>
      invoke<ExamResult[]>("get_exam_results", { examId }),

    enter: (p: {
      examId: string;
      studentId: string;
      answers: QuestionAnswerInput[];
      totalMax: number;
    }) => invoke<void>("enter_exam_results", p),
  },

  compiler: {
    /**
     * booklet: kitapçık türü ("A", "B", ...). null ise tek kitapçık — kâğıda
     * etiket basılmaz ve şık sırası yalnızca sınav kimliğinden türetilir.
     */
    generateTypst: (examId: string, answerKey: boolean, booklet: string | null = null) =>
      invoke<string>("generate_exam_pdf", { examId, answerKey, booklet }),

    exportPdf: (examId: string, answerKey: boolean, booklet: string | null = null) =>
      invoke<string>("export_exam_pdf", { examId, answerKey, booklet }),

    exportTypstFile: (examId: string, answerKey: boolean, booklet: string | null = null) =>
      invoke<string>("export_typst_file", { examId, answerKey, booklet }),

    previewTypst: (source: string) =>
      invoke<string>("compile_typst_preview", { source }),

    /**
     * Canlı önizleme yolu: sayfa başına bir SVG dizesi.
     *
     * previewTypst (base64 PDF) dışa aktarma içindir. PDF her derlemede iframe'i
     * baştan yükletir; kaydırma konumu sıfırlanır ve ekran titrer. SVG doğrudan
     * DOM'a girer.
     */
    previewSvg: (source: string) =>
      invoke<string[]>("compile_typst_preview_svg", { source }),

    /**
     * Soru editörü önizlemesi. Gövdeyi Rust tarafında sınavın gerçek önsözüyle
     * sarmalar — önsöz burada kopyalanmaz, bu yüzden sürüklenemez.
     */
    previewQuestion: (body: string) =>
      invoke<string[]>("compile_question_preview_svg", { body }),
  },

  // convenience: build a plain-text ContentNode body
  textBody,
};
