<script lang="ts">
  import { onMount } from "svelte";
  import PageHead from "$lib/components/shell/PageHead.svelte";
  import RuledField from "$lib/components/shell/RuledField.svelte";
  import SelectBox from "$lib/components/shell/SelectBox.svelte";
  import ScoreHistogram from "$lib/components/measure/ScoreHistogram.svelte";
  import AnswerGrid from "$lib/components/measure/AnswerGrid.svelte";
  import ResultEntry from "$lib/components/measure/ResultEntry.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import type { Classroom, Exam, ExamResult, Question, Student } from "$lib/types";

  let exams = $state<Exam[]>([]);
  let classrooms = $state<Classroom[]>([]);
  let students = $state<Student[]>([]);
  let results = $state<ExamResult[]>([]);
  /** Banka; sınavın soru atıflarını çözmek için gerekli. */
  let bank = $state<Question[]>([]);

  type Sekme = "giris" | "analiz";
  let sekme = $state<Sekme>("giris");

  let examId = $state<string>("");
  let classroomId = $state<string>("");

  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      [exams, classrooms, bank] = await Promise.all([
        api.exams.list(),
        api.students.listClassrooms(),
        api.questions.list(),
      ]);
      const published = exams.filter((e) => e.status !== "Draft");
      if (published.length > 0) examId = published[0].id;
      if (classrooms.length > 0) classroomId = classrooms[0].id;
      error = null;
    } catch (err: unknown) {
      error = errorText(err);
    } finally {
      loading = false;
    }
  });

  $effect(() => {
    if (examId) void loadResults(examId);
  });

  $effect(() => {
    if (classroomId) void loadStudents(classroomId);
  });

  async function loadResults(id: string) {
    try {
      results = await api.results.getByExam(id);
      error = null;
    } catch (err: unknown) {
      error = errorText(err);
    }
  }

  async function loadStudents(id: string) {
    try {
      students = await api.students.listByClassroom(id);
      error = null;
    } catch (err: unknown) {
      error = errorText(err);
    }
  }

  let selectedExam = $derived(exams.find((e) => e.id === examId) ?? null);

  let questionIds = $derived(
    selectedExam
      ? selectedExam.questions
          .slice()
          .sort((a, b) => a.display_order - b.display_order)
          .map((q) => q.question_id)
      : [],
  );

  let classResults = $derived(
    results.filter((r) => students.some((s) => s.id === r.student_id)),
  );

  let percentages = $derived(
    classResults
      .filter((r) => r.total_points_max > 0)
      .map((r) => (r.total_points_earned / r.total_points_max) * 100),
  );

  let summary = $derived.by(() => {
    if (percentages.length === 0) return null;
    const sorted = [...percentages].sort((a, b) => a - b);
    return {
      count: sorted.length,
      mean: sorted.reduce((sum, p) => sum + p, 0) / sorted.length,
      min: sorted[0],
      max: sorted[sorted.length - 1],
      median: sorted[Math.floor(sorted.length / 2)],
      failing: sorted.filter((p) => p < 50).length,
    };
  });
</script>

<div class="flex h-full min-h-0 flex-col">
  <PageHead title="Sınav analizi" />

  <div class="ruled-bottom flex shrink-0 flex-wrap items-end gap-rule bg-paper px-rule py-half paper-plain">
    <div class="w-[260px]">
      <RuledField label="Sınav">
        <SelectBox
          value={examId}
          options={exams.map((e) => ({ value: e.id, label: e.meta.title }))}
          emptyLabel="— seç —"
          onchange={(v) => (examId = v)}
        />
      </RuledField>
    </div>

    <div class="w-[160px]">
      <RuledField label="Sınıf">
        <SelectBox
          value={classroomId}
          options={classrooms.map((c) => ({ value: c.id, label: c.name }))}
          emptyLabel="— seç —"
          onchange={(v) => (classroomId = v)}
        />
      </RuledField>
    </div>

    <div class="flex items-stretch border border-rule-strong">
      {#each [["giris", "Sonuç girişi"], ["analiz", "Analiz"]] as [id, label]}
        <button
          type="button"
          class="border-r border-rule px-half py-quarter text-[12px] leading-rule
                 transition-colors last:border-r-0 hover:text-red-deep"
          class:bg-paper-sunk={sekme === id}
          class:font-semibold={sekme === id}
          class:text-ink={sekme === id}
          class:text-pencil={sekme !== id}
          aria-pressed={sekme === id}
          onclick={() => (sekme = id as Sekme)}
        >
          {label}
        </button>
      {/each}
    </div>

    {#if summary && sekme === "analiz"}
      <dl class="ml-auto flex items-baseline gap-rule">
        <div>
          <dt class="stamp">Ortalama</dt>
          <dd class="text-[19px] font-bold leading-rule tnum">{summary.mean.toFixed(0)}%</dd>
        </div>
        <div>
          <dt class="stamp">Ortanca</dt>
          <dd class="text-[19px] font-bold leading-rule tnum">{summary.median.toFixed(0)}%</dd>
        </div>
        <div>
          <dt class="stamp">En düşük / yüksek</dt>
          <dd class="text-[19px] font-bold leading-rule tnum">
            {summary.min.toFixed(0)} / {summary.max.toFixed(0)}
          </dd>
        </div>
        <div>
          <dt class="stamp">Eşiğin altında</dt>
          <dd class="text-[19px] font-bold leading-rule tnum text-red-deep">
            {summary.failing} / {summary.count}
          </dd>
        </div>
      </dl>
    {/if}
  </div>

  {#if error}
    <p class="ruled-bottom annot shrink-0 bg-red-wash px-rule py-quarter">{error}</p>
  {/if}

  <div class="flex min-h-0 flex-1 flex-col">
    {#if loading}
      <p class="pencil px-rule py-rule">Okunuyor…</p>
    {:else if !examId || !classroomId}
      <p class="pencil px-rule py-rule">Bir sınav ve bir sınıf seç.</p>
    {:else if selectedExam === null}
      <p class="pencil px-rule py-rule">Sınav bulunamadı.</p>
    {:else if sekme === "giris"}
      <ResultEntry
        exam={selectedExam}
        {students}
        {results}
        {bank}
        onsaved={() => {
          // Ölçüm her kayıtta yeniden hesaplanıyor; listeyi de tazele ki
          // öğrencinin yanındaki işaret hemen görünsün.
          void loadResults(examId);
        }}
      />
    {:else if classResults.length === 0}
      <p class="pencil px-rule py-rule">
        Bu sınav için bu sınıfta girilmiş sonuç yok. Sonuç girişi sekmesinden başla.
      </p>
    {:else}
      <div class="grid gap-rule px-rule py-half" style="grid-template-columns: minmax(280px, 380px) 1fr">
        <ScoreHistogram {percentages} />
        <AnswerGrid results={classResults} {students} {questionIds} />
      </div>
    {/if}
  </div>
</div>
