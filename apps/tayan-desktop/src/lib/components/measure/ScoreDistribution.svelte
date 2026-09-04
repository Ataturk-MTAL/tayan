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

<figure class="m-0">
  <figcaption class="stamp">Puan dağılımı</figcaption>

  {#if stats === null}
    <p class="pencil mt-quarter">Sonuç girilmemiş.</p>
  {:else}
    <svg
      class="mt-half w-full"
      viewBox="0 0 {W} {H + SERIT}"
      role="img"
      aria-label="Puan dağılım eğrisi: yatay eksen puan, dikey eksen frekans"
    >
      <line x1={SOL} y1={0} x2={SOL} y2={H - ALT} stroke="var(--color-rule-strong)" />
      <line x1={SOL} y1={H - ALT} x2={W} y2={H - ALT} stroke="var(--color-rule-strong)" />

      {#each yTikler as t (t)}
        <text
          x={SOL - 4}
          y={y(t) + 3}
          text-anchor="end"
          class="fill-ink-mid"
          style="font-size: 8px; font-variant-numeric: tabular-nums"
        >
          {t}
        </text>
      {/each}

      {#if egri.length > 0}
        <path d={alan(egri) ?? ""} class="fill-ink" opacity="0.14" />
        <path d={cizgi(egri) ?? ""} fill="none" class="stroke-ink" stroke-width="1.6" />
      {/if}

      <line
        x1={x(kirp(threshold))}
        y1={0}
        x2={x(kirp(threshold))}
        y2={H - ALT}
        stroke="var(--color-red)"
        stroke-dasharray="3 3"
      >
        <title>Geçme eşiği %{threshold}</title>
      </line>

      {#each isaretler as m (m.ad)}
        <line
          x1={x(kirp(m.deger))}
          y1={0}
          x2={x(kirp(m.deger))}
          y2={H - ALT}
          stroke="var(--color-red-deep)"
        >
          <title>{m.ad}: %{m.deger.toFixed(1)}</title>
        </line>
        <text
          x={x(kirp(m.deger)) + 3}
          y={m.dy}
          class="fill-red-deep"
          style="font-size: 8px"
        >
          {m.ad}
        </text>
      {/each}

      <!--
        Ham puanlar. Eğri bir KESTİRİM; noktalar gerçeğin kendisi. İkisi aynı
        eksende yan yana durunca eğrinin nerede yumuşattığı da görünüyor.
      -->
      {#each percentages as p, i (i)}
        <circle
          cx={x(kirp(p))}
          cy={H - ALT + 9}
          r="2.5"
          class={p >= threshold ? "fill-ink" : "fill-red"}
        >
          <title>%{p.toFixed(0)}</title>
        </circle>
      {/each}

      {#each xTikler as t (t)}
        <text
          x={x(t)}
          y={H + SERIT - 2}
          text-anchor={t === 0 ? "start" : t >= 100 ? "end" : "middle"}
          class="fill-ink-mid"
          style="font-size: 8px; font-variant-numeric: tabular-nums"
        >
          {t}
        </text>
      {/each}
    </svg>

    {#if egri.length === 0}
      <p class="pencil mt-quarter">
        Eğri çizilmedi: {stats.n} öğrenciyle ya da herkes aynı puanı aldığında
        dağılımın şekli hesaplanamıyor. Alttaki noktalar ham puanları gösteriyor.
      </p>
    {:else}
      <p class="pencil mt-quarter">
        Yatay: puan · Dikey: frekans. Kesikli çizgi geçme eşiği; koyu çizgiler
        mod, medyan ve ortalama. Alttaki noktalar tek tek öğrenciler.
      </p>
    {/if}

    <dl class="ruled-top mt-quarter grid grid-cols-[auto_1fr] gap-x-half">
      <dt class="pencil">Mod</dt>
      <dd class="annot tnum text-right">
        {stats.mode === null ? "—" : stats.mode.toFixed(0)}
      </dd>
      <dt class="pencil">Medyan</dt>
      <dd class="annot tnum text-right">{stats.median.toFixed(1)}</dd>
      <dt class="pencil">Ortalama</dt>
      <dd class="annot tnum text-right">{stats.mean.toFixed(1)}</dd>
      <dt class="pencil">Standart sapma</dt>
      <dd class="annot tnum text-right">{stats.sd.toFixed(1)}</dd>
      <dt class="pencil">Çarpıklık</dt>
      <dd class="annot tnum text-right">
        {stats.skewness === null ? "—" : stats.skewness.toFixed(2)}
      </dd>
    </dl>

    <p class="annot mt-quarter">{skewLabel(stats.skewness)}</p>

    {#if stats.mode === null}
      <p class="pencil mt-quarter">
        Tepe noktası yok: öğrenciler ayrı ayrı aralıklara düşmüş, hiçbir puan
        aralığında yığılma oluşmamış.
      </p>
    {/if}

    {#if stats.n < MIN_CARPIKLIK_N}
      <p class="pencil mt-quarter">
        Çarpıklık {stats.n} öğrenciyle oynak: tek bir çok düşük ya da çok
        yüksek not katsayıyı savurur. {MIN_CARPIKLIK_N} kişiden itibaren
        yorumlanabilir hâle gelir.
      </p>
    {/if}
  {/if}
</figure>
