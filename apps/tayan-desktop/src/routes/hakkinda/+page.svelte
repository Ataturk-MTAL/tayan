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

      <p class="pencil mt-rule border-t border-rule pt-half">
        Telif hakkı &copy; 2026 Hakan Gülen. TAYAN açık kaynaktır ve Apache License,
        Version 2.0 koşullarıyla dağıtılır. Dizgi motoru Typst, ayrı kendi lisansı
        altındadır; üçüncü taraf bileşenlerin tam listesi için depodaki
        <span class="font-mono text-[12px]">NOTICE</span> dosyasına bakınız.
      </p>

      <p class="pencil mt-half">
        Uygulama tamamen çevrimdışıdır: hesap açılmaz, veri toplanmaz, hiçbir bilgi
        dışarı gönderilmez. Soru bankası, sınavlar, sınıflar ve sonuçlar yalnızca bu
        bilgisayarda durur.
      </p>
    </div>
  </div>
</div>
