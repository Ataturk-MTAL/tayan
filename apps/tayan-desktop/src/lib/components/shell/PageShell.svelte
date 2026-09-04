<script lang="ts">
  /**
   * Sayfa kabı: sabit başlık şeridi + kendi içinde kayan içerik.
   *
   * HER SAYFA AYNI İSKELETİ KULLANIR. Kaydırmayı sayfa sayfa kurmak bu
   * projede zaten bir kez ters gitti: kabı olmayan sayfada belge kayıyor,
   * sol çekmece de içerikle birlikte yukarı gidiyordu. Tek kap, tek davranış.
   *
   * `actions` yuvası başlığın sağına düşer — "Yeni soru", "PDF kaydet" gibi
   * sayfanın ana eylemi her sayfada aynı yerde durur.
   */
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    /** Başlığın altındaki tek satırlık açıklama. */
    subtitle?: string | null;
    /** Başlığın sağındaki eylemler. */
    actions?: Snippet;
    children: Snippet;
    /**
     * İçeriğin KENDİ kaydırıcısı varsa (editör, tam yükseklikli düzen) kabuk
     * kaydırmayı üstlenmemeli: iç içe iki kaydırıcıda fare tekerleğinin
     * hangisini süreceği belirsizleşir.
     */
    scroll?: boolean;
  };

  let { title, subtitle = null, actions, children, scroll = true }: Props = $props();
</script>

<div class="flex h-full min-h-0 flex-col">
  <header
    class="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b
           border-gray-200 bg-white px-6 py-4 dark:border-gray-700 dark:bg-gray-800"
  >
    <div class="min-w-0">
      <h1 class="truncate text-xl font-semibold text-gray-900 dark:text-white">
        {title}
      </h1>
      {#if subtitle}
        <p class="mt-0.5 truncate text-sm text-gray-500 dark:text-gray-400">
          {subtitle}
        </p>
      {/if}
    </div>

    {#if actions}
      <div class="flex shrink-0 flex-wrap items-center gap-2">
        {@render actions()}
      </div>
    {/if}
  </header>

  <div class="min-h-0 flex-1" class:overflow-auto={scroll} class:p-6={scroll}>
    {@render children()}
  </div>
</div>
