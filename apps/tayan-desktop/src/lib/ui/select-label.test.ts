import { describe, expect, test } from "vitest";
import { selectedLabel } from "./select-label";

const KITAPCIK = [
  { value: "A", label: "A" },
  { value: "B", label: "B" },
];

describe("selectedLabel", () => {
  test("boş değer emptyLabel'i gösterir", () => {
    // Asıl hata: "Tek" seçiliyken kutu boş görünüyordu.
    expect(selectedLabel(KITAPCIK, "", "Tek")).toBe("Tek");
  });

  test("seçili seçeneğin etiketi", () => {
    expect(selectedLabel(KITAPCIK, "B", "Tek")).toBe("B");
  });

  test("emptyLabel yoksa boş değer boş kalır", () => {
    expect(selectedLabel(KITAPCIK, "", null)).toBe("");
  });

  test("listede olmayan değer olduğu gibi gösterilir", () => {
    // allowCustom ile serbest girilen ders adı böyle korunur.
    expect(selectedLabel(KITAPCIK, "Sayısal Elektronik", "Tek")).toBe(
      "Sayısal Elektronik",
    );
  });
});
