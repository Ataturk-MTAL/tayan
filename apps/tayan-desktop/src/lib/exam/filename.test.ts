import { describe, expect, test } from "vitest";
import { examFileName, slugify } from "./filename";
import type { Exam } from "$lib/types";

function sinav(over: Partial<Exam["meta"]> = {}): Exam {
  return {
    id: "e1",
    status: "Published",
    questions: [],
    created_at: "",
    updated_at: "",
    meta: {
      title: "1. Dönem 1. Yazılı",
      subject: "Matematik",
      classroom: "9-A",
      teacher: "Hakan GÜLEN",
      duration_min: 40,
      date: "2026-11-12",
      instructions: null,
      columns: 1,
      school: null,
      department: null,
      signers: [],
      ...over,
    },
  };
}

describe("slugify", () => {
  test("Türkçe harfler ASCII'ye iner", () => {
    // Dosya e-postayla gidiyor, Windows'a kopyalanıyor: "ı" ve "ğ" orada bozulur.
    expect(slugify("Işık Ölçümü Şubat Çağ")).toBe("Isik-Olcumu-Subat-Cag");
  });

  test("noktalama ve boşluk tek tireye iner", () => {
    expect(slugify("1. Dönem  2.  Yazılı!")).toBe("1-Donem-2-Yazili");
  });

  test("baştaki ve sondaki tireler atılır", () => {
    expect(slugify("  --Matematik--  ")).toBe("Matematik");
  });

  test("tamamen sembolden oluşan parça boş döner", () => {
    expect(slugify("!!!")).toBe("");
  });
});

describe("examFileName", () => {
  test("temel biçim", () => {
    expect(examFileName(sinav(), { answerKey: false, booklet: null, extension: "pdf" }))
      .toBe("9-A_Matematik_1-Donem-1-Yazili_2026-11-12.pdf");
  });

  test("cevap anahtarı EN SONDA", () => {
    expect(examFileName(sinav(), { answerKey: true, booklet: null, extension: "pdf" }))
      .toBe("9-A_Matematik_1-Donem-1-Yazili_2026-11-12_cevap.pdf");
  });

  test("kitapçık cevaptan önce gelir", () => {
    expect(examFileName(sinav(), { answerKey: true, booklet: "B", extension: "pdf" }))
      .toBe("9-A_Matematik_1-Donem-1-Yazili_2026-11-12_kitapcik-B_cevap.pdf");
  });

  test("boş alanlar atlanır, ad bozulmaz", () => {
    expect(
      examFileName(sinav({ classroom: "", subject: "" }), {
        answerKey: false, booklet: null, extension: "pdf",
      }),
    ).toBe("1-Donem-1-Yazili_2026-11-12.pdf");
  });

  test("her alan boşsa bile kullanılabilir ad kalır", () => {
    expect(
      examFileName(sinav({ classroom: "", subject: "", title: "", date: "" }), {
        answerKey: false, booklet: null, extension: "typ",
      }),
    ).toBe("sinav.typ");
  });

  test("uzantı verildiği gibi kullanılır", () => {
    expect(examFileName(sinav(), { answerKey: false, booklet: null, extension: "typ" }))
      .toMatch(/\.typ$/);
  });
});
