<script lang="ts">
  /**
   * Hakkında — künye, atıf ve lisans.
   *
   * Yardım sayfasının bir bölümü olarak başlamıştı; oradan buraya TAŞINDI.
   * Kopyalanmadı: iki yerde duran bir sürüm numarası er ya da geç ayrışır ve
   * hangisinin doğru olduğunu kimse bilemez.
   */
  import PageHead from "$lib/components/shell/PageHead.svelte";

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
  const CEKIRDEK = [
    { ad: "typst", surum: "0.15.1", lisans: "Apache-2.0", ne: "dizgi motoru" },
    { ad: "tauri", surum: "2.11.1", lisans: "Apache-2.0 / MIT", ne: "masaüstü çatısı" },
    { ad: "svelte", surum: "5.55.5", lisans: "MIT", ne: "arayüz" },
    { ad: "@sveltejs/kit", surum: "2.59.1", lisans: "MIT", ne: "yönlendirme" },
    { ad: "@codemirror/view", surum: "6.43.9", lisans: "MIT", ne: "kod editörü" },
    { ad: "d3", surum: "7.9.0", lisans: "ISC", ne: "grafik ölçekleri" },
    { ad: "sqlx", surum: "0.8.6", lisans: "MIT / Apache-2.0", ne: "veritabanı" },
    { ad: "tokio", surum: "1.52.3", lisans: "MIT", ne: "eşzamansız çalışma" },
    { ad: "serde", surum: "1.0.228", lisans: "MIT / Apache-2.0", ne: "veri dönüşümü" },
    { ad: "include_dir", surum: "0.7.4", lisans: "MIT", ne: "paket gömme" },
  ];

  const PAKETLER = [
    { ad: "cetz", surum: "0.4.2", lisans: "LGPL-3.0", ne: "çizim ve grafik" },
    { ad: "zap", surum: "0.5.0", lisans: "LGPL-3.0", ne: "devre şemaları" },
    { ad: "oxifmt", surum: "1.0.0", lisans: "Apache-2.0 / MIT", ne: "biçimlendirme" },
  ];

  /**
   * Kâğıdın yazı tipleri ayrı dosya olarak GELMİYOR: typst-assets crate'inin
   * içinden çıkıyorlar (Libertinus Serif, DejaVu Sans Mono, New Computer
   * Modern). Doğru atıf o crate'e.
   */
  const YAZI_TIPLERI = [
    { ad: "@fontsource/public-sans", lisans: "OFL-1.1", ne: "arayüz" },
    { ad: "@fontsource/jetbrains-mono", lisans: "OFL-1.1", ne: "kod editörü" },
    { ad: "typst-assets", lisans: "Apache-2.0", ne: "sınav kâğıdının yazı tipleri" },
  ];

  const NPM_SAYISI = 176;
  const CRATE_SAYISI = 709;

  /**
   * Sürüm elle yazılıyor ve Cargo.toml ile tauri.conf.json'daki 0.1.0 ile aynı
   * kalmak zorunda — sürüm yükseltirken üçü birden güncellenmeli.
   */
  const SURUM = "0.1.0";
</script>

