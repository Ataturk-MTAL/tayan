<script lang="ts">
  /**
   * Uygulama kabuğu — başlık şeridinin altından açılan navigasyon çekmecesi.
   *
   * ÜST ŞERİTTEN SOL ÇEKMECEYE. Yatay şerit altı bölümü yan yana sıkıştırıyor
   * ve pencere daraldığında taşıyordu; dikey çekmece bölüm adlarını tam
   * gösteriyor, ikon veriyor ve büyümeye yer bırakıyor.
   *
   * KAYDIRMA KABUKTA DEĞİL. Pencere sabit; her bölge kendi içinde kayıyor.
   * Belge kaydırması menüyü de yukarı taşıyordu.
   *
   * ÇEKMECE HER GENİŞLİKTE ÜSTE BİNER (`fixed` + karartma), içeriğe yer
   * açmaz. Önceki sürüm geniş pencerede yer açıp yalnız dar pencerede üste
   * biniyordu; iki kip iki ayrı yerleşim demekti ve içerik alanı çekmece her
   * açılıp kapandığında yeniden diziliyordu. Tek kipte içerik hiç kımıldamıyor
   * ve menü, gezinme durumundan bağımsız her zaman açılabiliyor.
   *
   * ÇEKMECE BAŞLIK ŞERİDİNİN ALTINDAN BAŞLAR. Şerit açıkken de görünür kalıyor
   * ki çekmeceyi kapatan ✕ düğmesi erişilebilir olsun; şeridi de örtseydi tek
   * kapatma yolu karartma ve Esc olurdu.
   *
   * BAŞLANGIÇ NOKTASI SABİT: `--tayan-header-h` (app.css). Şerit bir ara
   * içeriğine göre büyüyüp küçülüyor ve `ResizeObserver` ile ölçülüyordu; o
   * yükseklik sayfa değiştikçe zıpladığı için çekmecenin üst kenarı da
   * oynuyordu. Şerit artık sabit yükseklikte, ölçüme gerek kalmadı.
   *
   * ÜSTE BİNEN ÇEKMECE BİR KALIP KATMANIDIR, o yüzden kalıbın üç borcunu da
   * ödüyor: arkasında karartma, Esc ile kapanma, ve odağın içeri alınıp
   * kapanışta geldiği yere iade edilmesi.
   *
   * BÖLÜM LİSTESİ BURADA DEĞİL. `$lib/ui/nav.svelte.ts` hem bu çekmecenin hem
   * başlık şeridinin ortak kaynağı; bölüm eklemek iki yeri birden günceller.
   */
  import "../app.css";
  import { page } from "$app/state";
  import { installPageZoomGuard, resetPageZoom } from "$lib/ui/page-zoom";
  import { initTheme } from "$lib/ui/theme.svelte";
  import { closeDrawer, drawer } from "$lib/ui/drawer.svelte";
  import { isInSection, nav } from "$lib/ui/nav.svelte";
  import { pushEscapeLayer } from "$lib/ui/escape-stack";
  import ThemeToggle from "$lib/components/shell/ThemeToggle.svelte";

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

  /**
   * Esc açık çekmeceyi kapatır.
   *
   * Katman yığına yalnız çekmece AÇIKKEN giriyor. Sürekli kayıtlı olsaydı
   * kapalı çekmece, altındaki gerçek katmanın (kalıp paleti, tamamlama
   * kutusu) Esc'ini yutardı — merdiven her zaman EN ÜSTTEKİNİ kapatır.
   */
  $effect(() => {
    if (!drawer.open) return;
    return pushEscapeLayer(closeDrawer);
  });

  let drawerEl = $state<HTMLElement | null>(null);
  let previousFocus: HTMLElement | null = null;

  /**
   * Odak açılışta çekmeceye girer, kapanışta geldiği yere döner.
   *
   * ODAK İADESİ ŞART. Çekmece kapandığında odak hiçbir yerde kalmasaydı
   * tarayıcı onu `<body>`ye atar ve sonraki Sekme tuşu kullanıcıyı sayfanın
   * EN BAŞINA götürürdü — klavyeyle gezinen öğretmen menüyü her kapattığında
   * yerini kaybederdi. Temizleme işlevi kapanışta çalışıyor ve odağı ✕
   * düğmesine geri veriyor.
   */
  $effect(() => {
    if (!drawer.open) return;

    previousFocus = document.activeElement as HTMLElement | null;
    drawerEl?.querySelector("a")?.focus();

    return () => previousFocus?.focus();
  });

  let path = $derived(page.url.pathname);

  /**
   * Yol değiştiğinde çekmece kapanır.
   *
   * Çekmecedeki bölüm bağlantılarının kendi `onclick`i var ama o tek başına
   * YETMİYOR: başlık şeridi karartılmadığı için marka, çekmece açıkken de
   * tıklanabilir durumda ve ana sayfaya götürüyor — çekmece de yeni sayfanın
   * üstünde açık kalırdı. Kapanmayı bağlantı bağlantı değil GEZİNMEYE
   * bağlamak, ileride nereden gelirse gelsin her yön değişimini kapsıyor.
   *
   * Bağlantılardaki `onclick` yine de duruyor: BULUNULAN bölüme tıklamak yolu
   * değiştirmez, bu etki tetiklenmez ve çekmece açık kalırdı.
   */
  $effect(() => {
    void path;
    closeDrawer();
  });

  const LINK =
    "flex items-center gap-3 rounded-lg px-3 py-2 text-sm no-underline transition-colors";
  const ACTIVE =
    "bg-primary-100 font-semibold text-primary-800 dark:bg-primary-900/40 dark:text-primary-200";
  const IDLE =
    "text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700/60";
