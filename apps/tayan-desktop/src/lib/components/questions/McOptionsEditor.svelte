<script lang="ts">
  /**
   * McOptionsEditor — manage multiple-choice options.
   *
   * Each option has an id (A–E or auto), a text body, and a correct flag.
   * Only one option can be correct at a time.
   *
   * Usage:
   *   <McOptionsEditor options={opts} correctId={cId}
   *                    onchange={(opts, cId) => { ... }} />
   */

  export type McOption = { id: string; text: string };

  let {
    options   = [
      { id: 'A', text: '' },
      { id: 'B', text: '' },
      { id: 'C', text: '' },
      { id: 'D', text: '' },
    ],
    correctId = 'A',
    onchange,
  }: {
    options?:   McOption[];
    correctId?: string;
    onchange?:  (options: McOption[], correctId: string) => void;
  } = $props();

  const LABELS = 'ABCDEFGHIJ';

  function setOptionText(idx: number, text: string) {
    const next = options.map((o, i) => (i === idx ? { ...o, text } : o));
    onchange?.(next, correctId);
  }

  function setCorrect(id: string) {
    onchange?.(options, id);
  }

  function addOption() {
    if (options.length >= LABELS.length) return;
    const id = LABELS[options.length];
    onchange?.([...options, { id, text: '' }], correctId);
  }

  function removeOption(idx: number) {
    if (options.length <= 2) return;
    const next = options.filter((_, i) => i !== idx);
    const newCorrect = next.some((o) => o.id === correctId) ? correctId : next[0].id;
    onchange?.(next, newCorrect);
  }
</script>

<div class="space-y-2">
  {#each options as opt, idx}
    <div class="flex items-center gap-2">
      <!-- Correct radio -->
      <button
        type="button"
        onclick={() => setCorrect(opt.id)}
        class="flex-shrink-0 w-6 h-6 rounded-full border-2 transition-colors flex items-center justify-center
               {correctId === opt.id
                 ? 'border-primary bg-primary'
                 : 'border-muted-foreground hover:border-primary'}"
        aria-label="Doğru cevap {opt.id}"
        title="Doğru cevap olarak işaretle"
      >
        {#if correctId === opt.id}
          <span class="w-2.5 h-2.5 rounded-full bg-white"></span>
        {/if}
      </button>

      <!-- Label badge -->
      <span class="flex-shrink-0 w-6 text-center text-sm font-semibold text-muted-foreground">
        {opt.id}
      </span>

      <!-- Text input -->
      <input
        type="text"
        value={opt.text}
        oninput={(e) => setOptionText(idx, (e.currentTarget as HTMLInputElement).value)}
        placeholder="Seçenek metni…"
        class="flex-1 rounded-md border border-input bg-background px-3 py-1.5 text-sm
               focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      />

      <!-- Remove -->
      {#if options.length > 2}
        <button
          type="button"
          onclick={() => removeOption(idx)}
          class="flex-shrink-0 text-muted-foreground hover:text-destructive text-lg leading-none px-1"
          aria-label="Seçeneği kaldır"
        >×</button>
      {/if}
    </div>
  {/each}

  {#if options.length < LABELS.length}
    <button
      type="button"
      onclick={addOption}
      class="mt-1 text-sm text-primary hover:underline"
    >+ Seçenek ekle</button>
  {/if}
</div>
