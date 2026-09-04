<script lang="ts">
  /**
   * Puan yayılımı — her öğrenci bir nokta.
   *
   * HİSTOGRAMIN YERİNE GEÇTİ. Altı öğrenci ve 20 puanlık kutularla her kutuda
   * bir kişi çıkıyordu: grafik veriyi değil kutu genişliğini gösteriyordu.
   * Sınıf mevcudu 30 olsa da bu sürer — binleme küçük örneklemde yayılımı
   * gizler, üstelik kutu sınırı kaydıkça şekil değişir.
   *
   * Nokta grafiği bunu yapmaz: her öğrenci kendi yerinde durur, kümelenme ve
   * boşluk doğrudan görünür. Aynı puanı alanlar üst üste binmesin diye dikey
   * olarak kaydırılıyor (arı kovanı dizilimi).
   */
  import type { Spread } from "$lib/analysis/item-stats";

  type Props = {
    /** Öğrenci başına yüzde. */
    percentages: number[];
    stats: Spread | null;
    /** Geçme eşiği, yüzde. */
    threshold?: number;
  };

  let { percentages, stats, threshold = 50 }: Props = $props();

  const YUKSEKLIK = 96;
  const NOKTA = 7;
  /** Aynı sütuna düşen noktalar arasındaki dikey adım. */
  const KATMAN = 9;
  /** Yakınlık eşiği: bu kadar yüzde içindeki puanlar aynı yığın sayılır. */
  const YIGIN_ARALIGI = 2.5;

  /**
   * Noktaları yerleştirir. Aynı yerdekiler yukarı doğru istiflenir, böylece
   * altı öğrencinin de altısı görünür — üst üste binen nokta öğrenci kaybeder.
   */
  let noktalar = $derived.by(() => {
    const sirali = percentages
      .map((p, i) => ({ p, i }))
      .sort((a, b) => a.p - b.p);

    const yigin: number[] = [];
    return sirali.map(({ p, i }) => {
      let katman = 0;
      for (const oncekiP of yigin) {
        if (Math.abs(oncekiP - p) < YIGIN_ARALIGI) katman += 1;
      }
      yigin.push(p);
      return { p, i, katman };
    });
  });
</script>

<figure class="m-0">
  <figcaption class="stamp">Puan yayılımı</figcaption>

  {#if stats === null}
    <p class="pencil mt-quarter">Sonuç girilmemiş.</p>
  {:else}
    <div class="relative mt-half" style="height: {YUKSEKLIK}px">
      <!--
        Çeyrekler arası kutu: öğrencilerin ortadaki yarısı burada. Sınıfın
        "çoğunluğu nerede" sorusunun cevabı, ortalamadan daha sağlam — tek bir
        çok düşük not ortalamayı aşağı çeker, kutuyu çekmez.
      -->
      <div
        class="absolute inset-y-0 bg-ink/[0.06]"
        style="left: {stats.q1}%; width: {Math.max(stats.q3 - stats.q1, 0.4)}%"
        title="Ortadaki yarı: %{stats.q1.toFixed(0)} – %{stats.q3.toFixed(0)}"
      ></div>

      <!-- Geçme eşiği: kesikli. -->
      <div
        class="absolute inset-y-0 border-l border-dashed border-red"
        style="left: {threshold}%"
        title="Geçme eşiği %{threshold}"
      ></div>

      <!-- Sınıf ortalaması: kesiksiz. -->
      <div
        class="absolute inset-y-0 border-l border-red"
        style="left: {stats.mean}%"
        title="Sınıf ortalaması %{stats.mean.toFixed(0)}"
      ></div>

      {#each noktalar as n (n.i)}
        <span
          class="absolute rounded-full"
          class:bg-ink={n.p >= threshold}
          class:bg-red={n.p < threshold}
          style="
            left: {n.p}%;
            bottom: {n.katman * KATMAN}px;
            width: {NOKTA}px;
            height: {NOKTA}px;
            margin-left: {-NOKTA / 2}px;
          "
          title="%{n.p.toFixed(0)}"
        ></span>
      {/each}
    </div>

    <div class="ruled-top mt-quarter flex justify-between">
      {#each [0, 20, 40, 60, 80, 100] as tik (tik)}
        <span class="pencil tnum">{tik}</span>
      {/each}
    </div>

    <p class="annot mt-quarter">
      {stats.n} öğrenci. En düşük <b class="tnum">{stats.min.toFixed(0)}</b>,
      en yüksek <b class="tnum">{stats.max.toFixed(0)}</b>; ortadaki yarı
      <b class="tnum">{stats.q1.toFixed(0)}–{stats.q3.toFixed(0)}</b> arasında.
      Kesiksiz çizgi ortalama (<span class="tnum">{stats.mean.toFixed(0)}</span>),
      kesikli çizgi geçme eşiği (<span class="tnum">{threshold}</span>).
      Eşiğin altındaki noktalar kırmızı.
    </p>
  {/if}
</figure>
