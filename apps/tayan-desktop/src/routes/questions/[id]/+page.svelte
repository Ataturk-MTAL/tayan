<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import RichBodyEditor from '$lib/components/editor/RichBodyEditor.svelte';
  import TypstBodyEditor from '$lib/components/editor/TypstBodyEditor.svelte';
  import BlankBodyEditor, { type BlankDraft } from '$lib/components/questions/BlankBodyEditor.svelte';
  import McOptionsEditor, { type McOption } from '$lib/components/questions/McOptionsEditor.svelte';
  import OutcomeInput from '$lib/components/questions/OutcomeInput.svelte';
  import RubricEditor, { type RubricDraft } from '$lib/components/questions/RubricEditor.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import {
    QUESTION_TYPE_LABELS,
    type AnswerSpace,
    type ClassicQuestion,
    type ContentNode,
    type FillInBlankQuestion,
    type MultipleChoiceQuestion,
    type Question,
    type TrueFalseQuestion,
  } from '$lib/types';
  import { onMount } from 'svelte';

  type BodyEditorMode = 'rich' | 'typst';

  // ── route param ─────────────────────────────────────────────────────────────
  const questionId = $derived($page.params.id);

  // ── load state ──────────────────────────────────────────────────────────────
  let loadError  = $state<string | null>(null);
  let loaded     = $state(false);
  let original   = $state<Question | null>(null);

  // ── common form fields ───────────────────────────────────────────────────────
  let points   = $state(5);
  let outcomes = $state<string[]>([]);

  // ── MC / TF shared body ──────────────────────────────────────────────────────
  let mcTfBodyNodes   = $state<ContentNode[]>([]);
  let classicBodyNodes = $state<ContentNode[]>([]);
  let mcTfEditorMode = $state<BodyEditorMode>('rich');
  let classicEditorMode = $state<BodyEditorMode>('rich');
  let mcTfTypstBody = $state('');
  let classicTypstBody = $state('');

  // ── MC ──────────────────────────────────────────────────────────────────────
  let mcOptions   = $state<McOption[]>([]);
  let mcCorrectId = $state('A');
  let shuffle     = $state(false);

  // ── TF ──────────────────────────────────────────────────────────────────────
  let tfAnswer = $state<boolean>(true);

  // ── FiB ─────────────────────────────────────────────────────────────────────
  let fibBodyText = $state('');
  let fibBlanks   = $state<BlankDraft[]>([]);

  // ── Classic ──────────────────────────────────────────────────────────────────
  let rubric          = $state<RubricDraft[]>([]);
  let answerSpaceType = $state<'lines' | 'height' | 'grid'>('lines');
  let answerLines     = $state(6);
  let answerHeightCm  = $state(8);
  let answerGridRows  = $state(5);
  let answerGridCols  = $state(10);

  // ── submit ───────────────────────────────────────────────────────────────────
  let submitting  = $state(false);
  let submitError = $state<string | null>(null);

  // ── helpers ──────────────────────────────────────────────────────────────────
  function extractText(nodes: ContentNode[]): string {
    return nodes
      .map((n) => {
        if (n.type === 'text') return n.text;
        if (n.type === 'math') return `$${(n as { type: 'math'; raw: string }).raw}$`;
        if (n.type === 'typst_raw') return n.code;
        return '';
      })
      .join('');
  }

  function bodyHasRichContent(ns: ContentNode[]) {
    return ns.some(
      (n) => (n.type === 'text' && n.text.trim()) || n.type === 'math' || n.type === 'image',
    );
  }

  function extractTypstRaw(nodes: ContentNode[]): string | null {
    if (nodes.length !== 1) return null;
    const n = nodes[0];
    return n.type === 'typst_raw' ? n.code : null;
  }

  function asTypstRaw(code: string): ContentNode[] {
    return [{ type: 'typst_raw', code }];
  }

  // ── populate form from loaded question ───────────────────────────────────────
  function populate(q: Question) {
    outcomes = [...q.outcomes];

    if (q.question_type === 'multiple_choice') {
      const mc = q as MultipleChoiceQuestion;
      points        = mc.points;
      const rawBody = extractTypstRaw(mc.body);
      if (rawBody !== null) {
        mcTfEditorMode = 'typst';
        mcTfTypstBody = rawBody;
        mcTfBodyNodes = [];
      } else {
        mcTfEditorMode = 'rich';
        mcTfTypstBody = '';
        mcTfBodyNodes = mc.body;
      }
      mcOptions     = mc.options.map((o) => ({ id: o.id, text: extractText(o.body) }));
      mcCorrectId   = mc.options.find((o) => o.correct)?.id ?? 'A';
      shuffle       = mc.shuffle;
    } else if (q.question_type === 'true_false') {
      const tf = q as TrueFalseQuestion;
      points        = tf.points;
      const rawBody = extractTypstRaw(tf.body);
      if (rawBody !== null) {
        mcTfEditorMode = 'typst';
        mcTfTypstBody = rawBody;
        mcTfBodyNodes = [];
      } else {
        mcTfEditorMode = 'rich';
        mcTfTypstBody = '';
        mcTfBodyNodes = tf.body;
      }
      tfAnswer      = tf.correct_answer;
    } else if (q.question_type === 'fill_in_blank') {
      const fib = q as FillInBlankQuestion;
      fibBodyText = extractText(fib.body);
      fibBlanks   = fib.blanks.map((b) => ({
        id:               b.id,
        accepted_answers: b.accepted_answers.join(', '),
        points:           b.points,
        case_sensitive:   b.case_sensitive,
      }));
    } else if (q.question_type === 'classic') {
      const cl = q as ClassicQuestion;
      points           = cl.points;
      const rawBody = extractTypstRaw(cl.body);
      if (rawBody !== null) {
        classicEditorMode = 'typst';
        classicTypstBody = rawBody;
        classicBodyNodes = [];
      } else {
        classicEditorMode = 'rich';
        classicTypstBody = '';
        classicBodyNodes = cl.body;
      }
      rubric           = cl.rubric.map((r) => ({ criterion: r.criterion, points: r.points }));
      const as_ = cl.answer_space as AnswerSpace;
      if ('Lines' in as_)     { answerSpaceType = 'lines';  answerLines    = as_.Lines;        }
      else if ('HeightCm' in as_) { answerSpaceType = 'height'; answerHeightCm = as_.HeightCm; }
      else if ('Grid' in as_) { answerSpaceType = 'grid';   answerGridRows = as_.Grid.rows; answerGridCols = as_.Grid.cols; }
    }
  }

  onMount(async () => {
    try {
      const list = await api.questions.list();
      const q    = list.find((x) => x.id === questionId) ?? null;
      if (!q) { loadError = 'Soru bulunamadı.'; return; }
      original = q;
      populate(q);
    } catch (e) {
      loadError = String(e);
    } finally {
      loaded = true;
    }
  });

  function onFibChange(v: { bodyText: string; blanks: BlankDraft[] }) {
    fibBodyText = v.bodyText;
    fibBlanks   = v.blanks;
  }

  // ── build updated Question keeping original id + stats ───────────────────────
  async function submit() {
    if (!original) return;
    submitError = null;
    submitting  = true;
    try {
      const type = original.question_type;

      if (type === 'multiple_choice') {
        const body = mcTfEditorMode === 'typst' ? asTypstRaw(mcTfTypstBody) : mcTfBodyNodes;
        if (mcTfEditorMode === 'typst' ? !mcTfTypstBody.trim() : !bodyHasRichContent(mcTfBodyNodes)) {
          submitError = 'Soru gövdesi boş olamaz.'; return;
        }
        if (mcOptions.filter((o) => o.text.trim()).length < 2) { submitError = 'En az 2 seçenek gereklidir.'; return; }
        const updated: MultipleChoiceQuestion = {
          ...(original as MultipleChoiceQuestion),
          points,
          outcomes,
          body,
          options: mcOptions.map((o) => ({
            id:      o.id,
            body:    api.textBody(o.text.trim() || '—'),
            correct: o.id === mcCorrectId,
          })),
          shuffle,
        };
        await api.questions.update(updated);

      } else if (type === 'true_false') {
        const body = mcTfEditorMode === 'typst' ? asTypstRaw(mcTfTypstBody) : mcTfBodyNodes;
        if (mcTfEditorMode === 'typst' ? !mcTfTypstBody.trim() : !bodyHasRichContent(mcTfBodyNodes)) {
          submitError = 'Soru gövdesi boş olamaz.'; return;
        }
        const updated: TrueFalseQuestion = {
          ...(original as TrueFalseQuestion),
          points,
          outcomes,
          body,
          correct_answer: tfAnswer,
        };
        await api.questions.update(updated);

      } else if (type === 'fill_in_blank') {
        if (!fibBodyText.trim()) { submitError = 'Soru gövdesi boş olamaz.'; return; }
        if (fibBlanks.length === 0) { submitError = 'En az bir [B1] boşluğu ekleyin.'; return; }
        const emptyBlanks = fibBlanks.filter((b) => !b.accepted_answers.trim());
        if (emptyBlanks.length > 0) { submitError = `${emptyBlanks.map((b) => b.id).join(', ')} için doğru cevap girilmemiş.`; return; }
        const updated: FillInBlankQuestion = {
          ...(original as FillInBlankQuestion),
          outcomes,
          body:   api.textBody(fibBodyText.trim()),
          blanks: fibBlanks.map((b) => ({
            id:               b.id,
            accepted_answers: b.accepted_answers.split(',').map((s) => s.trim()).filter(Boolean),
            points:           b.points,
            case_sensitive:   b.case_sensitive,
          })),
        };
        await api.questions.update(updated);

      } else if (type === 'classic') {
        const body = classicEditorMode === 'typst' ? asTypstRaw(classicTypstBody) : classicBodyNodes;
        if (classicEditorMode === 'typst' ? !classicTypstBody.trim() : !bodyHasRichContent(classicBodyNodes)) {
          submitError = 'Soru gövdesi boş olamaz.'; return;
        }
        const answer_space: AnswerSpace =
          answerSpaceType === 'lines'  ? { Lines: answerLines }
          : answerSpaceType === 'height' ? { HeightCm: answerHeightCm }
          : { Grid: { rows: answerGridRows, cols: answerGridCols } };
        const updated: ClassicQuestion = {
          ...(original as ClassicQuestion),
          points,
          outcomes,
          body,
          rubric:  rubric.map((r) => ({ criterion: r.criterion, points: r.points })),
          answer_space,
        };
        await api.questions.update(updated);
      }

      goto('/questions');
    } catch (e) {
      submitError = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

{#if !loaded}
  <div class="p-6 text-muted-foreground text-sm">Yükleniyor…</div>
{:else if loadError || !original}
  <div class="p-6">
    <div class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
      {loadError ?? 'Soru bulunamadı.'}
    </div>
  </div>
{:else}

<div class="p-6 max-w-2xl mx-auto">
  <div class="flex items-center gap-3 mb-6">
    <a href="/questions" class="text-muted-foreground hover:text-foreground text-sm">← Soru Bankası</a>
    <span class="text-muted-foreground">/</span>
    <h1 class="text-xl font-bold">Soruyu Düzenle</h1>
    <span class="text-xs rounded-full px-2.5 py-0.5 font-medium bg-muted text-muted-foreground">
      {QUESTION_TYPE_LABELS[original.question_type]}
    </span>
  </div>

  <form onsubmit={(e) => { e.preventDefault(); submit(); }} class="space-y-5">

    {#if original.question_type === 'multiple_choice' || original.question_type === 'true_false'}
      <div class="space-y-1.5">
        <div class="flex items-center justify-between gap-3">
          <p class="text-sm font-medium">Soru Gövdesi</p>
          <div class="flex items-center gap-1 rounded-md border border-input p-0.5 text-xs">
            <button
              type="button"
              onclick={() => (mcTfEditorMode = 'rich')}
              class="rounded px-2 py-1 transition-colors {mcTfEditorMode === 'rich' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}"
            >
              Zengin
            </button>
            <button
              type="button"
              onclick={() => (mcTfEditorMode = 'typst')}
              class="rounded px-2 py-1 transition-colors {mcTfEditorMode === 'typst' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}"
            >
              Typst
            </button>
          </div>
        </div>
        {#if mcTfEditorMode === 'rich'}
          <RichBodyEditor nodes={mcTfBodyNodes} onchange={(v) => (mcTfBodyNodes = v)} />
        {:else}
          <TypstBodyEditor value={mcTfTypstBody} onchange={(v) => (mcTfTypstBody = v)} rows={12} />
        {/if}
      </div>
    {/if}

    {#if original.question_type === 'classic'}
      <div class="space-y-1.5">
        <div class="flex items-center justify-between gap-3">
          <p class="text-sm font-medium">Soru Gövdesi</p>
          <div class="flex items-center gap-1 rounded-md border border-input p-0.5 text-xs">
            <button
              type="button"
              onclick={() => (classicEditorMode = 'rich')}
              class="rounded px-2 py-1 transition-colors {classicEditorMode === 'rich' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}"
            >
              Zengin
            </button>
            <button
              type="button"
              onclick={() => (classicEditorMode = 'typst')}
              class="rounded px-2 py-1 transition-colors {classicEditorMode === 'typst' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}"
            >
              Typst
            </button>
          </div>
        </div>
        {#if classicEditorMode === 'rich'}
          <RichBodyEditor nodes={classicBodyNodes} onchange={(v) => (classicBodyNodes = v)} />
        {:else}
          <TypstBodyEditor value={classicTypstBody} onchange={(v) => (classicTypstBody = v)} rows={12} />
        {/if}
      </div>
    {/if}

    {#if original.question_type === 'multiple_choice'}
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

    {#if original.question_type === 'true_false'}
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

    {#if original.question_type === 'fill_in_blank'}
      <BlankBodyEditor bodyText={fibBodyText} blanks={fibBlanks} onchange={onFibChange} />
    {/if}

    {#if original.question_type === 'classic'}
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

    {#if original.question_type !== 'fill_in_blank'}
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
{/if}
