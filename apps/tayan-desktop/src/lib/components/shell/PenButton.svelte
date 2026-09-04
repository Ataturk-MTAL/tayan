<script lang="ts">
  /**
   * PenButton — flowbite-svelte `Button`'ın ince bir sarmalayıcısı.
   *
   * NEDEN SİLİNMEDİ, SARMALANDI: QuestionEditor, ResultEntry ve yardım /
   * sınavlar / sorular / öğrenciler / sınav-yeni rotaları bu bileşeni
   * `kind` prop'uyla çağırıyor. O dosyalar bu görevin kapsamı DIŞINDA —
   * bileşeni silmek ya da `kind`'ı kaldırmak hepsini derleme zamanında
   * kırardı. Bu yüzden `kind`'ı Flowbite'ın `color`'una çeviren tek satırlık
   * bir katman bırakıldı: çağıran taraf HİÇ değişmeden Flowbite görünümüne
   * kavuşuyor.
   *
   * Renk eşlemesi:
   * - "ink"   → primary (dolgun, birincil eylem — "Kaydet", "Yeni sınav")
   * - "red"   → red     (uyarı/vurgu; app.css'te kırmızı artık değerlendirme
   *             rengi olarak ayrıldı, bu yüzden burada da kırmızı kalması
   *             tutarlı: "zayıf ayırt edici" gibi dikkat çeken filtreler)
   * - "quiet" → alternative (kenarlıklı, sessiz buton — ikincil eylemler)
   */
  import { Button } from "flowbite-svelte";

  type Props = {
    kind?: "ink" | "red" | "quiet";
    type?: "button" | "submit";
    disabled?: boolean;
    onclick?: () => void;
    children: import("svelte").Snippet;
  };
  let { kind = "ink", type = "button", disabled = false, onclick, children }: Props = $props();

  const KIND_TO_COLOR = {
    ink: "primary",
    red: "red",
    quiet: "alternative",
  } as const;
</script>

<Button color={KIND_TO_COLOR[kind]} size="sm" {type} {disabled} {onclick}>
  {@render children()}
</Button>
