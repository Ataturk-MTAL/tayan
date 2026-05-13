<script lang="ts">
  import { api } from '$lib/api';
  import { EXAM_STATUS_LABELS, type Exam, type ExamStatus } from '$lib/types';
  import { onMount } from 'svelte';

  let exams    = $state<Exam[]>([]);
  let loading  = $state(true);
  let error    = $state<string | null>(null);
  let deleting = $state<string | null>(null);

  onMount(async () => {
    try { exams = await api.exams.list(); }
    catch (e) { error = String(e); }
    finally { loading = false; }
  });

  async function deleteExam(id: string) {
    if (deleting !== id) { deleting = id; return; }
    try {
      await api.exams.delete(id);
      exams = exams.filter((e) => e.id !== id);
      deleting = null;
    } catch (e) { deleting = null; }
  }

  const STATUS_COLORS: Record<ExamStatus, string> = {
    Draft:     'bg-muted text-muted-foreground',
    Published: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
    Archived:  'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400',
  };

  function fmtDate(d: string) {
    return new Date(d).toLocaleDateString('tr-TR', { day: '2-digit', month: 'short', year: 'numeric' });
  }
</script>

<div class="p-6 max-w-5xl mx-auto">
  <div class="flex items-center justify-between mb-6">
    <h1 class="text-2xl font-bold">Sınavlar</h1>
    <a
      href="/exams/new"
      class="inline-flex items-center gap-1.5 rounded-md bg-primary px-4 py-2 text-sm
             font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
    >
      + Yeni Sınav
    </a>
  </div>

  {#if loading}
    <p class="text-muted-foreground">Yükleniyor…</p>
  {:else if error}
    <div class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">{error}</div>
  {:else if exams.length === 0}
    <div class="rounded-lg border bg-card p-12 text-center text-muted-foreground">
      Henüz sınav oluşturulmadı.
    </div>
  {:else}
    <div class="rounded-lg border overflow-hidden">
      <table class="w-full text-sm">
        <thead class="bg-muted/50 text-muted-foreground">
          <tr>
            <th class="px-4 py-2.5 text-left font-medium">Sınav Adı</th>
            <th class="px-4 py-2.5 text-left font-medium">Ders</th>
            <th class="px-4 py-2.5 text-left font-medium">Sınıf</th>
            <th class="px-4 py-2.5 text-left font-medium">Tarih</th>
            <th class="px-4 py-2.5 text-left font-medium">Soru</th>
            <th class="px-4 py-2.5 text-left font-medium">Durum</th>
            <th class="px-4 py-2.5"></th>
          </tr>
        </thead>
        <tbody class="divide-y">
          {#each exams as exam (exam.id)}
            <tr class="hover:bg-muted/30 transition-colors">
              <td class="px-4 py-3 font-medium">
                <a href="/exams/{exam.id}" class="hover:underline">{exam.meta.title}</a>
              </td>
              <td class="px-4 py-3 text-muted-foreground">{exam.meta.subject}</td>
              <td class="px-4 py-3 text-muted-foreground">{exam.meta.classroom}</td>
              <td class="px-4 py-3 text-muted-foreground">{fmtDate(exam.meta.date)}</td>
              <td class="px-4 py-3 text-muted-foreground">{exam.questions.length}</td>
              <td class="px-4 py-3">
                <span class="rounded-full px-2 py-0.5 text-xs font-medium {STATUS_COLORS[exam.status]}">
                  {EXAM_STATUS_LABELS[exam.status]}
                </span>
              </td>
              <td class="px-4 py-3 text-right">
                <div class="flex items-center justify-end gap-2">
                  <a href="/exams/{exam.id}"
                    class="rounded px-2 py-1 text-xs hover:bg-muted transition-colors">Düzenle</a>
                  <button
                    type="button"
                    onclick={() => deleteExam(exam.id)}
                    title={deleting === exam.id ? 'Tekrar tıkla — sil' : 'Sil'}
                    class="rounded px-2 py-1 text-xs transition-colors
                           {deleting === exam.id
                             ? 'bg-destructive text-white'
                             : 'text-destructive hover:bg-destructive/10'}"
                  >{deleting === exam.id ? 'Emin misin?' : 'Sil'}</button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
