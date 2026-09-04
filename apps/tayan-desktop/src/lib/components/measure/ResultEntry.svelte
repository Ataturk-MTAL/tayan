<script lang="ts">
  /**
   * Sınav sonucu girişi — öğrenci öğrenci.
   *
   * Izgara (öğrenci × soru) daha hızlı görünür ama fiziksel işe uymaz:
   * öğretmenin elinde 30 kâğıtlık bir deste var, birini alıp cevaplarını girip
   * sonrakine geçiyor. Öğrenci başına form bu akışın birebir karşılığı; ızgarada
   * gözün satır kaydırması yanlış öğrenciye yazma riski üretir.
   *
   * CEVAP KODLAMASI doğrudan ScoringService::score_answer'a gider ve birebir
   * uymak zorundadır — yanlış kodlama sessizce yanlış not demektir:
   *   Çoktan seçmeli : şık kimliği, "A" / "B" / ...
   *   Doğru-yanlış   : "true" / "false"  (Rust tarafı parse::<bool>() yapar)
   *   Boşluk doldurma: JSON eşlemesi, {"b1": "180", "b2": "..."}
   *   Klasik         : cevap yok; puan ELLE girilir ve olduğu gibi kaydedilir
   */
  import PenButton from "../shell/PenButton.svelte";
  import { Alert, Badge, Card, Checkbox, Input, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from "flowbite-svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { bodyPreview } from "$lib/types";
  import type {
    Exam,
    ExamResult,
    Question,
    QuestionAnswerInput,
    RubricItem,
    Student,
  } from "$lib/types";

  type Props = {
    exam: Exam;
    students: Student[];
    results: ExamResult[];
    /** Bankadaki tüm sorular; sınavın atıfları buradan çözülür. */
    bank: Question[];
    onsaved: () => void;
  };

  let { exam, students, results, bank, onsaved }: Props = $props();

  let studentId = $state("");
  let saving = $state(false);
  let saveError = $state<string | null>(null);
  let saved = $state<string | null>(null);

  /** Girilen cevaplar: anahtar → ham metin. Kodlama kaydederken yapılır. */
  let answers = $state<Record<string, string>>({});
  /** Klasik sorular için elle girilen puan. */
  let manualPoints = $state<Record<string, number>>({});

  /**
   * Klasik soruda karşılanan rubrik ölçütleri: soru kimliği → ölçüt sıraları.
   *
   * Ölçüt işaretlendikçe puan kendiliğinden toplanır. Puanın KAYNAĞI yine
   * `manualPoints`; buradan hesaplanıp oraya yazılır. Böylece rubriksiz soru,
   * rubrikli soru ve öğretmenin elle düzelttiği puan tek yoldan kaydedilir.
   */
  let rubricMet = $state<Record<string, number[]>>({});

  function rubrikOf(q: Question): RubricItem[] {
    return q.question_type === "classic" ? q.rubric : [];
  }

  /** Ölçütü işaretle/kaldır ve puanı yeniden topla. */
  function toggleCriterion(q: Question, index: number) {
    const mevcut = rubricMet[q.id] ?? [];
    const sonraki = mevcut.includes(index)
      ? mevcut.filter((i) => i !== index)
      : [...mevcut, index].sort((a, b) => a - b);

    rubricMet = { ...rubricMet, [q.id]: sonraki };

    const rubrik = rubrikOf(q);
    const toplam = sonraki.reduce((sum, i) => sum + (rubrik[i]?.points ?? 0), 0);
    manualPoints = { ...manualPoints, [q.id]: toplam };
  }

  /**
   * Sınavın soruları, display_order sırasında ve bankada BULUNANLAR.
   *
   * Bankada olmayan atıf sessizce atlanmıyor: öğretmen kâğıtta o soruyu görüyor
   * ve puanının nereye gittiğini sorar. Aşağıda açıkça uyarılıyor.
   */
  let sorular = $derived.by(() => {
    const sirali = [...exam.questions].sort((a, b) => a.display_order - b.display_order);
    return sirali.map((ref) => ({
      ref,
      question: bank.find((q) => q.id === ref.question_id) ?? null,
    }));
  });

  let eksikSoru = $derived(sorular.filter((s) => s.question === null).length);

  /** Sınavın toplam puanı: sınava özgü puan varsa o, yoksa sorunun kendi puanı. */
  let toplamPuan = $derived(
    sorular.reduce((sum, s) => {
      const q = s.question;
      if (q === null) return sum;
      if (s.ref.points_override !== null) return sum + s.ref.points_override;
      if (q.question_type === "fill_in_blank")
        return sum + q.blanks.reduce((a, b) => a + b.points, 0);
      return sum + q.points;
    }, 0),
  );

  let girilmis = $derived(new Set(results.map((r) => r.student_id)));

  function selectStudent(id: string) {
    studentId = id;
    answers = {};
    manualPoints = {};
    rubricMet = {};
    saved = null;
    saveError = null;
  }

  function setAnswer(key: string, value: string) {
    answers = { ...answers, [key]: value };
  }

  function setPoints(qid: string, value: number) {
    manualPoints = { ...manualPoints, [qid]: value };
  }

  function maxPoints(ref: { points_override: number | null }, q: Question): number {
    if (ref.points_override !== null) return ref.points_override;
    return q.question_type === "fill_in_blank"
      ? q.blanks.reduce((a, b) => a + b.points, 0)
      : q.points;
  }

  /** Ham girdileri ScoringService'in beklediği biçime çevirir. */
  function buildPayload(): QuestionAnswerInput[] {
    const out: QuestionAnswerInput[] = [];

    for (const { question } of sorular) {
      if (question === null) continue;
      const qid = question.id;

      if (question.question_type === "classic") {
        out.push({
          question_id: qid,
          given_answer: null,
          points_earned: manualPoints[qid] ?? 0,
          is_correct: null,
          rubric_met: rubricMet[qid] ?? [],
        });
        continue;
      }

      if (question.question_type === "fill_in_blank") {
        const map: Record<string, string> = {};
        for (const b of question.blanks) {
          const v = answers[`${qid}::${b.id}`];
          if (v !== undefined && v !== "") map[b.id] = v;
        }
        out.push({
          question_id: qid,
          given_answer: JSON.stringify(map),
          points_earned: 0,
          is_correct: null,
          rubric_met: [],
        });
        continue;
      }

      // Çoktan seçmeli ve doğru-yanlış. Boş bırakılan soru CEVAPSIZ sayılır:
      // null gönderilir, yanlış bir şık uydurulmaz.
      const raw = answers[qid];
      out.push({
        question_id: qid,
        given_answer: raw === undefined || raw === "" ? null : raw,
        points_earned: 0,
        is_correct: null,
        rubric_met: [],
      });
    }

    return out;
  }

  async function save() {
    if (studentId === "") {
      saveError = "Önce öğrenci seç.";
      return;
    }

    saving = true;
    saveError = null;
    saved = null;
    try {
      await api.results.enter({
        examId: exam.id,
        studentId,
        answers: buildPayload(),
        totalMax: toplamPuan,
      });
      const s = students.find((x) => x.id === studentId);
      saved = s ? `${s.first_name} ${s.last_name}` : "Sonuç";
      onsaved();
    } catch (err: unknown) {
      saveError = errorText(err);
    } finally {
      saving = false;
    }
  }
</script>

<div class="grid min-h-0 flex-1 grid-cols-[220px_1fr]">
  <!-- Öğrenci listesi. Girilmiş olanlar işaretli: deste ilerledikçe kalan görünür. -->
  <nav class="min-h-0 overflow-auto border-r border-gray-300 dark:border-gray-600">
    {#if students.length === 0}
      <p class="p-2.5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">Bu sınıfta öğrenci yok.</p>
    {:else}
      <ul>
        {#each students as s (s.id)}
          <li>
            <button
              type="button"
              class="flex w-full items-center gap-2.5 border-b border-gray-200 px-2.5 py-[5px]
                     text-left text-[13px] leading-5 transition-colors hover:bg-gray-50
                     dark:border-gray-700 dark:hover:bg-gray-700
                     {s.id === studentId ? 'bg-primary-50 font-semibold dark:bg-primary-900/30' : ''}"
              onclick={() => selectStudent(s.id)}
            >
              <span class="tnum w-[28px] text-gray-500 dark:text-gray-400">{s.number}</span>
              <span class="flex-1">{s.first_name} {s.last_name}</span>
              {#if girilmis.has(s.id)}
                <!-- Sonucu girilmiş: yeşil "tamam" rozeti. Bu bir "doğru cevap" değil,
                     bir iş durumu — kırmızı/gri değerlendirme ekseninin dışında. -->
                <Badge color="green" class="px-1.5 py-0 text-[11px]">✓</Badge>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </nav>

  <div class="min-h-0 overflow-auto px-5 py-2.5">
    {#if eksikSoru > 0}
      <Alert color="red" class="mb-2.5 text-[12px] leading-5">
        Bu sınavın {eksikSoru} sorusu bankada bulunamadı. O sorular puanlanamaz;
        toplam puan onlar hariç hesaplandı.
      </Alert>
    {/if}

    {#if studentId === ""}
      <p class="text-[12px] leading-5 text-gray-500 dark:text-gray-400">Soldan bir öğrenci seç.</p>
    {:else}
      <div class="mb-2.5 flex flex-wrap items-center gap-2.5 border-b border-gray-300 pb-[5px] dark:border-gray-600">
        <span class="text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
          Toplam
        </span>
        <span class="tnum font-bold text-gray-900 dark:text-white">{toplamPuan}</span>
        <span class="text-[12px] text-gray-500 dark:text-gray-400">puan</span>
        {#if girilmis.has(studentId)}
          <!-- Uyarı, hata değil: amber. Kırmızı yalnız değerlendirme/yanlış içindir. -->
          <span class="text-[12px] leading-5 text-amber-600 dark:text-amber-400">
            Bu öğrencinin sonucu daha önce girilmiş; kaydetmek üzerine yazar.
          </span>
        {/if}
        <span class="ml-auto"></span>
        <PenButton kind="ink" disabled={saving} onclick={save}>
          {saving ? "Kaydediliyor…" : "Kaydet"}
        </PenButton>
      </div>

      {#if saveError}
        <Alert color="red" class="mb-2.5 text-[12px] leading-5">{saveError}</Alert>
      {/if}
      {#if saved}
        <p class="mb-2.5 text-[12px] leading-5 text-gray-500 dark:text-gray-400">
          {saved} kaydedildi. Ölçüm yeniden hesaplandı.
        </p>
      {/if}

      <Card size="xl" class="p-0">
        <Table>
          <TableHead>
            <TableHeadCell class="w-[2rem]">#</TableHeadCell>
            <TableHeadCell>Soru</TableHeadCell>
            <TableHeadCell>Cevap</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each sorular as { ref, question }, i (ref.question_id)}
              {#if question !== null}
                <TableBodyRow>
                  <TableBodyCell class="tnum align-top text-gray-500 dark:text-gray-400">
                    {i + 1}.
                  </TableBodyCell>
                  <TableBodyCell class="align-top">
                    <span class="text-[13px] leading-5 text-gray-700 dark:text-gray-300">
                      {bodyPreview(question.body, 90)}
                    </span>
                    <span class="tnum block text-[12px] text-gray-500 dark:text-gray-400">
                      {maxPoints(ref, question)} p
                    </span>
                  </TableBodyCell>
                  <TableBodyCell class="align-top">
                    {#if question.question_type === "multiple_choice"}
                      <div class="flex flex-wrap items-center gap-[5px]">
                        {#each question.options as opt (opt.id)}
                          <button
                            type="button"
                            class="w-[30px] border border-gray-300 bg-white py-[5px] text-[12px]
                                   leading-5 transition-colors hover:border-red-600
                                   dark:border-gray-600 dark:bg-gray-800 dark:hover:border-red-400
                                   {answers[question.id] === opt.id
                                     ? 'bg-primary-50 font-semibold dark:bg-primary-900/30'
                                     : ''}"
                            onclick={() =>
                              setAnswer(question.id, answers[question.id] === opt.id ? "" : opt.id)}
                          >
                            {opt.id}
                          </button>
                        {/each}
                        <span class="ml-2.5 text-[12px] text-gray-500 dark:text-gray-400">
                          boş bırakılırsa cevapsız sayılır
                        </span>
                      </div>
                    {:else if question.question_type === "true_false"}
                      <div class="flex flex-wrap items-center gap-[5px]">
                        <button
                          type="button"
                          class="border border-gray-300 bg-white px-2.5 py-[5px] text-[12px]
                                 leading-5 transition-colors hover:border-red-600
                                 dark:border-gray-600 dark:bg-gray-800 dark:hover:border-red-400
                                 {answers[question.id] === 'true'
                                   ? 'bg-primary-50 font-semibold dark:bg-primary-900/30'
                                   : ''}"
                          onclick={() =>
                            setAnswer(question.id, answers[question.id] === "true" ? "" : "true")}
                        >
                          Doğru
                        </button>
                        <button
                          type="button"
                          class="border border-gray-300 bg-white px-2.5 py-[5px] text-[12px]
                                 leading-5 transition-colors hover:border-red-600
                                 dark:border-gray-600 dark:bg-gray-800 dark:hover:border-red-400
                                 {answers[question.id] === 'false'
                                   ? 'bg-primary-50 font-semibold dark:bg-primary-900/30'
                                   : ''}"
                          onclick={() =>
                            setAnswer(question.id, answers[question.id] === "false" ? "" : "false")}
                        >
                          Yanlış
                        </button>
                      </div>
                    {:else if question.question_type === "fill_in_blank"}
                      <div class="flex flex-wrap items-center gap-2.5">
                        {#each question.blanks as b (b.id)}
                          <label class="flex items-center gap-[5px]">
                            <span class="text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                              {b.id}
                            </span>
                            <Input
                              type="text"
                              size="sm"
                              class="w-[110px]"
                              value={answers[`${question.id}::${b.id}`] ?? ""}
                              oninput={(e) =>
                                setAnswer(`${question.id}::${b.id}`, e.currentTarget.value)}
                            />
                          </label>
                        {/each}
                      </div>
                    {:else}
                      <!--
                        Açık uçlu soru. Rubrik varsa ölçüt ölçüt işaretlenir ve puan
                        kendiliğinden toplanır: hangi ölçütün verilmediği kayda geçer,
                        itiraz geldiğinde gerekçe elde durur.

                        Puan kutusu YİNE DE düzenlenebilir kalıyor. Ölçüte tam
                        uymayan ama karşılığı olan bir cevabı öğretmen takdir
                        edebilmeli; rubrik yardımcıdır, kelepçe değil.
                      -->
                      {#if rubrikOf(question).length > 0}
                        <ul class="mb-[5px] space-y-[2px]">
                          {#each rubrikOf(question) as olcut, oi (oi)}
                            <li>
                              <Checkbox
                                checked={(rubricMet[question.id] ?? []).includes(oi)}
                                onchange={() => toggleCriterion(question, oi)}
                                labelProps={{ class: "flex w-full items-start gap-[5px]" }}
                              >
                                <span class="flex-1 text-[12px] leading-5 text-gray-700 dark:text-gray-300">
                                  {olcut.criterion}
                                </span>
                                <span class="tnum shrink-0 text-[12px] text-gray-500 dark:text-gray-400">
                                  {olcut.points}
                                </span>
                              </Checkbox>
                            </li>
                          {/each}
                        </ul>
                      {/if}

                      <label class="flex items-center gap-[5px]">
                        <span class="text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                          Puan
                        </span>
                        <Input
                          type="number"
                          size="sm"
                          min="0"
                          max={maxPoints(ref, question)}
                          class="tnum w-[70px]"
                          value={manualPoints[question.id] ?? 0}
                          oninput={(e) => setPoints(question.id, Number(e.currentTarget.value))}
                        />
                        <span class="text-[12px] text-gray-500 dark:text-gray-400">
                          / {maxPoints(ref, question)}
                        </span>
                      </label>
                    {/if}
                  </TableBodyCell>
                </TableBodyRow>
              {/if}
            {/each}
          </TableBody>
        </Table>
      </Card>
    {/if}
  </div>
</div>
