<script lang="ts">
  /**
   * Puan dağılımı — ölçme-değerlendirme kitaplarındaki çan eğrisi.
   *
   * FREKANS DİKEY, PUAN YATAY. Öğretmen mod, medyan ve ortalamanın birbirine
   * göre yerinden sınıfın durumunu okuyor: mod > medyan > ortalama sola
   * çarpıktır ve sınıf başarılıdır, ters sıra sınıfın zorlandığını gösterir.
   *
   * ÇUBUK DEĞİL EĞRİ. Histogram aralık sınırına duyarlı: sınır bir puan kaysa
   * şekil değişiyor. Altı öğrencide her aralığa bir kişi düşüyor ve grafik
   * veriyi değil aralık genişliğini gösteriyordu — hiçbir şey anlaşılmıyordu.
   * Eğri (çekirdek yoğunluk kestirimi) bunu yapmaz: her puan kendi etrafına
   * bir tümsek koyar, tümsekler toplanır, dağılımın şekli görünür.
   *
   * D3 YALNIZ HESAP İÇİN: ölçekler ve eğri üreteci. Çizimi Svelte yapıyor;
   * D3'ün kendi belgesinin önerdiği ayrım bu.
   */
  import { area, curveBasis, line, scaleLinear } from "d3";
  import { BIN_WIDTH, densityCurve, skewLabel, type Spread } from "$lib/analysis/item-stats";

  type Props = {
    /** Öğrenci başına yüzde. */
    percentages: number[];
    stats: Spread | null;
    /** Geçme eşiği — yatay eksenin anlam ortası. */
    threshold?: number;
  };

  let { percentages, stats, threshold = 50 }: Props = $props();

  /** Çarpıklığın güvenilir okunabildiği en küçük sınıf. */
  const MIN_CARPIKLIK_N = 15;

  const W = 320;
  const H = 140;
  const SERIT = 20;
  const SOL = 22;
  const ALT = 14;

  let egri = $derived(densityCurve(percentages, BIN_WIDTH));

  /**
   * Dikey eksenin tepesi. Eğrinin tepesiyle kutunun tavanı arasında bir tutam
   * boşluk bırakılıyor; eğri tavana yapışırsa tepenin nerede olduğu görünmez.
   */
  let tavan = $derived(Math.max(1, ...egri.map((p) => p.y)) * 1.15);

  let x = $derived(scaleLinear<number, number>().domain([0, 100]).range([SOL, W]));
  let y = $derived(scaleLinear<number, number>().domain([0, tavan]).range([H - ALT, 0]));

  /** Frekans ekseni TAM SAYI: "2.5 öğrenci" diye bir şey yok. */
  let yTikler = $derived(
    Array.from(new Set(y.ticks(3).map(Math.round))).filter((t) => t <= tavan),
  );
  let xTikler = $derived(x.ticks(5));

  let cizgi = $derived(
    line<{ x: number; y: number }>()
      .x((p) => x(p.x))
      .y((p) => y(p.y))
      .curve(curveBasis),
  );

  let alan = $derived(
    area<{ x: number; y: number }>()
      .x((p) => x(p.x))
      .y0(y(0))
      .y1((p) => y(p.y))
      .curve(curveBasis),
  );

  /**
   * Mod null olabilir: tepe noktası yoksa çizgisi de çizilmez. Olmayan bir
   * tepeyi grafiğe koymak dağılımın yanlış okunmasına yol açardı.
   */
  let isaretler = $derived(
    stats === null
      ? []
      : [
          ...(stats.mode === null ? [] : [{ ad: "Mod", deger: stats.mode, dy: 10 }]),
          { ad: "Medyan", deger: stats.median, dy: 24 },
          { ad: "Ort", deger: stats.mean, dy: 38 },
        ],
  );

  function kirp(v: number): number {
    return Math.min(Math.max(v, 0), 100);
  }
</script>

<figure
  class="m-0 rounded-lg border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800"
