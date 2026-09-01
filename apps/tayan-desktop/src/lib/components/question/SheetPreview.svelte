<script lang="ts">
  import SheetPage from "./SheetPage.svelte";
  import PreviewZoom from "./PreviewZoom.svelte";

  type Props = {
    pages: string[];
    /**
     * Yalnızca derleme GERÇEKTEN uzun sürdüğünde true. Her derlemede değil:
     * tipik derleme 5-30 ms ve debounce 120 ms; bu ritimde bir şeyi soldurup
     * geri getirmek bilgi vermez, yalnızca göz yorar.
     */
    stale: boolean;
    error: string | null;
  };

  let { pages, stale, error }: Props = $props();

  const A4_WIDTH_PX = 794;
  const MIN_ZOOM = 0.5;
  const MAX_ZOOM = 3;
  const ZOOM_KEY = "tayan.preview.zoom";

  /**
   * Zoom tarayıcıda saklanır: öğretmen bir kez ayarlar, her soruda yeniden
   * ayarlamak zorunda kalmaz. Saklama başarısız olursa (özel pencere, site
   * verisi kapalı) uygulama çalışmaya devam eder, yalnızca hatırlamaz.
   */
  function loadZoom(): number {
    try {
      const raw = localStorage.getItem(ZOOM_KEY);
      const value = raw === null ? NaN : Number(raw);
      if (!Number.isFinite(value)) return 1;
      return Math.min(Math.max(value, MIN_ZOOM), MAX_ZOOM);
    } catch {
      return 1;
    }
  }

  let zoom = $state(1);
  let scroller = $state<HTMLDivElement | undefined>(undefined);

  $effect(() => {
    zoom = loadZoom();
  });

  function setZoom(value: number) {
    zoom = Math.min(Math.max(value, MIN_ZOOM), MAX_ZOOM);
    try {
      localStorage.setItem(ZOOM_KEY, String(zoom));
    } catch {
      // Hatırlamamak, çalışmamaktan iyidir.
    }
  }

  /** Sayfayı panelin genişliğine sığdır. Kenar boşluğu ızgaranın bir karesi. */
  function fit() {
    if (!scroller) return;
    const available = scroller.clientWidth - 40;
    setZoom(available / A4_WIDTH_PX);
  }

  function onWheel(event: WheelEvent) {
    if (!event.ctrlKey && !event.metaKey) return;
    event.preventDefault();
    // Küçük adımlar: tekerlek sürekli bir denetim, basamaklı değil.
    setZoom(zoom * (event.deltaY < 0 ? 1.08 : 1 / 1.08));
  }

  function onKeydown(event: KeyboardEvent) {
    if (!event.metaKey && !event.ctrlKey) return;
    if (event.key === "=" || event.key === "+") {
      event.preventDefault();
      setZoom(zoom * 1.2);
    } else if (event.key === "-") {
      event.preventDefault();
      setZoom(zoom / 1.2);
    } else if (event.key === "0") {
      event.preventDefault();
      setZoom(1);
    }
  }

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

<svelte:window onkeydown={onKeydown} />

<div class="flex h-full min-h-0 flex-col">
  <PreviewZoom {zoom} onzoom={setZoom} onfit={fit} />

  <!--
    Kâğıt masadan kalkan tek nesnedir; gölge burada gerçektir, süs değil.
    Uzun süren derlemede kâğıt soldurulmaz — kenarda ince bir çizgi belirir,
    böylece okunan metin bozulmadan durur.
  -->
  {#if stale}
    <div class="h-[2px] w-full shrink-0 bg-red" aria-hidden="true"></div>
  {/if}

  <div
    class="min-h-0 flex-1 overflow-auto bg-paper-sunk paper-grid"
    bind:this={scroller}
    onwheel={onWheel}
  >
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

    <div class="flex min-w-fit flex-col items-center gap-rule p-rule">
      {#each safePages as page, i (i)}
        <SheetPage svg={page} {zoom} />
      {/each}
    </div>
  </div>
</div>
