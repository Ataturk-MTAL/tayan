<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import QuestionForm from "$lib/components/question/QuestionForm.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { bodySource, isLegacyBody } from "$lib/question/body";
  import type { Question } from "$lib/types";

  let question = $state<Question | null>(null);
  let loading = $state(true);
  let loadError = $state<string | null>(null);

  onMount(async () => {
    try {
      const all = await api.questions.list();
      question = all.find((q) => q.id === page.params.id) ?? null;
      if (!question) loadError = "Bu soru bankada bulunamadı.";
    } catch (err: unknown) {
      loadError = errorText(err);
    } finally {
      loading = false;
    }
  });
</script>

{#if loading}
  <p class="pencil p-rule">Soru okunuyor…</p>
{:else if loadError}
  <p class="annot p-rule">{loadError}</p>
{:else if question}
  <QuestionForm
    initialType={question.question_type}
    initialBody={bodySource(question.body)}
    stats={question.stats}
    legacyWarning={isLegacyBody(question.body)}
    existing={question}
  />
{/if}
