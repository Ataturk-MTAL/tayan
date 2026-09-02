<script lang="ts">
  import { onMount } from "svelte";
  import PageHead from "$lib/components/shell/PageHead.svelte";
  import PenButton from "$lib/components/shell/PenButton.svelte";
  import QuestionCard from "$lib/components/question/QuestionCard.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { bodySource } from "$lib/question/body";
  import {
    QUESTION_TYPE_LABELS,
    questionPoints,
    scoreBadge,
    type Question,
    type ScoreBadge,
  } from "$lib/types";
  import { goto } from "$app/navigation";

  let questions = $state<Question[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let filter = $state<"all" | "untested" | "weak">("all");

  const BADGE_LABEL: Record<ScoreBadge, string> = {
    excellent: "Çok iyi",
    good: "İyi",
    fair: "Orta",
    poor: "Zayıf",
    untested: "Denenmemiş",
  };
  const BADGE_COLOR: Record<ScoreBadge, string> = {
    excellent: "var(--color-mark-excellent)",
    good: "var(--color-mark-good)",
    fair: "var(--color-mark-fair)",
    poor: "var(--color-mark-poor)",
    untested: "var(--color-mark-untested)",
  };

  onMount(load);

  async function load() {
    loading = true;
    try {
      questions = await api.questions.list();
      loadError = null;
    } catch (err: unknown) {
      loadError = errorText(err);
    } finally {
      loading = false;
    }
  }

  let shown = $derived(
    questions.filter((q) => {
      if (filter === "untested") return q.stats.times_used === 0;
      if (filter === "weak")
        return q.stats.times_used > 0 && q.stats.discrimination_index < 0.2;
      return true;
    }),
  );

</script>

<div class="flex h-full min-h-0 flex-col">
  <PageHead title="Soru bankası" count={loading ? null : `${shown.length} / ${questions.length}`}>
    <PenButton kind="quiet" onclick={() => (filter = "all")}>Tümü</PenButton>
    <PenButton kind="quiet" onclick={() => (filter = "untested")}>Denenmemiş</PenButton>
    <PenButton kind="red" onclick={() => (filter = "weak")}>Zayıf ayırt edici</PenButton>
    <PenButton kind="ink" onclick={() => goto("/questions/new")}>Soru yaz</PenButton>
  </PageHead>

  <div class="min-h-0 flex-1 overflow-auto">
    {#if loading}
      <p class="pencil p-rule">Banka okunuyor…</p>
    {:else if loadError}
      <p class="annot p-rule">{loadError}</p>
    {:else if shown.length === 0}
      <div class="p-rule">
        <p class="pencil">
          {questions.length === 0
            ? "Bankada henüz soru yok."
            : "Bu süzgece uyan soru yok."}
        </p>
      </div>
    {:else}
      <!--
        Fiş dizimi: her soru dizilmiş hâliyle bir kâğıt parçası. Tablo, soruyu
        ham Typst kaynağı olarak gösteriyordu; öğretmen kendi bankasında
        `#secenekler(dogru:` görüyordu.
      -->
      <div class="grid gap-rule p-rule" style="grid-template-columns: repeat(auto-fill, minmax(260px, 1fr))">
        {#each shown as q (q.id)}
          {@const badge = scoreBadge(q.stats)}
          <button
            type="button"
            class="sheet flex flex-col text-left transition-shadow hover:shadow-lg"
            onclick={() => goto(`/questions/${q.id}`)}
          >
            <QuestionCard body={bodySource(q.body)} />

            <div class="ruled-top flex flex-col gap-quarter bg-paper-lift px-half py-quarter">
              <div class="flex items-baseline gap-quarter">
                <span class="stamp">{QUESTION_TYPE_LABELS[q.question_type]}</span>
                <span class="pencil tnum ml-auto">{questionPoints(q)} p</span>
              </div>

              <div class="flex items-baseline gap-quarter text-[11px] leading-rule">
                {#if q.meta?.subject}
                  <span class="text-ink-mid">{q.meta.subject}</span>
                {/if}
                {#if q.meta?.grade}
                  <span class="pencil">{q.meta.grade}. sınıf</span>
                {/if}
              </div>

              {#if q.outcomes.length > 0}
                <span class="pencil font-mono text-[10px]">{q.outcomes.join(" · ")}</span>
              {/if}

              <!-- Ölçüm rengi 3px çizgi: kırmızı yalnız zayıf ayırt edicilikte. -->
              <div class="mt-quarter flex items-center gap-quarter">
                <span class="h-[3px] w-[26px]" style="background: {BADGE_COLOR[badge]}"></span>
                <span class="text-[11px] leading-rule" style="color: {BADGE_COLOR[badge]}">
                  {BADGE_LABEL[badge]}
                </span>
                {#if q.stats.times_used > 0}
                  <span class="pencil tnum ml-auto text-[11px]">
                    {q.stats.discrimination_index.toFixed(2)}
                  </span>
                {/if}
              </div>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>
