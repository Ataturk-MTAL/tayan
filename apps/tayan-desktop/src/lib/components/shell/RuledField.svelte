<script lang="ts">
  /**
   * RuledField — etiketli form alanı sarmalayıcısı.
   *
   * `label` / `hint` / `children` props'u QuestionInspector ve dört rotada
   * (`analysis`, `students`, `exams/new`, `exams/[id]`) kullanılıyor — o
   * dosyalar bu görevin kapsamı DIŞINDA, o yüzden props SABİT kaldı. Aynı
   * neden `id`/`for` çifti YERİNE `<label>` çağıranın verdiği kontrolü
   * SARMASIN diye tercih edilmedi: çağıran `RuledField`'e hangi elemente
   * `id` vereceğini söylemiyor, dolayısıyla dolaylı ilişkilendirme (label
   * kontrolü SARAR) tek seçenek — böylece etikete tıklamak da alanı
   * odaklıyor.
   *
   * Flowbite'ın `Label`'ı bu sarmalayıcı görevi zaten görüyor (metin
   * boyutu/rengi + `<label>` elementi); üstüne yalnız ipucu satırını ekliyoruz.
   */
  import { Label } from "flowbite-svelte";

  type Props = {
    label: string;
    hint?: string | null;
    children: import("svelte").Snippet;
  };
  let { label, hint = null, children }: Props = $props();
</script>

<Label class="ruled-field block space-y-1">
  <span class="block">{label}</span>
  {@render children()}
  {#if hint}
    <span class="block text-xs font-normal text-gray-500 dark:text-gray-300">{hint}</span>
  {/if}
</Label>

<style>
  /*
    NEDEN :global() VE ÖZEL SINIF: `children` çağıranın verdiği ham
    `<input>`/`<select>`/`<textarea>` (SelectBox kendi sınıflarını taşıyor,
    bkz. SelectBox.svelte — bu kural onun için de geçerli ama zaten
    override ediyor). Bu snippet'in içeriğine dışarıdan class EKLEYEMEYİZ,
    üstelik `<label>` elementini artık biz değil Flowbite'ın Label bileşeni
    çiziyor — Svelte'in stil kapsamlama karması (scope hash) o elemente
    iğnelenmiyor. Tek yol: kendi seçtiğimiz sabit bir sınıf (`ruled-field`)
    üzerinden tam `:global()` seçici. Bu sınıf yalnız BU bileşenin verdiği
    `<label>`'a gidiyor, uygulamadaki başka `<label>`'ları etkilemiyor.

    Bu, Flowbite'ın `Input` bileşeninin `md` boyutundaki varsayılan
    görünümünün elle taşınmış hâli (kenarlık, yuvarlak köşe, odak halkası,
    koyu kip) — çağıranlar `Input` bileşenini DEĞİL çıplak `<input>` yazdığı
    için bu görünümü buradan vermek zorundayız.
  */
  :global(.ruled-field input),
  :global(.ruled-field select),
  :global(.ruled-field textarea) {
    display: block;
    width: 100%;
    border-radius: 0.5rem;
    border: 1px solid var(--color-gray-300);
    background-color: var(--color-gray-50);
    padding: 0.625rem;
    font-size: 0.875rem;
    line-height: 1.25rem;
    color: var(--color-gray-900);
    /* Puan, seviye gibi sayısal alanlar hizalı kalsın diye. */
    font-variant-numeric: tabular-nums;
  }

  :global(.ruled-field input:disabled),
  :global(.ruled-field select:disabled),
  :global(.ruled-field textarea:disabled) {
    cursor: not-allowed;
    opacity: 0.5;
  }

  :global(.ruled-field input:focus),
  :global(.ruled-field select:focus),
  :global(.ruled-field textarea:focus) {
    outline: none;
    border-color: var(--color-primary-500);
    box-shadow: 0 0 0 1px var(--color-primary-500);
  }

  :global(.dark) :global(.ruled-field input),
  :global(.dark) :global(.ruled-field select),
  :global(.dark) :global(.ruled-field textarea) {
    border-color: var(--color-gray-600);
    background-color: var(--color-gray-700);
    color: var(--color-white);
  }
</style>
