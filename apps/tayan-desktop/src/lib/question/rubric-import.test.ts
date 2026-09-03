import { describe, expect, test } from "vitest";
import { hasRubricCall, importRubric, removeRange } from "./rubric-import";

/**
 * BU ÇÖZÜMLEYİCİ NOT VERMEZ. Testlerin ağırlığı "doğru okuyor mu"dan çok
 * "yanlışı sessizce kabul ediyor mu" üstünde: yarım bir makroyu geçerli
 * sayarsa öğretmen ölçütlerin taşındığını sanır, taşınmaz.
 */
describe("importRubric", () => {
  test("öğretmenin kendi dosyasındaki biçim okunur", () => {
    const kaynak = `Devreyi çiziniz.

#rubrik((
  ([Çalışma prensibi doğru açıklanmış], 6),
  ([Formül $R = (V_("pin") - V_F)/I$ yazılmış], 3),
))`;
    const r = importRubric(kaynak);
    expect(r?.ok).toBe(true);
    if (!r?.ok) return;
    expect(r.items).toEqual([
      { criterion: "Çalışma prensibi doğru açıklanmış", points: 6 },
      { criterion: 'Formül $R = (V_("pin") - V_F)/I$ yazılmış', points: 3 },
    ]);
  });

  test("içerikteki köşeli parantez ölçütü kesmez", () => {
    const r = importRubric("#rubrik((([Dizi #[iç blok] ile], 5),))");
    expect(r?.ok).toBe(true);
    if (r?.ok) expect(r.items[0].criterion).toBe("Dizi #[iç blok] ile");
  });

  test("bizim ürettiğimiz goster: true yutulur", () => {
    const r = importRubric("#rubrik((([Ölçüt], 7),), goster: true)");
    expect(r?.ok).toBe(true);
    if (r?.ok) expect(r.items).toEqual([{ criterion: "Ölçüt", points: 7 }]);
  });

  test("blok yoksa null — hata değil", () => {
    expect(importRubric("Sadece soru metni.")).toBeNull();
  });

  // ── Sessizce kabul EDİLMEMESİ gerekenler ──────────────────────────────────

  test("yarım makro reddedilir", () => {
    const r = importRubric("#rubrik((([Ölçüt], 5),");
    expect(r?.ok).toBe(false);
  });

  test("değişkenli çağrı reddedilir", () => {
    // #let olcutler = (...) ile yazan öğretmen duvara çarpmalı; uydurma bir
    // ölçüt listesi üretmek, hiç üretmemekten kötü.
    const r = importRubric("#rubrik(olcutler)");
    expect(r?.ok).toBe(false);
  });

  test("hesaplanmış puan reddedilir", () => {
    const r = importRubric("#rubrik((([Ölçüt], 3 + 2),))");
    expect(r?.ok).toBe(false);
    if (r && !r.ok) expect(r.reason).toContain("tam sayı");
  });

  test("köşeli parantezsiz ölçüt reddedilir", () => {
    const r = importRubric('#rubrik((("Ölçüt", 5),))');
    expect(r?.ok).toBe(false);
  });

  test("boş ölçüt metni reddedilir", () => {
    expect(importRubric("#rubrik((([], 5),))")?.ok).toBe(false);
  });

  test("boş liste reddedilir", () => {
    expect(importRubric("#rubrik(())")?.ok).toBe(false);
  });

  test("dizedeki parantez taramayı yanıltmaz", () => {
    const r = importRubric('#rubrik((([Metin "a) b" içeren], 5),))');
    expect(r?.ok).toBe(true);
    if (r?.ok) expect(r.items[0].criterion).toBe('Metin "a) b" içeren');
  });
});

describe("hasRubricCall", () => {
  test("kaydetme kapısı çağrıyı görür", () => {
    expect(hasRubricCall("metin #rubrik((...))")).toBe(true);
    expect(hasRubricCall("rubrik kelimesi geçiyor ama çağrı yok")).toBe(false);
  });
});

describe("removeRange", () => {
  test("blok çıkar, gövde derli toparlanır", () => {
    const kaynak = "Soru metni.\n\n#rubrik((([Ölçüt], 5),))\n\nDevamı.";
    const r = importRubric(kaynak);
    expect(r?.ok).toBe(true);
    if (!r?.ok) return;
    expect(removeRange(kaynak, r.from, r.to)).toBe("Soru metni.\n\nDevamı.");
  });
});
