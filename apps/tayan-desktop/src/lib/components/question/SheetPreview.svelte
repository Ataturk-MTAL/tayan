<script lang="ts">
  type Props = {
    pages: string[];
    stale: boolean;
    error: string | null;
  };

  let { pages, stale, error }: Props = $props();

  /**
   * Typst'in SVG dışa aktarıcısı script üretmez. Yine de temizliyoruz: bu
   * içerik Tauri webview'ine giriyor ve orada IPC yüzeyi var. Kaynak öğretmenin
   * kendi belgesi olsa bile, ucuz sigortayı atlamak için sebep yok.
   */
  function sanitize(svg: string): string {
    return svg
      .replace(/<script\b[\s\S]*?<\/script>/gi, "")
      .replace(/<foreignObject\b[\s\S]*?<\/foreignObject>/gi, "")
      .replace(/\son\w+\s*=\s*"[^"]*"/gi, "")
      .replace(/\son\w+\s*=\s*'[^']*'/gi, "");
  }

  let safePages = $derived(pages.map(sanitize));
</script>

<div class="h-full min-h-0 overflow-auto bg-paper-sunk paper-plain">
  {#if error}
    <div class="p-rule">
      <p class="stamp mb-half">Derlenmedi</p>
      <pre class="annot whitespace-pre-wrap font-mono text-[12px] leading-[20px]">{error}</pre>
      <p class="pencil mt-rule">Son çalışan sayfa aşağıda duruyor.</p>
    </div>
  {/if}

  {#if safePages.length === 0 && !error}
    <div class="flex h-full items-center justify-center">
      <p class="pencil">Sayfa henüz derlenmedi.</p>
    </div>
  {/if}

  <div class="flex flex-col items-center gap-rule p-rule" class:opacity-45={stale}>
    {#each safePages as page, i (i)}
      <!--
        Kâğıt masadan kalkan tek nesnedir; gölge burada gerçektir, süs değil.
        {@html} kullanımı bilinçli: içerik yerel Typst derleyicisinden geliyor
        ve yukarıdaki sanitize'dan geçiyor.
      -->
      <div class="sheet sheet-set">
        {@html page}
      </div>
    {/each}
  </div>
</div>

<style>
  .sheet :global(svg) {
    display: block;
    width: 100%;
    height: auto;
  }
</style>
