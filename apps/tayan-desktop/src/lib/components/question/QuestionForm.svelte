<script lang="ts">
  import QuestionEditor from "./QuestionEditor.svelte";
  import { typstBody } from "$lib/question/body";
  import type { ContentNode } from "$lib/types";
  import {
    parseOptions,
    parseTrueFalse,
    parseBlanks,
    parseAnswerLines,
    OPTION_LETTERS,
  } from "$lib/question/templates";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { QUESTION_TYPE_LABELS, questionPoints, type Question, type QuestionStats } from "$lib/types";
  import { goto } from "$app/navigation";

  type QuestionType = Question["question_type"];

  type Props = {
    initialType?: QuestionType;
    initialBody?: string;
    stats?: QuestionStats | null;
    legacyWarning?: boolean;
    /**
     * Var olan bir soru düzenleniyorsa o soru. null ise yeni soru.
     *
     * Bu ayrım olmadan düzenleme her kaydedişte YENİ soru üretiyordu: banka
     * kopyalarla doluyor, sınavlar eski kimliğe atıf yapmaya devam ediyor ve
     * yapılan değişiklik sınavda hiç görünmüyordu.
     */
    existing?: Question | null;
  };

  let {
    initialType = "multiple_choice",
    initialBody = "",
    stats = null,
    legacyWarning = false,
    existing = null,
  }: Props = $props();

  let questionType = $state<QuestionType>(initialType);
  let body = $state(initialBody);
  /**
   * Sorunun kendi puanı artık editörde düzenlenmiyor: puan soruya değil,
   * sorunun SINAVDAKİ kullanımına aittir. Aynı soru bir yazılıda 5, başkasında
   * 10 puan edebilir; sınav ekranında belirlenir.
   *
   * Bu alan yalnızca sınavda puan verilmediğinde kullanılan yedektir. Var olan
   * soruda korunur, yenide varsayılana düşer.
   */
  const DEFAULT_POINTS = 5;
  let points = $state(existing ? questionPoints(existing) : DEFAULT_POINTS);
  let outcomeText = $state(existing ? existing.outcomes.join(" ") : "");

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

  /**
   * Var olan soruyu, kimliğini ve istatistiğini KORUYARAK günceller.
   *
   * stats korunmak zorunda: madde analizi geçmişi soruya bağlıdır. Yeni bir
   * nesne üretip stats'ı sıfırlamak, o sorunun ölçülmüş tarihini silmek olur.
   */
  function buildUpdated(base: Question, bodyNodes: ContentNode[]): Question {
    const common = {
      id: base.id,
      outcomes,
      body: bodyNodes,
      stats: base.stats,
    };

    if (questionType === "multiple_choice") {
      const { options, correctIndex, shuffle } = parsed as {
        options: string[];
        correctIndex: number;
        shuffle: boolean;
      };
      return {
        ...common,
        question_type: "multiple_choice",
        points,
        options: options.map((source, i) => ({
          id: OPTION_LETTERS[i],
          body: typstBody(source),
          correct: i === correctIndex,
        })),
        shuffle,
      };
    }

    if (questionType === "true_false") {
      return {
        ...common,
        question_type: "true_false",
        points,
        correct_answer: parsed as boolean,
      };
    }

    if (questionType === "fill_in_blank") {
      const blanks = parsed as Array<{ accepted: string[] }>;
      return {
        ...common,
        question_type: "fill_in_blank",
        blanks: blanks.map((b, i) => ({
          id: `b${i + 1}`,
          accepted_answers: b.accepted,
          points,
          case_sensitive: false,
        })),
      };
    }

    return {
      ...common,
      question_type: "classic",
      points,
      sample_answer: null,
      rubric: [],
      answer_space: { Lines: parsed as number },
    };
  }

  async function save() {
    if (structureError !== null) {
      saveError = structureError;
      return;
    }

    saving = true;
    saveError = null;

    try {
      const bodyNodes = typstBody(body);

      if (existing) {
        await api.questions.update(buildUpdated(existing, bodyNodes));
        await goto("/questions");
        return;
      }

      if (questionType === "multiple_choice") {
        const { options, correctIndex, shuffle } = parsed as {
          options: string[];
          correctIndex: number;
          shuffle: boolean;
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
          shuffle,
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
  {#if legacyWarning}
    <p class="ruled-bottom annot shrink-0 bg-red-wash px-rule py-quarter">
      Bu soru eski zengin metin editörüyle yazılmış. Typst kaynağına çevrildi;
      kaydedersen bu çeviri kalıcı olur.
    </p>
  {/if}

  <!--
    Kalıp hatası (structureError) artık burada BASILMAZ: panelde, kazanım ve
    puanın hemen altında duruyor. Kaydetme hatası ise ayrı bir şey — ağa/diske
    giden bir çağrının başarısızlığı — ve üstte kalır.
  -->
  {#if saveError}
    <p class="ruled-bottom annot shrink-0 bg-red-wash px-rule py-quarter">{saveError}</p>
  {/if}

  <div class="min-h-0 flex-1">
    <QuestionEditor
      {body}
      {questionType}
      {outcomeText}
      {points}
      {stats}
      {structureError}
      {saving}
      saveLabel={saving ? "Kaydediliyor…" : existing ? "Güncelle" : "Kaydet"}
      onbodychange={(next) => (body = next)}
      onquestiontypechange={(next) => (questionType = next)}
      onoutcometextchange={(next) => (outcomeText = next)}
      onpointschange={(next) => (points = next)}
      onsave={save}
    />
  </div>
</div>
