<script lang="ts">
  /**
   * Yana kenetlenen panel: taraf değiştirir, katlanır, sürüklenerek boyutlanır.
   *
   * Öncesi sabit 240 px bir sütundu (`MeasureRail`) ve hep sağdaydı. Sabit
   * genişlik içeriği ekranın ortasından çalıyordu; taraf seçilemediği için de
   * kâğıt hep kenara sıkışıyordu. Yerleşim artık öğretmenin kararı.
   *
   * Durum `lib/ui/layout.svelte.ts`'te tek yerde durur ve tarayıcıda saklanır;
   * bu bileşen onu okur ve değiştirir, kendi kopyasını tutmaz.
   */
  import {
    layout,
    flipSide,
    setWidth,
    toggleCollapsed,
    MIN_PANEL_WIDTH,
    MAX_PANEL_WIDTH,
  } from "$lib/ui/layout.svelte";

  type Props = {
    title: string;
    children: import("svelte").Snippet;
  };

  let { title, children }: Props = $props();

  /** Katlıyken görünen ray. Başlığı dikey basar; tıklayınca açılır. */
  const RAIL_WIDTH = 26;

  /** Klavyeyle boyutlandırmada tek basışın değiştirdiği piksel. */
  const KEY_STEP = 20;

  let panel = $state<HTMLElement | undefined>(undefined);
  let dragging = $state(false);

  function onHandlePointerDown(event: PointerEvent) {
    if (!panel) return;
    event.preventDefault();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    dragging = true;
  }

  function onHandlePointerMove(event: PointerEvent) {
    if (!dragging || !panel) return;
    const rect = panel.getBoundingClientRect();
    // Sürükleme panelin DIŞ kenarından ölçülür: sol panelde sol kenar,
    // sağ panelde sağ kenar sabit kalır.
    const next =
      layout.side === "left" ? event.clientX - rect.left : rect.right - event.clientX;
    setWidth(next);
  }

  function onHandlePointerUp(event: PointerEvent) {
    if (!dragging) return;
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
    dragging = false;
  }

  /**
   * Klavyeyle boyutlandırma. Fare tek yol olmamalı: sürükleme hedefi 5 px'lik
   * bir çizgi ve titreyen bir eli olan öğretmen onu tutturamaz.
   */
  function onHandleKeydown(event: KeyboardEvent) {
    const grow = layout.side === "left" ? "ArrowRight" : "ArrowLeft";
    const shrink = layout.side === "left" ? "ArrowLeft" : "ArrowRight";

    if (event.key === grow) {
      event.preventDefault();
      setWidth(layout.width + KEY_STEP);
    } else if (event.key === shrink) {
      event.preventDefault();
      setWidth(layout.width - KEY_STEP);
    }
  }
</script>

{#if layout.collapsed}
  <button
    type="button"
    class="flex shrink-0 items-start justify-center border-gray-200 bg-white py-3
           text-gray-500 transition-colors hover:text-primary-600 dark:border-gray-700
           dark:bg-gray-800 dark:text-gray-400 dark:hover:text-primary-400"
    style="width: {RAIL_WIDTH}px"
    class:border-r={layout.side === "left"}
    class:border-l={layout.side === "right"}
    aria-expanded="false"
    title="{title} panelini aç"
    onclick={toggleCollapsed}
  >
    <span class="rail-title text-sm font-semibold text-gray-700 dark:text-gray-200">{title}</span>
  </button>
{:else}
  <aside
    bind:this={panel}
    class="relative flex min-h-0 shrink-0 flex-col border-gray-200 bg-white
           dark:border-gray-700 dark:bg-gray-800"
    style="width: {layout.width}px"
    class:border-r={layout.side === "left"}
    class:border-l={layout.side === "right"}
  >
    <div
      class="flex shrink-0 items-center gap-1 border-b border-gray-200 px-3 py-2
             dark:border-gray-700"
    >
      <span class="text-sm font-semibold text-gray-700 dark:text-gray-200">{title}</span>
      <button
        type="button"
        class="ml-auto rounded p-1 text-gray-400 transition-colors hover:bg-gray-100
               hover:text-primary-600 dark:text-gray-500 dark:hover:bg-gray-700
               dark:hover:text-primary-400"
        title={layout.side === "left" ? "Paneli sağa taşı" : "Paneli sola taşı"}
        aria-label={layout.side === "left" ? "Paneli sağa taşı" : "Paneli sola taşı"}
        onclick={flipSide}
      >
        {layout.side === "left" ? "⇥" : "⇤"}
      </button>
      <button
        type="button"
        class="rounded p-1 text-gray-400 transition-colors hover:bg-gray-100
               hover:text-primary-600 dark:text-gray-500 dark:hover:bg-gray-700
               dark:hover:text-primary-400"
        title="Paneli katla"
        aria-label="Paneli katla"
        aria-expanded="true"
        onclick={toggleCollapsed}
      >
        {layout.side === "left" ? "⇤" : "⇥"}
      </button>
    </div>

    <!--
      KAYDIRMA KABI — DOKUNMA. `min-h-0 flex-1 overflow-auto` olmadan bu
      `<aside>` flex ebeveyninde büzülmez ve içerik taşar; panel içeriğine
      (sorunun tamamı, uzun bir liste) erişilemez hâle gelir.
    -->
    <div class="min-h-0 flex-1 overflow-auto p-3">
      {@render children()}
    </div>

    <!--
      Boyutlandırma tutamacı panelin İÇ kenarında durur: sol panelde sağda,
      sağ panelde solda. Görünmez ama 5 px geniş ve klavyeyle de sürülebilir.
    -->
    <!--
      Svelte'in a11y linteri `role="separator"` + `tabindex` kalıbını
      etkileşimsiz sayıyor. WAI-ARIA'da yanlış: odaklanabilir bir ayırıcı
      (window splitter) geçerli bir widget'tır ve `aria-valuenow` /
      `aria-valuemin` / `aria-valuemax` tam da bunun için tanımlıdır. Klavyeyle
      boyutlandırmayı silmek, uyarıyı susturmak için erişilebilirliği bozmak
      olurdu — tersi.
    -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="absolute top-0 bottom-0 w-[5px] cursor-col-resize touch-none bg-transparent
             outline-none hover:bg-primary-400 focus-visible:bg-primary-400
             dark:hover:bg-primary-600 dark:focus-visible:bg-primary-600"
      class:right-0={layout.side === "left"}
      class:left-0={layout.side === "right"}
      class:bg-primary-400={dragging}
      class:dark:bg-primary-600={dragging}
      role="separator"
      aria-orientation="vertical"
      aria-label="{title} panelinin genişliği"
      aria-valuenow={layout.width}
      aria-valuemin={MIN_PANEL_WIDTH}
      aria-valuemax={MAX_PANEL_WIDTH}
      tabindex="0"
      onpointerdown={onHandlePointerDown}
      onpointermove={onHandlePointerMove}
      onpointerup={onHandlePointerUp}
      onpointercancel={onHandlePointerUp}
      onkeydown={onHandleKeydown}
    ></div>
  </aside>
{/if}

<style>
  /*
    YALNIZ `writing-mode` KALDI: geri kalan her şey (renk, kenarlık, boyut)
    artık şablondaki Tailwind sınıflarında. Bu özellik utility olarak
    Tailwind'de karşılığı olmadığı için (arbitrary value ile yazılabilirdi
    ama dikey başlık tek kullanım yeri burası — ayrı bir sınıfta durması
    daha okunur) küçük bir kalıntı olarak bırakıldı.
  */
  .rail-title {
    writing-mode: vertical-rl;
    text-orientation: mixed;
  }
</style>
