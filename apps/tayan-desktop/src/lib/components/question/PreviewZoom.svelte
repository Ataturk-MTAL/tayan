<script lang="ts">
  type Props = {
    zoom: number;
    onzoom: (value: number) => void;
    onfit: () => void;
  };
  let { zoom, onzoom, onfit }: Props = $props();

  /** Basamaklı seviyeler: rastgele bir sayıya değil, tahmin edilebilir yere gider. */
  export const ZOOM_STEPS = [0.5, 0.65, 0.8, 1, 1.25, 1.5, 2, 3];

  function step(direction: 1 | -1) {
    const found = ZOOM_STEPS.findIndex((s) => s >= zoom - 0.001);
    const index = found === -1 ? ZOOM_STEPS.length - 1 : found;
    const next = ZOOM_STEPS[Math.min(Math.max(index + direction, 0), ZOOM_STEPS.length - 1)];
    onzoom(next);
  }
</script>

<div class="ruled-bottom flex shrink-0 items-center gap-quarter bg-paper px-half py-quarter paper-plain">
  <button
    type="button"
    class="border border-rule-strong bg-paper-lift px-half leading-rule text-ink
           transition-colors hover:border-red hover:text-red-deep disabled:opacity-40"
    disabled={zoom <= ZOOM_STEPS[0]}
    title="Uzaklaştır (⌘−)"
    aria-label="Uzaklaştır"
    onclick={() => step(-1)}
  >
    −
  </button>

  <span class="tnum w-[46px] text-center text-[12px] leading-rule">
    {Math.round(zoom * 100)}%
  </span>

  <button
    type="button"
    class="border border-rule-strong bg-paper-lift px-half leading-rule text-ink
           transition-colors hover:border-red hover:text-red-deep disabled:opacity-40"
    disabled={zoom >= ZOOM_STEPS[ZOOM_STEPS.length - 1]}
    title="Yakınlaştır (⌘+)"
    aria-label="Yakınlaştır"
    onclick={() => step(1)}
  >
    +
  </button>

  <button
    type="button"
    class="ml-half border border-rule-strong bg-paper-lift px-half text-[12px] leading-rule
           text-ink transition-colors hover:border-red hover:text-red-deep"
    title="Sayfayı panele sığdır"
    onclick={onfit}
  >
    Sığdır
  </button>

  <button
    type="button"
    class="border border-rule-strong bg-paper-lift px-half text-[12px] leading-rule
           text-ink transition-colors hover:border-red hover:text-red-deep"
    title="Gerçek boyut (⌘0)"
    onclick={() => onzoom(1)}
  >
    100%
  </button>

  <span class="pencil ml-auto">⌘ + tekerlek</span>
</div>
