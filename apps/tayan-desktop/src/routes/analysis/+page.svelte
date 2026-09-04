<script lang="ts">
  import { onMount } from "svelte";
  import { Alert, Button } from "flowbite-svelte";
  import PageShell from "$lib/components/shell/PageShell.svelte";
  import RuledField from "$lib/components/shell/RuledField.svelte";
  import SelectBox from "$lib/components/shell/SelectBox.svelte";
  import ScoreDistribution from "$lib/components/measure/ScoreDistribution.svelte";
  import ItemAnalysis from "$lib/components/measure/ItemAnalysis.svelte";
  import AnswerGrid from "$lib/components/measure/AnswerGrid.svelte";
  import ResultEntry from "$lib/components/measure/ResultEntry.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { itemStats, spread } from "$lib/analysis/item-stats";
  import { buildReport } from "$lib/analysis/report";
  import { examFileName } from "$lib/exam/filename";
  import { save } from "@tauri-apps/plugin-dialog";
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

  /** Geçme eşiği. Şimdilik sabit; sınav ayarına bağlanana kadar tek yerde. */
  const GECME_ESIGI = 50;

  let raporYaziliyor = $state(false);
  let raporDurumu = $state<string | null>(null);

  /**
   * Analiz raporunu PDF olarak kaydeder.
   *
   * ÖLÇÜLER EKRANDAN GİDER. Rust ikinci bir hesap yapmıyor; öğretmenin veliye
   * gösterdiği kâğıt ile ekranda gördüğü aynı sayıları taşımak zorunda.
   */
  async function raporKaydet() {
    if (!selectedExam) return;

    const rapor = buildReport({
      exam: selectedExam,
      items: maddeler,
      bank,
      results: classResults,
      students,
      threshold: GECME_ESIGI,
    });
    if (rapor === null) {
      raporDurumu = "Sonuç girilmemiş; rapor alınamaz.";
      return;
    }

    const hedef = await save({
      defaultPath: examFileName(selectedExam, {
        answerKey: false,
        booklet: null,
        extension: "pdf",
        suffix: "analiz",
      }),
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!hedef) return; // vazgeçildi

    raporYaziliyor = true;
    raporDurumu = null;
    try {
      const yol = await api.compiler.exportAnalysisPdf(rapor, hedef);
      raporDurumu = `Rapor kaydedildi: ${yol}`;
    } catch (err: unknown) {
      raporDurumu = errorText(err);
    } finally {
      raporYaziliyor = false;
    }
  }

  /** Yayılım ölçüleri: ortalama, ortanca, çeyrekler. */
  let dagilim = $derived(spread(percentages));

  /** Soru soru madde analizi — bu sınavın kendi sonuçlarından. */
  let maddeler = $derived(itemStats(classResults, questionIds, bank));

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

<PageShell title="Sınav analizi" subtitle={selectedExam?.meta.title ?? null} scroll={false}>
  {#snippet actions()}
    <!--
      PDF düğmesi başlığa taşındı (PageShell'in ortak eylem yuvası). Görünürlük
      eski davranışla aynı: yalnız analiz sekmesinde VE sınıfta sonuç varken.
    -->
    {#if sekme === "analiz" && classResults.length > 0}
      <Button size="sm" disabled={raporYaziliyor} onclick={raporKaydet}>
        {raporYaziliyor ? "Yazılıyor…" : "Analiz PDF"}
      </Button>
    {/if}
  {/snippet}

  <div class="flex h-full min-h-0 flex-col">
    <div
      class="flex shrink-0 flex-wrap items-end gap-5 border-b border-gray-300 bg-white px-5 py-2.5
             dark:border-gray-600 dark:bg-gray-800"
    >
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

      <div class="flex items-stretch border border-gray-300 dark:border-gray-600">
        {#each [["giris", "Sonuç girişi"], ["analiz", "Analiz"]] as [id, label]}
          <button
            type="button"
            class="border-r border-gray-200 px-2.5 py-[5px] text-[12px] leading-5
                   transition-colors last:border-r-0 hover:text-red-600 dark:border-gray-700
                   dark:hover:text-red-400
                   {sekme === id
                     ? 'bg-primary-50 font-semibold text-primary-700 dark:bg-primary-900/30 dark:text-primary-300'
                     : 'text-gray-500 dark:text-gray-400'}"
            aria-pressed={sekme === id}
            onclick={() => (sekme = id as Sekme)}
          >
            {label}
          </button>
        {/each}
      </div>

      {#if summary && sekme === "analiz"}
        <dl class="ml-auto flex items-baseline gap-5">
          <div>
            <dt class="text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
              Ortalama
            </dt>
            <dd class="tnum text-[19px] font-bold leading-5 text-gray-900 dark:text-white">
              {summary.mean.toFixed(0)}%
            </dd>
          </div>
          <div>
            <dt class="text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
              Ortanca
            </dt>
            <dd class="tnum text-[19px] font-bold leading-5 text-gray-900 dark:text-white">
              {summary.median.toFixed(0)}%
            </dd>
          </div>
          <div>
            <dt class="text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
              En düşük / yüksek
            </dt>
            <dd class="tnum text-[19px] font-bold leading-5 text-gray-900 dark:text-white">
              {summary.min.toFixed(0)} / {summary.max.toFixed(0)}
            </dd>
          </div>
          <div>
            <dt class="text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
              Eşiğin altında
            </dt>
            <!-- Eşiğin altındaki sayı gerçek bir değerlendirme bulgusu: kırmızı burada doğru yerinde. -->
            <dd class="tnum text-[19px] font-bold leading-5 text-red-600 dark:text-red-400">
              {summary.failing} / {summary.count}
            </dd>
          </div>
        </dl>
      {/if}
    </div>

    {#if error}
      <Alert color="red" rounded={false} class="shrink-0 border-b border-gray-300 text-[12px] leading-5 dark:border-gray-600">
        {error}
      </Alert>
    {/if}

    <div class="flex min-h-0 flex-1 flex-col">
      {#if loading}
        <p class="px-5 py-5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">Okunuyor…</p>
      {:else if !examId || !classroomId}
        <p class="px-5 py-5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">Bir sınav ve bir sınıf seç.</p>
      {:else if selectedExam === null}
        <p class="px-5 py-5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">Sınav bulunamadı.</p>
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
        <p class="px-5 py-5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">
          Bu sınav için bu sınıfta girilmiş sonuç yok. Sonuç girişi sekmesinden başla.
        </p>
      {:else}
        <p class="border-b border-gray-300 px-5 py-[5px] text-[12px] leading-5 text-gray-500 dark:border-gray-600 dark:text-gray-400">
          {#if raporDurumu}{raporDurumu}{:else}Yayılım, soru soru analiz ve öğrenci listesi tek PDF'te.{/if}
        </p>

        <!--
          KAYDIRMA BURADA. Belge kaydırması ana menüyü de yukarı taşıyordu;
          pencere sabit kalmalı, yalnız bu bölge kaymalı.
        -->
        <div class="min-h-0 flex-1 overflow-auto">
          <!--
            Eğri geniş sütunda: dağılımın şekli grafiğin asıl ürünü ve 380 px'lik
            sütunda tek bir tümsek gibi eziliyordu. Cevap ızgarası soru sayısı
            kadar yer kaplar, `auto` ile kendi genişliğini alıyor.
          -->
          <div
            class="grid items-start gap-5 px-5 py-2.5"
            style="grid-template-columns: minmax(420px, 1fr) auto"
          >
            <ScoreDistribution {percentages} stats={dagilim} threshold={GECME_ESIGI} />
            <AnswerGrid results={classResults} {students} {questionIds} />
          </div>

          <div class="px-5 pb-5">
            <ItemAnalysis items={maddeler} {bank} studentCount={classResults.length} />
          </div>
        </div>
      {/if}
    </div>
  </div>
</PageShell>
