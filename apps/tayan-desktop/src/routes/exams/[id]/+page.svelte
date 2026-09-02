<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import PenButton from "$lib/components/shell/PenButton.svelte";
  import BudgetGauge from "$lib/components/measure/BudgetGauge.svelte";
  import QuestionStrip from "$lib/components/measure/QuestionStrip.svelte";
  import SheetPreview from "$lib/components/question/SheetPreview.svelte";
  import SelectBox from "$lib/components/shell/SelectBox.svelte";
  import RuledField from "$lib/components/shell/RuledField.svelte";
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

      <button
        type="button"
        class="border border-rule-strong bg-paper px-half py-quarter text-[12px]
               leading-rule text-ink transition-colors hover:border-red hover:text-red-deep"
        aria-expanded={ayarlarAcik}
        onclick={() => (ayarlarAcik = !ayarlarAcik)}
      >
        Kâğıt ayarları
      </button>

      <div class="ml-auto flex items-center gap-half">
        <label class="pencil flex items-center gap-quarter">
          Kitapçık
          <span class="inline-block w-[86px]">
            <SelectBox
              value={booklet ?? ""}
              options={BOOKLETS.map((b) => ({ value: b, label: b }))}
              emptyLabel="Tek"
              onchange={(v) => {
                // Boş dize = tek kitapçık. null'a çevrilmezse kâğıda
                // "KİTAPÇIK " diye boş bir etiket basılırdı.
                booklet = v === "" ? null : v;
                refreshPreview();
              }}
            />
          </span>
        </label>

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

    {#if ayarlarAcik}
      <div class="ruled-bottom shrink-0 bg-paper px-rule py-half paper-plain">
        {#if ayarHatasi}
          <p class="annot mb-half bg-red-wash px-half py-quarter">{ayarHatasi}</p>
        {/if}

        <div class="grid max-w-[720px] grid-cols-2 gap-x-rule gap-y-half">
          <RuledField label="Sütun" hint="Çift sütun kısa sorularda kâğıt kazandırır">
            <SelectBox
              value={String(exam.meta.columns)}
              options={[
                { value: "1", label: "Tek sütun" },
                { value: "2", label: "Çift sütun" },
              ]}
              onchange={(v) => exam && (exam.meta.columns = Number(v))}
            />
          </RuledField>

          <RuledField label="Süre" hint="dakika">
            <input type="number" min="1" bind:value={exam.meta.duration_min} />
          </RuledField>

          <RuledField label="Okul" hint="Boşsa kâğıda basılmaz">
            <input type="text" bind:value={exam.meta.school} />
          </RuledField>

          <RuledField label="Alan / Bölüm" hint="Boşsa kâğıda basılmaz">
            <input type="text" bind:value={exam.meta.department} />
          </RuledField>

          <div class="col-span-2">
            <div class="flex items-center gap-half">
              <span class="stamp">İmzalar</span>
              <span class="pencil">Boşsa imza bloğu basılmaz</span>
              <button
                type="button"
                class="ml-auto border border-rule-strong bg-paper-lift px-half py-quarter
                       text-[12px] leading-rule text-ink transition-colors
                       hover:border-red hover:text-red-deep"
                onclick={imzaEkle}
              >
                + İmza ekle
              </button>
            </div>

            {#each exam.meta.signers as _, i (i)}
              <div class="mt-quarter flex items-end gap-half">
                <div class="flex-1">
                  <RuledField label="Ad Soyad">
                    <input type="text" bind:value={exam.meta.signers[i].name} />
                  </RuledField>
                </div>
                <div class="flex-1">
                  <RuledField label="Unvan">
                    <input type="text" bind:value={exam.meta.signers[i].title} />
                  </RuledField>
                </div>
                <button
                  type="button"
                  class="border border-rule-strong bg-paper-lift px-half py-quarter
                         text-[12px] leading-rule text-pencil transition-colors
                         hover:border-red hover:text-red-deep"
                  aria-label="{i + 1}. imzayı sil"
                  onclick={() => imzaSil(i)}
                >
                  Sil
                </button>
              </div>
            {/each}
          </div>
        </div>

        <div class="mt-half flex items-center gap-half">
          <PenButton kind="ink" disabled={ayarKaydediliyor} onclick={ayarlariKaydet}>
            {ayarKaydediliyor ? "Kaydediliyor…" : "Ayarları kaydet"}
          </PenButton>
          <span class="pencil">Kaydedince önizleme yenilenir</span>
        </div>
      </div>
    {/if}

    <QuestionStrip questions={selected} />

    <div class="ruled-bottom flex shrink-0 flex-wrap items-center gap-rule bg-paper px-rule py-half paper-plain">
      <BudgetGauge label="Puan" value={totalPoints} target={POINT_TARGET} unit="puan" />
      <BudgetGauge label="Soru" value={selected.length} target={20} unit="soru" />
      {#if missing.length > 0}
        <span class="annot font-semibold">{missing.length} soru bankada yok</span>
      {/if}
      <span class="pencil">{exam.meta.duration_min} dk</span>
      {#if booklet !== null && shuffledCount === 0}
        <span class="annot">
          Hiçbir sorunun şıkları karıştırılmıyor — kitapçıklar birebir aynı çıkar.
          Soru gövdesinde karistir: true yap.
        </span>
      {:else if booklet !== null}
        <span class="pencil">{shuffledCount} soru karıştırılıyor</span>
      {/if}
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
          {#each missing as row (row.id)}
            <p class="annot border-b border-rule bg-red-wash px-rule py-half">
              Bankada bulunamadı: <span class="font-mono">{row.id.slice(0, 8)}</span> —
              soru silinmiş olabilir. Bu sınav eksik basılır.
            </p>
          {/each}
          <ol>
            {#each selected as q, i (q.id)}
              <li class="flex items-start gap-half border-b border-rule px-rule py-half">
                <span class="stamp tnum w-[18px] shrink-0 pt-[2px]">{i + 1}</span>
                <div class="min-w-0 flex-1">
                  <p class="truncate font-mono text-[12px] text-ink-mid">{preview(q)}</p>
                  <p class="pencil">{QUESTION_TYPE_LABELS[q.question_type]}</p>
                </div>

                <!--
                  Puan burada belirlenir, soruda değil: aynı soru bir yazılıda 5,
                  başkasında 10 puan edebilir.
                -->
                <label class="flex shrink-0 items-baseline gap-quarter">
                  <input
                    type="number"
                    min="1"
                    class="w-[48px] border-0 border-b border-rule-strong bg-transparent pb-[2px]
                           text-right leading-rule tnum focus:border-red focus:outline-none"
                    value={pointsInExam(q)}
                    disabled={busy}
                    onchange={(e) => {
                      const v = Number((e.currentTarget as HTMLInputElement).value);
                      run(() =>
                        api.exams.setQuestionPoints(exam!.id, q.id, Number.isFinite(v) && v > 0 ? v : null),
                      );
                    }}
                  />
                  <span class="pencil">puan</span>
                </label>

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
