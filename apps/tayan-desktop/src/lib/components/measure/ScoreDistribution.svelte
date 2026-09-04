<script lang="ts">
  /**
   * Puan dağılımı — ölçme-değerlendirmenin standart görünümü.
   *
   * FREKANS DİKEY, PUAN YATAY. Öğretmen bu grafiği kendi alanının diliyle
   * okuyor: mod, medyan ve ortalamanın birbirine göre yeri sınıfın durumunu
   * söylüyor. Mod > Medyan > Ortalama sola çarpıktır ve sınıf başarılıdır;
   * ters sıra sınıfın zorlandığını gösterir.
   *
   * D3 YALNIZ HESAP İÇİN. Çizimi Svelte yapıyor; D3'ten yalnız DOM'a
   * dokunmayan ölçek ve tik hesapları alınıyor. D3'ün kendi belgesi de bunu
   * söylüyor: "you can use Svelte exclusively for rendering if you like, and
   * only use D3 modules that don't manipulate the DOM." İkisinin aynı düğümü
   * yönetmesi, bu oturumda çift imleç sorununu yaratan hatanın aynısı olurdu.
   *
   * ÇUBUKLARIN ALTINDA HAM NOKTALAR VAR. Binleme küçük sınıfta yanıltır:
   * altı öğrencide her aralığa bir kişi düşer ve grafik veriyi değil aralık
   * genişliğini gösterir. Nokta şeridi her öğrenciyi kendi puanında gösterir;
   * iki görünüm birbirinin yerine değil, birbirinin denetimi.
   */
  // `d3` demetinden içe aktarılıyor; d3-scale ayrı bir paket olarak kurulu
  // değil. Vite yalnız kullanılan parçaları paketliyor.
  import { scaleLinear } from "d3";
  import { BIN_WIDTH, histogram, skewLabel, type Spread } from "$lib/analysis/item-stats";

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

  // Görünüm kutusu birimleri. SVG ölçeklenebilir olduğu için piksel değil
  // kendi birimimizle çiziyoruz; panel daralınca grafik de daralıyor.
  const W = 320;
  const H = 130;
  const SERIT = 18;
  const SOL = 22;
  const ALT = 14;

  let bins = $derived(histogram(percentages));
  let enYuksek = $derived(Math.max(1, ...bins.map((b) => b.count)));

  // Tip parametreleri şart: ticks() aksi hâlde unknown döndürüyor ve
  // eksen etiketleri yazılamıyor.
  let x = $derived(scaleLinear<number, number>().domain([0, 100]).range([SOL, W]));
  let y = $derived(scaleLinear<number, number>().domain([0, enYuksek]).range([H - ALT, 0]));

  /**
   * Frekans tikleri TAM SAYI olmalı — "2.5 öğrenci" diye bir şey yok.
   * d3'ün ticks() önerisini tam sayıya süzüyoruz.
   */
  let yTikler = $derived(
    Array.from(new Set(y.ticks(Math.min(enYuksek, 4)).map(Math.round))).filter(
      (t) => t <= enYuksek,
    ),
  );

  let xTikler = $derived(x.ticks(5));

  /** Mod/medyan/ortalama çizgileri. Etiketler farklı yükseklikte: çakışsalar
   *  bile hangisinin nerede olduğu okunabilmeli. */
  let isaretler = $derived(
    stats === null
      ? []
      : [
          { ad: "Mod", deger: stats.mode, dy: 10 },
          { ad: "Medyan", deger: stats.median, dy: 24 },
          { ad: "Ort", deger: stats.mean, dy: 38 },
        ],
  );
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
      aria-label="Puan dağılımı: yatay eksen puan, dikey eksen frekans"
    >
      <!-- Eksenler. Recessive: veriyi değil çerçeveyi çiziyorlar. -->
      <line x1={SOL} y1={0} x2={SOL} y2={H - ALT} stroke="var(--color-rule-strong)" />
      <line
        x1={SOL}
        y1={H - ALT}
        x2={W}
        y2={H - ALT}
        stroke="var(--color-rule-strong)"
      />

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

      <!-- Çubuklar. Aralarında kâğıt boşluğu var; bitişik çubuklar tek parça
           gibi okunur ve öğretmen frekansı yanlış sayar. -->
      {#each bins as b (b.from)}
        {#if b.count > 0}
          <rect
            x={x(b.from) + 1}
            y={y(b.count)}
            width={Math.max(x(b.to) - x(b.from) - 2, 1)}
            height={y(0) - y(b.count)}
            class="fill-ink"
          >
            <title>{b.from}–{b.to} puan: {b.count} öğrenci</title>
          </rect>
        {/if}
      {/each}

      <!-- Geçme eşiği. -->
      <line
        x1={x(threshold)}
        y1={0}
        x2={x(threshold)}
        y2={H - ALT}
        stroke="var(--color-red)"
        stroke-dasharray="3 3"
      >
        <title>Geçme eşiği %{threshold}</title>
      </line>

      {#each isaretler as m (m.ad)}
        <line
          x1={x(m.deger)}
          y1={0}
          x2={x(m.deger)}
          y2={H - ALT}
          stroke="var(--color-red-deep)"
        >
          <title>{m.ad}: %{m.deger.toFixed(1)}</title>
        </line>
        <text
          x={x(m.deger) + 3}
          y={m.dy}
          class="fill-red-deep"
          style="font-size: 8px"
        >
          {m.ad}
        </text>
      {/each}

      <!-- Ham puanlar: her öğrenci bir nokta, binleme yok. -->
      {#each percentages as p, i (i)}
        <circle
          cx={x(Math.min(Math.max(p, 0), 100))}
          cy={H - ALT + 8}
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
          text-anchor="middle"
          class="fill-ink-mid"
          style="font-size: 8px; font-variant-numeric: tabular-nums"
        >
          {t}
        </text>
      {/each}
    </svg>

    <p class="pencil mt-quarter">
      Yatay: puan · Dikey: frekans ({BIN_WIDTH} puanlık aralıklar). Kesikli
      çizgi geçme eşiği; koyu çizgiler mod, medyan ve ortalama. Alttaki
      noktalar tek tek öğrenciler.
    </p>

    <dl class="ruled-top mt-quarter grid grid-cols-[auto_1fr] gap-x-half">
      <dt class="pencil">Mod</dt>
      <dd class="annot tnum text-right">{stats.mode.toFixed(0)}</dd>
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

    {#if stats.n < MIN_CARPIKLIK_N}
      <p class="pencil mt-quarter">
        Çarpıklık {stats.n} öğrenciyle oynak: tek bir çok düşük ya da çok
        yüksek not katsayıyı savurur. {MIN_CARPIKLIK_N} kişiden itibaren
        yorumlanabilir hâle gelir.
      </p>
    {/if}
  {/if}
</figure>
