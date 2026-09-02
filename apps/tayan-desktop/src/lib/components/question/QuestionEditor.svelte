<script lang="ts">
  import TypstSource from "./TypstSource.svelte";
  import SheetPreview from "./SheetPreview.svelte";
  import FloatingPalette from "./FloatingPalette.svelte";
  import QuestionInspector from "./QuestionInspector.svelte";
  import DockPanel from "../shell/DockPanel.svelte";
  import PenButton from "../shell/PenButton.svelte";
  import { layout, setMode, type ViewMode } from "$lib/ui/layout.svelte";
  import { api } from "$lib/api";
  import { parseDiagnostics, errorText, type TypstDiagnostic } from "$lib/editor/diagnostics";
  import type { QuestionMeta, QuestionStats, Question } from "$lib/types";

  type QuestionType = Question["question_type"];

  type Props = {
    body: string;
    questionType: QuestionType;
    outcomeText: string;
    points: number;
    stats?: QuestionStats | null;
    structureError: string | null;
    meta: QuestionMeta;
    subjectOptions: string[];
    bank: Question[];
    onmetachange: (next: QuestionMeta) => void;
    saving: boolean;
    saveLabel: string;
    onbodychange: (body: string) => void;
    onquestiontypechange: (value: QuestionType) => void;
    onoutcometextchange: (value: string) => void;
    onpointschange: (value: number) => void;
    onsave: () => void;
    /** Cevap bölgesi. Kırmızı cetvelin sağı yalnızca ölçümündür, cevabın değil. */
    answer?: import("svelte").Snippet;
  };

  let {
    body,
    questionType,
    outcomeText,
    points,
    stats = null,
    structureError,
    meta,
    subjectOptions,
    bank,
    onmetachange,
    saving,
    saveLabel,
    onbodychange,
    onquestiontypechange,
    onoutcometextchange,
    onpointschange,
    onsave,
    answer,
  }: Props = $props();

  /**
   * Derleme her tuş vuruşunda değil, yazma durunca çalışır.
   *
   * 120 ms, ölçüme dayanıyor: font kaydı ısındıktan sonra tek bir sorunun
   * derlenmesi 4-30 ms sürüyor (debug profilinde ölçüldü). Bu hızda daha uzun
   * bir bekleme yalnızca yapay gecikme olur.
   *
   * Sıfır değil, çünkü derlemeler sıraya alınsa da her tuş vuruşunda IPC turu
   * yapmanın anlamı yok; 120 ms yazma ritmindeki vuruşları toparlar.
   */
  const DEBOUNCE_MS = 120;

  let pages = $state<string[]>([]);
  let compileError = $state<string | null>(null);
  let diagnostics = $state<TypstDiagnostic[]>([]);
  let compiling = $state(false);

  /**
   * Derleme göstergesi yalnızca uzun süren derlemede görünür.
   *
   * Tipik derleme 5-30 ms. Her derlemede bir gösterge yakıp söndürmek, yazma
   * hızında saniyede birkaç kez yanıp sönen bir arayüz demek — bilgi vermiyor,
   * yalnızca göz yoruyor.
   */
  const SLOW_COMPILE_MS = 400;
  let slowCompile = $state(false);
  let slowTimer: ReturnType<typeof setTimeout> | null = null;

  let sourceRef = $state<ReturnType<typeof TypstSource> | null>(null);

  /**
   * Aynı anda en fazla bir derleme. Üst üste binen derlemeler paralel
   * TayanWorld örneği demektir; her biri kendi belleğini tutar ve hızlı yazan
   * bir öğretmen süreci kolayca şişirir. Sıraya alınan yalnızca EN SON kaynak
   * tutulur — aradakiler zaten ekranda görünmeyecek.
   */
  let pendingSource: string | null = null;

  let imageError = $state<string | null>(null);

  const MODES: Array<{ id: ViewMode; label: string; title: string }> = [
    { id: "editor", label: "Editör", title: "Yalnızca kaynak" },
    { id: "split", label: "Yan yana", title: "Kaynak ve kâğıt yan yana" },
    { id: "preview", label: "Kâğıt", title: "Yalnızca basılacak sayfa" },
  ];

  $effect(() => {
    const current = body;
    const timer = setTimeout(() => void compile(current), DEBOUNCE_MS);
    return () => clearTimeout(timer);
  });

  async function compile(source: string) {
    if (compiling) {
      pendingSource = source;
      return;
    }

    compiling = true;
    slowTimer = setTimeout(() => (slowCompile = true), SLOW_COMPILE_MS);

    try {
      const result = await api.compiler.previewQuestion(source);
      pages = result;
      compileError = null;
      diagnostics = [];
    } catch (err: unknown) {
      // Önceki sayfalar ekranda kalır: hata anında kâğıdı boşaltmak,
      // öğretmenin neyi bozduğunu görmesini engeller.
      const message = errorText(err);
      compileError = message;
      diagnostics = parseDiagnostics(message);
    } finally {
      compiling = false;
      if (slowTimer !== null) {
        clearTimeout(slowTimer);
        slowTimer = null;
      }
      slowCompile = false;

      const queued = pendingSource;
      pendingSource = null;
      if (queued !== null && queued !== source) void compile(queued);
    }
  }

  function handleInsert(snippet: string) {
    sourceRef?.insert(snippet);
  }
