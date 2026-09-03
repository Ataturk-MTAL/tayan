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
  const durumlar = new Map<string, EditorState>();
  /** Görünümde şu an hangi belge duruyor. onMount'ta kurulur; prop'un ilk
   * değerini burada okumak "yalnız ilk değeri yakalar" uyarısını hak eder. */
  let aktifDoc: string | null = null;

  function durumKur(doc: string, metin: string): EditorState {
    return EditorState.create({
      doc: metin,
      // Değişiklik hangi belgeden geldiyse onunla bildirilir; sekme
      // değiştikten sonra gecikmeli bir olay yanlış alana yazamaz.
      extensions: typstEditorExtensions((v) => onchange(v, doc), handlePaste),
    });
  }

  onMount(() => {
    aktifDoc = docId;
    const state = durumKur(docId, value);
    durumlar.set(docId, state);
    view = new EditorView({ parent: host, state });
  });

  onDestroy(() => view?.destroy());

  // Sekme değişince: mevcut durumu sakla, hedefin durumunu geri yükle.
  $effect(() => {
    const hedef = docId;
    if (!view || aktifDoc === null || hedef === aktifDoc) return;

    durumlar.set(aktifDoc, view.state);
    aktifDoc = hedef;

    const kayitli = durumlar.get(hedef) ?? durumKur(hedef, value);
    durumlar.set(hedef, kayitli);
    view.setState(kayitli);
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

<div class="h-full min-h-0 paper-plain bg-paper-lift" bind:this={host}></div>
