<script lang="ts">
  import type { ContentNode, Exam, Question } from '$lib/types';
  import { api } from '$lib/api';
  import katex from 'katex';
  import 'katex/dist/katex.min.css';

  let {
    exam,
    examItems,
    showAnswerKey = $bindable(false),
    onclose,
  }: {
    exam:        Exam;
    examItems:   Array<{ ref: Exam['questions'][number]; question: Question | undefined }>;
    showAnswerKey: boolean;
    onclose:     () => void;
  } = $props();

  let pdfExporting = $state(false);
  let pdfError     = $state<string | null>(null);

  async function downloadPdf() {
    pdfExporting = true;
    pdfError = null;
    try {
      const b64 = await api.compiler.exportPdf(exam.id, showAnswerKey);
      const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
      const blob = new Blob([bytes], { type: 'application/pdf' });
      const url  = URL.createObjectURL(blob);
      const a    = document.createElement('a');
      const suffix = showAnswerKey ? '_cevap' : '';
      a.href     = url;
      a.download = `${exam.meta.title}${suffix}.pdf`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      pdfError = String(e);
    } finally {
      pdfExporting = false;
    }
  }

  // ── Render helpers ────────────────────────────────────────────────────────

  function renderMath(raw: string, display: boolean): string {
    try {
      return katex.renderToString(raw, { displayMode: display, throwOnError: false });
    } catch {
      return `<code>${raw}</code>`;
    }
  }

  function renderNodes(nodes: ContentNode[]): string {
    return nodes.map((n) => {
      if (n.type === 'text') {
        let t = n.text
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;');
        if (n.style.bold)          t = `<strong>${t}</strong>`;
        if (n.style.italic)        t = `<em>${t}</em>`;
        if (n.style.underline)     t = `<u>${t}</u>`;
        if (n.style.strikethrough) t = `<s>${t}</s>`;
        return t;
      }
      if (n.type === 'math') return renderMath(n.raw, n.display === 'block');
      if (n.type === 'chem') return `<code class="font-mono text-sm bg-muted px-1 rounded">${n.raw}</code>`;
      if (n.type === 'image') return `<img src="${n.src}" alt="${n.alt}" style="max-width:${n.width ?? '100%'}" class="inline-block" />`;
      if (n.type === 'blank') return `<span class="inline-block border-b-2 border-current min-w-16 mx-1">&nbsp;</span>`;
      if (n.type === 'newline') return '<br />';
      return '';
    }).join('');
  }

  function totalPoints(): number {
    return examItems.reduce((sum, item) => {
      if (!item.question) return sum;
      const pts = item.ref.points_override ?? (
        item.question.question_type === 'fill_in_blank'
          ? item.question.blanks.reduce((s, b) => s + b.points, 0)
          : (item.question as { points: number }).points
      );
      return sum + pts;
    }, 0);
  }

  function qPoints(item: typeof examItems[number]): number {
    if (!item.question) return 0;
    if (item.ref.points_override != null) return item.ref.points_override;
    if (item.question.question_type === 'fill_in_blank')
      return item.question.blanks.reduce((s, b) => s + b.points, 0);
    return (item.question as { points: number }).points;
  }

  const LETTER = ['A', 'B', 'C', 'D', 'E', 'F'];
</script>

<!-- Backdrop -->
<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-40 bg-black/50" onclick={onclose}></div>

