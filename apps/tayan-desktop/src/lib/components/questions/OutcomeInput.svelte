<script lang="ts">
  /**
   * OutcomeInput — comma-separated outcome code entry with tag display.
   * Binds to a string[] via the `outcomes` prop (two-way via callback).
   *
   * Usage:
   *   <OutcomeInput outcomes={myOutcomes} onchange={(v) => (myOutcomes = v)} />
   */

  let {
    outcomes = [],
    onchange,
    placeholder = 'M.7.2.3, M.7.2.4',
  }: {
    outcomes?:    string[];
    onchange?:    (v: string[]) => void;
    placeholder?: string;
  } = $props();

  // eslint-disable-next-line svelte/state-referenced-locally
  let raw = $state(outcomes.join(', '));

  function parse(text: string): string[] {
    return text
      .split(',')
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
  }

  function handleInput(e: Event) {
    raw = (e.currentTarget as HTMLInputElement).value;
    onchange?.(parse(raw));
  }

  function removeTag(code: string) {
    const next = outcomes.filter((o) => o !== code);
    raw = next.join(', ');
    onchange?.(next);
  }

  let tags = $derived(parse(raw));
</script>

<div class="space-y-2">
  <input
    type="text"
    value={raw}
    oninput={handleInput}
    {placeholder}
    class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
           ring-offset-background placeholder:text-muted-foreground
           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
  />
  {#if tags.length > 0}
    <div class="flex flex-wrap gap-1.5">
      {#each tags as code}
        <span class="inline-flex items-center gap-1 rounded-full bg-primary/10 text-primary
                     px-2.5 py-0.5 text-xs font-medium">
          {code}
          <button
            type="button"
            onclick={() => removeTag(code)}
            class="ml-0.5 text-primary/60 hover:text-primary leading-none"
            aria-label="Kaldır"
          >×</button>
        </span>
      {/each}
    </div>
  {/if}
</div>
