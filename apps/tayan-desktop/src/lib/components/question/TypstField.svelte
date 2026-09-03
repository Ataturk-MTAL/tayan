<script lang="ts">
  /**
   * Kısa Typst metni için tek satırlık alan — rubrik ölçütü gibi.
   *
   * Gövde editörüyle AYNI tamamlama kaynağını kullanır: `$` yazınca semboller,
   * `#` yazınca fonksiyonlar gelir. Ölçüt metinleri çoğu zaman matematik
   * içeriyor ("Formül $R = (V_("pin") - V_F)/I$ yazılmış"); düz bir textarea
   * öğretmeni sembolleri elle yazmaya zorluyordu.
   *
   * DIŞARIDAN GELEN DEĞER GERİ YAZILMAZ. CodeMirror kendi belgesinin sahibi;
   * her prop değişiminde belgeyi ezmek imleci başa atar ve yazarken metni
   * bozar. Değer yalnız kurulurken okunur, sonrası `onchange` ile dışarı akar.
   */
  import { onDestroy, onMount } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, placeholder as cmPlaceholder } from "@codemirror/view";
  import { typstFieldExtensions } from "$lib/editor/setup";

  type Props = {
    value: string;
    placeholder?: string;
    ariaLabel?: string;
    /**
     * Alt çizgiyi alanın kendisi mi çizsin?
     *
     * Rubrik satırında ÇİZMEZ: orada ölçüt, puan ve sil düğmesi tek bir
     * çizginin üstünde durmalı. Her alan kendi çizgisini çizince çizgiler
     * ayrı ayrı bitiyor ve satır kırık görünüyordu.
     */
    bordered?: boolean;
    onchange: (value: string) => void;
  };

  let {
    value,
    placeholder = "",
    ariaLabel,
    bordered = true,
    onchange,
  }: Props = $props();

  let host: HTMLDivElement;
  let view: EditorView | null = null;

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: value,
        extensions: [...typstFieldExtensions(onchange), cmPlaceholder(placeholder)],
      }),
    });
    if (ariaLabel) view.contentDOM.setAttribute("aria-label", ariaLabel);
  });

  onDestroy(() => view?.destroy());
</script>

<div
  bind:this={host}
  class="min-w-0 flex-1"
  class:border-b={bordered}
  class:border-rule-strong={bordered}
  class:focus-within:border-red={bordered}
></div>
