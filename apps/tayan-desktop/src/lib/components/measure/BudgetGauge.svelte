<script lang="ts">
  /**
   * Bütçe köşede bir sayı değil, dolan ve GÖRÜNÜR biçimde taşan bir ölçüdür.
   * Taşma gizlenmez: taştığı an kırmızı bölge çubuğun dışına çıkar, çünkü
   * 100 puanı aşan bir sınav öğretmenin fark etmesi gereken bir hatadır.
   */
  type Props = {
    label: string;
    value: number;
    target: number;
    unit: string;
  };

  let { label, value, target, unit }: Props = $props();

  let ratio = $derived(target > 0 ? value / target : 0);
  let fillPct = $derived(Math.min(ratio, 1) * 100);
  let overPct = $derived(ratio > 1 ? Math.min((ratio - 1) * 100, 100) : 0);
  let over = $derived(value > target);
</script>

<div class="flex items-center gap-2.5">
  <span class="whitespace-nowrap text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
    {label}
  </span>

  <div class="relative h-[10px] w-[160px] border border-gray-300 bg-gray-100 dark:border-gray-600 dark:bg-gray-700">
    <div class="absolute inset-y-0 left-0 bg-gray-800 dark:bg-gray-200" style="width: {fillPct}%"></div>
    {#if over}
      <!-- Taşan kısım çubuğun dışına, kenar boşluğuna taşar. -->
      <div
        class="absolute inset-y-[-2px] left-full bg-red-600 dark:bg-red-400"
        style="width: {overPct}%"
        aria-hidden="true"
      ></div>
    {/if}
  </div>

  <span
    class="tnum text-[13px] leading-5"
    class:text-red-600={over}
    class:dark:text-red-400={over}
    class:font-semibold={over}
  >
    {value} / {target} {unit}
  </span>
</div>
