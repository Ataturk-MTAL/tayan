<script module lang="ts">
  import { api } from "$lib/api";

  /**
   * Aynı anda en fazla bu kadar derleme.
   *
   * Her derleme kendi TayanWorld örneğini kurar; sınırsız bırakmak 40 soruluk
   * bir bankada 40 paralel derleyici demek. Bu, süreç belleğini bir kez
   * 31,63 GB'a çıkaran kod yolunun aynısı — sınır ihtiyattan konuldu.
   * Ölçülüp artırılabilir, ama sınırsız bırakılamaz.
   */
  const MAX_CONCURRENT = 3;

  /** Gövde metni → SVG. Aynı soru iki kez derlenmez. */
  const cache = new Map<string, string>();
  let running = 0;
  const queue: Array<() => void> = [];

  function acquireSlot(): Promise<void> {
    if (running < MAX_CONCURRENT) {
      running += 1;
      return Promise.resolve();
    }
    return new Promise((resolve) => queue.push(resolve));
  }

  function releaseSlot() {
    const next = queue.shift();
    if (next) next();
    else running -= 1;
  }

  export async function compileThumbnail(body: string): Promise<string> {
    const hit = cache.get(body);
    if (hit !== undefined) return hit;

    await acquireSlot();
    try {
      const svg = await api.compiler.questionThumbnail(body);
      cache.set(body, svg);
      return svg;
    } finally {
      releaseSlot();
    }
  }
</script>

<script lang="ts">
  /**
   * Banka kartı — sorunun DİZGİSİ, kaynağı değil.
   *
   * Banka listesi soruları ham Typst olarak gösteriyordu: öğretmen kendi
   * bankasına bakınca `$x^2 - 5x + 6 = 0$ #secenekler(dogru:...` görüyordu.
   * "Ürün kâğıttır" ilkesinin en sert ihlali editörde değil, listedeydi.
   *
   * BELLEK: her kart bir Typst derlemesi. Üç önlem yukarıdaki modül bloğunda
   * ve burada:
   *   1. IntersectionObserver — yalnız EKRANA GİREN kart derlenir
   *   2. Eşzamanlılık sınırı — aynı anda en fazla MAX_CONCURRENT derleme
   *   3. Önbellek — aynı gövde iki kez derlenmez
   * Sınır ve önbellek modül düzeyinde: bütün kartlar aynı bütçeyi paylaşır.
   */
  import { errorText } from "$lib/editor/diagnostics";

  type Props = { body: string };
  let { body }: Props = $props();

  let host = $state<HTMLDivElement | undefined>(undefined);
  let svg = $state<string | null>(null);
  let failed = $state(false);

  /**
   * Typst'in SVG dışa aktarıcısı script üretmez, yine de temizliyoruz: bu
   * içerik Tauri webview'ine giriyor ve orada IPC yüzeyi var.
   */
  function sanitize(s: string): string {
    return s
      .replace(/<script\b[\s\S]*?<\/script>/gi, "")
      .replace(/<foreignObject\b[\s\S]*?<\/foreignObject>/gi, "")
      .replace(/\son\w+\s*=\s*"[^"]*"/gi, "")
      .replace(/\son\w+\s*=\s*'[^']*'/gi, "");
  }

  $effect(() => {
    const el = host;
    const source = body;
    if (!el) return;

    let cancelled = false;
    const observer = new IntersectionObserver(
      (entries) => {
        if (!entries.some((entry) => entry.isIntersecting)) return;
        observer.disconnect();
        void (async () => {
          try {
            const raw = await compileThumbnail(source);
            if (!cancelled) svg = sanitize(raw);
          } catch (err: unknown) {
            // Hata metni karta sığmaz; kart "Dizilemedi" der ve soru yine
            // açılabilir. Ayrıntı konsola bırakılmıyor, gizlenmiyor da:
            // editörde açınca aynı hata tanılama olarak görünür.
            void errorText(err);
            if (!cancelled) failed = true;
          }
        })();
      },
      // 200px önden başlat: kaydırırken kart boş görünmesin.
      { rootMargin: "200px" },
    );

    observer.observe(el);
    return () => {
      cancelled = true;
      observer.disconnect();
    };
  });
</script>

<div bind:this={host} class="thumb">
  {#if svg !== null}
    {@html svg}
  {:else if failed}
    <p class="p-2 text-[11px] text-red-600">Dizilemedi — açıp bakınca hata görünür</p>
  {:else}
    <div class="skeleton animate-pulse"></div>
  {/if}
</div>

<style>
  /*
    Kâğıt her zaman beyaz — koyu kipte bile. Kartın dizgisi basılacak sayfanın
    kendisi; içindeki metin ve iskelet rengi de bu yüzden koyu kipten
    ETKİLENMİYOR, tıpkı SheetPreview'deki kâğıt gibi.
  */
  .thumb {
    background: #ffffff;
    overflow: hidden;
    /* Uzun soru kartı ezmesin: kırpılır, tamamı editörde görülür. */
    max-height: 190px;
  }
  .thumb :global(svg) {
    display: block;
    width: 100%;
    height: auto;
  }
  .skeleton {
    height: 110px;
    background: #e5e7eb;
  }
</style>
