<script lang="ts">
  import QuestionEditor from "./QuestionEditor.svelte";
  import RuledField from "../shell/RuledField.svelte";
  import PenButton from "../shell/PenButton.svelte";
  import { typstBody } from "$lib/question/body";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { QUESTION_TYPE_LABELS, type Question, type QuestionStats } from "$lib/types";
  import { goto } from "$app/navigation";

  type QuestionType = Question["question_type"];

  type Props = {
    initialType?: QuestionType;
    initialBody?: string;
    stats?: QuestionStats | null;
    legacyWarning?: boolean;
  };

  let {
    initialType = "multiple_choice",
    initialBody = "",
    stats = null,
    legacyWarning = false,
  }: Props = $props();

  const OPTION_LETTERS = ["A", "B", "C", "D", "E"];

  let questionType = $state<QuestionType>(initialType);
  let body = $state(initialBody);
  let points = $state(5);
  let outcomeText = $state("");

  // Çoktan seçmeli
  let options = $state<string[]>(["", "", "", "", ""]);
  let correctIndex = $state(0);
  let shuffle = $state(false);

  // Doğru / yanlış
  let trueFalseAnswer = $state(true);

  // Boşluk doldurma
  let blanks = $state<Array<{ accepted: string; points: number }>>([
    { accepted: "", points: 5 },
  ]);

  // Klasik
  let rubric = $state<Array<{ criterion: string; points: number }>>([
    { criterion: "", points: 5 },
  ]);
  let answerLines = $state(6);

  let saving = $state(false);
  let saveError = $state<string | null>(null);

  let outcomes = $derived(
    outcomeText
      .split(/[,\s]+/)
      .map((o) => o.trim())
      .filter(Boolean),
  );

  let effectivePoints = $derived(
    questionType === "fill_in_blank"
      ? blanks.reduce((sum, b) => sum + b.points, 0)
      : points,
  );

  function addBlank() {
    blanks = [...blanks, { accepted: "", points: 5 }];
  }
  function removeBlank(index: number) {
    blanks = blanks.filter((_, i) => i !== index);
  }
  function addRubricItem() {
    rubric = [...rubric, { criterion: "", points: 5 }];
  }
  function removeRubricItem(index: number) {
    rubric = rubric.filter((_, i) => i !== index);
  }

  function validate(): string | null {
    if (body.trim() === "") return "Soru gövdesi boş.";

    if (questionType === "multiple_choice") {
      const filled = options.filter((o) => o.trim() !== "");
      if (filled.length < 2) return "En az iki seçenek gerekli.";
      if (options[correctIndex].trim() === "") return "Doğru olarak işaretlenen seçenek boş.";
    }

    if (questionType === "fill_in_blank") {
      if (blanks.length === 0) return "En az bir boşluk gerekli.";
      if (blanks.some((b) => b.accepted.trim() === ""))
        return "Her boşluğun en az bir kabul edilen cevabı olmalı.";
    }

    if (questionType === "classic") {
      if (rubric.some((r) => r.criterion.trim() === ""))
        return "Rubrik ölçütlerinden biri boş.";
    }

    return null;
  }

  async function save() {
    const problem = validate();
    if (problem) {
      saveError = problem;
      return;
    }

    saving = true;
    saveError = null;

    try {
      const bodyNodes = typstBody(body);

      if (questionType === "multiple_choice") {
        await api.questions.addMultipleChoice({
          points,
          outcomes,
          body: bodyNodes,
          options: options
            .map((source, i) => ({
              id: OPTION_LETTERS[i],
              body: typstBody(source),
              correct: i === correctIndex,
            }))
            .filter((o, i) => options[i].trim() !== ""),
          shuffle,
        });
      } else if (questionType === "true_false") {
        await api.questions.addTrueFalse({
          points,
          outcomes,
          body: bodyNodes,
          correct_answer: trueFalseAnswer,
        });
      } else if (questionType === "fill_in_blank") {
        await api.questions.addFillInBlank({
          outcomes,
          body: bodyNodes,
          blanks: blanks.map((b, i) => ({
            id: `b${i + 1}`,
            accepted_answers: b.accepted.split("|").map((a) => a.trim()).filter(Boolean),
            points: b.points,
          })),
        });
      } else {
        await api.questions.addClassic({
          points,
          outcomes,
          body: bodyNodes,
          sample_answer: null,
          rubric: rubric.filter((r) => r.criterion.trim() !== ""),
          answer_space: { Lines: answerLines },
        });
      }

      await goto("/questions");
    } catch (err: unknown) {
      saveError = errorText(err);
    } finally {
      saving = false;
    }
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="ruled-bottom flex shrink-0 flex-wrap items-end gap-rule bg-paper-lift px-rule py-half paper-plain">
    <div class="w-[160px]">
      <RuledField label="Soru tipi">
        <select bind:value={questionType}>
          {#each Object.entries(QUESTION_TYPE_LABELS) as [value, label]}
            <option {value}>{label}</option>
          {/each}
        </select>
      </RuledField>
    </div>

    {#if questionType !== "fill_in_blank"}
      <div class="w-[80px]">
        <RuledField label="Puan">
          <input type="number" min="1" bind:value={points} />
        </RuledField>
      </div>
    {:else}
      <div class="w-[80px]">
        <span class="stamp block">Puan</span>
        <span class="block border-b border-rule-strong pb-[3px] leading-rule tnum">
          {effectivePoints}
        </span>
        <span class="pencil block">boşluklardan</span>
      </div>
    {/if}

    <div class="min-w-[200px] flex-1">
      <RuledField label="Kazanım" hint="Boşluk veya virgülle ayır — MAT.9.1.2">
        <input type="text" bind:value={outcomeText} placeholder="MAT.9.1.2" />
      </RuledField>
    </div>

    <PenButton kind="ink" disabled={saving} onclick={save}>
      {saving ? "Kaydediliyor…" : "Kaydet"}
    </PenButton>
  </div>

  {#if legacyWarning}
    <p class="ruled-bottom annot shrink-0 bg-red-wash px-rule py-quarter">
      Bu soru eski zengin metin editörüyle yazılmış. Typst kaynağına çevrildi;
      kaydedersen bu çeviri kalıcı olur.
    </p>
  {/if}

  {#if saveError}
    <p class="ruled-bottom annot shrink-0 bg-red-wash px-rule py-quarter">{saveError}</p>
  {/if}

  <div class="min-h-0 flex-1">
    <QuestionEditor
      {body}
      {questionType}
      points={effectivePoints}
      {outcomes}
      {stats}
      onbodychange={(next) => (body = next)}
    >
      {#snippet answer()}
        {#if questionType === "multiple_choice"}
          <div class="flex items-center gap-half">
            <h3 class="stamp">Seçenekler</h3>
            <label class="pencil flex items-center gap-quarter">
              <input type="checkbox" bind:checked={shuffle} />
              Sınavda karıştır
            </label>
          </div>
          {#each options as _, i}
            <div class="mt-quarter flex items-center gap-half">
              <label class="flex items-center gap-quarter">
                <input type="radio" name="correct" value={i} bind:group={correctIndex} />
                <span class="stamp w-[14px]">{OPTION_LETTERS[i]}</span>
              </label>
              <input
                type="text"
                class="flex-1 border-0 border-b border-rule-strong bg-transparent pb-[2px]
                       font-mono text-[13px] leading-rule focus:border-red focus:outline-none"
                bind:value={options[i]}
                placeholder="Typst kaynağı"
              />
            </div>
          {/each}

        {:else if questionType === "true_false"}
          <h3 class="stamp">Doğru cevap</h3>
          <div class="mt-quarter flex gap-rule">
            <label class="flex items-center gap-quarter">
              <input type="radio" value={true} bind:group={trueFalseAnswer} /> Doğru
            </label>
            <label class="flex items-center gap-quarter">
              <input type="radio" value={false} bind:group={trueFalseAnswer} /> Yanlış
            </label>
          </div>

        {:else if questionType === "fill_in_blank"}
          <div class="flex items-center gap-half">
            <h3 class="stamp">Boşluklar</h3>
            <span class="pencil">Gövdeye #blank(...) ekle, karşılığını buraya yaz</span>
          </div>
          {#each blanks as blank, i}
            <div class="mt-quarter flex items-center gap-half">
              <span class="stamp w-[24px]">b{i + 1}</span>
              <input
                type="text"
                class="flex-1 border-0 border-b border-rule-strong bg-transparent pb-[2px]
                       leading-rule focus:border-red focus:outline-none"
                bind:value={blank.accepted}
                placeholder="Kabul edilen cevaplar, | ile ayrılmış"
              />
              <input
                type="number"
                min="1"
                class="w-[56px] border-0 border-b border-rule-strong bg-transparent pb-[2px]
                       text-right leading-rule tnum focus:border-red focus:outline-none"
                bind:value={blank.points}
              />
              <PenButton kind="quiet" onclick={() => removeBlank(i)}>Sil</PenButton>
            </div>
          {/each}
          <div class="mt-half">
            <PenButton kind="quiet" onclick={addBlank}>Boşluk ekle</PenButton>
          </div>

        {:else}
          <div class="flex items-center gap-rule">
            <h3 class="stamp">Rubrik</h3>
            <label class="pencil flex items-center gap-quarter">
              Cevap için satır
              <input
                type="number"
                min="1"
                class="w-[56px] border-0 border-b border-rule-strong bg-transparent pb-[2px]
                       text-right leading-rule tnum focus:border-red focus:outline-none"
                bind:value={answerLines}
              />
            </label>
          </div>
          {#each rubric as item, i}
            <div class="mt-quarter flex items-center gap-half">
              <input
                type="text"
                class="flex-1 border-0 border-b border-rule-strong bg-transparent pb-[2px]
                       leading-rule focus:border-red focus:outline-none"
                bind:value={item.criterion}
                placeholder="Ölçüt"
              />
              <input
                type="number"
                min="1"
                class="w-[56px] border-0 border-b border-rule-strong bg-transparent pb-[2px]
                       text-right leading-rule tnum focus:border-red focus:outline-none"
                bind:value={item.points}
              />
              <PenButton kind="quiet" onclick={() => removeRubricItem(i)}>Sil</PenButton>
            </div>
          {/each}
          <div class="mt-half">
            <PenButton kind="quiet" onclick={addRubricItem}>Ölçüt ekle</PenButton>
          </div>
        {/if}
      {/snippet}
    </QuestionEditor>
  </div>
</div>
