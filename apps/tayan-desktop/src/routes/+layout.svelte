<script lang="ts">
  /**
   * Uygulama kabuğu — solda kalıcı navigasyon çekmecesi, sağda içerik.
   *
   * ÜST ŞERİTTEN SOL ÇEKMECEYE. Yatay şerit altı bölümü yan yana sıkıştırıyor
   * ve pencere daraldığında taşıyordu; dikey çekmece bölüm adlarını tam
   * gösteriyor, ikon veriyor ve büyümeye yer bırakıyor.
   *
   * KAYDIRMA KABUKTA DEĞİL. Pencere sabit; her bölge kendi içinde kayıyor.
   * Belge kaydırması menüyü de yukarı taşıyordu.
   */
  import "../app.css";
  import { page } from "$app/state";
  import { installPageZoomGuard, resetPageZoom } from "$lib/ui/page-zoom";
  import { initTheme } from "$lib/ui/theme.svelte";
  import ThemeToggle from "$lib/components/shell/ThemeToggle.svelte";
  import {
    ChartPieOutline,
    ClipboardListOutline,
    FileLinesOutline,
    InfoCircleOutline,
    QuestionCircleOutline,
    UsersGroupOutline,
  } from "flowbite-svelte-icons";

  let { children } = $props();

  /**
   * Sayfa zoom'u uygulama genelinde kilitli.
   *
   * Kilitsiz hâlde `⌘+tekerlek` — ve macOS'ta trackpad kıstırması — fare
   * önizlemenin dışındayken doğrudan webview'e düşüyor ve TÜM arayüzü
   * ölçekliyordu. Önizlemenin kendi zoom'u bundan etkilenmez.
   */
  $effect(() => {
    void resetPageZoom();
    return installPageZoomGuard();
  });

  $effect(() => initTheme());

  const NAV = [
    { href: "/questions", label: "Sorular", icon: FileLinesOutline },
    { href: "/exams", label: "Sınavlar", icon: ClipboardListOutline },
    { href: "/students", label: "Öğrenciler", icon: UsersGroupOutline },
    { href: "/analysis", label: "Analiz", icon: ChartPieOutline },
  ];

  const ALT_NAV = [
    { href: "/yardim", label: "Yardım", icon: QuestionCircleOutline },
    { href: "/hakkinda", label: "Hakkında", icon: InfoCircleOutline },
  ];

  let path = $derived(page.url.pathname);

  /**
   * `/questions` ile `/questions/new` aynı bölüm; `startsWith` tek başına
   * yetmez çünkü ileride `/exams-arsiv` gibi bir yol da eşleşirdi.
   */
  function aktifMi(href: string): boolean {
    return path === href || path.startsWith(`${href}/`);
  }

  const BAGLANTI =
    "flex items-center gap-3 rounded-lg px-3 py-2 text-sm no-underline transition-colors";
  const AKTIF =
    "bg-primary-100 font-semibold text-primary-800 dark:bg-primary-900/40 dark:text-primary-200";
  const PASIF =
    "text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700/60";
</script>

<div class="flex h-full bg-gray-50 dark:bg-gray-900">
  <!--
    Çekmece sabit genişlikte ve KENDİ İÇİNDE kayıyor: bölüm sayısı arttığında
    içerik alanını itmemeli.
  -->
  <aside
    class="flex w-56 shrink-0 flex-col border-r border-gray-200 bg-white
           dark:border-gray-700 dark:bg-gray-800"
  >
    <a
      href="/"
      class="flex shrink-0 items-center gap-2 border-b border-gray-200 px-4 py-4
             no-underline dark:border-gray-700"
    >
      <span
        class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary-600
               text-sm font-bold text-white"
      >
        T
      </span>
      <span class="text-base font-bold tracking-tight text-gray-900 dark:text-white">
        TAYAN
      </span>
    </a>

    <nav class="min-h-0 flex-1 space-y-1 overflow-auto p-3">
      {#each NAV as item (item.href)}
        <a
          href={item.href}
          class="{BAGLANTI} {aktifMi(item.href) ? AKTIF : PASIF}"
          aria-current={aktifMi(item.href) ? "page" : undefined}
        >
          <item.icon class="h-5 w-5 shrink-0" />
          {item.label}
        </a>
      {/each}
    </nav>

    <div class="shrink-0 space-y-1 border-t border-gray-200 p-3 dark:border-gray-700">
      {#each ALT_NAV as item (item.href)}
        <a
          href={item.href}
          class="{BAGLANTI} {aktifMi(item.href) ? AKTIF : PASIF}"
          aria-current={aktifMi(item.href) ? "page" : undefined}
        >
          <item.icon class="h-5 w-5 shrink-0" />
          {item.label}
        </a>
      {/each}

      <div class="pt-1">
        <ThemeToggle />
      </div>
    </div>
  </aside>

  <main class="flex min-h-0 min-w-0 flex-1 flex-col">
    {@render children()}
  </main>
</div>
