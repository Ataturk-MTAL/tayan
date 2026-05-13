<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { Exam, ExamResult, Student, Classroom } from '$lib/types';

  let exams          = $state<Exam[]>([]);
  let selectedExamId = $state<string | null>(null);
  let results        = $state<ExamResult[]>([]);
  let studentMap     = $state<Map<string, Student>>(new Map());
  let loading        = $state(true);
  let resultsLoading = $state(false);
  let error          = $state<string | null>(null);

  let selectedExam = $derived(exams.find((e) => e.id === selectedExamId) ?? null);

  // ── Stats ──────────────────────────────────────────────────────────────────
  let stats = $derived((() => {
    if (results.length === 0) return null;
    const pcts = results.map((r) =>
      r.total_points_max > 0 ? (r.total_points_earned / r.total_points_max) * 100 : 0
    );
    const avg  = pcts.reduce((s, v) => s + v, 0) / pcts.length;
    const max  = Math.max(...pcts);
    const min  = Math.min(...pcts);
    const pass = pcts.filter((p) => p >= 50).length;
    return { avg, max, min, pass, total: pcts.length };
  })());

  // ── Outcome breakdown ──────────────────────────────────────────────────────
  type OutcomeStat = { outcome: string; avg_pct: number; count: number };
  let outcomeStats = $derived((() => {
    const map = new Map<string, { sum: number; count: number }>();
    for (const r of results) {
      for (const op of r.outcome_performance) {
        const entry = map.get(op.outcome) ?? { sum: 0, count: 0 };
        entry.sum   += op.score_pct;
        entry.count += 1;
        map.set(op.outcome, entry);
      }
    }
    return Array.from(map.entries())
      .map(([outcome, { sum, count }]): OutcomeStat => ({
        outcome,
        avg_pct: sum / count,
        count,
      }))
      .sort((a, b) => a.avg_pct - b.avg_pct);
  })());

  onMount(async () => {
    try {
      [exams] = await Promise.all([api.exams.list()]);
      // Pre-load all students for name lookup
      await loadAllStudents();
    } catch (e) { error = String(e); }
    finally { loading = false; }
  });

  async function loadAllStudents() {
    try {
      const classes: Classroom[] = await api.students.listClassrooms();
      const allStudents = (
        await Promise.all(classes.map((c) => api.students.listByClassroom(c.id)))
      ).flat();
      studentMap = new Map(allStudents.map((s) => [s.id, s]));
    } catch {
      // non-critical: names won't show but results still display
    }
  }

  async function selectExam(id: string) {
    selectedExamId = id;
    resultsLoading = true;
    try {
      results = await api.results.getByExam(id);
      results = results.slice().sort((a, b) => b.total_points_earned - a.total_points_earned);
    } catch (e) { error = String(e); }
    finally { resultsLoading = false; }
  }

  function studentName(id: string): string {
    const s = studentMap.get(id);
    return s ? `${s.first_name} ${s.last_name}` : id.slice(0, 8) + '…';
  }

  function studentNo(id: string): string {
    return studentMap.get(id)?.number ?? '—';
  }

  function pctBar(pct: number) {
    const w   = Math.round(pct);
    const col = pct >= 70 ? 'bg-emerald-500' : pct >= 50 ? 'bg-amber-500' : 'bg-red-500';
    return { w, col };
  }

  function fmtDate(d: string) {
    return new Date(d).toLocaleDateString('tr-TR', { day: '2-digit', month: 'short', year: 'numeric' });
  }
</script>

