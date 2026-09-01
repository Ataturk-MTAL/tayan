<script lang="ts">
  import { onMount } from "svelte";
  import PageHead from "$lib/components/shell/PageHead.svelte";
  import PenButton from "$lib/components/shell/PenButton.svelte";
  import { api } from "$lib/api";
  import { errorText } from "$lib/editor/diagnostics";
  import { EXAM_STATUS_LABELS, type Exam } from "$lib/types";
  import { goto } from "$app/navigation";

  let exams = $state<Exam[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);

  onMount(load);

  async function load() {
    loading = true;
    try {
      exams = await api.exams.list();
      loadError = null;
    } catch (err: unknown) {
      loadError = errorText(err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <PageHead title="Sınavlar" count={loading ? null : `${exams.length}`}>
    <PenButton kind="ink" onclick={() => goto("/exams/new")}>Yeni sınav</PenButton>
  </PageHead>

  <div class="min-h-0 flex-1 overflow-auto">
    {#if loading}
      <p class="pencil p-rule">Sınavlar okunuyor…</p>
    {:else if loadError}
      <p class="annot p-rule">{loadError}</p>
    {:else if exams.length === 0}
      <p class="pencil p-rule">Henüz sınav yok.</p>
    {:else}
      <table class="w-full border-collapse text-[13px]">
        <thead>
          <tr class="ruled-bottom">
            <th class="stamp px-rule py-quarter text-left">Sınav</th>
            <th class="stamp px-half py-quarter text-left">Ders</th>
            <th class="stamp px-half py-quarter text-left">Sınıf</th>
            <th class="stamp px-half py-quarter text-right">Soru</th>
            <th class="stamp px-half py-quarter text-right">Süre</th>
            <th class="stamp px-rule py-quarter text-right">Durum</th>
          </tr>
        </thead>
        <tbody>
          {#each exams as exam (exam.id)}
            <tr
              class="cursor-pointer border-b border-rule hover:bg-paper-lift"
              onclick={() => goto(`/exams/${exam.id}`)}
            >
              <td class="px-rule py-half font-semibold">{exam.meta.title}</td>
              <td class="px-half py-half">{exam.meta.subject}</td>
              <td class="px-half py-half">{exam.meta.classroom}</td>
              <td class="px-half py-half text-right tnum">{exam.questions.length}</td>
              <td class="px-half py-half text-right tnum">{exam.meta.duration_min} dk</td>
              <td
                class="px-rule py-half text-right"
                class:text-red-deep={exam.status === "Published"}
              >
                {EXAM_STATUS_LABELS[exam.status]}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>
