<script lang="ts">
  import { Alert } from "flowbite-svelte";
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
  /**
   * Taban 0.5 ve bilinçli olarak öyle KALIYOR.
   *
   * Bu sabit yalnız o anki görünümü değil, `tayan.preview.zoom` altında
   * saklanan değeri de kırpıyor (hem loadZoom hem setZoom). Yani tabanı
   * indirmek bir yerleşim düzeltmesi değil, oturumlar arası kalıcı bir
   * davranış değişikliğidir; kendi başına karara bağlanmalı, taşma düzeltmesi
   * diye geçiştirilmemeli. Taşmanın kendisi zaten aşağıdaki `overflow-auto`
   * kaydırma kabında tutuluyor — kâğıt kesilmiyor, kaydırılıyor.
   *
   * Bilinen ödün: "Sığdır" bölme 794 × 0.5 + 40 = 437 px'ten darken sözünü tam
   * tutamaz. Ölçüm — 1024 px pencere (tauri.conf.json minWidth), 224 px
   * çekmece + 1 px kenarlık, varsayılan 260 px panel, yan yana mod: önizleme
   * bölmesine 217 px kalıyor, fit() (217 − 40) / 794 ≈ 0,22 istiyor, setZoom
   * 0,5'e kırpıyor ve sayfa yatayda kaydırılabilir kalıyor.
   */
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
    Basılacak kâğıt uzun süren derlemede SOLDURULMAZ — önceki sayfa okunur
    kalır. Bunun yerine üstte ince bir çubuk belirir. Kırmızı DEĞİL: bu bir
    hata değil, yalnızca "hâlâ hesaplanıyor" bilgisi — kırmızı bu uygulamada
    yalnızca değerlendirme/hata kanalına ayrılmış (bkz. app.css).
  -->
  {#if stale}
    <div class="h-0.5 w-full shrink-0 bg-primary-400 dark:bg-primary-500" aria-hidden="true"></div>
  {/if}

  <!--
    Kâğıdın çevresi koyu kipe uyar, kâğıdın KENDİSİ (SheetPage içinde) her
    zaman beyaz kalır — basılacak sayfa gerçekte de beyaz kâğıttır.
  -->
  <div
    class="min-h-0 flex-1 overflow-auto bg-gray-100 dark:bg-gray-900"
    bind:this={scroller}
    onwheel={onWheel}
  >
    {#if error}
      <div class="p-4">
        <Alert color="red">
          <span class="font-semibold">Derlenmedi</span>
          <pre
            class="mt-2 max-h-64 overflow-auto whitespace-pre-wrap font-mono text-xs leading-5"
          >{error}</pre>
          <p class="mt-2 text-xs opacity-80">Son çalışan sayfa aşağıda duruyor.</p>
        </Alert>
      </div>
    {/if}

    {#if safePages.length === 0 && !error}
      <div class="flex h-full items-center justify-center">
        <p class="text-sm text-gray-500 dark:text-gray-400">Sayfa henüz derlenmedi.</p>
      </div>
    {/if}

    <div class="flex min-w-fit flex-col items-center gap-5 p-5">
      {#each safePages as page, i (i)}
        <SheetPage svg={page} {zoom} />
      {/each}
    </div>
  </div>
</div>