<div class="flex h-full min-h-0 flex-col">
  <PageHead title="Hakkında" />

  <div class="min-h-0 flex-1 overflow-auto">
    <div class="mx-auto max-w-[680px] px-rule py-rule">
      <img
        src="/tayan-logo.png"
        alt="TAYAN — Soru Bankası ve Typst Editörü"
        class="block w-[280px] max-w-full"
      />

      <p class="mt-rule leading-rule">
        <strong>TAYAN</strong>, öğretmenlerin sınav sorusu yazması, sınav kurması ve
        sonuçları çözümlemesi için yapılmış çevrimdışı bir masaüstü uygulamasıdır.
        Sorular <a href="https://typst.app" target="_blank" rel="noreferrer">Typst</a>
        ile dizilir; kâğıda basılan ile ekranda görülen aynıdır.
      </p>

      <dl
        class="mt-rule grid grid-cols-[130px_1fr] gap-x-half gap-y-half border-t border-rule pt-half"
      >
        <dt class="stamp">Sürüm</dt>
        <dd class="tnum">{SURUM}</dd>

        <dt class="stamp">Geliştiren</dt>
        <dd>Hakan Gülen</dd>

        <dt class="stamp">Kurum</dt>
        <dd>
          Atatürk Mesleki ve Teknik Anadolu Lisesi<br />
          <span class="pencil">Elektrik-Elektronik Teknolojisi Alanı</span>
        </dd>

        <dt class="stamp">Lisans</dt>
        <dd>Apache License 2.0</dd>

        <dt class="stamp">Kaynak kod</dt>
        <dd>
          <a
            href="https://github.com/Ataturk-MTAL/tayan"
            target="_blank"
            rel="noreferrer"
            class="font-mono text-[12px]">github.com/Ataturk-MTAL/tayan</a
          >
        </dd>
      </dl>

      <h2 class="stamp mt-rule border-t border-rule-strong pt-half">
        Kullanılan açık kaynak bileşenler
      </h2>
      <p class="pencil mt-quarter">
        TAYAN bu bileşenler olmadan var olamazdı. Her biri kendi lisansı
        altındadır.
      </p>

      <h3 class="stamp mt-half">Dizgi ve uygulama çatısı</h3>
      <ul class="mt-quarter">
        {#each CEKIRDEK as b (b.ad)}
          <li class="flex justify-between gap-half border-b border-rule py-[2px]">
            <span class="annot">
              <span class="font-mono text-[12px]">{b.ad}</span>
              <span class="pencil">{b.surum}</span> — {b.ne}
            </span>
            <span class="pencil shrink-0">{b.lisans}</span>
          </li>
        {/each}
      </ul>

      <h3 class="stamp mt-half">Gömülü Typst paketleri</h3>
      <p class="pencil mt-quarter">
        Uygulamanın içine gömülüdürler; internet olmadan da çalışsın diye.
        Kaynak hâlleriyle dağıtılırlar, yani değiştirilebilirler.
      </p>
      <ul class="mt-quarter">
        {#each PAKETLER as p (p.ad)}
          <li class="flex justify-between gap-half border-b border-rule py-[2px]">
            <span class="annot">
              <span class="font-mono text-[12px]">{p.ad}</span>
              <span class="pencil">{p.surum}</span> — {p.ne}
            </span>
            <span class="pencil shrink-0">{p.lisans}</span>
          </li>
        {/each}
      </ul>

      <h3 class="stamp mt-half">Yazı tipleri</h3>
      <ul class="mt-quarter">
        {#each YAZI_TIPLERI as y (y.ad)}
          <li class="flex justify-between gap-half border-b border-rule py-[2px]">
            <span class="annot">
              <span class="font-mono text-[12px]">{y.ad}</span> — {y.ne}
            </span>
            <span class="pencil shrink-0">{y.lisans}</span>
          </li>
        {/each}
      </ul>

      <p class="pencil mt-half">
        Yukarıdakiler doğrudan kullanılan bileşenler. Onların da kendi
        bağımlılıkları var: toplam <b>{NPM_SAYISI}</b> JavaScript paketi ve
        <b>{CRATE_SAYISI}</b> Rust crate'i. Büyük çoğunluğu MIT, ISC ve
        Apache-2.0. Tam liste depodaki
        <span class="font-mono text-[12px]">NOTICE</span> ve
        <span class="font-mono text-[12px]">THIRD-PARTY.md</span> dosyalarındadır.
      </p>

      <p class="pencil mt-rule border-t border-rule pt-half">
        Telif hakkı &copy; 2026 Hakan Gülen. TAYAN açık kaynaktır ve Apache License,
        Version 2.0 koşullarıyla dağıtılır.
      </p>

      <p class="pencil mt-half">
        Uygulama tamamen çevrimdışıdır: hesap açılmaz, veri toplanmaz, hiçbir bilgi
        dışarı gönderilmez. Soru bankası, sınavlar, sınıflar ve sonuçlar yalnızca bu
        bilgisayarda durur.
      </p>
    </div>
  </div>
</div>
