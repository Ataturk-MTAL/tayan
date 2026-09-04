<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { Alert, Badge, Button, Checkbox, Input, Label, Select, Spinner } from "flowbite-svelte";
  import PageShell from "$lib/components/shell/PageShell.svelte";
  import BudgetGauge from "$lib/components/measure/BudgetGauge.svelte";
  import QuestionStrip from "$lib/components/measure/QuestionStrip.svelte";
  import SheetPreview from "$lib/components/question/SheetPreview.svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import { examFileName } from "$lib/exam/filename";
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

  const COLUMN_OPTIONS = [
    { value: "1", name: "Tek sütun" },
    { value: "2", name: "Çift sütun" },
  ];

  let exam = $state<Exam | null>(null);
  let bank = $state<Question[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);

  /**
   * Kâğıt ayarları paneli. Sütun, okul, alan ve imzalar yalnız oluşturma
   * formunda girilebiliyordu; kurulmuş sınavda değiştirilemiyordu. Oysa bunlar
   * BASKININ özellikleri ve öğretmen kâğıdı gördükten sonra fikir değiştirir.
   */
  let ayarlarAcik = $state(false);
  let ayarKaydediliyor = $state(false);
  let ayarHatasi = $state<string | null>(null);

  async function ayarlariKaydet() {
    if (!exam) return;
    ayarKaydediliyor = true;
    ayarHatasi = null;
    try {
      await api.exams.updateMeta(exam.id, {
        ...exam.meta,
        school: exam.meta.school?.trim() || null,
        department: exam.meta.department?.trim() || null,
        // Adı da unvanı da boş satırlar gönderilmez: kâğıtta boş bir imza
        // çizgisi olarak basılırdı.
        signers: exam.meta.signers
          .map((sg) => ({ name: sg.name.trim(), title: sg.title.trim() }))
          .filter((sg) => sg.name !== "" || sg.title !== ""),
      });
      ayarlarAcik = false;
      await refreshPreview();
    } catch (err: unknown) {
      ayarHatasi = errorText(err);
    } finally {
      ayarKaydediliyor = false;
    }
  }

  function imzaEkle() {
    if (!exam) return;
    exam.meta.signers = [...exam.meta.signers, { name: "", title: "" }];
  }

  function imzaSil(i: number) {
    if (!exam) return;
    exam.meta.signers = exam.meta.signers.filter((_, k) => k !== i);
  }
  let actionError = $state<string | null>(null);
  let busy = $state(false);

  let answerKey = $state(false);

  /** null = tek kitapçık. Etiket basılmaz, sıra yalnızca sınav kimliğinden. */
  let booklet = $state<string | null>(null);
  const BOOKLETS = ["A", "B", "C", "D"];
  const BOOKLET_OPTIONS = [
    { value: "", name: "Tek" },
    ...BOOKLETS.map((b) => ({ value: b, name: b })),
  ];
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
      const source = await api.compiler.generateTypst(exam.id, answerKey, booklet);
      pages = await api.compiler.previewSvg(source);
      previewError = null;
    } catch (err: unknown) {
      previewError = errorText(err);
    } finally {
      compiling = false;
    }
  }

  let ordered = $derived.by(() => {
    if (!exam) return [];
    return exam.questions
      .slice()
      .sort((a, b) => a.display_order - b.display_order)
      .map((ref) => ({
        id: ref.question_id,
        question: bank.find((q) => q.id === ref.question_id) ?? null,
      }));
  });

  let selected = $derived(
    ordered.map((row) => row.question).filter((q): q is Question => q !== null),
  );

  /**
   * Bankada karşılığı olmayan atıflar. Sessizce atmak, öğretmene eksik bir
   * sınavı tam sanarak bastırmak demektir; toplam puan da yanlış çıkar.
   */
  let missing = $derived(ordered.filter((row) => row.question === null));

  let available = $derived(
    bank.filter((q) => !exam?.questions.some((ref) => ref.question_id === q.id)),
  );

  /** Sınavdaki puan: override varsa o, yoksa sorunun kendi puanı. */
  function pointsInExam(q: Question): number {
    const ref = exam?.questions.find((r) => r.question_id === q.id);
    return ref?.points_override ?? questionPoints(q);
  }

  let totalPoints = $derived(selected.reduce((sum, q) => sum + pointsInExam(q), 0));

  /**
   * Kitapçıklar yalnızca karıştırılan sorular varsa farklılaşır. Hiçbiri
   * karıştırılmıyorsa A ile B birebir aynı çıkar; bunu söylememek, öğretmenin
   * kopya çekilmediğini sanmasına yol açar.
   */
  let shuffledCount = $derived(
    selected.filter((q) => q.question_type === "multiple_choice" && q.shuffle).length,
  );

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
    // Nereye kaydedileceğini ÖĞRETMEN seçer. Önceden dosya sormadan
    // İndirilenler'e düşüyordu; daha kötüsü, PDF hiç yazılmıyordu.
    const hedef = await save({
      defaultPath: examFileName(exam, { answerKey, booklet, extension: "pdf" }),
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!hedef) return; // vazgeçildi

    await run(async () => {
      const path = await api.compiler.exportPdf(exam!.id, answerKey, booklet, hedef);
      actionError = `PDF kaydedildi: ${path}`;
    });
  }
