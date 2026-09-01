<script lang="ts">
  import TypstSource from "./TypstSource.svelte";
  import SheetPreview from "./SheetPreview.svelte";
  import BlockPalette from "./BlockPalette.svelte";
  import MeasureRail from "../measure/MeasureRail.svelte";
  import { api } from "$lib/api";
  import { parseDiagnostics, errorText, type TypstDiagnostic } from "$lib/editor/diagnostics";
  import type { QuestionStats } from "$lib/types";
  import { QUESTION_TYPE_LABELS, type Question } from "$lib/types";

  type Props = {
    body: string;
    questionType: Question["question_type"];
    points: number;
    outcomes: string[];
    stats?: QuestionStats | null;
    onbodychange: (body: string) => void;
    /** Cevap bölgesi. Kırmızı cetvelin sağı yalnızca ölçümündür, cevabın değil. */
    answer?: import("svelte").Snippet;
  };

  let {
    body,
    questionType,
    points,
    outcomes,
    stats = null,
    onbodychange,
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

<div class="flex h-full min-h-0 flex-col">
  <!-- Künye: sorunun kimliği. Kart değil, cetvelli satır. -->
  <div class="ruled-bottom flex shrink-0 items-center gap-rule bg-paper px-rule py-half paper-plain">
    <span class="stamp">{QUESTION_TYPE_LABELS[questionType]}</span>
    <span class="pencil">{points} puan</span>
    {#if outcomes.length > 0}
      <span class="pencil font-mono">{outcomes.join(" · ")}</span>
    {/if}

    <span class="ml-auto annot" class:invisible={!slowCompile}>derleniyor…</span>
  </div>

  <BlockPalette oninsert={handleInsert} />

  <div class="grid min-h-0 flex-1 grid-cols-[minmax(320px,1fr)_minmax(360px,1.15fr)_240px]">
    <section class="min-h-0 border-r border-rule-strong">
      <TypstSource bind:this={sourceRef} value={body} {diagnostics} onchange={onbodychange} />
    </section>

    <section class="min-h-0">
      <SheetPreview {pages} stale={slowCompile} error={compileError} />
    </section>

    <MeasureRail {stats} {points} {outcomes} />
  </div>

  {#if answer}
    <div class="ruled-top max-h-[240px] shrink-0 overflow-auto bg-paper px-rule py-half paper-plain">
      {@render answer()}
    </div>
  {/if}
</div>
