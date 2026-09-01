<script lang="ts">
  /**
   * Tek bir basılı sayfa.
   *
   * Neden {@html} değil: {@html} her güncellemede innerHTML'i yeniden yazar,
   * yani sayfa hiç değişmemiş olsa bile SVG ağacı sökülüp kurulur ve yeniden
   * boyanır. Yazarken bu, saniyede birkaç kez tekrarlanan görünür bir titreme
   * demek.
   *
   * Typst değişmeyen bir sayfa için bayt bayt aynı SVG üretir. Bu yüzden
   * karşılaştırıp yalnızca GERÇEKTEN değişmişse DOM'a dokunuyoruz.
   */
  type Props = { svg: string; zoom: number };
  let { svg, zoom }: Props = $props();

  /** A4 genişliği, 96 dpi. Zoom bunun katıdır. */
  const A4_WIDTH_PX = 794;

  // $state şart: bind:this hedefi reaktif olmazsa ilk $effect koşusunda host
  // henüz atanmamış olabilir ve efekt bir daha tetiklenmez — ilk sayfa hiç
  // çizilmez.
  let host = $state<HTMLDivElement | undefined>(undefined);
  let painted: string | null = null;

  $effect(() => {
    const next = svg;
    const el = host;
    if (!el || next === painted) return;
    el.innerHTML = next;
    painted = next;
  });
</script>

<!--
  Zoom transform ile değil GENİŞLİKLE yapılır. transform: scale() sayfayı
  bulanıklaştırır ve kaydırma alanını bozar; genişlik değişince SVG kendi
  vektörlerinden yeniden çizilir ve her ölçekte keskin kalır.
-->
<div
  class="sheet sheet-set"
  style="width: {Math.round(A4_WIDTH_PX * zoom)}px"
  bind:this={host}
></div>

<style>
  /*
    Bu kural SheetPreview'de duruyordu; .sheet buraya taşınınca Svelte onu
    "hiçbir şeye uymuyor" diye attı ve sayfa doğal boyutunda çizilmeye başladı.
    Kapsamlı stil, kapsadığı işaretlemeyle aynı dosyada durmak zorunda.
  */
  .sheet :global(svg) {
    display: block;
    width: 100%;
    height: auto;
  }
</style>
