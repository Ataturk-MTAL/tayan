<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import type { Exam, Question } from "$lib/types";

  let questions = $state<Question[]>([]);
  let exams = $state<Exam[]>([]);
  let loadError = $state<string | null>(null);
  let loading = $state(true);

  onMount(async () => {
    try {
      [questions, exams] = await Promise.all([api.questions.list(), api.exams.list()]);
    } catch (err: unknown) {
      loadError = errorText(err);
    } finally {
      loading = false;
    }
  });

  let untested = $derived(questions.filter((q) => q.stats.times_used === 0).length);
  let weak = $derived(
    questions.filter((q) => q.stats.times_used > 0 && q.stats.discrimination_index < 0.2).length,
  );
  let drafts = $derived(exams.filter((e) => e.status === "Draft").length);
</script>

<!--
  Üç çekirdek eşit ağırlıkta: dizgi, ölçüm, hız. Hiçbiri diğerinin alt sekmesi
  değil, bu yüzden üçü de aynı genişlikte cetvelli bölge.
-->
<div class="h-full overflow-auto">
  <div class="mx-auto max-w-[900px] px-rule py-rule">
    <h1>Bugün ne yapacaksın?</h1>

    {#if loadError}
      <p class="annot mt-half">{loadError}</p>
    {/if}

    <div class="mt-rule grid grid-cols-3 border-t border-l border-rule-strong">
      <a
        href="/questions/new"
        class="border-r border-b border-rule-strong bg-paper-lift p-rule no-underline
               transition-colors hover:bg-paper-sunk"
      >
        <h2>Soru yaz</h2>
        <p class="pencil mt-quarter">
          Typst kaynağını yaz, kâğıda ne basılacağını yanında gör.
        </p>
        <p class="mt-half text-[28px] font-bold leading-[40px] tnum">
          {loading ? "—" : questions.length}
        </p>
        <p class="stamp">bankadaki soru</p>
      </a>

      <a
        href="/exams/new"
        class="border-r border-b border-rule-strong bg-paper-lift p-rule no-underline
               transition-colors hover:bg-paper-sunk"
      >
        <h2>Sınav kur</h2>
        <p class="pencil mt-quarter">
          Bankadan soru seç, puan bütçesini doldur, baskıya çıkar.
        </p>
        <p class="mt-half text-[28px] font-bold leading-[40px] tnum">
          {loading ? "—" : drafts}
        </p>
        <p class="stamp">taslak sınav</p>
      </a>

      <a
        href="/analysis"
        class="border-r border-b border-rule-strong bg-paper-lift p-rule no-underline
               transition-colors hover:bg-paper-sunk"
      >
        <h2>Sonucu oku</h2>
        <p class="pencil mt-quarter">
          Sınav sonuçları soruya geri döner; zayıf soru kendini belli eder.
        </p>
        <p class="mt-half text-[28px] font-bold leading-[40px] tnum" style="color: var(--color-red)">
          {loading ? "—" : weak}
        </p>
        <p class="stamp">ayırt ediciliği düşük soru</p>
      </a>
    </div>

    {#if !loading && untested > 0}
      <p class="annot mt-rule border-t border-rule pt-half">
        {untested} soru hiç uygulanmadı. Ölçümü olmayan soru, kalitesi bilinmeyen sorudur.
      </p>
    {/if}
  </div>
</div>