<div class="p-6 max-w-6xl mx-auto space-y-6">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div class="flex items-center justify-between">
    <h1 class="text-2xl font-bold tracking-tight">Sınav Analizi</h1>
  </div>

  {#if loading}
    <p class="text-muted-foreground">Yükleniyor…</p>
  {:else if error}
    <div class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
      {error}
    </div>
  {:else}

    <!-- ── Exam selector ──────────────────────────────────────────────────── -->
    <div class="flex items-center gap-3">
      <label class="text-sm font-medium shrink-0" for="exam-sel">Sınav:</label>
      {#if exams.length === 0}
        <p class="text-sm text-muted-foreground">Henüz sınav yok.</p>
      {:else}
        <select
          id="exam-sel"
          value={selectedExamId ?? ''}
          onchange={(e) => {
            const v = (e.currentTarget as HTMLSelectElement).value;
            if (v) selectExam(v);
          }}
          class="rounded-md border border-input bg-background px-3 py-1.5 text-sm
                 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <option value="" disabled>Sınav seçin…</option>
          {#each exams as exam (exam.id)}
            <option value={exam.id}>
              {exam.meta.title} — {exam.meta.subject} / {exam.meta.classroom}
              ({fmtDate(exam.meta.date)})
            </option>
          {/each}
        </select>
      {/if}
    </div>

    {#if selectedExamId}
      {#if resultsLoading}
        <p class="text-muted-foreground text-sm">Sonuçlar yükleniyor…</p>
      {:else if results.length === 0}
        <div class="rounded-lg border bg-card p-12 text-center text-muted-foreground text-sm">
          Bu sınav için henüz sonuç girilmemiş.
        </div>
      {:else}

        <!-- ── Stats cards ──────────────────────────────────────────────────── -->
        {#if stats}
          <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
            <div class="rounded-lg border bg-card px-4 py-3">
              <p class="text-xs text-muted-foreground">Ortalama</p>
              <p class="text-2xl font-bold mt-0.5">{stats.avg.toFixed(1)}<span class="text-sm font-normal text-muted-foreground ml-0.5">%</span></p>
            </div>
            <div class="rounded-lg border bg-card px-4 py-3">
              <p class="text-xs text-muted-foreground">En Yüksek</p>
              <p class="text-2xl font-bold mt-0.5 text-emerald-600">{stats.max.toFixed(1)}<span class="text-sm font-normal text-muted-foreground ml-0.5">%</span></p>
            </div>
            <div class="rounded-lg border bg-card px-4 py-3">
              <p class="text-xs text-muted-foreground">En Düşük</p>
              <p class="text-2xl font-bold mt-0.5 text-red-500">{stats.min.toFixed(1)}<span class="text-sm font-normal text-muted-foreground ml-0.5">%</span></p>
            </div>
            <div class="rounded-lg border bg-card px-4 py-3">
              <p class="text-xs text-muted-foreground">Geçme Oranı ≥50%</p>
              <p class="text-2xl font-bold mt-0.5">{stats.pass}<span class="text-sm font-normal text-muted-foreground ml-0.5">/ {stats.total}</span></p>
            </div>
          </div>
        {/if}

        <!-- ── Results table ────────────────────────────────────────────────── -->
        <div class="space-y-2">
          <h2 class="font-semibold">Öğrenci Sonuçları</h2>
          <div class="rounded-lg border overflow-hidden">
            <table class="w-full text-sm">
              <thead class="bg-muted/50 text-muted-foreground">
                <tr>
                  <th class="px-3 py-2.5 text-left font-medium w-10">Sıra</th>
                  <th class="px-4 py-2.5 text-left font-medium w-20">No</th>
                  <th class="px-4 py-2.5 text-left font-medium">Ad Soyad</th>
                  <th class="px-4 py-2.5 text-right font-medium">Puan</th>
                  <th class="px-4 py-2.5 text-left font-medium w-48">Başarı</th>
                  <th class="px-4 py-2.5 text-right font-medium">%</th>
                </tr>
              </thead>
              <tbody class="divide-y">
                {#each results as r, i (r.id)}
                  {@const pct = r.total_points_max > 0
                    ? (r.total_points_earned / r.total_points_max) * 100 : 0}
                  {@const bar = pctBar(pct)}
                  <tr class="hover:bg-muted/30 transition-colors">
                    <td class="px-3 py-2.5 text-center text-muted-foreground">{i + 1}</td>
                    <td class="px-4 py-2.5 font-mono text-muted-foreground">{studentNo(r.student_id)}</td>
                    <td class="px-4 py-2.5 font-medium">{studentName(r.student_id)}</td>
                    <td class="px-4 py-2.5 text-right tabular-nums">
                      {r.total_points_earned.toFixed(1)} / {r.total_points_max.toFixed(0)}
                    </td>
                    <td class="px-4 py-2.5">
                      <div class="h-2 w-full rounded-full bg-muted overflow-hidden">
                        <div class="h-full rounded-full {bar.col} transition-all"
                          style="width: {bar.w}%"></div>
                      </div>
                    </td>
                    <td class="px-4 py-2.5 text-right tabular-nums font-medium
                               {pct >= 70 ? 'text-emerald-600' : pct >= 50 ? 'text-amber-600' : 'text-red-500'}">
                      {pct.toFixed(1)}%
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>

        <!-- ── Outcome breakdown ────────────────────────────────────────────── -->
        {#if outcomeStats.length > 0}
          <div class="space-y-2">
            <h2 class="font-semibold">Kazanım Analizi</h2>
            <div class="rounded-lg border overflow-hidden">
              <table class="w-full text-sm">
                <thead class="bg-muted/50 text-muted-foreground">
                  <tr>
                    <th class="px-4 py-2.5 text-left font-medium">Kazanım</th>
                    <th class="px-4 py-2.5 text-left font-medium w-48">Başarı</th>
                    <th class="px-4 py-2.5 text-right font-medium">Ortalama</th>
                  </tr>
                </thead>
                <tbody class="divide-y">
                  {#each outcomeStats as os (os.outcome)}
                    {@const bar = pctBar(os.avg_pct)}
                    <tr class="hover:bg-muted/30 transition-colors">
                      <td class="px-4 py-2.5 font-mono text-sm">{os.outcome}</td>
                      <td class="px-4 py-2.5">
                        <div class="h-2 w-full rounded-full bg-muted overflow-hidden">
                          <div class="h-full rounded-full {bar.col}"
                            style="width: {bar.w}%"></div>
                        </div>
                      </td>
                      <td class="px-4 py-2.5 text-right tabular-nums font-medium
                                 {os.avg_pct >= 70 ? 'text-emerald-600' : os.avg_pct >= 50 ? 'text-amber-600' : 'text-red-500'}">
                        {os.avg_pct.toFixed(1)}%
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}

      {/if}
    {/if}
  {/if}
</div>
