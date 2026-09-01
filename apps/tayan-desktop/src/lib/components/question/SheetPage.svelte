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
  type Props = { svg: string };
  let { svg }: Props = $props();

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
  sheet-set yalnızca ilk görünüşte oynar; güncellemede sınıf yeniden eklenmediği
  için animasyon tekrarlanmaz.
-->
<div class="sheet sheet-set" bind:this={host}></div>
