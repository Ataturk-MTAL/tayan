import { describe, expect, test } from "vitest";
import { typstPlain } from "./plain-text";

describe("typstPlain", () => {
  test("asıl şikâyet: ham kod yerine okunur satır", () => {
    // Analiz ekranında bu satır aynen şöyle görünüyordu:
    //   "[typst] Aşağıdaki denklemin köklerini bulunuz. $ x^2 - 5x + 6 = 0 $"
    const kaynak = "Aşağıdaki denklemin köklerini bulunuz.\n\n$ x^2 - 5x + 6 = 0 $";
    expect(typstPlain(kaynak)).toBe("Aşağıdaki denklemin köklerini bulunuz. x² - 5x + 6 = 0");
  });

  test("gövdesiz çağrı düşer — cevap alanı okunacak bir şey değil", () => {
    const kaynak = 'Grafiği çiziniz.\n\n#cevap-alani(satir: 10, bicim: "kareli")';
    expect(typstPlain(kaynak)).toBe("Grafiği çiziniz.");
  });

  test("gövdeli çağrının İÇERİĞİ kalır", () => {
    expect(typstPlain("#text(8pt)[Not: dikkat]")).toBe("Not: dikkat");
  });

  test("iç içe gövde özyineleniyor", () => {
    expect(typstPlain("#text(8pt)[alan $x^2$ olur]")).toBe("alan x² olur");
  });

  test("vurgu imleri düşer, metin kalır", () => {
    expect(typstPlain("*kalın* ve _eğik_ metin")).toBe("kalın ve eğik metin");
  });

  test("kaçırılmış karakter olduğu gibi kalır", () => {
    expect(typstPlain("C\\# dili ve \\$5")).toBe("C# dili ve $5");
  });

  test("alt simge: taban gösterimi okunur olur", () => {
    expect(typstPlain("$(1011 thin 0110)_2$ sayısı")).toBe("(1011 0110)₂ sayısı");
  });

  test("matematikteki tırnak metin kipidir, görünmez", () => {
    expect(typstPlain('$(2"F")_16$')).toBe("(2F)₁₆");
  });

  test("saydam sarmalayıcı açılır", () => {
    expect(typstPlain("$R = bold(300 Omega)$")).toBe("R = 300 Ω");
  });

  test("sözlük: dot, times, ok", () => {
    expect(typstPlain("$2 dot 16$")).toBe("2 · 16");
    expect(typstPlain("$a != b$")).toBe("a ≠ b");
    expect(typstPlain("$0 -> 1$")).toBe("0 → 1");
  });

  test("yorumlar düşer", () => {
    expect(typstPlain("Soru // burası not\nmetin")).toBe("Soru metin");
    expect(typstPlain("Soru /* gizli */ metin")).toBe("Soru metin");
  });

  test("kapanmamış ayraç metni yutmaz, çökmez", () => {
    // Öğretmen yazarken gövde sürekli yarım kalır; önizleme patlamamalı.
    expect(typstPlain("Metin #kutu(")).toBe("Metin");
    expect(typstPlain("Metin $x^2")).toBe("Metin x²");
  });

  test("boş kaynak boş metin", () => {
    expect(typstPlain("")).toBe("");
    expect(typstPlain("   \n  ")).toBe("");
  });
});
