<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView } from "@codemirror/view";
  import { typstEditorExtensions, setDiagnostics } from "$lib/editor/setup";
  import type { TypstDiagnostic } from "$lib/editor/diagnostics";

  type Props = {
    value: string;
    diagnostics?: TypstDiagnostic[];
    onchange: (value: string) => void;
  };

  let { value, diagnostics = [], onchange }: Props = $props();

  let host: HTMLDivElement;
  let view: EditorView | null = null;

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: value,
        extensions: typstEditorExtensions(onchange),
      }),
    });
  });

  onDestroy(() => view?.destroy());

  // Hata işaretleri dışarıdan gelir; editör kendi başına derleme yapmaz.
  $effect(() => {
    view?.dispatch({ effects: setDiagnostics.of(diagnostics) });
  });

  /**
   * Blok şeridinin metni imlece yerleştirmesi için.
   *
   * Parçacık {|} işareti taşıyorsa imleç oraya konur. İşaret olmadan imleç
   * her zaman parçacığın SONUNA düşüyordu: `$x$` eklendiğinde kapanış
   * dolarının sağına, yani yazılacak yerin dışına.
   */
  const CARET = "{|}";

  export function insert(snippet: string) {
    if (!view) return;

    const caretAt = snippet.indexOf(CARET);
    const text = caretAt === -1 ? snippet : snippet.replace(CARET, "");
    const offset = caretAt === -1 ? text.length : caretAt;

    const { from, to } = view.state.selection.main;
    view.dispatch({
      changes: { from, to, insert: text },
      selection: { anchor: from + offset },
    });
    view.focus();
  }
</script>

<div class="h-full min-h-0 paper-plain bg-paper-lift" bind:this={host}></div>
