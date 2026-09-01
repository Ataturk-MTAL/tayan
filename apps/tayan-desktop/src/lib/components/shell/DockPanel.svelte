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
    class="dock-edge flex shrink-0 items-start justify-center bg-paper-lift py-half
           transition-colors hover:text-red-deep"
    style="width: {RAIL_WIDTH}px"
    class:dock-left={layout.side === "left"}
    class:dock-right={layout.side === "right"}
    aria-expanded="false"
    title="{title} panelini aç"
    onclick={toggleCollapsed}
  >
    <span class="stamp rail-title">{title}</span>
  </button>
{:else}
  <aside
    bind:this={panel}
    class="dock-edge relative flex min-h-0 shrink-0 flex-col bg-paper-lift"
    style="width: {layout.width}px"
    class:dock-left={layout.side === "left"}
    class:dock-right={layout.side === "right"}
  >
    <div class="ruled-bottom flex shrink-0 items-center gap-quarter px-half py-quarter">
      <span class="stamp">{title}</span>
      <button
        type="button"
        class="stamp ml-auto px-quarter transition-colors hover:text-red-deep"
        title={layout.side === "left" ? "Paneli sağa taşı" : "Paneli sola taşı"}
        aria-label={layout.side === "left" ? "Paneli sağa taşı" : "Paneli sola taşı"}
        onclick={flipSide}
      >
        {layout.side === "left" ? "⇥" : "⇤"}
      </button>
      <button
        type="button"
        class="stamp px-quarter transition-colors hover:text-red-deep"
        title="Paneli katla"
        aria-label="Paneli katla"
        aria-expanded="true"
        onclick={toggleCollapsed}
      >
        {layout.side === "left" ? "⇤" : "⇥"}
      </button>
    </div>

    <div class="min-h-0 flex-1 overflow-auto px-half py-half">
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
      class="resize-handle"
      class:handle-right={layout.side === "left"}
      class:handle-left={layout.side === "right"}
      class:is-dragging={dragging}
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
  /* Panelin içeriğe bakan kenarı. Taraf değişince çizgi de taraf değiştirir. */
  .dock-edge {
    border-color: var(--color-rule-strong);
    border-style: solid;
    border-width: 0;
  }
  .dock-edge.dock-left {
    border-right-width: 1px;
  }
  .dock-edge.dock-right {
    border-left-width: 1px;
  }

  .rail-title {
    writing-mode: vertical-rl;
    text-orientation: mixed;
  }

  .resize-handle {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 5px;
    cursor: col-resize;
    /* Kenar çizgisinin üstüne biner; kendi rengi yok, yalnız hedef alanıdır. */
    background: transparent;
    touch-action: none;
  }
  .resize-handle.handle-right {
    right: 0;
  }
  .resize-handle.handle-left {
    left: 0;
  }
  .resize-handle:hover,
  .resize-handle:focus-visible,
  .resize-handle.is-dragging {
    background: var(--color-red);
    outline: none;
  }
</style>
