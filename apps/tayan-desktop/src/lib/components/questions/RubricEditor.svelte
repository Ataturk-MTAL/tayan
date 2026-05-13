<script lang="ts">
  /**
   * RubricEditor — manage rubric items for classic (open-ended) questions.
   * Shows a running total and warns when total ≠ question points.
   *
   * Usage:
   *   <RubricEditor rubric={items} totalPoints={10}
   *                 onchange={(items) => (rubric = items)} />
   */

  export type RubricDraft = { criterion: string; points: number };

  let {
    rubric      = [] as RubricDraft[],
    totalPoints = 0,
    onchange,
  }: {
    rubric?:      RubricDraft[];
    totalPoints?: number;
    onchange?:    (rubric: RubricDraft[]) => void;
  } = $props();

  function addItem() {
    onchange?.([...rubric, { criterion: '', points: 1 }]);
  }

  function removeItem(idx: number) {
    onchange?.(rubric.filter((_, i) => i !== idx));
  }

  function updateItem(idx: number, patch: Partial<RubricDraft>) {
    onchange?.(rubric.map((r, i) => (i === idx ? { ...r, ...patch } : r)));
  }

  let rubricTotal = $derived(rubric.reduce((s, r) => s + r.points, 0));
  let mismatch    = $derived(totalPoints > 0 && rubric.length > 0 && rubricTotal !== totalPoints);
</script>

<div class="space-y-2">
  {#if rubric.length > 0}
    <div class="space-y-2">
      {#each rubric as item, idx}
        <div class="flex items-center gap-2">
          <input
            type="text"
            value={item.criterion}
            oninput={(e) =>
              updateItem(idx, { criterion: (e.currentTarget as HTMLInputElement).value })}
            placeholder="Kriter açıklaması…"
            class="flex-1 rounded-md border border-input bg-background px-3 py-1.5 text-sm
                   focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          />
          <input
            type="number"
            min="1"
            value={item.points}
            oninput={(e) =>
              updateItem(idx, {
                points: Number((e.currentTarget as HTMLInputElement).value) || 1,
              })}
            class="w-16 rounded-md border border-input bg-background px-2 py-1.5 text-sm
                   text-center focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            title="Puan"
          />
          <span class="text-xs text-muted-foreground">puan</span>
          <button
            type="button"
            onclick={() => removeItem(idx)}
            class="text-muted-foreground hover:text-destructive text-lg leading-none px-1"
            aria-label="Kriteri kaldır"
          >×</button>
        </div>
      {/each}
    </div>
  {/if}

  <div class="flex items-center justify-between">
    <button
      type="button"
      onclick={addItem}
      class="text-sm text-primary hover:underline"
    >+ Kriter ekle</button>

    {#if rubric.length > 0}
      <span class="text-sm {mismatch ? 'text-destructive font-semibold' : 'text-muted-foreground'}">
        Toplam: {rubricTotal}
        {#if totalPoints > 0}/ {totalPoints}{/if}
        {#if mismatch}
          — soru puanıyla eşleşmiyor
        {/if}
      </span>
    {/if}
  </div>
</div>
