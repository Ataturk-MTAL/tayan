<script lang="ts">
  import type { QuestionStats, ScoreBadge } from "$lib/types";

  type Props = {
    stats: QuestionStats | null;
    points: number;
    outcomes: string[];
  };

  let { stats, points, outcomes }: Props = $props();

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

  let badge = $derived.by<ScoreBadge>(() => {
    if (!stats || stats.times_used === 0) return "untested";
    const s = stats.performance_score;
    if (s >= 80) return "excellent";
    if (s >= 50) return "good";
    if (s >= 20) return "fair";
    return "poor";
  });

  /** Kaç öğretmenin gözden kaçırdığı uyarılar. Sessiz kalmak burada zarardır. */
  let warnings = $derived.by(() => {
    if (!stats || stats.times_used === 0) return [];
    const out: string[] = [];
    if (stats.difficulty_index > 0.9) out.push("Fazla kolay — neredeyse herkes doğru yaptı.");
    if (stats.difficulty_index < 0.2) out.push("Fazla zor — neredeyse kimse doğru yapamadı.");
    if (stats.discrimination_index < 0.2)
      out.push("Ayırt ediciliği düşük: iyi ve zayıf öğrenciyi ayırmıyor.");
    return out;
  });

  function pct(n: number): string {
    return `${Math.round(n * 100)}%`;
  }
</script>

<!--
  Kenar cetvelinin sağı değerlendirme alanıdır. Soldaki içerikten kırmızı bir
  dikey çizgiyle ayrılır; defterdeki kenar çizgisinin işlevi budur.
-->
<aside class="margin-rule h-full min-h-0 overflow-auto bg-paper paper-plain px-rule py-rule">
  <h3 class="stamp">Ölçüm</h3>

  <div class="mt-half flex items-baseline gap-half">
    <span class="text-[28px] font-bold leading-[40px] tnum" style="color: {BADGE_COLOR[badge]}">
      {stats && stats.times_used > 0 ? Math.round(stats.performance_score) : "—"}
    </span>
    <span class="annot">{BADGE_LABEL[badge]}</span>
  </div>

  {#if !stats || stats.times_used === 0}
    <p class="pencil mt-half">
      Bu soru hiç uygulanmadı. Ölçüm, sınav sonuçları girildiğinde oluşur.
    </p>
  {:else}
    <dl class="mt-rule grid grid-cols-[1fr_auto] gap-x-half">
      <dt class="pencil">Güçlük</dt>
      <dd class="annot tnum text-right">{pct(stats.difficulty_index)}</dd>

      <dt class="pencil">Ayırt edicilik</dt>
      <dd class="annot tnum text-right">{stats.discrimination_index.toFixed(2)}</dd>

      <dt class="pencil">Uygulanma</dt>
      <dd class="annot tnum text-right">{stats.times_used} kez</dd>

      <dt class="pencil">Doğru / cevap</dt>
      <dd class="annot tnum text-right">{stats.correct_responses} / {stats.total_responses}</dd>

      <dt class="pencil">Ortalama puan</dt>
      <dd class="annot tnum text-right">{stats.avg_points_earned.toFixed(1)}</dd>
    </dl>

    {#each warnings as warning}
      <p class="annot mt-half border-t border-rule pt-half">{warning}</p>
    {/each}
  {/if}

  <h3 class="stamp mt-rule border-t border-rule pt-half">Puan</h3>
  <p class="mt-quarter text-[19px] font-bold leading-rule tnum">{points}</p>
  <p class="pencil">sınavda değiştirilebilir</p>

  <h3 class="stamp mt-rule border-t border-rule pt-half">Kazanım</h3>
  {#if outcomes.length === 0}
    <p class="pencil mt-quarter">Kazanım girilmedi.</p>
  {:else}
    <ul class="mt-quarter">
      {#each outcomes as outcome}
        <li class="font-mono text-[12px] leading-rule text-ink-mid">{outcome}</li>
      {/each}
    </ul>
  {/if}
</aside>
