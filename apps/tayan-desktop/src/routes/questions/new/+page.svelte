<script lang="ts">
  import { goto } from '$app/navigation';
  import { api } from '$lib/api';
  import { QUESTION_TYPE_LABELS, type ContentNode } from '$lib/types';
  import OutcomeInput from '$lib/components/questions/OutcomeInput.svelte';
  import McOptionsEditor, { type McOption } from '$lib/components/questions/McOptionsEditor.svelte';
  import BlankBodyEditor, { type BlankDraft } from '$lib/components/questions/BlankBodyEditor.svelte';
  import RubricEditor, { type RubricDraft } from '$lib/components/questions/RubricEditor.svelte';
  import RichBodyEditor from '$lib/components/editor/RichBodyEditor.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  type QuestionType = 'multiple_choice' | 'true_false' | 'fill_in_blank' | 'classic';
  const TYPES: QuestionType[] = ['multiple_choice', 'true_false', 'fill_in_blank', 'classic'];

  let selectedType = $state<QuestionType>('multiple_choice');
  let submitting   = $state(false);
  let submitError  = $state<string | null>(null);

  let mcTfBodyNodes   = $state<ContentNode[]>([]);
  let classicBodyNodes = $state<ContentNode[]>([]);
  let points   = $state(5);
  let outcomes = $state<string[]>([]);

  function bodyHasContent(ns: ContentNode[]) {
    return ns.some(n => (n.type === 'text' && n.text.trim()) || n.type === 'math' || n.type === 'image');
  }

  let mcOptions   = $state<McOption[]>([
    { id: 'A', text: '' },
    { id: 'B', text: '' },
    { id: 'C', text: '' },
    { id: 'D', text: '' },
  ]);
  let mcCorrectId = $state('A');
  let shuffle     = $state(false);

  let tfAnswer = $state<boolean>(true);

  let fibBodyText = $state('');
  let fibBlanks   = $state<BlankDraft[]>([]);

  let rubric          = $state<RubricDraft[]>([]);
  let answerSpaceType = $state<'lines' | 'height' | 'grid'>('lines');
  let answerLines     = $state(6);
  let answerHeightCm  = $state(8);
  let answerGridRows  = $state(5);
  let answerGridCols  = $state(10);

  function changeType(t: QuestionType) { selectedType = t; submitError = null; }

  async function submit() {
    submitError = null; submitting = true;
    try {
      if (selectedType === 'multiple_choice') {
        if (!bodyHasContent(mcTfBodyNodes)) { submitError = 'Soru gövdesi boş olamaz.'; return; }
        if (mcOptions.filter((o) => o.text.trim()).length < 2) { submitError = 'En az 2 seçenek gereklidir.'; return; }
        await api.questions.addMultipleChoice({ points, outcomes, body: mcTfBodyNodes,
          options: mcOptions.map((o) => ({ id: o.id, body: api.textBody(o.text.trim() || '—'), correct: o.id === mcCorrectId })), shuffle });
      } else if (selectedType === 'true_false') {
        if (!bodyHasContent(mcTfBodyNodes)) { submitError = 'Soru gövdesi boş olamaz.'; return; }
        await api.questions.addTrueFalse({ points, outcomes, body: mcTfBodyNodes, correct_answer: tfAnswer });
      } else if (selectedType === 'fill_in_blank') {
        if (!fibBodyText.trim()) { submitError = 'Soru gövdesi boş olamaz.'; return; }
        if (fibBlanks.length === 0) { submitError = 'En az bir [B1] boşluğu ekleyin.'; return; }
        const empty = fibBlanks.filter((b) => !b.accepted_answers.trim());
        if (empty.length > 0) { submitError = `${empty.map((b) => b.id).join(', ')} için doğru cevap girilmemiş.`; return; }
        await api.questions.addFillInBlank({ outcomes, body: api.textBody(fibBodyText.trim()),
          blanks: fibBlanks.map((b) => ({ id: b.id,
            accepted_answers: b.accepted_answers.split(',').map((s) => s.trim()).filter(Boolean),
            points: b.points, case_sensitive: b.case_sensitive })) });
      } else if (selectedType === 'classic') {
        if (!bodyHasContent(classicBodyNodes)) { submitError = 'Soru gövdesi boş olamaz.'; return; }
        const answer_space = answerSpaceType === 'lines' ? { Lines: answerLines }
          : answerSpaceType === 'height' ? { HeightCm: answerHeightCm }
          : { Grid: { rows: answerGridRows, cols: answerGridCols } };
        await api.questions.addClassic({ points, outcomes, body: classicBodyNodes,
          rubric: rubric.map((r) => ({ criterion: r.criterion, points: r.points })), answer_space });
      }
      goto('/questions');
    } catch (e) { submitError = String(e); } finally { submitting = false; }
  }

  function onFibChange(v: { bodyText: string; blanks: BlankDraft[] }) {
    fibBodyText = v.bodyText; fibBlanks = v.blanks;
  }
</script>