</script>

{#snippet inspector()}
  <DockPanel title="Soru">
    <QuestionInspector
      {questionType}
      {outcomeText}
      {points}
      {stats}
      {structureError}
      {meta}
      {subjectOptions}
      {bank}
      {onmetachange}
      {onquestiontypechange}
      {onoutcometextchange}
      {onpointschange}
    />
  </DockPanel>
{/snippet}

<div class="flex h-full min-h-0 flex-col">
  <!--
    Tek ince araç satırı. Öncesinde burada iki şerit vardı: form başlığı
    (tip + kazanım + Kaydet) ve künye (tip + puan + kazanım). İkisi de aynı
    gerçekleri iki kez basıyordu; ikisi de panele indi. Kalan yalnızca
    görünüm modu ve kaydetme — ikisi de bir bölmeye ait değil.
  -->
  <div
    class="ruled-bottom paper-plain flex shrink-0 items-center gap-half bg-paper-lift
           px-half py-quarter"
  >
    <div class="flex items-stretch border border-rule-strong">
      {#each MODES as m (m.id)}
        <button
          type="button"
          class="border-r border-rule px-half py-quarter text-[12px] leading-rule
                 transition-colors last:border-r-0 hover:text-red-deep"
          class:bg-paper-sunk={layout.mode === m.id}
          class:font-semibold={layout.mode === m.id}
          class:text-ink={layout.mode === m.id}
          class:text-pencil={layout.mode !== m.id}
          aria-pressed={layout.mode === m.id}
          title={m.title}
          onclick={() => setMode(m.id)}
        >
          {m.label}
        </button>
      {/each}
    </div>

    {#if imageError}
      <span class="annot">{imageError}</span>
    {/if}

    <span class="annot ml-auto" class:invisible={!slowCompile}>derleniyor…</span>

    <PenButton kind="ink" disabled={saving || structureError !== null} onclick={onsave}>
      {saveLabel}
    </PenButton>
  </div>

  <div class="flex min-h-0 flex-1">
    {#if layout.side === "left"}{@render inspector()}{/if}

    <!--
      `min-w-0` ve `min-w-[320px]` SÜS DEĞİL, yerleşimin taşıyıcı kısıtı.

      `SheetPage` kâğıda açık piksel genişliği verir (794 × zoom): %159'da
      1263 px'lik gerçek bir öğe. Flex öğelerinin varsayılanı `min-width: auto`,
      yani içeriğinin min-content genişliğinin altına inmeyi REDDEDER — önizleme
      1263 px talep eder ve editörü sıfıra ezer.

      Eski grid bunu `minmax(320px,1fr)_minmax(360px,1.15fr)` ile karşılıyordu.
      Flex'e geçerken o taban düştü; buradaki iki sınıf onu geri koyuyor:
      editörün tabanı var, önizleme küçülüp KENDİ içinde kaydırıyor.
    -->
    <div class="flex min-h-0 min-w-0 flex-1">
      {#if layout.mode !== "preview"}
        <!--
          Palet kaynağın ÜZERİNDE yüzer; `relative` bu yüzden burada. Kâğıdın
          üstüne taşmaz — basılacak sayfayı hiçbir şey örtmez.
        -->
        <section
          class="relative min-h-0 min-w-[320px] flex-1 basis-0"
          class:border-r={layout.mode === "split"}
          class:border-rule-strong={layout.mode === "split"}
        >
          <TypstSource
            bind:this={sourceRef}
            value={body}
            {diagnostics}
            onchange={onbodychange}
            onimageerror={(m) => (imageError = m)}
          />
          <FloatingPalette {questionType} oninsert={handleInsert} />
        </section>
      {/if}

      {#if layout.mode !== "editor"}
        <!--
          `min-w-0`: kâğıt ne kadar büyürse büyüsün bu bölme küçülebilmeli.
          İçindeki kaydırıcı taşmayı zaten üstleniyor; bu sınıf olmadan kâğıdın
          piksel genişliği yerleşimi dışarı iter ve editörü ekrandan siler.
        -->
        <section class="min-h-0 min-w-0 flex-[1.15] basis-0">
          <SheetPreview {pages} stale={slowCompile} error={compileError} />
        </section>
      {/if}
    </div>

    {#if layout.side === "right"}{@render inspector()}{/if}
  </div>

  {#if answer}
    <div class="ruled-top paper-plain max-h-[240px] shrink-0 overflow-auto bg-paper px-rule py-half">
      {@render answer()}
    </div>
  {/if}
</div>
