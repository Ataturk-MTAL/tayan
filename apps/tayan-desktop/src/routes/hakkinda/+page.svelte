<script lang="ts">
  /**
   * Hakkında — künye, atıf ve lisans.
   *
   * Yardım sayfasının bir bölümü olarak başlamıştı; oradan buraya TAŞINDI.
   * Kopyalanmadı: iki yerde duran bir sürüm numarası er ya da geç ayrışır ve
   * hangisinin doğru olduğunu kimse bilemez.
   */
  import PageShell from "$lib/components/shell/PageShell.svelte";
  import { A, Badge, Heading, P } from "flowbite-svelte";

  /**
   * Açık kaynak künyesi.
   *
   * SÜRÜM VE LİSANSLAR ELLE YAZILI ama uydurma değil: `pnpm licenses list
   * --prod` ve `cargo metadata` çıktısından alındı. Bağımlılık yükseltilince
   * burası da güncellenmeli — yanlış lisans bildirmek, bildirmemekten kötüdür.
   *
   * Buraya yalnız DOĞRUDAN kullanılanlar giriyor. 176 npm paketinin ve 709
   * Rust crate'inin tamamını listelemek kimsenin okumadığı bir duvar üretir;
   * tam liste NOTICE ve THIRD-PARTY.md dosyalarında.
   */
  const CORE_COMPONENTS = [
    { name: "typst", version: "0.15.1", license: "Apache-2.0", role: "dizgi motoru" },
    { name: "tauri", version: "2.11.1", license: "Apache-2.0 / MIT", role: "masaüstü çatısı" },
    { name: "svelte", version: "5.55.5", license: "MIT", role: "arayüz" },
    { name: "@sveltejs/kit", version: "2.59.1", license: "MIT", role: "yönlendirme" },
    { name: "@codemirror/view", version: "6.43.9", license: "MIT", role: "kod editörü" },
    { name: "d3", version: "7.9.0", license: "ISC", role: "grafik ölçekleri" },
    { name: "sqlx", version: "0.8.6", license: "MIT / Apache-2.0", role: "veritabanı" },
    { name: "tokio", version: "1.52.3", license: "MIT", role: "eşzamansız çalışma" },
    { name: "serde", version: "1.0.228", license: "MIT / Apache-2.0", role: "veri dönüşümü" },
    { name: "include_dir", version: "0.7.4", license: "MIT", role: "paket gömme" },
  ];

  const TYPST_PACKAGES = [
    { name: "cetz", version: "0.4.2", license: "LGPL-3.0", role: "çizim ve grafik" },
    { name: "zap", version: "0.5.0", license: "LGPL-3.0", role: "devre şemaları" },
    { name: "oxifmt", version: "1.0.0", license: "Apache-2.0 / MIT", role: "biçimlendirme" },
  ];

  /**
   * Kâğıdın yazı tipleri ayrı dosya olarak GELMİYOR: typst-assets crate'inin
   * içinden çıkıyorlar (Libertinus Serif, DejaVu Sans Mono, New Computer
   * Modern). Doğru atıf o crate'e.
   */
  const FONTS = [
    { name: "@fontsource/public-sans", license: "OFL-1.1", role: "arayüz" },
    { name: "@fontsource/jetbrains-mono", license: "OFL-1.1", role: "kod editörü" },
    { name: "typst-assets", license: "Apache-2.0", role: "sınav kâğıdının yazı tipleri" },
  ];

  const NPM_PACKAGE_COUNT = 176;
  const RUST_CRATE_COUNT = 709;

  /**
   * Sürüm elle yazılıyor ve Cargo.toml ile tauri.conf.json'daki 0.1.0 ile aynı
   * kalmak zorunda — sürüm yükseltirken üçü birden güncellenmeli.
   */
  const APP_VERSION = "0.1.0";
</script>

