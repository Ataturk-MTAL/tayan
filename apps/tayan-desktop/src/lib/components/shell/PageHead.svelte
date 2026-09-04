<script lang="ts">
  /**
   * PageHead — üst başlık şeridi.
   *
   * ARTIK YENİ SAYFALAR İÇİN DEĞİL: `PageShell.svelte` başlığı da içeren
   * tam sayfa kabuğunu üstlendi. Bu bileşen yine de `hakkında`, `yardım`,
   * `analiz`, `sınavlar`, `sorular`, `öğrenciler` ve `sınav/yeni`
   * rotalarında kullanılıyor — o dosyalar bu görevin kapsamı DIŞINDA, o
   * yüzden props (`title`, `count`, `children`) SABİT kaldı.
   *
   * PageShell'e YÖNLENDİRMEDİ çünkü PageShell kendi kaydırma kabını da
   * sarıyor (`children` = sayfanın TAMAMI); PageHead'i çağıran sayfalarsa
   * kaydırma kabını (`min-h-0 flex-1 overflow-auto`) PageHead'İN DIŞINDA,
   * kendi düzeninde kuruyor. PageShell'e sarsaydık iki kaydırıcı iç içe
   * girer, fare tekerleğinin hangisini süreceği belirsizleşirdi (PageShell
   * bu yüzden zaten "tek kap, tek davranış" diyor). Bunun yerine PageShell
   * ile GÖRSEL OLARAK birebir aynı `<header>` sınıfları buraya taşındı —
   * ikisi yan yana kullanılsa fark edilmez, ileride bir sayfa PageHead'den
   * PageShell'e geçince de görünüm değişmez.
   */
  type Props = {
    title: string;
    count?: string | null;
    children?: import("svelte").Snippet;
  };
  let { title, count = null, children }: Props = $props();
</script>

<div
  class="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b
         border-gray-200 bg-white px-6 py-4 dark:border-gray-700 dark:bg-gray-800"
>
  <div class="flex min-w-0 items-baseline gap-3">
    <h1 class="truncate text-xl font-semibold text-gray-900 dark:text-white">{title}</h1>
    {#if count}
      <span class="tnum shrink-0 text-sm text-gray-500 dark:text-gray-400">{count}</span>
    {/if}
  </div>

  <div class="flex shrink-0 flex-wrap items-center gap-2">
    {@render children?.()}
  </div>
</div>
