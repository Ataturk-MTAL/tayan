<script lang="ts">
  import { onMount } from "svelte";
  import PageHead from "$lib/components/shell/PageHead.svelte";
  import PenButton from "$lib/components/shell/PenButton.svelte";
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

  function preview(q: Question): string {
    const source = bodySource(q.body).replace(/\s+/g, " ").trim();
    return source.length > 110 ? `${source.slice(0, 110)}…` : source || "(boş)";
  }
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
      <table class="w-full border-collapse text-[13px]">
        <thead>
          <tr class="ruled-bottom">
            <th class="stamp px-rule py-quarter text-left">Soru</th>
            <th class="stamp px-half py-quarter text-left">Tip</th>
            <th class="stamp px-half py-quarter text-right">Puan</th>
            <th class="stamp px-half py-quarter text-right">Güçlük</th>
            <th class="stamp px-half py-quarter text-right">Ayırt edicilik</th>
            <th class="stamp px-rule py-quarter text-right">Ölçüm</th>
          </tr>
        </thead>
        <tbody>
          {#each shown as q (q.id)}
            {@const badge = scoreBadge(q.stats)}
            <tr
              class="cursor-pointer border-b border-rule align-top hover:bg-paper-lift"
              onclick={() => goto(`/questions/${q.id}`)}
            >
              <td class="px-rule py-half font-mono text-ink-mid">{preview(q)}</td>
              <td class="px-half py-half whitespace-nowrap">{QUESTION_TYPE_LABELS[q.question_type]}</td>
              <td class="px-half py-half text-right tnum">{questionPoints(q)}</td>
              <td class="px-half py-half text-right tnum">
                {q.stats.times_used > 0 ? `${Math.round(q.stats.difficulty_index * 100)}%` : "—"}
              </td>
              <td
                class="px-half py-half text-right tnum"
                class:text-red-deep={q.stats.times_used > 0 && q.stats.discrimination_index < 0.2}
              >
                {q.stats.times_used > 0 ? q.stats.discrimination_index.toFixed(2) : "—"}
              </td>
              <td class="px-rule py-half text-right whitespace-nowrap" style="color: {BADGE_COLOR[badge]}">
                {BADGE_LABEL[badge]}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>