</script>

<div class="relative h-full overflow-hidden bg-gray-50 dark:bg-gray-900">
  <!--
    İÇERİK ÇEKMECEDEN ÖNCE VE ONDAN BAĞIMSIZ. Çekmece `fixed` olduğu için
    yerleşim akışında hiç yer kaplamıyor; açılıp kapanması içeriği yeniden
    dizmiyor, yalnız üstünü örtüyor.
  -->
  <main class="flex h-full min-h-0 min-w-0 flex-col">
    {@render children()}
  </main>

  <!--
    Karartma başlık şeridinin ALTINDAN başlıyor — şerit karartılmıyor ki ✕
    düğmesi hem görünür hem tıklanabilir kalsın. Şeridi de karartsaydık düğme
    görünürdü ama üstündeki karartma tıklamayı yutardı.

    `<button>`, `<div onclick>` değil: kapatma bir eylem, klavyeyle de
    erişilebilir olmalı ve ekran okuyucuya adıyla duyurulmalı.
  -->
  {#if drawer.open}
    <button
      type="button"
      class="fixed inset-x-0 bottom-0 z-30 bg-gray-900/50"
      style="top: var(--tayan-header-h)"
      aria-label="Menüyü kapat"
      onclick={closeDrawer}
    ></button>
  {/if}

  <!--
    Çekmece KENDİ İÇİNDE kayıyor: bölüm sayısı arttığında pencereyi uzatmıyor.

    `inert` KAPALIYKEN ŞART. Kapalı çekmece yalnız GÖRÜNMEZ — ekranın dışında,
    ama hâlâ belgede. `inert` olmadan Sekme tuşu görünmeyen altı bağlantıyı
    tek tek dolaşır ve klavye kullanıcısı odağı kaybeder.

    `motion-reduce:transition-none`: hareketi azaltılmış sistemde çekmece
    kayarak değil, anında yer değiştirir.
  -->
  <aside
    id="main-drawer"
    bind:this={drawerEl}
    class="fixed bottom-0 left-0 z-40 flex w-56 flex-col border-r
           border-gray-200 bg-white shadow-xl transition-transform duration-200
           ease-out motion-reduce:transition-none dark:border-gray-700
           dark:bg-gray-800 {drawer.open ? 'translate-x-0' : '-translate-x-full'}"
    style="top: var(--tayan-header-h)"
    inert={!drawer.open}
  >
    <!--
      MARKA ARTIK BAŞLIK ŞERİDİNDE, burada değil. Şerit her zaman görünür
      olduğu için markanın çekmecenin içinde durması onu menü açılana kadar
      gizliyordu — kimlik, uygulamanın açılışta görünen yüzü olmalı. Ana
      sayfaya (`/`) giden bağlantı da markayla birlikte şeride taşındı; bu
      çekmecede artık yalnız BÖLÜMLER var.
    -->
    <nav class="min-h-0 flex-1 space-y-1 overflow-auto p-3">
      {#each nav.main as item (item.href)}
        <a
          href={item.href}
          class="{LINK} {isInSection(item.href, path) ? ACTIVE : IDLE}"
          aria-current={isInSection(item.href, path) ? "page" : undefined}
          onclick={closeDrawer}
        >
          <item.icon class="h-5 w-5 shrink-0" />
          {item.label}
        </a>
      {/each}
    </nav>

    <div class="shrink-0 space-y-1 border-t border-gray-200 p-3 dark:border-gray-700">
      {#each nav.secondary as item (item.href)}
        <a
          href={item.href}
          class="{LINK} {isInSection(item.href, path) ? ACTIVE : IDLE}"
          aria-current={isInSection(item.href, path) ? "page" : undefined}
          onclick={closeDrawer}
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
</div>
