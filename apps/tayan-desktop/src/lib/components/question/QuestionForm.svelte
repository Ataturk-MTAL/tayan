<script lang="ts">
  import QuestionEditor from "./QuestionEditor.svelte";
  import RuledField from "../shell/RuledField.svelte";
  import PenButton from "../shell/PenButton.svelte";
  import { typstBody } from "$lib/question/body";
  import {
    parseOptions,
    parseTrueFalse,
    parseBlanks,
    parseAnswerLines,
    OPTION_LETTERS,
  } from "$lib/question/templates";
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

  let questionType = $state<QuestionType>(initialType);
  let body = $state(initialBody);
  let points = $state(5);
  let outcomeText = $state("");

  let saving = $state(false);
  let saveError = $state<string | null>(null);

  let outcomes = $derived(
    outcomeText
      .split(/[,\s]+/)
      .map((o) => o.trim())
      .filter(Boolean),
  );

  /**
   * Sorunun yapısı gövdenin KENDİSİNDEN okunur; ayrı bir form paneli yok.
   * Bu, öğretmenin tek yere bakmasını sağlar ve cevap anahtarının kâğıtta
   * görünenden ayrı düşmesini imkânsız kılar.
   */
  let parsed = $derived.by(() => {
    if (body.trim() === "") return "Soru gövdesi boş.";

    if (questionType === "multiple_choice") return parseOptions(body);
    if (questionType === "true_false") return parseTrueFalse(body);
    if (questionType === "fill_in_blank") return parseBlanks(body);
    return parseAnswerLines(body);
  });

  /** Kalıp hatası yazarken de görünür; kaydetmeyi beklemeye gerek yok. */
  let structureError = $derived(typeof parsed === "string" ? parsed : null);

  let blankCount = $derived(
    questionType === "fill_in_blank" && Array.isArray(parsed) ? parsed.length : 0,
  );

  let effectivePoints = $derived(
    questionType === "fill_in_blank" ? points * Math.max(blankCount, 1) : points,
  );

  async function save() {
    if (structureError !== null) {
      saveError = structureError;
      return;
    }

    saving = true;
    saveError = null;

    try {
      const bodyNodes = typstBody(body);

      if (questionType === "multiple_choice") {
        const { options, correctIndex } = parsed as {
          options: string[];
          correctIndex: number;
        };
        await api.questions.addMultipleChoice({
          points,
          outcomes,
          body: bodyNodes,
          options: options.map((source, i) => ({
            id: OPTION_LETTERS[i],
            body: typstBody(source),
            correct: i === correctIndex,
          })),
          shuffle: false,
        });
      } else if (questionType === "true_false") {
        await api.questions.addTrueFalse({
          points,
          outcomes,
          body: bodyNodes,
          correct_answer: parsed as boolean,
        });
      } else if (questionType === "fill_in_blank") {
        const blanks = parsed as Array<{ accepted: string[] }>;
        await api.questions.addFillInBlank({
          outcomes,
          body: bodyNodes,
          blanks: blanks.map((b, i) => ({
            id: `b${i + 1}`,
            accepted_answers: b.accepted,
            points,
          })),
        });
      } else {
        await api.questions.addClassic({
          points,
          outcomes,
          body: bodyNodes,
          sample_answer: null,
          rubric: [],
          answer_space: { Lines: parsed as number },
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

    <div class="w-[110px]">
      <RuledField
        label="Puan"
        hint={questionType === "fill_in_blank" ? `boşluk başına — toplam ${effectivePoints}` : null}
      >
        <input type="number" min="1" bind:value={points} />
      </RuledField>
    </div>

    <div class="min-w-[200px] flex-1">
      <RuledField label="Kazanım" hint="Boşluk veya virgülle ayır — MAT.9.1.2">
        <input type="text" bind:value={outcomeText} placeholder="MAT.9.1.2" />
      </RuledField>
    </div>

    <PenButton kind="ink" disabled={saving || structureError !== null} onclick={save}>
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
  {:else if structureError}
    <p class="ruled-bottom annot shrink-0 bg-red-wash px-rule py-quarter">{structureError}</p>
  {/if}

  <div class="min-h-0 flex-1">
    <QuestionEditor
      {body}
      {questionType}
      points={effectivePoints}
      {outcomes}
      {stats}
      onbodychange={(next) => (body = next)}
    />
  </div>
</div>
