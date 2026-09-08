<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import PageShell from "$lib/components/shell/PageShell.svelte";
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

<!--
  scroll={false}: editör kendi yüksekliğini ve kendi kaydırıcılarını
  yönetiyor (kaynak, önizleme, panel — üçü de kendi içinde kayar). Kabuk
  kaydırmayı üstlenirse iç içe iki kaydırıcı fare tekerleğinin hangisini
  süreceğini belirsizleştirirdi.
-->
<PageShell title="Soruyu düzenle" subtitle={question?.meta?.title || null} scroll={false}>
  {#if loading}
    <p class="p-4 text-sm text-gray-500 dark:text-gray-400">Soru okunuyor…</p>
  {:else if loadError}
    <p class="p-4 text-sm text-red-600 dark:text-red-400">{loadError}</p>
  {:else if question}
    <QuestionForm
      initialType={question.question_type}
      initialBody={bodySource(question.body)}
      stats={question.stats}
      legacyWarning={isLegacyBody(question.body)}
      existing={question}
    />
  {/if}
</PageShell>
