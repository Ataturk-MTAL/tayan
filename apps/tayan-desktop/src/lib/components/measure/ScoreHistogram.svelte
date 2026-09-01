<script lang="ts">
  /**
   * Puan dağılımı. Tek seri, bu yüzden gösterge kutusu yok — başlık seriyi
   * adlandırır. Çubuklar olgudur ve mürekkeple çizilir; kırmızı olan tek şey
   * geçme eşiği ve sınıf ortalaması, çünkü onlar yargıdır.
   */
  type Props = {
    percentages: number[];
    passMark?: number;
  };

  let { percentages, passMark = 50 }: Props = $props();

  const BIN_SIZE = 10;
  const BIN_COUNT = 10;
  const BAR_AREA_PX = 160;

  let bins = $derived.by(() => {
    const counts = new Array<number>(BIN_COUNT).fill(0);
    for (const p of percentages) {
      const index = Math.min(Math.floor(p / BIN_SIZE), BIN_COUNT - 1);
      counts[index] += 1;
    }
    return counts;
  });

  let maxCount = $derived(Math.max(1, ...bins));
  let mean = $derived(
    percentages.length === 0
      ? 0
      : percentages.reduce((sum, p) => sum + p, 0) / percentages.length,
  );
</script>

<figure class="m-0">
  <figcaption class="stamp">Puan dağılımı</figcaption>

  {#if percentages.length === 0}
    <p class="pencil mt-quarter">Sonuç girilmemiş.</p>
  {:else}
    <div class="relative mt-half" style="height: {BAR_AREA_PX}px">
      <!-- Geçme eşiği ve ortalama: kırmızı, çünkü ikisi de değerlendirmedir. -->
      <div
        class="pointer-events-none absolute inset-y-0 border-l border-dashed border-red"
        style="left: {passMark}%"
        aria-hidden="true"
      ></div>
      <div
        class="pointer-events-none absolute inset-y-0 border-l border-red"
        style="left: {mean}%"
        aria-hidden="true"
      ></div>

      <div class="flex h-full items-end gap-[2px]">
        {#each bins as count, i}
          <div
            class="group relative flex-1"
            style="height: {(count / maxCount) * 100}%"
            title="{i * BIN_SIZE}–{(i + 1) * BIN_SIZE}%: {count} öğrenci"
          >
            <div class="h-full w-full bg-ink transition-colors group-hover:bg-ink-mid"></div>
            {#if count > 0}
              <span
                class="pointer-events-none absolute -top-[18px] left-0 right-0 text-center
                       text-[11px] leading-rule text-pencil tnum"
              >
                {count}
              </span>
            {/if}
          </div>
        {/each}
      </div>
    </div>

    <div class="mt-quarter flex justify-between border-t border-rule-strong pt-quarter">
      {#each Array(6) as _, i}
        <span class="pencil tnum">{i * 20}</span>
      {/each}
    </div>

    <p class="annot mt-quarter">
      Kesiksiz kırmızı çizgi sınıf ortalaması ({mean.toFixed(0)}%), kesikli çizgi
      geçme eşiği ({passMark}%).
    </p>
  {/if}
</figure>
