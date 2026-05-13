<script lang="ts">
  import { page } from '$app/state';
  import { api } from '$lib/api';
  import ExamPreview from '$lib/components/ExamPreview.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import {
    EXAM_STATUS_LABELS, QUESTION_TYPE_LABELS,
    type Classroom,
    type Exam,
    type ExamResult,
    type ExamStatus,
    type Question,
    type QuestionAnswerInput,
    type Student,
  } from '$lib/types';
  import { onMount } from 'svelte';

  const examId = $derived(page.params.id!);

  let exam        = $state<Exam | null>(null);
  let questions   = $state<Question[]>([]);   // bank — all available
  let loading     = $state(true);
  let error       = $state<string | null>(null);
  let adding      = $state<string | null>(null);
  let removing    = $state<string | null>(null);
  let publishing  = $state(false);
  let showPicker  = $state(false);
  let exporting      = $state(false);
  let filterText     = $state('');
  let publishError   = $state<string | null>(null);
  let exportMessage  = $state<{ ok: boolean; text: string } | null>(null);
  let confirmPublish = $state(false);
  let showPreview    = $state(false);
  let previewAnswers = $state(false);

  // ── Result entry state ────────────────────────────────────────────────────
  let showEntry       = $state(false);
  let classrooms      = $state<Classroom[]>([]);
  let students        = $state<Student[]>([]);
  let existingResults = $state<ExamResult[]>([]);
  let entryClassId    = $state('');
  let entryStudId     = $state('');
  let entryStep       = $state<1 | 2>(1);
  let answerDraft     = $state<Record<string, string | null>>({});
  let classicPts      = $state<Record<string, number>>({});
  let entryError      = $state<string | null>(null);
  let entrySaving     = $state(false);
  let entrySuccess    = $state(false);

  // questions already in this exam (by id)
  let examQIds    = $derived(new Set(exam?.questions.map((q) => q.question_id) ?? []));

  // question bank sorted by display_order, then enriched with Question data
  type ExamItem = {
    ref:      Exam['questions'][number];
    question: Question | undefined;
  };
  let examItems = $derived<ExamItem[]>(
    (exam?.questions ?? [])
      .slice()
      .sort((a, b) => a.display_order - b.display_order)
      .map((ref) => ({ ref, question: questions.find((q) => q.id === ref.question_id) }))
  );

  let totalPoints = $derived(
    examItems.reduce((sum, item) => {
      const pts = item.ref.points_override ?? (item.question ? questionPoints(item.question) : 0);
      return sum + pts;
    }, 0)
  );

  // filtered bank for picker
  let filteredBank = $derived(
    questions
      .filter((q) => !examQIds.has(q.id))
      .filter((q) => {
        const t = filterText.trim().toLowerCase();
        if (!t) return true;
        const label = QUESTION_TYPE_LABELS[q.question_type].toLowerCase();
        const body  = q.body
          .map((n) => (n.type === 'text' ? n.text : ''))
          .join(' ')
          .toLowerCase();
        return label.includes(t) || body.includes(t);
      })
  );

  onMount(async () => {
    try {
      [exam, questions] = await Promise.all([
        api.exams.get(examId),
        api.questions.list(),
      ]);
    } catch (e) { error = String(e); }
    finally { loading = false; }
  });

  async function addQuestion(questionId: string) {
    if (!exam) return;
    adding = questionId;
    try {
      await api.exams.addQuestion(examId, questionId);
      exam = await api.exams.get(examId);
    } catch (e) { alert(String(e)); }
    finally { adding = null; }
  }

  async function removeQuestion(questionId: string) {
    if (!exam) return;
    if (removing !== questionId) { removing = questionId; return; } // first click = arm
    try {
      await api.exams.removeQuestion(examId, questionId);
      exam = await api.exams.get(examId);
      removing = null;
    } catch (e) { removing = null; }
  }

  async function publish() {
    if (!exam || exam.questions.length === 0) return;
    if (!confirmPublish) { confirmPublish = true; return; }
    confirmPublish = false;
    publishError   = null;
    publishing     = true;
    try {
      await api.exams.publish(examId);
      exam = await api.exams.get(examId);
    } catch (e) { publishError = String(e); }
    finally { publishing = false; }
  }

  async function exportTypst(answerKey: boolean) {
    if (!exam) return;
    exportMessage = null;
    exporting     = true;
    try {
      const path = await api.compiler.exportTypstFile(examId, answerKey);
      exportMessage = { ok: true,  text: `Kaydedildi: ${path}` };
    } catch (e) { exportMessage = { ok: false, text: String(e) }; }
    finally { exporting = false; }
  }

  function questionPoints(q: Question): number {
    if (q.question_type === 'fill_in_blank') return q.blanks.reduce((s, b) => s + b.points, 0);
    return q.points;
  }

  function bodyPreview(q: Question): string {
    return q.body
      .map((n) => {
        if (n.type === 'text')    return n.text;
        if (n.type === 'math')    return `$${n.raw}$`;
        if (n.type === 'image')   return '[resim]';
        if (n.type === 'newline') return ' ';
        return '';
      })
      .join('')
      .slice(0, 90) + (q.body.map(n => n.type === 'text' ? n.text : '').join('').length > 90 ? '…' : '');
  }

  const STATUS_COLORS: Record<ExamStatus, string> = {
    Draft:     'bg-muted text-muted-foreground',
    Published: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
    Archived:  'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400',
  };

  // ── Result entry helpers ──────────────────────────────────────────────────
  async function openResultEntry() {
    showEntry       = true;
    entryStep       = 1;
    entryClassId    = '';
    entryStudId     = '';
    answerDraft     = {};
    classicPts      = {};
    entryError      = null;
    entrySuccess    = false;
    if (classrooms.length === 0) {
      try { classrooms = await api.students.listClassrooms(); } catch { /* ignore */ }
    }
    // always refresh results so we have up-to-date data
    try { existingResults = await api.results.getByExam(examId); } catch { /* ignore */ }
  }

  async function changeEntryClassroom(classId: string) {
    entryClassId = classId;
    entryStudId  = '';
    students     = [];
    if (classId) {
      try { students = await api.students.listByClassroom(classId); } catch { /* ignore */ }
    }
  }

  function goToAnswerStep() {
    if (!entryStudId) return;
    const draft: Record<string, string | null> = {};
    const pts:   Record<string, number>        = {};

    // check for existing result for this student
    const existing = existingResults.find((r) => r.student_id === entryStudId);

    for (const item of examItems) {
      if (!item.question) continue;
      const q = item.question;
      const prevAns = existing?.answers.find((a) => a.question_id === q.id);

      if (q.question_type === 'fill_in_blank') {
        // prevAns.given_answer is JSON map of blank_id → value
        let prevMap: Record<string, string> = {};
        if (prevAns?.given_answer) {
          try { prevMap = JSON.parse(prevAns.given_answer); } catch { /* ignore */ }
        }
        for (const b of q.blanks) { draft[`${q.id}__${b.id}`] = prevMap[b.id] ?? ''; }
      } else if (q.question_type === 'classic') {
        pts[q.id] = prevAns?.points_earned ?? 0;
      } else {
        draft[q.id] = prevAns?.given_answer ?? null;
      }
    }
    answerDraft = draft;
    classicPts  = pts;
    entryStep   = 2;
  }

  async function submitEntry() {
    if (!exam || !entryStudId) return;
    entrySaving  = true;
    entryError   = null;
    try {
      const answers: QuestionAnswerInput[] = examItems
        .filter((item) => item.question)
        .map((item) => {
          const q      = item.question!;
          const maxPts = item.ref.points_override ?? questionPoints(q);
          if (q.question_type === 'fill_in_blank') {
            const map: Record<string, string> = {};
            for (const b of q.blanks) { map[b.id] = answerDraft[`${q.id}__${b.id}`] ?? ''; }
            return { question_id: q.id, given_answer: JSON.stringify(map), points_earned: 0, is_correct: null };
          } else if (q.question_type === 'classic') {
            const earned = Math.min(Math.max(0, classicPts[q.id] ?? 0), maxPts);
            return { question_id: q.id, given_answer: null, points_earned: earned, is_correct: null };
          } else {
            return { question_id: q.id, given_answer: answerDraft[q.id] ?? null, points_earned: 0, is_correct: null };
          }
        });
      await api.results.enter({ examId, studentId: entryStudId, answers, totalMax: totalPoints });
      existingResults = await api.results.getByExam(examId);
      entrySuccess = true;
      entryStep    = 1;
      entryStudId  = '';
    } catch (e) {
      entryError = String(e);
    } finally {
      entrySaving = false;
    }
  }

  let entryStudName = $derived(
    (() => {
      const s = students.find((x) => x.id === entryStudId);
      return s ? `${s.first_name} ${s.last_name}` : '';
    })()
  );
