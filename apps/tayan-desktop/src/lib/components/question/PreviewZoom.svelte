<script lang="ts">
  import { Button } from "flowbite-svelte";
  import { MinusOutline, PlusOutline } from "flowbite-svelte-icons";

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

<div
  class="flex shrink-0 items-center gap-2 border-b border-gray-200 bg-white px-3 py-1.5
         dark:border-gray-700 dark:bg-gray-800"
>
  <Button
    size="xs"
    color="light"
    class="!p-1.5"
    disabled={zoom <= ZOOM_STEPS[0]}
    title="Uzaklaştır (⌘−)"
    aria-label="Uzaklaştır"
    onclick={() => step(-1)}
  >
    <MinusOutline class="h-3.5 w-3.5" />
  </Button>

  <span class="tnum w-12 text-center text-xs text-gray-600 dark:text-gray-300">
    {Math.round(zoom * 100)}%
  </span>

  <Button
    size="xs"
    color="light"
    class="!p-1.5"
    disabled={zoom >= ZOOM_STEPS[ZOOM_STEPS.length - 1]}
    title="Yakınlaştır (⌘+)"
    aria-label="Yakınlaştır"
    onclick={() => step(1)}
  >
    <PlusOutline class="h-3.5 w-3.5" />
  </Button>

  <Button size="xs" color="light" class="ml-2" title="Sayfayı panele sığdır" onclick={onfit}>
    Sığdır
  </Button>

  <Button size="xs" color="light" title="Gerçek boyut (⌘0)" onclick={() => onzoom(1)}>
    100%
  </Button>

  <span class="ml-auto text-xs text-gray-400 dark:text-gray-500">⌘ + tekerlek</span>
</div>
