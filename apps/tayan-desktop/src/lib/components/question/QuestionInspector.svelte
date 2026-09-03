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
  import RubricEditor from "./RubricEditor.svelte";
  import SelectBox from "../shell/SelectBox.svelte";
  import { GRADE_OPTIONS } from "$lib/question/subjects";
  import { outcomePrefix, outcomeSuggestions, splitOutcomes } from "$lib/question/outcomes";
  import {
    DIFFICULTY_LABELS,
    MAX_GRADE,
    MIN_GRADE,
    QUESTION_TYPE_LABELS,
    type Difficulty,
    type Question,
    type QuestionMeta,
    type QuestionStats,
    type RubricItem,
    type ScoreBadge,
  } from "$lib/types";

  type QuestionType = Question["question_type"];

  type Props = {
    questionType: QuestionType;
    outcomeText: string;
    points: number;
    rubric: RubricItem[];
    stats: QuestionStats | null;
    structureError: string | null;
    /** Ders, sınıf seviyesi, zorluk. İlk ikisi zorunlu. */
    meta: QuestionMeta;
    /** Bankada kullanılan dersler + başlangıç listesi. */
    subjectOptions: string[];
    /** Kazanım önerileri için bankanın tamamı. */
    bank: Question[];
    onmetachange: (next: QuestionMeta) => void;
    onquestiontypechange: (value: QuestionType) => void;
    onoutcometextchange: (value: string) => void;
    onpointschange: (value: number) => void;
    onrubricchange: (next: RubricItem[]) => void;
  };

  let {
    questionType,
    outcomeText,
    points,
    rubric,
    stats,
    structureError,
    meta,
    subjectOptions,
    bank,
    onmetachange,
    onquestiontypechange,
    onoutcometextchange,
    onpointschange,
    onrubricchange,
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

  /**
   * Eksik zorunlu alanlar. Kaydete basılmadan görünür: hatayı ancak kaydetmeye
   * çalışınca öğrenmek, doldurulmuş bir formu geri çevirmek demektir.
   */
  let dersEksik = $derived(meta.subject.trim() === "");
  let seviyeEksik = $derived(
    !Number.isFinite(meta.grade) || meta.grade < MIN_GRADE || meta.grade > MAX_GRADE,
  );

  const ZORLUK_SECENEKLERI = (["kolay", "orta", "zor"] as Difficulty[]).map((z) => ({
    value: z,
    label: DIFFICULTY_LABELS[z],
  }));

  let tipSecenekleri = $derived(
    Object.entries(QUESTION_TYPE_LABELS).map(([value, label]) => ({ value, label })),
  );

  let dersSecenekleri = $derived(subjectOptions.map((s) => ({ value: s, label: s })));

  /**
   * Kazanım kodları yazılırken doğrulanır. Kural Rust tarafıyla birebir aynı;
   * kaydetmeye çalışınca "Geçersiz kazanım kodu" ile karşılaşmak, doldurulmuş
   * bir formu geri çevirmek olurdu.
   */
  let kazanimlar = $derived(splitOutcomes(outcomeText));
  let kazanimOnek = $derived(outcomePrefix(meta.subject, meta.grade));
  let kazanimOnerileri = $derived(outcomeSuggestions(bank, meta.subject, meta.grade));

  function addOutcome(code: string) {
    const varOlan = outcomeText.trim();
    if (varOlan.split(/[,\s]+/).includes(code)) return;
    onoutcometextchange(varOlan === "" ? code : `${varOlan} ${code}`);
  }

  function pct(n: number): string {
    return `${Math.round(n * 100)}%`;
  }
</script>

<div class="flex flex-col gap-rule">
  <div class="flex flex-col gap-half">
    <RuledField label="Soru tipi">
      <SelectBox
        value={questionType}
        options={tipSecenekleri}
        onchange={(v) => onquestiontypechange(v as QuestionType)}
      />
    </RuledField>

    <!--
      Başlık ZORUNLU DEĞİL ve yalnız cevap anahtarına basılır. Öğrenci
      nüshasında görünseydi konuyu ele verirdi: "LED Sürme" başlığı, sorunun
      neyi sorduğunu okumadan söyler.
    -->
    <RuledField label="Başlık" hint="Yalnız cevap anahtarında görünür">
      <input
        type="text"
        placeholder="Dijital Çıkış — LED Sürme"
        value={meta.title}
        oninput={(e) => onmetachange({ ...meta, title: e.currentTarget.value })}
      />
    </RuledField>

    <RuledField label="Ders" hint={dersEksik ? "Zorunlu" : null}>
      <SelectBox
        value={meta.subject}
        options={dersSecenekleri}
        allowCustom
        placeholder="Matematik"
        invalid={dersEksik}
        onchange={(v) => onmetachange({ ...meta, subject: v })}
      />
    </RuledField>

    <RuledField
      label="Sınıf seviyesi"
      hint={seviyeEksik ? `Zorunlu — ${MIN_GRADE} ile ${MAX_GRADE} arası` : null}
    >
      <SelectBox
        value={meta.grade === 0 ? "" : String(meta.grade)}
        options={GRADE_OPTIONS}
        placeholder="Seç"
        invalid={seviyeEksik}
        onchange={(v) => onmetachange({ ...meta, grade: Number(v) })}
      />
    </RuledField>

    <RuledField label="Zorluk" hint="İsteğe bağlı — ölçüm gelince gerçeği görürsün">
      <SelectBox
        value={meta.difficulty ?? ""}
        options={ZORLUK_SECENEKLERI}
        emptyLabel="Belirtilmedi"
        onchange={(v) => onmetachange({ ...meta, difficulty: v === "" ? null : (v as Difficulty) })}
      />
    </RuledField>

    <RuledField
      label="Kazanım"
      hint={kazanimlar.invalid.length > 0
        ? `Biçim hatalı: ${kazanimlar.invalid.join(", ")} — DERS.SINIF.ÜNİTE.KAZANIM`
        : "Boşluk veya virgülle ayır"}
    >
      <input
        type="text"
        value={outcomeText}
        placeholder={kazanimOnek === "" ? "MAT.9.1.2" : `${kazanimOnek}1.2`}
        aria-invalid={kazanimlar.invalid.length > 0}
        oninput={(e) => onoutcometextchange(e.currentTarget.value)}
      />
    </RuledField>

    {#if kazanimOnerileri.length > 0}
      <div>
        <span class="stamp">Bu ders ve seviyede kullandıkların</span>
        <div class="mt-quarter flex flex-wrap gap-quarter">
          {#each kazanimOnerileri as kod}
            <button
              type="button"
              class="border border-rule-strong bg-paper px-quarter py-[1px] font-mono text-[11px]
                     leading-rule text-ink-mid transition-colors hover:border-red hover:text-red-deep"
              onclick={() => addOutcome(kod)}
            >
              {kod}
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <RuledField label="Yedek puan" hint="Sınavda ayrıca belirlenir">
      <input
        type="number"
        min="1"
        value={points}
        oninput={(e) => onpointschange(Number(e.currentTarget.value))}
      />
    </RuledField>
  </div>

  <!--
    Rubrik yalnız açık uçlu soruda. Şıklı ve doğru-yanlış soruda puan
    kendiliğinden hesaplanıyor; oraya ölçüt koymak öğretmeni yanıltır.
  -->
  {#if questionType === "classic"}
    <RubricEditor {rubric} {points} onchange={onrubricchange} />
  {/if}

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