</script>

{#if loading}
  <div class="p-6 text-muted-foreground">Yükleniyor…</div>
{:else if error || !exam}
  <div class="p-6">
    <div class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
      {error ?? 'Sınav bulunamadı.'}
    </div>
  </div>
{:else}

<div class="p-6 max-w-5xl mx-auto space-y-6">

  <!-- Header -->
  <div class="flex items-start gap-4 flex-wrap">
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2 text-sm text-muted-foreground mb-1">
        <a href="/exams" class="hover:text-foreground">← Sınavlar</a>
        <span>/</span>
        <span>{exam.meta.subject} · {exam.meta.classroom}</span>
      </div>
      <h1 class="text-2xl font-bold truncate">{exam.meta.title}</h1>
      <p class="text-sm text-muted-foreground mt-0.5">
        {exam.meta.teacher} · {exam.meta.duration_min} dk ·
        {new Date(exam.meta.date).toLocaleDateString('tr-TR', { day: '2-digit', month: 'long', year: 'numeric' })}
      </p>
    </div>
    <div class="flex items-center gap-2 shrink-0">
      <span class="rounded-full px-2.5 py-0.5 text-xs font-medium {STATUS_COLORS[exam.status]}">
        {EXAM_STATUS_LABELS[exam.status]}
      </span>
      {#if exam.questions.length > 0}
        <div class="flex flex-col items-end gap-1">
          <div class="flex items-center gap-1">
            <button
              type="button"
              onclick={() => { previewAnswers = false; showPreview = true; }}
              disabled={exporting}
              title="Sınav önizlemesi"
              class="inline-flex items-center gap-1 rounded-md border px-3 py-1.5 text-xs font-medium
                     hover:bg-accent transition-colors"
            >
              👁 Önizle
            </button>
            <button
              type="button"
              onclick={() => exportTypst(false)}
              disabled={exporting}
              title="Typst kaynak dosyasını indir"
              class="inline-flex items-center gap-1 rounded-md border px-3 py-1.5 text-xs font-medium
                     hover:bg-accent transition-colors disabled:opacity-50"
            >
              ↓ Typst
            </button>
            <button
              type="button"
              onclick={() => exportTypst(true)}
              disabled={exporting}
              title="Cevap anahtarı ile Typst indir"
              class="inline-flex items-center gap-1 rounded-md border px-3 py-1.5 text-xs font-medium
                     hover:bg-accent transition-colors disabled:opacity-50"
            >
              ↓ Cevaplı
            </button>
          </div>
          {#if exportMessage}
            <p class="text-xs max-w-xs truncate {exportMessage.ok ? 'text-green-600' : 'text-destructive'}">
              {exportMessage.text}
            </p>
          {/if}
        </div>
      {/if}
      {#if exam.status === 'Draft'}
        <div class="flex flex-col items-end gap-1">
          {#if confirmPublish}
            <div class="flex items-center gap-2">
              <span class="text-xs text-muted-foreground">Yayınlanan sınav düzenlenemez.</span>
              <Button onclick={publish} disabled={publishing}>
                {publishing ? 'Yayınlanıyor…' : 'Evet, Yayınla'}
              </Button>
              <button
                type="button"
                onclick={() => (confirmPublish = false)}
                class="text-xs text-muted-foreground hover:text-foreground"
              >İptal</button>
            </div>
          {:else}
            <Button onclick={publish} disabled={publishing || exam.questions.length === 0}>
              {publishing ? 'Yayınlanıyor…' : 'Yayınla'}
            </Button>
          {/if}
          {#if publishError}
            <p class="text-xs text-destructive max-w-xs">{publishError}</p>
          {/if}
        </div>
      {/if}
      {#if exam.status === 'Published'}
        <Button onclick={openResultEntry}>
          + Sonuç Gir
        </Button>
      {/if}
    </div>
  </div>

  <!-- Questions in exam -->
  <div class="space-y-2">
    <div class="flex items-center justify-between">
      <h2 class="font-semibold">Sorular
        <span class="ml-1.5 text-sm font-normal text-muted-foreground">
          ({examItems.length} soru · {totalPoints} puan)
        </span>
      </h2>
      {#if exam.status === 'Draft'}
        <Button variant="outline" size="sm" onclick={() => (showPicker = !showPicker)}>
          {showPicker ? 'Gizle' : '+ Soru Ekle'}
        </Button>
      {/if}
    </div>

    {#if examItems.length === 0}
      <div class="rounded-lg border bg-card p-10 text-center text-sm text-muted-foreground">
        Henüz soru eklenmedi.
      </div>
    {:else}
      <div class="rounded-lg border overflow-hidden">
        <table class="w-full text-sm">
          <thead class="bg-muted/50 text-muted-foreground">
            <tr>
              <th class="w-10 px-3 py-2 text-center font-medium">#</th>
              <th class="px-4 py-2 text-left font-medium">Soru</th>
              <th class="px-4 py-2 text-left font-medium">Tip</th>
              <th class="px-4 py-2 text-right font-medium">Puan</th>
              {#if exam.status === 'Draft'}<th class="w-10 px-2 py-2"></th>{/if}
            </tr>
          </thead>
          <tbody class="divide-y">
            {#each examItems as item (item.ref.question_id)}
              <tr class="hover:bg-muted/30 transition-colors group">
                <td class="px-3 py-2.5 text-center text-muted-foreground">{item.ref.display_order}</td>
                <td class="px-4 py-2.5 max-w-xs truncate">
                  {#if item.question}
                    {bodyPreview(item.question)}
                  {:else}
                    <span class="text-muted-foreground italic">—</span>
                  {/if}
                </td>
                <td class="px-4 py-2.5 text-muted-foreground">
                  {item.question ? QUESTION_TYPE_LABELS[item.question.question_type] : '—'}
                </td>
                <td class="px-4 py-2.5 text-right">
                  {item.ref.points_override ?? (item.question ? questionPoints(item.question) : '—')}
                </td>
                {#if exam.status === 'Draft'}
                  <td class="px-2 py-2.5">
                    <button
                      type="button"
                      onclick={() => removeQuestion(item.ref.question_id)}
                      title={removing === item.ref.question_id ? 'Tekrar tıkla — onaylıyor' : 'Soruyu kaldır'}
                      class="opacity-0 group-hover:opacity-100 transition-opacity px-1.5 py-1
                             rounded text-xs disabled:opacity-50
                             {removing === item.ref.question_id
                               ? 'bg-destructive text-white opacity-100'
                               : 'text-destructive hover:bg-destructive/10'}"
                    >{removing === item.ref.question_id ? 'Emin misin?' : '✕'}</button>
                  </td>
                {/if}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>

  <!-- Question picker -->
  {#if showPicker && exam.status === 'Draft'}
    <div class="rounded-lg border bg-card space-y-3 p-4">
      <h3 class="font-medium text-sm">Soru Bankasından Ekle</h3>
      <input
        type="search"
        bind:value={filterText}
        placeholder="Ara (tip, metin)…"
        class="w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm
               focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring placeholder:text-muted-foreground"
      />
      {#if filteredBank.length === 0}
        <p class="text-sm text-muted-foreground py-4 text-center">
          {questions.length === examQIds.size ? 'Tüm sorular zaten eklendi.' : 'Eşleşen soru yok.'}
        </p>
      {:else}
        <div class="divide-y max-h-72 overflow-y-auto rounded-md border">
          {#each filteredBank as q (q.id)}
            <div class="flex items-center gap-3 px-3 py-2.5 hover:bg-muted/40 transition-colors">
              <span class="shrink-0 rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">
                {QUESTION_TYPE_LABELS[q.question_type]}
              </span>
              <span class="flex-1 text-sm truncate">{bodyPreview(q)}</span>
              <span class="shrink-0 text-xs text-muted-foreground">{questionPoints(q)} pt</span>
              <button
                type="button"
                disabled={adding === q.id}
                onclick={() => addQuestion(q.id)}
                class="shrink-0 rounded px-2.5 py-1 text-xs bg-primary text-primary-foreground
                       hover:bg-primary/90 transition-colors disabled:opacity-50"
              >
                {adding === q.id ? '…' : 'Ekle'}
              </button>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

</div>

<!-- ── Result Entry Modal ───────────────────────────────────────────────── -->
{#if showEntry}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-40 bg-black/40" onclick={() => (showEntry = false)}></div>
  <div class="fixed inset-0 z-50 overflow-y-auto flex items-start justify-center p-4 pointer-events-none">
    <div class="relative w-full max-w-xl bg-background rounded-xl border shadow-xl my-8 pointer-events-auto">

      <!-- Modal header -->
      <div class="flex items-center gap-3 px-5 py-4 border-b">
        {#if entryStep === 2}
          <button
            type="button"
            onclick={() => (entryStep = 1)}
            class="text-sm text-muted-foreground hover:text-foreground"
          >←</button>
        {/if}
        <h2 class="font-semibold flex-1">
          {#if entryStep === 1}Sonuç Gir — Öğrenci Seç{:else}Sonuç Gir — {entryStudName}{/if}
        </h2>
        <button
          type="button"
          onclick={() => (showEntry = false)}
          class="text-muted-foreground hover:text-foreground text-lg leading-none"
        >✕</button>
      </div>

      <!-- Step 1: Student picker -->
      {#if entryStep === 1}
        <div class="p-5 space-y-4">

          {#if entrySuccess}
            <div class="rounded-md border border-green-300 bg-green-50 dark:bg-green-900/20 px-3 py-2 text-sm text-green-700 dark:text-green-300">
              Sonuç kaydedildi. Başka bir öğrenci için tekrar seçebilirsiniz.
            </div>
          {/if}

          <div class="space-y-1.5">
            <label class="text-sm font-medium" for="entry-class">Sınıf</label>
            <select
              id="entry-class"
              class="w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm
                     focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              value={entryClassId}
              onchange={(e) => changeEntryClassroom((e.target as HTMLSelectElement).value)}
            >
              <option value="">— Sınıf seçin —</option>
              {#each classrooms as c}
                <option value={c.id}>{c.grade}/{c.branch} — {c.name} ({c.academic_year})</option>
              {/each}
            </select>
          </div>

          {#if students.length > 0}
            <div class="space-y-1.5">
              <p class="text-sm font-medium">Öğrenci</p>
              <div class="max-h-56 overflow-y-auto rounded-md border divide-y">
                {#each [...students].sort((a, b) => parseInt(a.number) - parseInt(b.number)) as s}
                  <button
                    type="button"
                    onclick={() => (entryStudId = s.id)}
                    class="w-full flex items-center gap-3 px-3 py-2.5 hover:bg-muted/40 text-left transition-colors
                           {entryStudId === s.id ? 'bg-primary/10 border-l-2 border-primary' : ''}"
                  >
                    <span class="font-mono text-xs text-muted-foreground w-10 shrink-0">{s.number}</span>
                    <span class="text-sm">{s.first_name} {s.last_name}</span>
                  </button>
                {/each}
              </div>
            </div>
          {:else if entryClassId}
            <p class="text-sm text-muted-foreground">Bu sınıfta öğrenci yok.</p>
          {/if}

          <div class="flex justify-end pt-1">
            <Button onclick={goToAnswerStep} disabled={!entryStudId}>
              Cevapları Gir →
            </Button>
          </div>
        </div>

      <!-- Step 2: Answer entry -->
      {:else}
        <div class="p-5 space-y-5 max-h-[70vh] overflow-y-auto">
          {#each examItems as item, idx (item.ref.question_id)}
            {#if item.question}
              {@const q      = item.question}
              {@const maxPts = item.ref.points_override ?? questionPoints(q)}
              <div class="rounded-lg border p-4 space-y-3">
                <!-- Question header -->
                <div class="flex items-start justify-between gap-2">
                  <div class="flex-1">
                    <span class="text-xs font-medium text-muted-foreground mr-1">{idx + 1}.</span>
                    <span class="text-sm">{bodyPreview(q)}</span>
                  </div>
                  <span class="text-xs bg-muted text-muted-foreground rounded px-1.5 py-0.5 shrink-0">
                    {QUESTION_TYPE_LABELS[q.question_type]} · {maxPts}p
                  </span>
                </div>

                <!-- MC options -->
                {#if q.question_type === 'multiple_choice'}
                  <div class="flex flex-wrap gap-2">
                    {#each q.options as opt}
                      <button
                        type="button"
                        onclick={() => { answerDraft = { ...answerDraft, [q.id]: opt.id }; }}
                        class="px-3 py-1.5 rounded-md border text-sm font-medium transition-colors
                               {answerDraft[q.id] === opt.id
                                 ? 'bg-primary text-primary-foreground border-primary'
                                 : 'hover:bg-accent'}"
                      >{opt.id}</button>
                    {/each}
                    <button
                      type="button"
                      onclick={() => { answerDraft = { ...answerDraft, [q.id]: null }; }}
                      class="px-3 py-1.5 rounded-md border text-sm text-muted-foreground transition-colors
                             {answerDraft[q.id] === null ? 'bg-muted' : 'hover:bg-accent'}"
                    >—</button>
                  </div>

                <!-- TF options -->
                {:else if q.question_type === 'true_false'}
                  <div class="flex gap-2">
                    {#each [['true', 'Doğru'], ['false', 'Yanlış']] as [val, label]}
                      <button
                        type="button"
                        onclick={() => { answerDraft = { ...answerDraft, [q.id]: val }; }}
                        class="px-4 py-1.5 rounded-md border text-sm font-medium transition-colors
                               {answerDraft[q.id] === val
                                 ? 'bg-primary text-primary-foreground border-primary'
                                 : 'hover:bg-accent'}"
                      >{label}</button>
                    {/each}
                    <button
                      type="button"
                      onclick={() => { answerDraft = { ...answerDraft, [q.id]: null }; }}
                      class="px-3 py-1.5 rounded-md border text-sm text-muted-foreground transition-colors
                             {answerDraft[q.id] === null ? 'bg-muted' : 'hover:bg-accent'}"
                    >—</button>
                  </div>

                <!-- FillInBlank inputs -->
                {:else if q.question_type === 'fill_in_blank'}
                  <div class="space-y-2">
                    {#each q.blanks as b}
                      <div class="flex items-center gap-2">
                        <span class="text-xs font-mono text-muted-foreground w-8 shrink-0">{b.id}</span>
                        <input
                          type="text"
                          placeholder="Cevap…"
                          value={answerDraft[`${q.id}__${b.id}`] ?? ''}
                          oninput={(e) => {
                            answerDraft = { ...answerDraft, [`${q.id}__${b.id}`]: (e.target as HTMLInputElement).value };
                          }}
                          class="flex-1 rounded-md border border-input bg-background px-2.5 py-1 text-sm
                                 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        />
                        <span class="text-xs text-muted-foreground shrink-0">{b.points}p</span>
                      </div>
                    {/each}
                  </div>

                <!-- Classic: manual points -->
                {:else if q.question_type === 'classic'}
                  <div class="flex items-center gap-2">
                    <label class="text-sm text-muted-foreground" for="cp-{q.id}">Puan:</label>
                    <input
                      id="cp-{q.id}"
                      type="number"
                      min="0"
                      max={maxPts}
                      step="0.5"
                      value={classicPts[q.id] ?? 0}
                      oninput={(e) => {
                        classicPts = { ...classicPts, [q.id]: parseFloat((e.target as HTMLInputElement).value) || 0 };
                      }}
                      class="w-20 rounded-md border border-input bg-background px-2.5 py-1 text-sm
                             focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    />
                    <span class="text-sm text-muted-foreground">/ {maxPts}</span>
                  </div>
                {/if}
              </div>
            {/if}
          {/each}
        </div>

        <!-- Footer -->
        <div class="px-5 py-4 border-t space-y-3">
          {#if entryError}
            <div class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
              {entryError}
            </div>
          {/if}
          <div class="flex justify-end gap-2">
            <Button variant="ghost" onclick={() => (entryStep = 1)}>← Geri</Button>
            <Button onclick={submitEntry} disabled={entrySaving}>
              {entrySaving
                ? 'Kaydediliyor…'
                : existingResults.some((r) => r.student_id === entryStudId)
                  ? 'Güncelle'
                  : 'Kaydet'}
            </Button>
          </div>
        </div>
      {/if}

    </div>
  </div>
{/if}

<!-- ── Exam Preview Modal ────────────────────────────────────────────────── -->
{#if showPreview}
  <ExamPreview
    {exam}
    {examItems}
    bind:showAnswerKey={previewAnswers}
    onclose={() => (showPreview = false)}
  />
{/if}

{/if}
