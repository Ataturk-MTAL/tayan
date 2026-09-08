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
    /**
     * Hangi belge düzenleniyor — "question" ya da "answer".
     *
     * TEK GÖRÜNÜM, ÇOK DURUM. Sekme başına ayrı bir `EditorView` monte etmek
     * iki imleç çiziyordu; ayrıca `display:none` içindeki bir CodeMirror
     * ölçüm yapamıyor ve göründüğünde bozuk çizebiliyor. Tek görünüm tutup
     * `setState` ile belge değiştirmek ikisini birden çözüyor.
     *
     * Geri alma geçmişi `EditorState` içinde yaşadığı için sekmeler arasında
     * gidip gelmek geçmişi KAYBETMEZ — her belgenin kendi durumu saklanıyor.
     */
    docId?: string;
    diagnostics?: TypstDiagnostic[];
    onchange: (value: string, docId: string) => void;
    /** Panodan görsel yapıştırıldığında. Hata mesajı üstte gösterilsin diye dışarı verilir. */
    onimageerror?: (message: string) => void;
  };

  let {
    value,
    docId = "question",
    diagnostics = [],
    onchange,
    onimageerror,
  }: Props = $props();

  let host: HTMLDivElement;
  let view: EditorView | null = null;

  /** Belge kimliği → durum. Sekme geçişinde geçmişi taşıyan yer burası. */
  const states = new Map<string, EditorState>();
  /** Görünümde şu an hangi belge duruyor. onMount'ta kurulur; prop'un ilk
   * değerini burada okumak "yalnız ilk değeri yakalar" uyarısını hak eder. */
  let activeDocId: string | null = null;

  function createState(doc: string, text: string): EditorState {
    return EditorState.create({
      doc: text,
      // Değişiklik hangi belgeden geldiyse onunla bildirilir; sekme
      // değiştikten sonra gecikmeli bir olay yanlış alana yazamaz.
      extensions: typstEditorExtensions((v) => onchange(v, doc), handlePaste),
    });
  }

  onMount(() => {
    activeDocId = docId;
    const state = createState(docId, value);
    states.set(docId, state);
    view = new EditorView({ parent: host, state });
  });

  onDestroy(() => view?.destroy());

  // Sekme değişince: mevcut durumu sakla, hedefin durumunu geri yükle.
  $effect(() => {
    const targetDocId = docId;
    if (!view || activeDocId === null || targetDocId === activeDocId) return;

    states.set(activeDocId, view.state);
    activeDocId = targetDocId;

    const savedState = states.get(targetDocId) ?? createState(targetDocId, value);
    states.set(targetDocId, savedState);
    view.setState(savedState);
    view.focus();
  });

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

<!--
  ZEMİN EDİTÖRÜN KENDİ DEĞİŞKENİNDEN, `bg-white`DAN DEĞİL.

  `setup.ts`teki tema `&` üzerinde bilerek `backgroundColor: "transparent"`
  diyor — zemini bu kap boyuyor. Kapta çıplak `bg-white` vardı ve `dark:`
  karşılığı YOKTU: koyu kipte editör beyaz kalıyor, metin ise `--cm-text`
  (#f3f4f6) ile açık geliyordu. Açık zemin üstünde açık metin — kod
  okunmuyordu. Ölçüldü: `.cm-editor` arka planı `rgba(0, 0, 0, 0)`, kabı
  `rgb(255, 255, 255)`, metin `rgb(243, 244, 246)`.

  `bg-white dark:bg-gray-800` yazmak da işi görürdü ama iki ayrı doğruluk
  kaynağı bırakırdı. `--cm-bg` zaten editörün gutter'ı, etkin satırı ve
  panelleri için kullanılan değişken; kap da onu okuyunca ikisi ayrı düşemez.
-->
<div class="h-full min-h-0 bg-[var(--cm-bg)]" bind:this={host}></div>
