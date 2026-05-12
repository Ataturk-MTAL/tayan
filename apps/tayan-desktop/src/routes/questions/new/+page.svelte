<script lang="ts">
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import { QUESTION_TYPE_LABELS } from '$lib/types';

  type QuestionType = 'multiple_choice' | 'true_false' | 'fill_in_blank' | 'classic';

  let selectedType = $state<QuestionType>('multiple_choice');
  let submitting   = $state(false);
  let submitError  = $state<string | null>(null);

  // ── Shared fields ────────────────────────────────────────────────────────────
  let bodyText = $state('');
  let points   = $state(5);
  let outcomesRaw = $state('');  // comma-separated, e.g. "M.7.2.3, M.7.2.4"

  function parseOutcomes(): string[] {
    return outcomesRaw
      .split(',')
      .map(s => s.trim())
      .filter(s => s.length > 0);
  }

  // ── Multiple choice ──────────────────────────────────────────────────────────
  type OptionDraft = { id: string; text: string };
  let options = $state<OptionDraft[]>([
    { id: 'A', text: '' },
    { id: 'B', text: '' },
    { id: 'C', text: '' },
    { id: 'D', text: '' },
    { id: 'E', text: '' },
  ]);
  let correctOption = $state('A');
  let shuffle = $state(false);

  // ── True / False ─────────────────────────────────────────────────────────────
  let tfAnswer = $state<boolean>(true);

  // ── Submit ───────────────────────────────────────────────────────────────────
  async function submit() {
    submitError = null;
    if (!bodyText.trim()) { submitError = 'Soru gövdesi boş olamaz.'; return; }

    const body     = api.textBody(bodyText.trim());
    const outcomes = parseOutcomes();

    try {
      submitting = true;

      if (selectedType === 'multiple_choice') {
        const filled = options.filter(o => o.text.trim());
        if (filled.length < 2) { submitError = 'En az 2 seçenek gereklidir.'; return; }
        await api.questions.addMultipleChoice({
          points,
          outcomes,
          body,
          options: options.map(o => ({
            id:      o.id,
            body:    api.textBody(o.text.trim() || '—'),
            correct: o.id === correctOption,
          })),
          shuffle,
        });

      } else if (selectedType === 'true_false') {
        await api.questions.addTrueFalse({ points, outcomes, body, correct_answer: tfAnswer });

      } else {
        submitError = 'Bu tip henüz uygulanmadı — T05 göreviyle eklenecek.';
        return;
      }

      goto('/questions');
    } catch (e) {
      submitError = String(e);
    } finally {
      submitting = false;
    }
  }

  const TYPES: QuestionType[] = ['multiple_choice', 'true_false', 'fill_in_blank', 'classic'];
</script>

