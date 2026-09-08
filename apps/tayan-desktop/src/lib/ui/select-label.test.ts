import { describe, expect, test } from "vitest";
import { selectedLabel } from "./select-label";

const BOOKLET_OPTIONS = [
  { value: "A", label: "A" },
  { value: "B", label: "B" },
];

describe("selectedLabel", () => {
  test("boş değer emptyLabel'i gösterir", () => {
    // Asıl hata: "Tek" seçiliyken kutu boş görünüyordu.
    expect(selectedLabel(BOOKLET_OPTIONS, "", "Tek")).toBe("Tek");
  });

  test("seçili seçeneğin etiketi", () => {
    expect(selectedLabel(BOOKLET_OPTIONS, "B", "Tek")).toBe("B");
  });

  test("emptyLabel yoksa boş değer boş kalır", () => {
    expect(selectedLabel(BOOKLET_OPTIONS, "", null)).toBe("");
  });

  test("listede olmayan değer olduğu gibi gösterilir", () => {
    // allowCustom ile serbest girilen ders adı böyle korunur.
    expect(selectedLabel(BOOKLET_OPTIONS, "Sayısal Elektronik", "Tek")).toBe(
      "Sayısal Elektronik",
    );
  });
});
