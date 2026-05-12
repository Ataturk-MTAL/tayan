<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import {
    type Question,
    bodyPreview,
    questionPoints,
    scoreBadge,
    QUESTION_TYPE_LABELS,
  } from '$lib/types';

  let questions = $state<Question[]>([]);
  let loading   = $state(true);
  let error     = $state<string | null>(null);

  onMount(async () => {
    try {
      questions = await api.questions.list();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const BADGE_STYLES = {
    excellent: 'bg-emerald-100 text-emerald-800',
    good:      'bg-blue-100 text-blue-800',
    fair:      'bg-amber-100 text-amber-800',
    poor:      'bg-red-100 text-red-800',
    untested:  'bg-muted text-muted-foreground',
  };

  const BADGE_LABELS = {
    excellent: 'Mükemmel',
    good:      'İyi',
    fair:      'Orta',
    poor:      'Zayıf',
    untested:  'Test Edilmedi',
  };

  const TYPE_STYLES: Record<Question['question_type'], string> = {
    multiple_choice: 'bg-violet-100 text-violet-800',
    true_false:      'bg-sky-100 text-sky-800',
    fill_in_blank:   'bg-orange-100 text-orange-800',
    classic:         'bg-stone-100 text-stone-700',
  };
</script>

<div class="p-6 max-w-5xl mx-auto">
  <!-- Header -->
  <div class="flex items-center justify-between mb-6">
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Soru Bankası</h1>
      {#if !loading}
        <p class="text-sm text-muted-foreground mt-0.5">
          {questions.length} soru
        </p>
      {/if}
    </div>
    <a
      href="/questions/new"
      class="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
    >
      + Soru Ekle
    </a>
  </div>

  <!-- States -->
  {#if loading}
    <div class="flex items-center justify-center py-24 text-muted-foreground text-sm">
      Yükleniyor…
    </div>

  {:else if error}
    <div class="rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
      {error}
    </div>

  {:else if questions.length === 0}
    <div class="flex flex-col items-center justify-center py-24 gap-3">
      <p class="text-muted-foreground text-sm">Henüz soru eklenmemiş.</p>
      <a
        href="/questions/new"
        class="text-sm text-primary hover:underline"
      >
        İlk soruyu ekle →
      </a>
    </div>

  {:else}
    <!-- Question list -->
    <div class="rounded-lg border overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="bg-muted/50 border-b text-muted-foreground text-xs uppercase tracking-wide">
            <th class="px-4 py-3 text-left font-medium">Soru</th>
            <th class="px-4 py-3 text-left font-medium">Tip</th>
            <th class="px-4 py-3 text-right font-medium">Puan</th>
            <th class="px-4 py-3 text-left font-medium">Kazanım</th>
            <th class="px-4 py-3 text-left font-medium">Kalite</th>
          </tr>
        </thead>
        <tbody class="divide-y">
          {#each questions as q (q.id)}
            {@const badge = scoreBadge(q.stats)}
            {@const pts   = questionPoints(q)}
            <tr class="hover:bg-muted/30 transition-colors group">
              <!-- Body preview -->
              <td class="px-4 py-3 max-w-xs">
                <span class="line-clamp-2 text-foreground leading-snug">
                  {bodyPreview(q.body)}
                </span>
                {#if q.stats.times_used > 0}
                  <span class="text-xs text-muted-foreground">
                    {q.stats.times_used}× kullanıldı
                  </span>
                {/if}
              </td>

              <!-- Type badge -->
              <td class="px-4 py-3">
                <span class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium {TYPE_STYLES[q.question_type]}">
                  {QUESTION_TYPE_LABELS[q.question_type]}
                </span>
              </td>

              <!-- Points -->
              <td class="px-4 py-3 text-right tabular-nums font-medium">
                {pts}p
              </td>

              <!-- Outcomes -->
              <td class="px-4 py-3">
                <div class="flex flex-wrap gap-1">
                  {#each q.outcomes.slice(0, 2) as code}
                    <span class="text-xs text-muted-foreground font-mono bg-muted px-1.5 py-0.5 rounded">
                      {code}
                    </span>
                  {/each}
                  {#if q.outcomes.length > 2}
                    <span class="text-xs text-muted-foreground">+{q.outcomes.length - 2}</span>
                  {/if}
                </div>
              </td>

              <!-- Score badge -->
              <td class="px-4 py-3">
                <span class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium {BADGE_STYLES[badge]}">
                  {BADGE_LABELS[badge]}
                </span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
