<script lang="ts">
  import { onMount } from "svelte";
  import { Alert, Card } from "flowbite-svelte";
  import PageShell from "$lib/components/shell/PageShell.svelte";
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

<PageShell title="Bugün ne yapacaksın?">
  <div class="mx-auto max-w-[900px]">
    {#if loadError}
      <Alert color="red" class="mb-6">{loadError}</Alert>
    {/if}

    <!--
      Üç çekirdek eşit ağırlıkta: dizgi, ölçüm, hız. Hiçbiri diğerinin alt sekmesi
      değil, bu yüzden üçü de aynı boyutta kart.
    -->
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
      <Card href="/questions/new" size="xl" class="hover:bg-gray-50 dark:hover:bg-gray-700">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Soru yaz</h2>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Typst kaynağını yaz, kâğıda ne basılacağını yanında gör.
        </p>
        <p class="mt-4 text-[28px] font-bold leading-[40px] tnum text-gray-900 dark:text-white">
          {loading ? "—" : questions.length}
        </p>
        <p class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
          bankadaki soru
        </p>
      </Card>

      <Card href="/exams/new" size="xl" class="hover:bg-gray-50 dark:hover:bg-gray-700">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Sınav kur</h2>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Bankadan soru seç, puan bütçesini doldur, baskıya çıkar.
        </p>
        <p class="mt-4 text-[28px] font-bold leading-[40px] tnum text-gray-900 dark:text-white">
          {loading ? "—" : drafts}
        </p>
        <p class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
          taslak sınav
        </p>
      </Card>

      <!--
        Bu kart tek başına kırmızı: uygulamada kırmızı yalnız değerlendirme
        için ayrılmış (bkz. app.css), "zayıf soru" ölçütü bu ayrımı hak ediyor.
      -->
      <Card href="/analysis" size="xl" class="hover:bg-gray-50 dark:hover:bg-gray-700">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Sonucu oku</h2>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Sınav sonuçları soruya geri döner; zayıf soru kendini belli eder.
        </p>
        <p class="mt-4 text-[28px] font-bold leading-[40px] tnum text-red-600 dark:text-red-500">
          {loading ? "—" : weak}
        </p>
        <p class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
          ayırt ediciliği düşük soru
        </p>
      </Card>
    </div>

    {#if !loading && untested > 0}
      <Alert color="gray" class="mt-6">
        {untested} soru hiç uygulanmadı. Ölçümü olmayan soru, kalitesi bilinmeyen sorudur.
      </Alert>
    {/if}
  </div>
</PageShell>