<PageShell title="Hakkında">
  <div class="mx-auto max-w-[680px]">
    <img
      src="/tayan-logo.png"
      alt="TAYAN — Soru Bankası ve Typst Editörü"
      class="block w-[280px] max-w-full"
    />

    <P class="mt-6 text-gray-700 dark:text-gray-300">
      <strong class="text-gray-900 dark:text-white">TAYAN</strong>, öğretmenlerin sınav sorusu
      yazması, sınav kurması ve sonuçları çözümlemesi için yapılmış çevrimdışı bir masaüstü
      uygulamasıdır. Sorular <A href="https://typst.app" target="_blank" rel="noreferrer"
        >Typst</A
      > ile dizilir; kâğıda basılan ile ekranda görülen aynıdır.
    </P>

    <dl
      class="mt-6 grid grid-cols-[130px_1fr] gap-x-2.5 gap-y-2.5 border-t border-gray-200 pt-2.5
             text-sm dark:border-gray-700"
    >
      <dt class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
        Sürüm
      </dt>
      <dd class="tnum text-gray-900 dark:text-white">{APP_VERSION}</dd>

      <dt class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
        Geliştiren
      </dt>
      <dd class="text-gray-900 dark:text-white">Hakan Gülen</dd>

      <dt class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
        Kurum
      </dt>
      <dd class="text-gray-900 dark:text-white">
        Atatürk Mesleki ve Teknik Anadolu Lisesi<br />
        <span class="text-xs text-gray-500 dark:text-gray-400"
          >Elektrik-Elektronik Teknolojisi Alanı</span
        >
      </dd>

      <dt class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
        Lisans
      </dt>
      <dd class="text-gray-900 dark:text-white">Apache License 2.0</dd>

      <dt class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
        Kaynak kod
      </dt>
      <dd>
        <A
          href="https://github.com/Ataturk-MTAL/tayan"
          target="_blank"
          rel="noreferrer"
          class="font-mono text-xs">github.com/Ataturk-MTAL/tayan</A
        >
      </dd>
    </dl>

    <Heading
      tag="h2"
      class="mt-6 border-t border-gray-200 pt-2.5 text-base font-semibold text-gray-900
             dark:border-gray-700 dark:text-white"
    >
      Kullanılan açık kaynak bileşenler
    </Heading>
    <P class="mt-1 text-sm text-gray-500 dark:text-gray-400">
      TAYAN bu bileşenler olmadan var olamazdı. Her biri kendi lisansı altındadır.
    </P>

    <Heading
      tag="h3"
      class="mt-4 text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
    >
      Dizgi ve uygulama çatısı
    </Heading>
    <ul class="mt-1 divide-y divide-gray-200 dark:divide-gray-700">
      {#each CORE_COMPONENTS as b (b.name)}
        <li class="flex items-center justify-between gap-2.5 py-1.5">
          <span class="min-w-0 text-sm text-gray-700 dark:text-gray-300">
            <span class="font-mono text-xs text-gray-900 dark:text-white">{b.name}</span>
            <span class="text-gray-500 dark:text-gray-400">{b.version}</span> — {b.role}
          </span>
          <Badge color="gray" class="shrink-0">{b.license}</Badge>
        </li>
      {/each}
    </ul>

    <Heading
      tag="h3"
      class="mt-4 text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
    >
      Gömülü Typst paketleri
    </Heading>
    <P class="mt-1 text-sm text-gray-500 dark:text-gray-400">
      Uygulamanın içine gömülüdürler; internet olmadan da çalışsın diye. Kaynak hâlleriyle
      dağıtılırlar, yani değiştirilebilirler.
    </P>
    <ul class="mt-1 divide-y divide-gray-200 dark:divide-gray-700">
      {#each TYPST_PACKAGES as p (p.name)}
        <li class="flex items-center justify-between gap-2.5 py-1.5">
          <span class="min-w-0 text-sm text-gray-700 dark:text-gray-300">
            <span class="font-mono text-xs text-gray-900 dark:text-white">{p.name}</span>
            <span class="text-gray-500 dark:text-gray-400">{p.version}</span> — {p.role}
          </span>
          <Badge color="gray" class="shrink-0">{p.license}</Badge>
        </li>
      {/each}
    </ul>

    <Heading
      tag="h3"
      class="mt-4 text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
    >
      Yazı tipleri
    </Heading>
    <ul class="mt-1 divide-y divide-gray-200 dark:divide-gray-700">
      {#each FONTS as y (y.name)}
        <li class="flex items-center justify-between gap-2.5 py-1.5">
          <span class="min-w-0 text-sm text-gray-700 dark:text-gray-300">
            <span class="font-mono text-xs text-gray-900 dark:text-white">{y.name}</span> — {y.role}
          </span>
          <Badge color="gray" class="shrink-0">{y.license}</Badge>
        </li>
      {/each}
    </ul>

    <P class="mt-4 text-sm text-gray-500 dark:text-gray-400">
      Yukarıdakiler doğrudan kullanılan bileşenler. Onların da kendi bağımlılıkları var: toplam
      <b class="text-gray-700 dark:text-gray-300">{NPM_PACKAGE_COUNT}</b> JavaScript paketi ve
      <b class="text-gray-700 dark:text-gray-300">{RUST_CRATE_COUNT}</b> Rust crate'i. Büyük çoğunluğu
      MIT, ISC ve Apache-2.0. Tam liste depodaki
      <span class="font-mono text-xs">NOTICE</span> ve
      <span class="font-mono text-xs">THIRD-PARTY.md</span> dosyalarındadır.
    </P>

    <P class="mt-6 border-t border-gray-200 pt-2.5 text-sm text-gray-500 dark:border-gray-700 dark:text-gray-400">
      Telif hakkı &copy; 2026 Hakan Gülen. TAYAN açık kaynaktır ve Apache License, Version 2.0
      koşullarıyla dağıtılır.
    </P>

    <P class="mt-2.5 text-sm text-gray-500 dark:text-gray-400">
      Uygulama tamamen çevrimdışıdır: hesap açılmaz, veri toplanmaz, hiçbir bilgi dışarı
      gönderilmez. Soru bankası, sınavlar, sınıflar ve sonuçlar yalnızca bu bilgisayarda durur.
    </P>
  </div>
</PageShell>
