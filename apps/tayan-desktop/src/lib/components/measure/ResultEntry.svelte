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
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { bodyPreview } from "$lib/types";
  import type {
    Exam,
    ExamResult,
    Question,
    QuestionAnswerInput,
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
  <nav class="min-h-0 overflow-auto border-r border-rule-strong">
    {#if students.length === 0}
      <p class="pencil p-half">Bu sınıfta öğrenci yok.</p>
    {:else}
      <ul>
        {#each students as s (s.id)}
          <li>
            <button
              type="button"
              class="flex w-full items-center gap-half border-b border-rule px-half py-quarter
                     text-left text-[13px] leading-rule transition-colors hover:bg-paper-lift"
              class:bg-paper-sunk={s.id === studentId}
              class:font-semibold={s.id === studentId}
              onclick={() => selectStudent(s.id)}
            >
              <span class="tnum w-[28px] text-pencil">{s.number}</span>
              <span class="flex-1">{s.first_name} {s.last_name}</span>
              {#if girilmis.has(s.id)}
                <span class="stamp" style="color: var(--color-mark-excellent)">✓</span>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </nav>

  <div class="min-h-0 overflow-auto px-rule py-half">
    {#if eksikSoru > 0}
      <p class="annot mb-half bg-red-wash px-half py-quarter">
        Bu sınavın {eksikSoru} sorusu bankada bulunamadı. O sorular puanlanamaz;
        toplam puan onlar hariç hesaplandı.
      </p>
    {/if}

    {#if studentId === ""}
      <p class="pencil">Soldan bir öğrenci seç.</p>
    {:else}
      <div class="ruled-bottom mb-half flex flex-wrap items-center gap-half pb-quarter">
        <span class="stamp">Toplam</span>
        <span class="tnum font-bold">{toplamPuan}</span>
        <span class="pencil">puan</span>
        {#if girilmis.has(studentId)}
          <span class="annot">Bu öğrencinin sonucu daha önce girilmiş; kaydetmek üzerine yazar.</span>
        {/if}
        <span class="ml-auto"></span>
        <PenButton kind="ink" disabled={saving} onclick={save}>
          {saving ? "Kaydediliyor…" : "Kaydet"}
        </PenButton>
      </div>

      {#if saveError}
        <p class="annot mb-half bg-red-wash px-half py-quarter">{saveError}</p>
      {/if}
      {#if saved}
        <p class="pencil mb-half">{saved} kaydedildi. Ölçüm yeniden hesaplandı.</p>
      {/if}

      {#each sorular as { ref, question }, i (ref.question_id)}
        {#if question !== null}
          <div class="border-b border-rule py-half">
            <div class="flex items-baseline gap-half">
              <span class="stamp">{i + 1}.</span>
              <span class="flex-1 text-[13px] leading-rule text-ink-mid">
                {bodyPreview(question.body, 90)}
              </span>
              <span class="pencil tnum">{maxPoints(ref, question)} p</span>
            </div>

            <div class="mt-quarter flex flex-wrap items-center gap-quarter">
              {#if question.question_type === "multiple_choice"}
                {#each question.options as opt (opt.id)}
                  <button
                    type="button"
                    class="w-[30px] border border-rule-strong bg-paper py-quarter text-[12px]
                           leading-rule transition-colors hover:border-red"
                    class:bg-paper-sunk={answers[question.id] === opt.id}
                    class:font-semibold={answers[question.id] === opt.id}
                    onclick={() =>
                      setAnswer(question.id, answers[question.id] === opt.id ? "" : opt.id)}
                  >
                    {opt.id}
                  </button>
                {/each}
                <span class="pencil ml-half">boş bırakılırsa cevapsız sayılır</span>
              {:else if question.question_type === "true_false"}
                <button
                  type="button"
                  class="border border-rule-strong bg-paper px-half py-quarter text-[12px]
                         leading-rule transition-colors hover:border-red"
                  class:bg-paper-sunk={answers[question.id] === "true"}
                  class:font-semibold={answers[question.id] === "true"}
                  onclick={() =>
                    setAnswer(question.id, answers[question.id] === "true" ? "" : "true")}
                >
                  Doğru
                </button>
                <button
                  type="button"
                  class="border border-rule-strong bg-paper px-half py-quarter text-[12px]
                         leading-rule transition-colors hover:border-red"
                  class:bg-paper-sunk={answers[question.id] === "false"}
                  class:font-semibold={answers[question.id] === "false"}
                  onclick={() =>
                    setAnswer(question.id, answers[question.id] === "false" ? "" : "false")}
                >
                  Yanlış
                </button>
              {:else if question.question_type === "fill_in_blank"}
                {#each question.blanks as b (b.id)}
                  <label class="flex items-center gap-quarter">
                    <span class="stamp">{b.id}</span>
                    <input
                      type="text"
                      class="w-[110px] border-0 border-b border-rule-strong bg-transparent
                             pb-[2px] text-[13px] leading-rule focus:border-red focus:outline-none"
                      value={answers[`${question.id}::${b.id}`] ?? ""}
                      oninput={(e) => setAnswer(`${question.id}::${b.id}`, e.currentTarget.value)}
                    />
                  </label>
                {/each}
              {:else}
                <label class="flex items-center gap-quarter">
                  <span class="stamp">Puan</span>
                  <input
                    type="number"
                    min="0"
                    max={maxPoints(ref, question)}
                    class="tnum w-[70px] border-0 border-b border-rule-strong bg-transparent
                           pb-[2px] text-[13px] leading-rule focus:border-red focus:outline-none"
                    value={manualPoints[question.id] ?? 0}
                    oninput={(e) => setPoints(question.id, Number(e.currentTarget.value))}
                  />
                  <span class="pencil">/ {maxPoints(ref, question)}</span>
                </label>
              {/if}
            </div>
          </div>
        {/if}
      {/each}
    {/if}
  </div>
</div>
