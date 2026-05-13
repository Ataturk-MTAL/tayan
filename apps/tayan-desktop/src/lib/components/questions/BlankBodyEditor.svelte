<script lang="ts">
  /**
   * BlankBodyEditor — text area where teacher writes body using [B1], [B2]… syntax.
   * Parses blanks in real time and lets teacher set accepted answers per blank.
   *
   * Emits:
   *   onchange({ bodyText, blanks })
   *     bodyText — raw text with [Bn] placeholders
   *     blanks   — array of { id, accepted_answers, points, case_sensitive }
   */

  export type BlankDraft = {
    id:               string;
    accepted_answers: string;  // comma-sep; split on save
    points:           number;
    case_sensitive:   boolean;
  };

  let {
    bodyText = '',
    blanks   = [] as BlankDraft[],
    onchange,
  }: {
    bodyText?: string;
    blanks?:   BlankDraft[];
    onchange?: (v: { bodyText: string; blanks: BlankDraft[] }) => void;
  } = $props();

  /** Parse [B1], [B2]… ids from text in order of appearance. */
  function parseIds(text: string): string[] {
    const seen = new Set<string>();
    const ids: string[] = [];
    for (const m of text.matchAll(/\[B(\d+)\]/gi)) {
      const id = `B${m[1]}`;
      if (!seen.has(id)) { seen.add(id); ids.push(id); }
    }
    return ids;
  }

  function syncBlanks(text: string, current: BlankDraft[]): BlankDraft[] {
    const ids = parseIds(text);
    const map = Object.fromEntries(current.map((b) => [b.id, b]));
    return ids.map((id) => map[id] ?? {
      id,
      accepted_answers: '',
      points: 1,
      case_sensitive: false,
    });
  }

  function handleBodyInput(e: Event) {
    const text = (e.currentTarget as HTMLTextAreaElement).value;
    const next = syncBlanks(text, blanks);
    onchange?.({ bodyText: text, blanks: next });
  }

  function updateBlank(id: string, patch: Partial<BlankDraft>) {
    const next = blanks.map((b) => b.id === id ? { ...b, ...patch } : b);
    onchange?.({ bodyText, blanks: next });
  }

  let detectedIds = $derived(parseIds(bodyText));
</script>

<div class="space-y-3">
  <!-- Body textarea -->
  <div>
    <label class="block text-sm font-medium mb-1">
      Soru Gövdesi
      <span class="text-muted-foreground font-normal">(boşluklar için [B1], [B2]… yazın)</span>
    </label>
    <textarea
      rows="4"
      value={bodyText}
      oninput={handleBodyInput}
      placeholder="Örn: Fotosentez _______ [B1] ve _______ [B2] ile gerçekleşir."
      class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm
             ring-offset-background placeholder:text-muted-foreground resize-y
             focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    ></textarea>
  </div>

  <!-- Per-blank config -->
  {#if detectedIds.length > 0}
    <div class="space-y-2">
      <p class="text-sm font-medium">Boşluk Ayarları</p>
      {#each blanks as blank}
        <div class="rounded-md border border-border p-3 space-y-2">
          <div class="flex items-center gap-2">
            <span class="text-xs font-bold bg-primary/10 text-primary rounded px-2 py-0.5">
              [{blank.id}]
            </span>
            <input
              type="text"
              value={blank.accepted_answers}
              oninput={(e) =>
                updateBlank(blank.id, {
                  accepted_answers: (e.currentTarget as HTMLInputElement).value,
                })}
              placeholder="Doğru cevaplar (virgülle ayır)"
              class="flex-1 rounded-md border border-input bg-background px-3 py-1.5 text-sm
                     focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            />
            <input
              type="number"
              min="1"
              value={blank.points}
              oninput={(e) =>
                updateBlank(blank.id, {
                  points: Number((e.currentTarget as HTMLInputElement).value) || 1,
                })}
              class="w-16 rounded-md border border-input bg-background px-2 py-1.5 text-sm
                     text-center focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              title="Puan"
            />
            <span class="text-xs text-muted-foreground">puan</span>
          </div>
          <label class="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer">
            <input
              type="checkbox"
              checked={blank.case_sensitive}
              onchange={(e) =>
                updateBlank(blank.id, {
                  case_sensitive: (e.currentTarget as HTMLInputElement).checked,
                })}
              class="rounded"
            />
            Büyük/küçük harf duyarlı
          </label>
        </div>
      {/each}
    </div>
  {/if}
</div>
