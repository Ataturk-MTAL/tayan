<script lang="ts">
  /**
   * Puan dağılımı — ölçme-değerlendirmenin standart görünümü.
   *
   * FREKANS DİKEY, PUAN YATAY. Öğretmen bu grafiği kendi alanının diliyle
   * okuyor: mod, medyan ve ortalamanın birbirine göre yeri sınıfın durumunu
   * söylüyor. Mod > Medyan > Ortalama ise dağılım sola çarpıktır ve sınıf
   * başarılıdır; ters sıra sınıfın zorlandığını gösterir.
   *
   * ÇUBUKLARIN ALTINDA HAM NOKTALAR VAR. Binleme küçük sınıfta yanıltır:
   * altı öğrencide her aralığa bir kişi düşer ve grafik veriyi değil aralık
   * genişliğini gösterir. Alttaki nokta şeridi her öğrenciyi kendi puanında
   * gösterir; çubukların gizlediği yayılım orada görünür.
   */
  import { BIN_WIDTH, histogram, skewLabel, type Spread } from "$lib/analysis/item-stats";

  type Props = {
    /** Öğrenci başına yüzde. */
    percentages: number[];
    stats: Spread | null;
    /** Geçme eşiği — yatay eksenin anlam ortası. */
    threshold?: number;
  };

  let { percentages, stats, threshold = 50 }: Props = $props();

  const YUKSEKLIK = 120;
  const SERIT = 16;
  /** Çarpıklığın güvenilir okunabildiği en küçük sınıf. */
  const MIN_CARPIKLIK_N = 15;

  let bins = $derived(histogram(percentages));
  let enYuksek = $derived(Math.max(1, ...bins.map((b) => b.count)));

  /** Frekans ekseni tam sayı olmalı; kesirli tik "yarım öğrenci" demek. */
  let tikler = $derived(
    enYuksek <= 4
      ? Array.from({ length: enYuksek + 1 }, (_, i) => enYuksek - i)
      : [enYuksek, Math.round(enYuksek / 2), 0],
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
    <div class="mt-half flex gap-quarter">
      <div
        class="flex w-[1.6rem] shrink-0 flex-col justify-between text-right"
        style="height: {YUKSEKLIK}px"
      >
        {#each tikler as t (t)}
          <span class="pencil tnum leading-none">{t}</span>
        {/each}
      </div>

      <div class="min-w-0 flex-1">
        <div
          class="relative border-b border-l border-rule-strong"
          style="height: {YUKSEKLIK}px"
        >
          <div
            class="absolute inset-y-0 border-l border-dashed border-red"
            style="left: {kirp(threshold)}%"
            title="Geçme eşiği %{threshold}"
          ></div>

          <!--
            Çubuklar arasında kâğıt boşluğu var; bitişik çubuklar tek parça
            gibi okunur ve öğretmen frekansı yanlış sayar.
          -->
          <div class="absolute inset-0 flex items-end gap-[2px]">
            {#each bins as b (b.from)}
              <div
                class="flex-1 bg-ink"
                class:opacity-0={b.count === 0}
                style="height: {(b.count / enYuksek) * 100}%"
                title="{b.from}–{b.to} puan: {b.count} öğrenci"
              ></div>
            {/each}
          </div>

          {#each [["mod", stats.mode, "Mod"], ["medyan", stats.median, "Medyan"], ["ort", stats.mean, "Ortalama"]] as [id, deger, ad] (id)}
            <div
              class="absolute inset-y-0 border-l border-red-deep"
              style="left: {kirp(Number(deger))}%"
              title="{ad}: %{Number(deger).toFixed(1)}"
            ></div>
          {/each}
        </div>

        <!-- Ham puanlar: her öğrenci bir nokta, binleme yok. -->
        <div class="relative" style="height: {SERIT}px">
          {#each percentages as p, i (i)}
            <span
              class="absolute top-[4px] h-[6px] w-[6px] rounded-full"
              class:bg-ink={p >= threshold}
              class:bg-red={p < threshold}
              style="left: {kirp(p)}%; margin-left: -3px"
              title="%{p.toFixed(0)}"
            ></span>
          {/each}
        </div>

        <div class="flex justify-between">
          {#each [0, 20, 40, 60, 80, 100] as t (t)}
            <span class="pencil tnum">{t}</span>
          {/each}
        </div>
      </div>
    </div>

    <p class="pencil mt-quarter">
      Yatay: puan · Dikey: frekans ({BIN_WIDTH} puanlık aralıklar). Kesikli
      çizgi geçme eşiği; koyu dikey çizgiler mod, medyan ve ortalama.
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
