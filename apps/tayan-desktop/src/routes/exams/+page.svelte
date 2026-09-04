<script lang="ts">
  import { onMount } from "svelte";
  import {
    Alert,
    Badge,
    Button,
    Spinner,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from "flowbite-svelte";
  import PageShell from "$lib/components/shell/PageShell.svelte";
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

<PageShell title="Sınavlar" subtitle={loading ? null : `${exams.length} sınav`}>
  {#snippet actions()}
    <Button size="sm" onclick={() => goto("/exams/new")}>Yeni sınav</Button>
  {/snippet}

  {#if loading}
    <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
      <Spinner size="5" />
      Sınavlar okunuyor…
    </div>
  {:else if loadError}
    <Alert color="red">{loadError}</Alert>
  {:else if exams.length === 0}
    <p class="text-gray-500 dark:text-gray-400">Henüz sınav yok.</p>
  {:else}
    <Table hoverable>
      <TableHead>
        <TableHeadCell>Sınav</TableHeadCell>
        <TableHeadCell>Ders</TableHeadCell>
        <TableHeadCell>Sınıf</TableHeadCell>
        <TableHeadCell class="text-right">Soru</TableHeadCell>
        <TableHeadCell class="text-right">Süre</TableHeadCell>
        <TableHeadCell class="text-right">Durum</TableHeadCell>
      </TableHead>
      <TableBody>
        {#each exams as exam (exam.id)}
          <TableBodyRow class="cursor-pointer" onclick={() => goto(`/exams/${exam.id}`)}>
            <TableBodyCell class="font-semibold">{exam.meta.title}</TableBodyCell>
            <TableBodyCell>{exam.meta.subject}</TableBodyCell>
            <TableBodyCell>{exam.meta.classroom}</TableBodyCell>
            <TableBodyCell class="text-right tnum">{exam.questions.length}</TableBodyCell>
            <TableBodyCell class="text-right tnum">{exam.meta.duration_min} dk</TableBodyCell>
            <TableBodyCell class="text-right">
              <!-- Yayına çıkmış sınav artık düzenlenemez; kırmızı rozet bu tekliği hatırlatıyor. -->
              <Badge color={exam.status === "Published" ? "red" : "gray"}>
                {EXAM_STATUS_LABELS[exam.status]}
              </Badge>
            </TableBodyCell>
          </TableBodyRow>
        {/each}
      </TableBody>
    </Table>
  {/if}
</PageShell>