</script>

{#if loading}
  <div class="flex items-center gap-2 p-6 text-sm text-gray-500 dark:text-gray-400">
    <Spinner size="5" />
    Sınav okunuyor…
  </div>
{:else if loadError}
  <Alert color="red" class="m-6">{loadError}</Alert>
{:else if exam}
  <PageShell
    title={exam.meta.title}
    subtitle="{exam.meta.subject} · {exam.meta.classroom} · {exam.meta.date}"
    scroll={false}
  >
    {#snippet actions()}
      <!--
        Yayına çıkmış sınav artık düzenlenemez; kırmızı rozet bu tekliği hatırlatıyor.
        `exam!`: snippet'ler ayrı bir kapsam olduğundan TypeScript üst kapsamdaki
        `{#if exam}` daraltmasını buraya taşımıyor — exam burada zaten garanti dolu.
      -->
      <Badge color={exam!.status === "Published" ? "red" : "gray"}>
        {EXAM_STATUS_LABELS[exam!.status]}
      </Badge>

      <Button
        size="sm"
        color="alternative"
        aria-expanded={ayarlarAcik}
        onclick={() => (ayarlarAcik = !ayarlarAcik)}
      >
        Kâğıt ayarları
      </Button>

      <span class="inline-block w-24">
        <Select
          items={BOOKLET_OPTIONS}
          value={booklet ?? ""}
          placeholder=""
          onchange={(e) => {
            // Boş dize = tek kitapçık. null'a çevrilmezse kâğıda
            // "KİTAPÇIK " diye boş bir etiket basılırdı.
            const v = (e.currentTarget as HTMLSelectElement).value;
            booklet = v === "" ? null : v;
            refreshPreview();
          }}
        />
      </span>

      <Checkbox bind:checked={answerKey} onchange={refreshPreview}>Cevap anahtarı</Checkbox>

      <Button size="sm" color="alternative" disabled={busy} onclick={exportPdf}>
        PDF kaydet
      </Button>
      <Button
        size="sm"
        disabled={busy || selected.length === 0 || exam!.status === "Published"}
        onclick={() => run(() => api.exams.publish(exam!.id))}
      >
        Yayınla
      </Button>
    {/snippet}

    <div class="flex h-full min-h-0 flex-col">
      {#if ayarlarAcik}
        <div class="shrink-0 border-b border-gray-200 bg-gray-50 px-6 py-4 dark:border-gray-700 dark:bg-gray-800">
          {#if ayarHatasi}
            <Alert color="red" class="mb-3">{ayarHatasi}</Alert>
          {/if}

          <div class="grid max-w-3xl grid-cols-2 gap-x-5 gap-y-2.5">
            <div>
              <Label class="mb-1">
                Sütun
                <span class="font-normal text-gray-500 dark:text-gray-400"
                  >— Çift sütun kısa sorularda kâğıt kazandırır</span
                >
              </Label>
              <Select
                items={COLUMN_OPTIONS}
                value={String(exam.meta.columns)}
                placeholder=""
                onchange={(e) =>
                  exam &&
                  (exam.meta.columns = Number((e.currentTarget as HTMLSelectElement).value))}
              />
            </div>

            <div>
              <Label for="ayar-sure" class="mb-1">
                Süre <span class="font-normal text-gray-500 dark:text-gray-400">(dakika)</span>
              </Label>
              <Input id="ayar-sure" type="number" min="1" bind:value={exam.meta.duration_min} />
            </div>

            <div>
              <Label for="ayar-okul" class="mb-1">
                Okul
                <span class="font-normal text-gray-500 dark:text-gray-400"
                  >— Boşsa kâğıda basılmaz</span
                >
              </Label>
              <!-- get/set çifti: Input null kabul etmiyor, exam.meta.school ise
                   string | null — boş girişte veritabanı alanı null kalabilmeli. -->
              <Input
                id="ayar-okul"
                type="text"
                bind:value={() => exam?.meta.school ?? "", (v) => exam && (exam.meta.school = v)}
              />
            </div>

            <div>
              <Label for="ayar-alan" class="mb-1">
                Alan / Bölüm
                <span class="font-normal text-gray-500 dark:text-gray-400"
                  >— Boşsa kâğıda basılmaz</span
                >
              </Label>
              <Input
                id="ayar-alan"
                type="text"
                bind:value={() => exam?.meta.department ?? "", (v) => exam && (exam.meta.department = v)}
              />
            </div>

            <div class="col-span-2">
              <div class="flex items-center gap-2.5">
                <span
                  class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
                >
                  İmzalar
                </span>
                <span class="text-sm text-gray-500 dark:text-gray-400">
                  Boşsa imza bloğu basılmaz
                </span>
                <Button size="xs" color="alternative" class="ml-auto" onclick={imzaEkle}>
                  + İmza ekle
                </Button>
              </div>

              {#each exam.meta.signers as _, i (i)}
                <div class="mt-2.5 flex items-end gap-2.5">
                  <div class="flex-1">
                    <Label class="mb-1">Ad Soyad</Label>
                    <Input type="text" bind:value={exam.meta.signers[i].name} />
                  </div>
                  <div class="flex-1">
                    <Label class="mb-1">Unvan</Label>
                    <Input type="text" bind:value={exam.meta.signers[i].title} />
                  </div>
                  <Button
                    size="sm"
                    color="alternative"
                    aria-label="{i + 1}. imzayı sil"
                    onclick={() => imzaSil(i)}
                  >
                    Sil
                  </Button>
                </div>
              {/each}
            </div>
          </div>

          <div class="mt-2.5 flex items-center gap-2.5">
            <Button size="sm" disabled={ayarKaydediliyor} onclick={ayarlariKaydet}>
              {ayarKaydediliyor ? "Kaydediliyor…" : "Ayarları kaydet"}
            </Button>
            <span class="text-sm text-gray-500 dark:text-gray-400">
              Kaydedince önizleme yenilenir
            </span>
          </div>
        </div>
      {/if}

      <QuestionStrip questions={selected} />

      <div
        class="flex shrink-0 flex-wrap items-center gap-5 border-b border-gray-200 bg-white
               px-6 py-2.5 dark:border-gray-700 dark:bg-gray-900"
      >
        <BudgetGauge label="Puan" value={totalPoints} target={POINT_TARGET} unit="puan" />
        <BudgetGauge label="Soru" value={selected.length} target={20} unit="soru" />
        {#if missing.length > 0}
          <span class="text-sm font-semibold text-red-600 dark:text-red-500">
            {missing.length} soru bankada yok
          </span>
        {/if}
        <span class="text-sm text-gray-500 dark:text-gray-400">{exam.meta.duration_min} dk</span>
        {#if booklet !== null && shuffledCount === 0}
          <span class="text-sm text-red-600 dark:text-red-500">
            Hiçbir sorunun şıkları karıştırılmıyor — kitapçıklar birebir aynı çıkar. Soru
            gövdesinde karistir: true yap.
          </span>
        {:else if booklet !== null}
          <span class="text-sm text-gray-500 dark:text-gray-400">
            {shuffledCount} soru karıştırılıyor
          </span>
        {/if}
        {#if compiling}
          <span class="ml-auto text-sm text-gray-500 dark:text-gray-400">derleniyor…</span>
        {/if}
      </div>

      {#if actionError}
        <Alert color="red" class="shrink-0 rounded-none">{actionError}</Alert>
      {/if}

      <div
        class="grid min-h-0 flex-1 grid-cols-[minmax(280px,1fr)_minmax(300px,1fr)_minmax(320px,1.1fr)]"
      >
        <section class="min-h-0 overflow-auto border-r border-gray-200 dark:border-gray-700">
          <h2
            class="sticky top-0 border-b border-gray-200 bg-white px-6 py-2 text-xs font-semibold
                   uppercase tracking-wide text-gray-500 dark:border-gray-700 dark:bg-gray-900
                   dark:text-gray-400"
          >
            Sınavdaki sorular
          </h2>
          {#if selected.length === 0}
            <p class="p-6 text-sm text-gray-500 dark:text-gray-400">Henüz soru eklenmedi.</p>
          {:else}
            {#each missing as row (row.id)}
              <Alert color="red" class="mx-6 mt-3 rounded-md">
                Bankada bulunamadı: <span class="font-mono">{row.id.slice(0, 8)}</span> — soru
                silinmiş olabilir. Bu sınav eksik basılır.
              </Alert>
            {/each}
            <ol class="divide-y divide-gray-200 dark:divide-gray-700">
              {#each selected as q, i (q.id)}
                <li class="flex items-start gap-2.5 px-6 py-4">
                  <span
                    class="tnum w-5 shrink-0 pt-0.5 text-xs font-semibold text-gray-500 dark:text-gray-400"
                  >
                    {i + 1}
                  </span>
                  <div class="min-w-0 flex-1">
                    <p class="truncate font-mono text-xs text-gray-700 dark:text-gray-300">
                      {preview(q)}
                    </p>
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                      {QUESTION_TYPE_LABELS[q.question_type]}
                    </p>
                  </div>

                  <!--
                    Puan burada belirlenir, soruda değil: aynı soru bir yazılıda 5,
                    başkasında 10 puan edebilir.
                  -->
                  <label class="flex shrink-0 items-baseline gap-1 text-sm text-gray-500 dark:text-gray-400">
                    <input
                      type="number"
                      min="1"
                      class="tnum w-12 border-0 border-b border-gray-300 bg-transparent pb-0.5
                             text-right leading-6 focus:border-primary-600 focus:outline-none
                             focus:ring-0 dark:border-gray-600 dark:focus:border-primary-500"
                      value={pointsInExam(q)}
                      disabled={busy}
                      onchange={(e) => {
                        const v = Number((e.currentTarget as HTMLInputElement).value);
                        run(() =>
                          api.exams.setQuestionPoints(
                            exam!.id,
                            q.id,
                            Number.isFinite(v) && v > 0 ? v : null,
                          ),
                        );
                      }}
                    />
                    puan
                  </label>

                  <Button
                    size="xs"
                    color="alternative"
                    disabled={busy}
                    onclick={() => run(() => api.exams.removeQuestion(exam!.id, q.id))}
                  >
                    Çıkar
                  </Button>
                </li>
              {/each}
            </ol>
          {/if}
        </section>

        <section class="min-h-0 overflow-auto border-r border-gray-200 dark:border-gray-700">
          <h2
            class="sticky top-0 border-b border-gray-200 bg-white px-6 py-2 text-xs font-semibold
                   uppercase tracking-wide text-gray-500 dark:border-gray-700 dark:bg-gray-900
                   dark:text-gray-400"
          >
            Bankadan ekle
          </h2>
          {#if available.length === 0}
            <p class="p-6 text-sm text-gray-500 dark:text-gray-400">
              Eklenebilecek başka soru yok.
            </p>
          {:else}
            <ul class="divide-y divide-gray-200 dark:divide-gray-700">
              {#each available as q (q.id)}
                <li class="flex items-start gap-2.5 px-6 py-4">
                  <div class="min-w-0 flex-1">
                    <p class="truncate font-mono text-xs text-gray-700 dark:text-gray-300">
                      {preview(q)}
                    </p>
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                      {QUESTION_TYPE_LABELS[q.question_type]} · {questionPoints(q)} puan
                      {#if q.stats.times_used > 0 && q.stats.discrimination_index < 0.2}
                        <span class="text-red-600 dark:text-red-500">· ayırt ediciliği düşük</span>
                      {/if}
                    </p>
                  </div>
                  <Button
                    size="xs"
                    color="alternative"
                    disabled={busy}
                    onclick={() => run(() => api.exams.addQuestion(exam!.id, q.id))}
                  >
                    Ekle
                  </Button>
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
  </PageShell>
{/if}
