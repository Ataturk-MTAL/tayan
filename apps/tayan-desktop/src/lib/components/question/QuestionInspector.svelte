<script lang="ts">
  /**
   * Sorunun künyesi ve ölçümü — kenetlenen panelin içeriği.
   *
   * Öncesi ikiye bölünmüştü: soru tipi ve kazanım üstteki form şeridinde
   * GİRİLİYOR, aynı bilgiler sağdaki 240 px'lik ölçüm sütununda tekrar
   * BASILIYORDU. Üç yerde aynı gerçek, hiçbirinde tam hikâye.
   *
   * Artık tek yer: girilen de okunan da burada. Ölçüm bloğu aynı panelin altına
   * iner çünkü aynı sorunun tarihçesidir — ayrı bir sütunu hak etmiyor.
   *
   * Durum burada TUTULMAZ. Sahibi `QuestionForm`; buraya props gelir, değişiklik
   * geri çağrıyla döner. Panel kendi kopyasını tutsa iki doğru kaynak olurdu.
   */
  import RuledField from "../shell/RuledField.svelte";
  import {
    QUESTION_TYPE_LABELS,
    type Question,
    type QuestionStats,
    type ScoreBadge,
  } from "$lib/types";

  type QuestionType = Question["question_type"];

  type Props = {
    questionType: QuestionType;
    outcomeText: string;
    points: number;
    stats: QuestionStats | null;
    structureError: string | null;
    onquestiontypechange: (value: QuestionType) => void;
    onoutcometextchange: (value: string) => void;
    onpointschange: (value: number) => void;
  };

  let {
    questionType,
    outcomeText,
    points,
    stats,
    structureError,
    onquestiontypechange,
    onoutcometextchange,
    onpointschange,
  }: Props = $props();

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

  let outcomes = $derived(
    outcomeText
      .split(/[,\s]+/)
      .map((o) => o.trim())
      .filter(Boolean),
  );

  function pct(n: number): string {
    return `${Math.round(n * 100)}%`;
  }
</script>

<div class="flex flex-col gap-rule">
  <div class="flex flex-col gap-half">
    <RuledField label="Soru tipi">
      <select
        value={questionType}
        onchange={(e) => onquestiontypechange(e.currentTarget.value as QuestionType)}
      >
        {#each Object.entries(QUESTION_TYPE_LABELS) as [value, label]}
          <option {value}>{label}</option>
        {/each}
      </select>
    </RuledField>

    <RuledField label="Kazanım" hint="Boşluk veya virgülle ayır — MAT.9.1.2">
      <input
        type="text"
        value={outcomeText}
        placeholder="MAT.9.1.2"
        oninput={(e) => onoutcometextchange(e.currentTarget.value)}
      />
    </RuledField>

    <RuledField label="Yedek puan" hint="Sınavda ayrıca belirlenir">
      <input
        type="number"
        min="1"
        value={points}
        oninput={(e) => onpointschange(Number(e.currentTarget.value))}
      />
    </RuledField>
  </div>

  {#if structureError}
    <p class="annot border-t border-rule pt-half">{structureError}</p>
  {/if}

  <div class="border-t border-rule pt-half">
    <h3 class="stamp">Ölçüm</h3>

    <div class="mt-quarter flex items-baseline gap-half">
      <span class="text-[28px] font-bold leading-[40px] tnum" style="color: {BADGE_COLOR[badge]}">
        {stats && stats.times_used > 0 ? Math.round(stats.performance_score) : "—"}
      </span>
      <span class="annot">{BADGE_LABEL[badge]}</span>
    </div>

    {#if !stats || stats.times_used === 0}
      <p class="pencil mt-quarter">
        Bu soru hiç uygulanmadı. Ölçüm, sınav sonuçları girildiğinde oluşur.
      </p>
    {:else}
      <dl class="mt-half grid grid-cols-[1fr_auto] gap-x-half">
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
  </div>

  {#if outcomes.length > 0}
    <div class="border-t border-rule pt-half">
      <h3 class="stamp">Girilen kazanımlar</h3>
      <ul class="mt-quarter">
        {#each outcomes as outcome}
          <li class="font-mono text-[12px] leading-rule text-ink-mid">{outcome}</li>
        {/each}
      </ul>
    </div>
  {/if}
</div>
