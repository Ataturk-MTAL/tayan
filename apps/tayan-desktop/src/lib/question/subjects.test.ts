import { describe, expect, test } from "vitest";
import { GRADE_OPTIONS, STARTER_SUBJECTS, subjectSuggestions } from "./subjects";
import type { Question } from "$lib/types";

function soru(subject: string): Question {
  return {
    question_type: "true_false",
    meta: { subject, grade: 9, difficulty: null, title: "" },
    id: `q-${subject}`,
    points: 5,
    outcomes: [],
    body: [],
    correct_answer: true,
    stats: {
      times_used: 0,
      total_responses: 0,
      correct_responses: 0,
      difficulty_index: 0,
      discrimination_index: 0,
      avg_points_earned: 0,
      last_used_at: null,
      performance_score: 0,
    },
  };
}

describe("subjectSuggestions", () => {
  test("bankada kullanılan dersler EN ÜSTE çıkar", () => {
    const r = subjectSuggestions([soru("Sayısal Elektronik"), soru("Kumanda Teknikleri")]);
    // Kullanılanlar alfabetik, sonra başlangıç listesinin kalanı.
    expect(r.slice(0, 2)).toEqual(["Kumanda Teknikleri", "Sayısal Elektronik"]);
  });

  test("kullanılan ders başlangıç listesinde tekrarlanmaz", () => {
    const r = subjectSuggestions([soru("Matematik")]);
    expect(r.filter((s) => s === "Matematik")).toHaveLength(1);
    expect(r[0]).toBe("Matematik");
  });

  test("boş banka yalnız başlangıç listesini verir", () => {
    expect(subjectSuggestions([])).toEqual(STARTER_SUBJECTS);
  });

  test("boş ve boşluklu ders adları elenir", () => {
    expect(subjectSuggestions([soru("   "), soru("")])).toEqual(STARTER_SUBJECTS);
  });

  test("Türkçe sıralama: Ç, İ, Ü doğru yerde", () => {
    const r = subjectSuggestions([soru("Ünite Testi"), soru("Çizim"), soru("İngilizce")]);
    expect(r.slice(0, 3)).toEqual(["Çizim", "İngilizce", "Ünite Testi"]);
  });
});

describe("GRADE_OPTIONS", () => {
  test("1'den 12'ye, eksiksiz", () => {
    expect(GRADE_OPTIONS).toHaveLength(12);
    expect(GRADE_OPTIONS[0]).toEqual({ value: "1", label: "1. sınıf" });
    expect(GRADE_OPTIONS[11]).toEqual({ value: "12", label: "12. sınıf" });
  });

  test("değerler DİZE — SelectBox dize bekliyor", () => {
    // Sayı gönderilseydi SelectBox eşleşmeyi kaçırır, seçili değer boş görünürdü.
    expect(GRADE_OPTIONS.every((o) => typeof o.value === "string")).toBe(true);
  });
});
