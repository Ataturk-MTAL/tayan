/**
 * SelectBox'ın kapalıyken gösterdiği etiket.
 *
 * NEDEN AYRI DOSYA: bu bir karar ve karar sınanabilmeli. Bileşenin içinde
 * kalınca sessizce bozuldu — "Tek" seçiliyken açılır listede seçili
 * görünüyor ama kapalı kutu BOMBOŞ duruyordu, çünkü boş değerin etiketi
 * `options` dizisinde değil, listeye çizim sırasında ekleniyor.
 */

export type SelectOption = { value: string; label: string };

export function selectedLabel(
  options: SelectOption[],
  value: string,
  emptyLabel: string | null,
): string {
  if (value === "" && emptyLabel !== null) return emptyLabel;
  const hit = options.find((o) => o.value === value);
  return hit ? hit.label : value;
}
