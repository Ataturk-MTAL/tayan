<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView } from "@codemirror/view";
  import { typstEditorExtensions, setDiagnostics } from "$lib/editor/setup";
  import { saveImageAsTypst } from "$lib/question/image";
  import { errorText } from "$lib/editor/diagnostics";
  import type { TypstDiagnostic } from "$lib/editor/diagnostics";

  type Props = {
    value: string;
    diagnostics?: TypstDiagnostic[];
    onchange: (value: string) => void;
    /** Panodan görsel yapıştırıldığında. Hata mesajı üstte gösterilsin diye dışarı verilir. */
    onimageerror?: (message: string) => void;
  };

  let { value, diagnostics = [], onchange, onimageerror }: Props = $props();

  let host: HTMLDivElement;
  let view: EditorView | null = null;

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: value,
        extensions: typstEditorExtensions(onchange, handlePaste),
      }),
    });
  });

  onDestroy(() => view?.destroy());

  /**
   * Panodaki görseli yakalar: öğretmen bir şekil ekran görüntüsü alıp doğrudan
   * yapıştırabilsin. Dosya kaydedip yolunu bulmak, gerçek kullanımda en çok
   * vazgeçilen adım.
   *
   * Panoda görsel yoksa false döner ve CodeMirror normal metin yapıştırmasını
   * kendisi yapar.
   */
  function handlePaste(event: ClipboardEvent): boolean {
    const items = event.clipboardData?.items;
    if (!items) return false;

    for (const item of items) {
      if (!item.type.startsWith("image/")) continue;
      const file = item.getAsFile();
      if (!file) continue;

      event.preventDefault();
      void saveImageAsTypst(file)
        .then((call) => insert(call))
        .catch((err: unknown) => onimageerror?.(errorText(err)));
      return true;
    }
    return false;
  }

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