>
  <figcaption class="text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
    Puan dağılımı
  </figcaption>

  {#if stats === null}
    <p class="mt-[5px] text-[12px] leading-5 text-gray-500 dark:text-gray-400">Sonuç girilmemiş.</p>
  {:else}
    <svg
      class="mt-2.5 w-full"
      viewBox="0 0 {W} {H + SERIT}"
      role="img"
      aria-label="Puan dağılım eğrisi: yatay eksen puan, dikey eksen frekans"
    >
      <!-- Eksen ve ızgara: veri değil, okuma çerçevesi — soluk gri. -->
      <line x1={SOL} y1={0} x2={SOL} y2={H - ALT} class="stroke-gray-300 dark:stroke-gray-600" />
      <line x1={SOL} y1={H - ALT} x2={W} y2={H - ALT} class="stroke-gray-300 dark:stroke-gray-600" />

      {#each yTikler as t (t)}
        <text
          x={SOL - 4}
          y={y(t) + 3}
          text-anchor="end"
          class="fill-gray-500 dark:fill-gray-400"
          style="font-size: 8px; font-variant-numeric: tabular-nums"
        >
          {t}
        </text>
      {/each}

      <!-- Ana veri: eğrinin kendisi, koyu gri/lacivert — değerlendirme değil, ölçüm. -->
      {#if egri.length > 0}
        <path d={alan(egri) ?? ""} class="fill-gray-800 dark:fill-gray-200" opacity="0.14" />
        <path d={cizgi(egri) ?? ""} fill="none" class="stroke-gray-800 dark:stroke-gray-200" stroke-width="1.6" />
      {/if}

      <!-- Geçme eşiği: değerlendirme çizgisi, kırmızı. -->
      <line
        x1={x(kirp(threshold))}
        y1={0}
        x2={x(kirp(threshold))}
        y2={H - ALT}
        class="stroke-red-600 dark:stroke-red-400"
        stroke-dasharray="3 3"
      >
        <title>Geçme eşiği %{threshold}</title>
      </line>

      <!-- Mod/medyan/ortalama çizgileri de değerlendirme okuması: aynı kırmızı. -->
      {#each isaretler as m (m.ad)}
        <line
          x1={x(kirp(m.deger))}
          y1={0}
          x2={x(kirp(m.deger))}
          y2={H - ALT}
          class="stroke-red-600 dark:stroke-red-400"
        >
          <title>{m.ad}: %{m.deger.toFixed(1)}</title>
        </line>
        <text
          x={x(kirp(m.deger)) + 3}
          y={m.dy}
          class="fill-red-600 dark:fill-red-400"
          style="font-size: 8px"
        >
          {m.ad}
        </text>
      {/each}

      <!--
        Ham puanlar. Eğri bir KESTİRİM; noktalar gerçeğin kendisi. İkisi aynı
        eksende yan yana durunca eğrinin nerede yumuşattığı da görünüyor.
        Eşiğin altındaki nokta kırmızı — bu da bir değerlendirme okuması.
      -->
      {#each percentages as p, i (i)}
        <circle
          cx={x(kirp(p))}
          cy={H - ALT + 9}
          r="2.5"
          class={p >= threshold ? "fill-gray-800 dark:fill-gray-200" : "fill-red-600 dark:fill-red-400"}
        >
          <title>%{p.toFixed(0)}</title>
        </circle>
      {/each}

      {#each xTikler as t (t)}
        <text
          x={x(t)}
          y={H + SERIT - 2}
          text-anchor={t === 0 ? "start" : t >= 100 ? "end" : "middle"}
          class="fill-gray-500 dark:fill-gray-400"
          style="font-size: 8px; font-variant-numeric: tabular-nums"
        >
          {t}
        </text>
      {/each}
    </svg>

    {#if egri.length === 0}
      <p class="mt-[5px] text-[12px] leading-5 text-gray-500 dark:text-gray-400">
        Eğri çizilmedi: {stats.n} öğrenciyle ya da herkes aynı puanı aldığında
        dağılımın şekli hesaplanamıyor. Alttaki noktalar ham puanları gösteriyor.
      </p>
    {:else}
      <p class="mt-[5px] text-[12px] leading-5 text-gray-500 dark:text-gray-400">
        Yatay: puan · Dikey: frekans. Kesikli çizgi geçme eşiği; koyu çizgiler
        mod, medyan ve ortalama. Alttaki noktalar tek tek öğrenciler.
      </p>
    {/if}

    <!--
      ÖLÇÜLER YATAY. Dikey liste dar bir sütun izlenimi veriyor ve grafiğin
      altına uzun bir metin kuyruğu ekliyordu; beş sayı yan yana tek bakışta
      okunuyor ve mod/medyan/ortalama sırası da böyle görünüyor.
    -->
    <dl class="mt-[5px] flex flex-wrap gap-x-5 gap-y-[5px] border-t border-gray-300 pt-[5px] dark:border-gray-600">
      {#each [["Mod", stats.mode === null ? "—" : stats.mode.toFixed(0)], ["Medyan", stats.median.toFixed(1)], ["Ortalama", stats.mean.toFixed(1)], ["Std. sapma", stats.sd.toFixed(1)], ["Çarpıklık", stats.skewness === null ? "—" : stats.skewness.toFixed(2)]] as [ad, deger] (ad)}
        <div>
          <dt class="text-[12px] leading-5 text-gray-500 dark:text-gray-400">{ad}</dt>
          <dd class="tnum text-[15px] leading-5 text-gray-900 dark:text-white">{deger}</dd>
        </div>
      {/each}
    </dl>

    <!--
      Çarpıklık sözü her zaman kırmızı DEĞİL: yön olumlu da olabilir ("sola
      çarpık — sınıf başarılı"). Kırmızı yalnız değerlendirme/olumsuz bulgu
      içindir; burası nötr açıklama.
    -->
    <p class="mt-[5px] text-[12px] leading-5 text-gray-700 dark:text-gray-300">{skewLabel(stats.skewness)}</p>

    <!--
      Uyarılar tek satıra indi. Paragraf paragraf açıklama, dashboard'da
      okunmuyor ve yerleşimi bozuyor; sebep kısa, ayrıntı yardım sayfasında.
    -->
    {#if stats.mode === null || stats.n < MIN_CARPIKLIK_N}
      <ul class="mt-[5px] text-[12px] leading-5 text-gray-500 dark:text-gray-400">
        {#if stats.mode === null}
          <li>Tepe noktası yok — hiçbir puan aralığında yığılma oluşmamış.</li>
        {/if}
        {#if stats.n < MIN_CARPIKLIK_N}
          <li>
            Çarpıklık {stats.n} öğrenciyle oynak; {MIN_CARPIKLIK_N} kişiden
            itibaren yorumlanabilir.
          </li>
        {/if}
      </ul>
    {/if}
  {/if}
</figure>