<div class="p-6 max-w-2xl mx-auto">
  <div class="flex items-center gap-3 mb-6">
    <a href="/questions" class="text-muted-foreground hover:text-foreground text-sm">← Soru Bankası</a>
    <span class="text-muted-foreground">/</span>
    <h1 class="text-xl font-bold">Yeni Soru</h1>
  </div>

  <div class="flex flex-wrap gap-2 mb-6 p-1 bg-muted rounded-lg w-fit">
    {#each TYPES as t}
      <button type="button" onclick={() => changeType(t)}
        class="px-3 py-1.5 rounded-md text-sm font-medium transition-colors
               {selectedType === t ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'}">
        {QUESTION_TYPE_LABELS[t]}
      </button>
    {/each}
  </div>

  <form onsubmit={(e) => { e.preventDefault(); submit(); }} class="space-y-5">

    {#if selectedType === 'multiple_choice' || selectedType === 'true_false'}
      <div class="space-y-1.5">
        <p class="text-sm font-medium">Soru Gövdesi</p>
        <RichBodyEditor nodes={mcTfBodyNodes} onchange={(v) => (mcTfBodyNodes = v)} />
      </div>
    {/if}

    {#if selectedType === 'classic'}
      <div class="space-y-1.5">
        <p class="text-sm font-medium">Soru Gövdesi</p>
        <RichBodyEditor nodes={classicBodyNodes} onchange={(v) => (classicBodyNodes = v)} />
      </div>
    {/if}

    {#if selectedType === 'multiple_choice'}
      <div class="space-y-1.5">
        <p class="text-sm font-medium">Seçenekler</p>
        <McOptionsEditor options={mcOptions} correctId={mcCorrectId}
          onchange={(opts, cid) => { mcOptions = opts; mcCorrectId = cid; }} />
      </div>
      <label class="flex items-center gap-2 text-sm cursor-pointer w-fit">
        <input type="checkbox" bind:checked={shuffle} class="accent-primary rounded" />
        PDF'de seçenekleri karıştır
      </label>
    {/if}

    {#if selectedType === 'true_false'}
      <div class="space-y-2">
        <p class="text-sm font-medium">Doğru Cevap</p>
        <div class="flex gap-4">
          <label class="flex items-center gap-2 text-sm cursor-pointer">
            <input type="radio" name="tf" value={true}  bind:group={tfAnswer} class="accent-primary" /> Doğru
          </label>
          <label class="flex items-center gap-2 text-sm cursor-pointer">
            <input type="radio" name="tf" value={false} bind:group={tfAnswer} class="accent-primary" /> Yanlış
          </label>
        </div>
      </div>
    {/if}

    {#if selectedType === 'fill_in_blank'}
      <BlankBodyEditor bodyText={fibBodyText} blanks={fibBlanks} onchange={onFibChange} />
    {/if}

    {#if selectedType === 'classic'}
      <div class="space-y-1.5">
        <p class="text-sm font-medium">Rubric <span class="font-normal text-muted-foreground">(isteğe bağlı)</span></p>
        <RubricEditor {rubric} totalPoints={points} onchange={(items) => (rubric = items)} />
      </div>
      <div class="space-y-2">
        <p class="text-sm font-medium">Cevap Alanı</p>
        <div class="flex gap-4 flex-wrap">
          {#each (['lines', 'height', 'grid'] as const) as t}
            <label class="flex items-center gap-1.5 text-sm cursor-pointer">
              <input type="radio" name="as" value={t} bind:group={answerSpaceType} class="accent-primary" />
              {t === 'lines' ? 'Satır' : t === 'height' ? 'Yükseklik (cm)' : 'İzgara'}
            </label>
          {/each}
        </div>
        {#if answerSpaceType === 'lines'}
          <div class="flex items-center gap-2 text-sm">
            <input type="number" min="1" max="30" bind:value={answerLines}
              class="w-20 rounded-md border border-input bg-background px-2 py-1.5 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" />
            <span class="text-muted-foreground">satır</span>
          </div>
        {:else if answerSpaceType === 'height'}
          <div class="flex items-center gap-2 text-sm">
            <input type="number" min="1" max="50" step="0.5" bind:value={answerHeightCm}
              class="w-20 rounded-md border border-input bg-background px-2 py-1.5 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" />
            <span class="text-muted-foreground">cm</span>
          </div>
        {:else}
          <div class="flex items-center gap-3 text-sm">
            <div class="flex items-center gap-1.5">
              <input type="number" min="1" max="30" bind:value={answerGridRows}
                class="w-16 rounded-md border border-input bg-background px-2 py-1.5 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" />
              <span class="text-muted-foreground">satır</span>
            </div>
            <span class="text-muted-foreground">×</span>
            <div class="flex items-center gap-1.5">
              <input type="number" min="1" max="50" bind:value={answerGridCols}
                class="w-16 rounded-md border border-input bg-background px-2 py-1.5 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" />
              <span class="text-muted-foreground">sütun</span>
            </div>
          </div>
        {/if}
      </div>
    {/if}

    {#if selectedType !== 'fill_in_blank'}
      <div class="space-y-1.5 w-32">
        <label class="text-sm font-medium" for="points">Puan</label>
        <input id="points" type="number" min="1" max="100" bind:value={points}
          class="w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" />
      </div>
    {/if}

    <div class="space-y-1.5">
      <p class="text-sm font-medium">Kazanım Kodları <span class="font-normal text-muted-foreground">(isteğe bağlı)</span></p>
      <OutcomeInput {outcomes} onchange={(v) => (outcomes = v)} />
    </div>

    {#if submitError}
      <div class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
        {submitError}
      </div>
    {/if}

    <div class="flex items-center justify-end gap-3 pt-2">
      <Button variant="ghost" onclick={() => goto('/questions')}>İptal</Button>
      <Button type="submit" disabled={submitting}>
        {submitting ? 'Kaydediliyor…' : 'Kaydet'}
      </Button>
    </div>

  </form>
</div>
