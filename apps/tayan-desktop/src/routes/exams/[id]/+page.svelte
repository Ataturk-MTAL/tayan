<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import PenButton from "$lib/components/shell/PenButton.svelte";
  import BudgetGauge from "$lib/components/measure/BudgetGauge.svelte";
  import QuestionStrip from "$lib/components/measure/QuestionStrip.svelte";
  import SheetPreview from "$lib/components/question/SheetPreview.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { bodySource } from "$lib/question/body";
  import {
    EXAM_STATUS_LABELS,
    QUESTION_TYPE_LABELS,
    questionPoints,
    type Exam,
    type Question,
  } from "$lib/types";

  /** Bir yazılının toplam puanı. Aşılırsa öğretmen bunu görmeli. */
  const POINT_TARGET = 100;

  let exam = $state<Exam | null>(null);
  let bank = $state<Question[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let actionError = $state<string | null>(null);
  let busy = $state(false);

  let answerKey = $state(false);
  let pages = $state<string[]>([]);
  let previewError = $state<string | null>(null);
  let compiling = $state(false);

  onMount(load);

  async function load() {
    loading = true;
    try {
      [exam, bank] = await Promise.all([
        api.exams.get(page.params.id as string),
        api.questions.list(),
      ]);
      loadError = null;
      await refreshPreview();
    } catch (err: unknown) {
      loadError = errorText(err);
    } finally {
      loading = false;
    }
  }

  async function refreshPreview() {
    if (!exam) return;
    compiling = true;
    try {
      const source = await api.compiler.generateTypst(exam.id, answerKey);
      pages = await api.compiler.previewSvg(source);
      previewError = null;
    } catch (err: unknown) {
      previewError = errorText(err);
    } finally {
      compiling = false;
    }
  }

  let selected = $derived.by(() => {
    if (!exam) return [] as Question[];
    return exam.questions
      .slice()
      .sort((a, b) => a.display_order - b.display_order)
      .map((ref) => bank.find((q) => q.id === ref.question_id))
      .filter((q): q is Question => q !== undefined);
  });

  let available = $derived(
    bank.filter((q) => !exam?.questions.some((ref) => ref.question_id === q.id)),
  );

  let totalPoints = $derived(selected.reduce((sum, q) => sum + questionPoints(q), 0));

  async function run(action: () => Promise<unknown>) {
    busy = true;
    actionError = null;
    try {
      await action();
      exam = await api.exams.get(page.params.id as string);
      await refreshPreview();
    } catch (err: unknown) {
      actionError = errorText(err);
    } finally {
      busy = false;
    }
  }

  function preview(q: Question): string {
    const source = bodySource(q.body).replace(/\s+/g, " ").trim();
    return source.length > 70 ? `${source.slice(0, 70)}…` : source || "(boş)";
  }

  async function exportPdf() {
    if (!exam) return;
    await run(async () => {
      const path = await api.compiler.exportPdf(exam!.id, answerKey);
      actionError = `PDF kaydedildi: ${path}`;
    });
  }
</script>

{#if loading}
  <p class="pencil p-rule">Sınav okunuyor…</p>
{:else if loadError}
  <p class="annot p-rule">{loadError}</p>
{:else if exam}
  <div class="flex h-full min-h-0 flex-col">
    <div class="ruled-bottom flex shrink-0 flex-wrap items-center gap-rule bg-paper-lift px-rule py-half paper-plain">
      <h1 class="text-[19px]">{exam.meta.title}</h1>
      <span class="pencil">
        {exam.meta.subject} · {exam.meta.classroom} · {exam.meta.date}
      </span>
      <span class="stamp" class:text-red-deep={exam.status === "Published"}>
        {EXAM_STATUS_LABELS[exam.status]}
      </span>

      <div class="ml-auto flex items-center gap-half">
        <label class="pencil flex items-center gap-quarter">
          <input type="checkbox" bind:checked={answerKey} onchange={refreshPreview} />
          Cevap anahtarı
        </label>
        <PenButton kind="quiet" disabled={busy} onclick={exportPdf}>PDF kaydet</PenButton>
        <PenButton
          kind="ink"
          disabled={busy || selected.length === 0 || exam.status === "Published"}
          onclick={() => run(() => api.exams.publish(exam!.id))}
        >
          Yayınla
        </PenButton>
      </div>
    </div>

    <QuestionStrip questions={selected} />

    <div class="ruled-bottom flex shrink-0 flex-wrap items-center gap-rule bg-paper px-rule py-half paper-plain">
      <BudgetGauge label="Puan" value={totalPoints} target={POINT_TARGET} unit="puan" />
      <BudgetGauge label="Soru" value={selected.length} target={20} unit="soru" />
      <span class="pencil">{exam.meta.duration_min} dk</span>
      {#if compiling}<span class="annot ml-auto">derleniyor…</span>{/if}
    </div>

    {#if actionError}
      <p class="ruled-bottom annot shrink-0 bg-red-wash px-rule py-quarter">{actionError}</p>
    {/if}

    <div class="grid min-h-0 flex-1 grid-cols-[minmax(280px,1fr)_minmax(300px,1fr)_minmax(320px,1.1fr)]">
      <section class="min-h-0 overflow-auto border-r border-rule-strong">
        <h2 class="stamp ruled-bottom sticky top-0 bg-paper px-rule py-quarter">
          Sınavdaki sorular
        </h2>
        {#if selected.length === 0}
          <p class="pencil p-rule">Henüz soru eklenmedi.</p>
        {:else}
          <ol>
            {#each selected as q, i (q.id)}
              <li class="flex items-start gap-half border-b border-rule px-rule py-half">
                <span class="stamp tnum w-[18px] shrink-0 pt-[2px]">{i + 1}</span>
                <div class="min-w-0 flex-1">
                  <p class="truncate font-mono text-[12px] text-ink-mid">{preview(q)}</p>
                  <p class="pencil">
                    {QUESTION_TYPE_LABELS[q.question_type]} · {questionPoints(q)} puan
                  </p>
                </div>
                <PenButton
                  kind="quiet"
                  disabled={busy}
                  onclick={() => run(() => api.exams.removeQuestion(exam!.id, q.id))}
                >
                  Çıkar
                </PenButton>
              </li>
            {/each}
          </ol>
        {/if}
      </section>

      <section class="min-h-0 overflow-auto border-r border-rule-strong">
        <h2 class="stamp ruled-bottom sticky top-0 bg-paper px-rule py-quarter">
          Bankadan ekle
        </h2>
        {#if available.length === 0}
          <p class="pencil p-rule">Eklenebilecek başka soru yok.</p>
        {:else}
          <ul>
            {#each available as q (q.id)}
              <li class="flex items-start gap-half border-b border-rule px-rule py-half">
                <div class="min-w-0 flex-1">
                  <p class="truncate font-mono text-[12px] text-ink-mid">{preview(q)}</p>
                  <p class="pencil">
                    {QUESTION_TYPE_LABELS[q.question_type]} · {questionPoints(q)} puan
                    {#if q.stats.times_used > 0 && q.stats.discrimination_index < 0.2}
                      <span class="text-red-deep">· ayırt ediciliği düşük</span>
                    {/if}
                  </p>
                </div>
                <PenButton
                  kind="quiet"
                  disabled={busy}
                  onclick={() => run(() => api.exams.addQuestion(exam!.id, q.id))}
                >
                  Ekle
                </PenButton>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="min-h-0">
        <SheetPreview {pages} stale={compiling} error={previewError} />
      </section>
    </div>
  </div>
{/if}