<div class="p-6 max-w-2xl mx-auto">

  <!-- Header -->
  <div class="flex items-center gap-3 mb-6">
    <a href="/questions" class="text-muted-foreground hover:text-foreground text-sm">← Soru Bankası</a>
    <span class="text-muted-foreground">/</span>
    <h1 class="text-xl font-bold">Yeni Soru</h1>
  </div>

  <!-- Type selector -->
  <div class="flex gap-2 mb-6 p-1 bg-muted rounded-lg w-fit">
    {#each TYPES as t}
      <button
        type="button"
        onclick={() => { selectedType = t; submitError = null; }}
        class="px-3 py-1.5 rounded-md text-sm font-medium transition-colors
               {selectedType === t
                 ? 'bg-background text-foreground shadow-sm'
                 : 'text-muted-foreground hover:text-foreground'}"
      >
        {QUESTION_TYPE_LABELS[t]}
      </button>
    {/each}
  </div>

  <form onsubmit={(e) => { e.preventDefault(); submit(); }} class="space-y-5">

    <!-- Body -->
    <div class="space-y-1.5">
      <label class="text-sm font-medium" for="body">Soru Gövdesi</label>
      <textarea
        id="body"
        bind:value={bodyText}
        rows="4"
        placeholder="Soru metnini buraya yazın…"
        class="w-full rounded-md border bg-background px-3 py-2 text-sm resize-none
               focus:outline-none focus:ring-2 focus:ring-ring placeholder:text-muted-foreground"
      ></textarea>
    </div>

    <!-- MC options -->
    {#if selectedType === 'multiple_choice'}
      <div class="space-y-2">
        <p class="text-sm font-medium">Seçenekler</p>
        {#each options as opt}
          <div class="flex items-center gap-3">
            <input
              type="radio"
              name="correct"
              value={opt.id}
              bind:group={correctOption}
              class="accent-primary shrink-0"
              title="Doğru cevap"
            />
            <span class="w-6 text-sm font-mono font-bold text-muted-foreground shrink-0">{opt.id}</span>
            <input
              type="text"
              bind:value={opt.text}
              placeholder="Seçenek {opt.id}…"
              class="flex-1 rounded-md border bg-background px-3 py-1.5 text-sm
                     focus:outline-none focus:ring-2 focus:ring-ring placeholder:text-muted-foreground"
            />
          </div>
        {/each}
        <p class="text-xs text-muted-foreground mt-1">Soldaki radyo butonu doğru cevabı işaretler.</p>
      </div>

      <label class="flex items-center gap-2 text-sm cursor-pointer w-fit">
        <input type="checkbox" bind:checked={shuffle} class="accent-primary" />
        PDF'de seçenekleri karıştır
      </label>
    {/if}

    <!-- T/F -->
    {#if selectedType === 'true_false'}
      <div class="space-y-2">
        <p class="text-sm font-medium">Doğru Cevap</p>
        <div class="flex gap-4">
          <label class="flex items-center gap-2 text-sm cursor-pointer">
            <input type="radio" name="tf" value={true} bind:group={tfAnswer} class="accent-primary" />
            Doğru
          </label>
          <label class="flex items-center gap-2 text-sm cursor-pointer">
            <input type="radio" name="tf" value={false} bind:group={tfAnswer} class="accent-primary" />
            Yanlış
          </label>
        </div>
      </div>
    {/if}

    <!-- FillInBlank / Classic placeholder -->
    {#if selectedType === 'fill_in_blank' || selectedType === 'classic'}
      <div class="rounded-md border border-dashed p-4 text-sm text-muted-foreground text-center">
        Bu tip T05 göreviyle uygulanacak.
      </div>
    {/if}

    <!-- Points + Outcomes -->
    <div class="grid grid-cols-2 gap-4">
      <div class="space-y-1.5">
        <label class="text-sm font-medium" for="points">Puan</label>
        <input
          id="points"
          type="number"
          min="1"
          max="100"
          bind:value={points}
          class="w-full rounded-md border bg-background px-3 py-1.5 text-sm
                 focus:outline-none focus:ring-2 focus:ring-ring"
        />
      </div>
      <div class="space-y-1.5">
        <label class="text-sm font-medium" for="outcomes">
          Kazanım Kodları
          <span class="font-normal text-muted-foreground">(virgülle ayır)</span>
        </label>
        <input
          id="outcomes"
          type="text"
          bind:value={outcomesRaw}
          placeholder="M.7.2.3, M.7.2.4"
          class="w-full rounded-md border bg-background px-3 py-1.5 text-sm
                 focus:outline-none focus:ring-2 focus:ring-ring placeholder:text-muted-foreground"
        />
      </div>
    </div>

    <!-- Error -->
    {#if submitError}
      <div class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
        {submitError}
      </div>
    {/if}

    <!-- Actions -->
    <div class="flex items-center justify-end gap-3 pt-2">
      <a
        href="/questions"
        class="rounded-md px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
      >
        İptal
      </a>
      <button
        type="submit"
        disabled={submitting}
        class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground
               hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {submitting ? 'Kaydediliyor…' : 'Kaydet'}
      </button>
    </div>

  </form>
</div>
