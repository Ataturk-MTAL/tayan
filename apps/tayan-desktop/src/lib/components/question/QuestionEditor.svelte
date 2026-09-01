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
  };

  let {
    body,
    questionType,
    points,
    outcomes,
    stats = null,
    onbodychange,
  }: Props = $props();

  /** Derleme her tuş vuruşunda değil, yazma durunca çalışır. */
  const DEBOUNCE_MS = 220;

  let pages = $state<string[]>([]);
  let compileError = $state<string | null>(null);
  let diagnostics = $state<TypstDiagnostic[]>([]);
  let compiling = $state(false);

  let sourceRef = $state<ReturnType<typeof TypstSource> | null>(null);

  $effect(() => {
    const current = body;
    const timer = setTimeout(() => void compile(current), DEBOUNCE_MS);
    return () => clearTimeout(timer);
  });

  async function compile(source: string) {
    compiling = true;
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

    <span class="ml-auto annot" class:invisible={!compiling}>derleniyor…</span>
  </div>

  <BlockPalette oninsert={handleInsert} />

  <div class="grid min-h-0 flex-1 grid-cols-[minmax(320px,1fr)_minmax(360px,1.15fr)_240px]">
    <section class="min-h-0 border-r border-rule-strong">
      <TypstSource bind:this={sourceRef} value={body} {diagnostics} onchange={onbodychange} />
    </section>

    <section class="min-h-0">
      <SheetPreview {pages} stale={compiling} error={compileError} />
    </section>

    <MeasureRail {stats} {points} {outcomes} />
  </div>
</div>
