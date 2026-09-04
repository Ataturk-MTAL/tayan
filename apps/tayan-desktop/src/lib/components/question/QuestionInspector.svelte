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

  /**
   * Ölçüm rozeti rengi artık CSS değişkeni değil, doğrudan Tailwind sınıfı.
   * Eski `--color-mark-*` değişkenleri app.css yeniden yazılırken kaldırıldı;
   * anlam aynı kalıyor (iyi=yeşil, zayıf=kırmızı) ama koyu kip karşılığı da var.
   */
  const BADGE_CLASS: Record<ScoreBadge, string> = {
    excellent: "text-green-600 dark:text-green-400",
    good: "text-blue-600 dark:text-blue-400",
    fair: "text-amber-600 dark:text-amber-400",
    poor: "text-red-600 dark:text-red-400",
    untested: "text-gray-400 dark:text-gray-500",
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

<div class="flex flex-col gap-5 text-sm">
  <div class="flex flex-col gap-3">
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
        <span class="text-[11px] font-semibold tracking-wide text-gray-500 uppercase dark:text-gray-400">
          Bu ders ve seviyede kullandıkların
        </span>
        <div class="mt-1 flex flex-wrap gap-1">
          {#each kazanimOnerileri as kod}
            <button
              type="button"
              class="rounded border border-gray-300 bg-white px-1.5 py-0.5 font-mono text-[11px]
                     text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-700
                     dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300
                     dark:hover:border-primary-400 dark:hover:text-primary-400"
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
    <p class="border-t border-gray-200 pt-3 text-xs text-red-600 dark:border-gray-700 dark:text-red-400">
      {structureError}
    </p>
  {/if}

  <div class="border-t border-gray-200 pt-3 dark:border-gray-700">
    <h3 class="text-[11px] font-semibold tracking-wide text-gray-500 uppercase dark:text-gray-400">
      Ölçüm
    </h3>

    <div class="mt-1 flex items-baseline gap-2">
      <span class="tnum text-[28px] leading-10 font-bold {BADGE_CLASS[badge]}">
        {stats && stats.times_used > 0 ? Math.round(stats.performance_score) : "—"}
      </span>
      <span class="text-xs text-gray-500 dark:text-gray-400">{BADGE_LABEL[badge]}</span>
    </div>

    {#if !stats || stats.times_used === 0}
      <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
        Bu soru hiç uygulanmadı. Ölçüm, sınav sonuçları girildiğinde oluşur.
      </p>
    {:else}
      <dl class="mt-2 grid grid-cols-[1fr_auto] gap-x-2 text-xs">
        <dt class="text-gray-500 dark:text-gray-400">Güçlük</dt>
        <dd class="tnum text-right text-gray-700 dark:text-gray-300">{pct(stats.difficulty_index)}</dd>

        <dt class="text-gray-500 dark:text-gray-400">Ayırt edicilik</dt>
        <dd class="tnum text-right text-gray-700 dark:text-gray-300">
          {stats.discrimination_index.toFixed(2)}
        </dd>

        <dt class="text-gray-500 dark:text-gray-400">Uygulanma</dt>
        <dd class="tnum text-right text-gray-700 dark:text-gray-300">{stats.times_used} kez</dd>

        <dt class="text-gray-500 dark:text-gray-400">Doğru / cevap</dt>
        <dd class="tnum text-right text-gray-700 dark:text-gray-300">
          {stats.correct_responses} / {stats.total_responses}
        </dd>

        <dt class="text-gray-500 dark:text-gray-400">Ortalama puan</dt>
        <dd class="tnum text-right text-gray-700 dark:text-gray-300">
          {stats.avg_points_earned.toFixed(1)}
        </dd>
      </dl>

      {#each warnings as warning}
        <p class="mt-2 border-t border-gray-200 pt-2 text-xs text-red-600 dark:border-gray-700 dark:text-red-400">
          {warning}
        </p>
      {/each}
    {/if}
  </div>

  {#if outcomes.length > 0}
    <div class="border-t border-gray-200 pt-3 dark:border-gray-700">
      <h3 class="text-[11px] font-semibold tracking-wide text-gray-500 uppercase dark:text-gray-400">
        Girilen kazanımlar
      </h3>
      <!--
        break-all ŞART: kazanım kodları nokta ve rakamdan oluşan tek parça
        belirteçler, içinde kırılacak boşluk yok. Varsayılan
        overflow-wrap: normal hiçbir yerden kıramadığı için uzun bir kod li'nin
        dışına taşıyor, kırpan bir ata olmadığından taşma DockPanel'in
        `min-h-0 flex-1 overflow-auto` kabına kadar gidiyor ve orada YATAY
        kaydırma açıyordu — panelin bütün içeriği sağa kayıyordu. break-words
        yetmez, o yalnız kelime sınırında kırar ve bu belirteçte sınır yoktur.
        Metin kısaltılmıyor, yalnız sarılıyor: kod tam okunur kalıyor.
      -->
      <ul class="mt-1">
        {#each outcomes as outcome}
          <li class="font-mono text-xs break-all text-gray-600 dark:text-gray-300">{outcome}</li>
        {/each}
      </ul>
    </div>
  {/if}
</div>