<!-- Modal -->
<div class="fixed inset-0 z-50 overflow-y-auto flex items-start justify-center p-6 pointer-events-none">
  <div class="pointer-events-auto w-full max-w-3xl bg-white dark:bg-zinc-900 rounded-xl shadow-2xl border my-4">

    <!-- Toolbar -->
    <div class="flex items-center gap-3 px-5 py-3 border-b bg-muted/30 rounded-t-xl">
      <span class="font-semibold text-sm flex-1 truncate">{exam.meta.title} — Önizleme</span>
      <label class="flex items-center gap-2 text-sm cursor-pointer select-none">
        <input type="checkbox" bind:checked={showAnswerKey} class="rounded" />
        Cevap Anahtarı
      </label>
      <button
        type="button"
        onclick={downloadPdf}
        disabled={pdfExporting}
        class="inline-flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50 transition-colors"
      >
        {#if pdfExporting}
          <span class="inline-block w-3 h-3 border-2 border-white/40 border-t-white rounded-full animate-spin"></span>
          Oluşturuluyor…
        {:else}
          PDF İndir
        {/if}
      </button>
      {#if pdfError}
        <span class="text-xs text-destructive max-w-xs truncate" title={pdfError}>{pdfError}</span>
      {/if}
      <button
        type="button"
        onclick={onclose}
        class="text-muted-foreground hover:text-foreground text-lg leading-none ml-2"
      >✕</button>
    </div>

    <!-- Exam sheet -->
    <div class="p-8 space-y-6 print:p-0" id="exam-preview-content">

      <!-- Header -->
      <div class="text-center space-y-1 pb-4 border-b-2">
        <h1 class="text-xl font-bold">{exam.meta.title}</h1>
        <p class="text-sm text-muted-foreground">
          {exam.meta.subject} &nbsp;|&nbsp;
          {exam.meta.classroom} &nbsp;|&nbsp;
          {exam.meta.teacher}
        </p>
        <p class="text-xs text-muted-foreground">
          Tarih: {new Date(exam.meta.date).toLocaleDateString('tr-TR', { day: '2-digit', month: 'long', year: 'numeric' })}
          &nbsp;·&nbsp; Süre: {exam.meta.duration_min} dk
          &nbsp;·&nbsp; Toplam: {totalPoints()} puan
        </p>
        <div class="flex justify-center gap-12 mt-3 text-sm">
          <span>Ad Soyad: ___________________________</span>
          <span>No: ________</span>
          <span>Puan: ________</span>
        </div>
      </div>

      <!-- Questions -->
      {#each examItems as item, idx (item.ref.question_id)}
        {#if item.question}
          {@const q = item.question}
          {@const pts = qPoints(item)}
          <div class="space-y-2">

            <!-- Question header -->
            <div class="flex gap-2 items-start">
              <span class="shrink-0 font-bold text-sm mt-0.5">{idx + 1}.</span>
              <div class="flex-1 min-w-0">
                <div class="prose prose-sm max-w-none dark:prose-invert leading-relaxed">
                  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                  {@html renderNodes(q.body)}
                </div>
              </div>
              <span class="shrink-0 text-xs text-muted-foreground whitespace-nowrap">({pts} pt)</span>
            </div>

            <!-- Multiple choice -->
            {#if q.question_type === 'multiple_choice'}
              <div class="ml-5 space-y-1">
                {#each q.options as opt, oi}
                  <div class="flex gap-2 items-start text-sm
                    {showAnswerKey && opt.correct ? 'font-semibold text-green-700 dark:text-green-400' : ''}">
                    <span class="shrink-0 font-medium">{LETTER[oi]})</span>
                    <span class="flex-1">
                      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                      {@html renderNodes(opt.body)}
                      {#if showAnswerKey && opt.correct}
                        <span class="ml-1 text-green-600">✓</span>
                      {/if}
                    </span>
                  </div>
                {/each}
              </div>

            <!-- True / False -->
            {:else if q.question_type === 'true_false'}
              <div class="ml-5 flex gap-4 text-sm">
                <span class="{showAnswerKey && q.correct_answer === true  ? 'font-bold text-green-700 dark:text-green-400' : ''}">
                  ( ) Doğru {#if showAnswerKey && q.correct_answer === true}✓{/if}
                </span>
                <span class="{showAnswerKey && q.correct_answer === false ? 'font-bold text-green-700 dark:text-green-400' : ''}">
                  ( ) Yanlış {#if showAnswerKey && q.correct_answer === false}✓{/if}
                </span>
              </div>

            <!-- Fill in blank -->
            {:else if q.question_type === 'fill_in_blank'}
              {#if showAnswerKey}
                <div class="ml-5 space-y-0.5">
                  {#each q.blanks as b, bi}
                    <p class="text-sm text-green-700 dark:text-green-400">
                      <span class="font-medium">Boşluk {bi + 1}:</span>
                      {b.accepted_answers.join(' / ')}
                      <span class="text-muted-foreground">({b.points} pt)</span>
                    </p>
                  {/each}
                </div>
              {/if}

            <!-- Classic -->
            {:else if q.question_type === 'classic'}
              {#if showAnswerKey && q.sample_answer && q.sample_answer.length > 0}
                <div class="ml-5 mt-1 rounded border-l-2 border-green-500 pl-3 text-sm text-green-800 dark:text-green-300">
                  <p class="text-xs font-semibold mb-0.5 text-green-600">Örnek Yanıt:</p>
                  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                  <div>{@html renderNodes(q.sample_answer)}</div>
                </div>
              {:else if !showAnswerKey}
                <div class="ml-5 mt-2">
                  {#if 'Lines' in q.answer_space}
                    {#each { length: q.answer_space.Lines } as _}
                      <div class="border-b border-gray-300 dark:border-gray-600 mt-5 w-full"></div>
                    {/each}
                  {:else if 'HeightCm' in q.answer_space}
                    <div class="border border-gray-300 dark:border-gray-600 rounded w-full"
                         style="height: {q.answer_space.HeightCm}cm"></div>
                  {:else if 'Grid' in q.answer_space}
                    <div class="border border-gray-300 dark:border-gray-600 rounded overflow-hidden"
                         style="display:grid; grid-template-columns: repeat({q.answer_space.Grid.cols}, 1fr)">
                      {#each { length: q.answer_space.Grid.rows * q.answer_space.Grid.cols } as _}
                        <div class="border border-gray-200 dark:border-gray-700 aspect-square"></div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}
            {/if}

          </div>
        {/if}
      {/each}

    </div>

    <!-- Footer -->
    <div class="flex justify-end gap-2 px-5 py-3 border-t bg-muted/30 rounded-b-xl">
      <span class="text-xs text-muted-foreground self-center">{examItems.length} soru · {totalPoints()} puan toplam</span>
      <button
        type="button"
        onclick={onclose}
        class="rounded-md border px-3 py-1.5 text-sm hover:bg-accent transition-colors"
      >Kapat</button>
    </div>

  </div>
</div>
