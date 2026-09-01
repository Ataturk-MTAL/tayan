<script lang="ts">
  import "../app.css";
  import { page } from "$app/state";
  import { installPageZoomGuard, resetPageZoom } from "$lib/ui/page-zoom";

  let { children } = $props();

  /**
   * Sayfa zoom'u uygulama genelinde kilitli.
   *
   * Kilitsiz hâlde `⌘+tekerlek` — ve macOS'ta trackpad kıstırması — fare
   * önizlemenin dışındayken doğrudan webview'e düşüyor ve TÜM arayüzü
   * ölçekliyordu: panel, gezinme bandı, satır numaraları. Öğretmen kâğıdı
   * büyütmek isterken uygulamayı büyütüyordu.
   *
   * Önizlemenin kendi zoom'u bundan etkilenmez; koruma yayılımı durdurmaz.
   */
  $effect(() => {
    void resetPageZoom();
    return installPageZoomGuard();
  });

  const NAV = [
    { href: "/questions", label: "Sorular" },
    { href: "/exams", label: "Sınavlar" },
    { href: "/students", label: "Öğrenciler" },
    { href: "/analysis", label: "Analiz" },
    { href: "/yardim", label: "Yardım" },
  ];

  let path = $derived(page.url.pathname);
</script>

<!--
  Gezinme kesintisiz tek bir şerittir, koyu bir kenar çubuğu değil. Sayfanın tek
  yatay çizgisi olarak durur; altındaki her şey kâğıttır.
-->
<div class="flex h-screen flex-col bg-paper">
  <header class="ruled-bottom flex shrink-0 items-stretch bg-paper-lift paper-plain">
    <a
      href="/"
      class="flex items-center px-half text-[13px] font-bold tracking-[0.12em] text-ink no-underline"
    >
      TAYAN
    </a>

    <nav class="flex items-stretch">
      {#each NAV as item}
        <a
          href={item.href}
          class="flex items-center border-l border-rule px-half py-quarter text-[12px]
                 no-underline transition-colors hover:text-red-deep"
          class:text-ink={path.startsWith(item.href)}
          class:font-semibold={path.startsWith(item.href)}
          class:text-pencil={!path.startsWith(item.href)}
          aria-current={path.startsWith(item.href) ? "page" : undefined}
        >
          {item.label}
          {#if path.startsWith(item.href)}
            <span class="ml-half block h-[3px] w-[3px] bg-red" aria-hidden="true"></span>
          {/if}
        </a>
      {/each}
    </nav>

    <div class="flex-1 border-l border-rule"></div>
  </header>

  <main class="min-h-0 flex-1">
    {@render children()}
  </main>
</div>
