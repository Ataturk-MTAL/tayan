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
  /*
    `curveMonotoneX`, `curveBasis` DEĞİL. `curveBasis` yaklaşık bir B-spline:
    örneklenen noktalardan GEÇMEZ, onları çeker. Zaten aşırı düzleşmiş bir
    eğriyi bir kez daha yumuşatıyordu. `curveMonotoneX` ızgara üzerinde
    örneklenmiş bir fonksiyon için doğru seçim — noktalardan geçer ve
    aşım (overshoot) üretmez, yani yoğunluk sahte biçimde eksiye inmez.
  */
  import {
    area,
    brushX as d3brushX,
    curveMonotoneX,
    line,
    scaleLinear,
    select as d3select,
    type D3BrushEvent,
  } from "d3";
  import {
    clearSelection,
    hoverStudent,
    selectStudents,
    selection,
  } from "$lib/ui/analysis-selection.svelte";
  import {
    BIN_WIDTH,
    densityCurve,
    MIN_CURVE_N,
    MIN_SKEWNESS_N,
    skewLabel,
    type Spread,
  } from "$lib/analysis/item-stats";

  type Props = {
    /** Öğrenci başına yüzde. */
    percentages: number[];
    /**
     * Puanların ait olduğu öğrenci kimlikleri — `percentages` ile AYNI SIRADA.
     *
     * Fırçalanan aralığın hangi öğrencilere karşılık geldiğini bulmanın tek
     * yolu bu. Verilmezse fırça çizilmiyor: seçim üretemeyecek bir etkileşimi
     * göstermek, tıklayınca hiçbir şey olmayan bir düğme koymak olurdu.
     */
    studentIds?: string[];
    /** Balonda öğrencinin adını gösterebilmek için — kimlik → "101 Ayşe YILMAZ". */
    studentLabel?: (studentId: string) => string;
    stats: Spread | null;
    /** Geçme eşiği — yatay eksenin anlam ortası. */
    threshold?: number;
  };

  let {
    percentages,
    studentIds = [],
    studentLabel,
    stats,
    threshold = 50,
  }: Props = $props();

  let figureEl = $state<HTMLElement | null>(null);

  /** Balonun konumu ve içeriği. `null` = balon yok. */
  let balon = $state<{ x: number; y: number; pct: number; studentId: string } | null>(null);

  /**
   * Balon konumu FİGÜRE GÖRE. SVG ölçekleniyor; viewBox koordinatını piksele
   * çevirmek kart genişliği değiştikçe yeniden hesap ister. İmleç olayı zaten
   * ekran koordinatı taşıyor.
   */
  function balonGoster(event: PointerEvent, pct: number, studentId: string) {
    const kap = figureEl?.getBoundingClientRect();
    if (!kap) return;
    balon = { x: event.clientX - kap.left, y: event.clientY - kap.top, pct, studentId };
    hoverStudent(studentId);
  }

  function balonGizle() {
    balon = null;
    hoverStudent(null);
  }

  /** Tek öğrenciyi seçime al/çıkar. */
  function ogrenciSec(id: string) {
    const var_ = selection.studentIds.includes(id);
    selectStudents(
      var_ ? selection.studentIds.filter((x) => x !== id) : [...selection.studentIds, id],
      KAYNAK,
    );
  }

  const KAYNAK = "puan aralığı";

  let brushHost = $state<SVGGElement | null>(null);
  let canBrush = $derived(studentIds.length === percentages.length && percentages.length > 0);

  /**
   * PUAN ARALIĞI FIRÇASI — "bu aralıktakiler kimler?"
   *
   * Dağılımın gösterdiği şey bir küme; öğretmenin sorusu ise hep bireysel:
   * "eşiğin hemen altındaki şu yığın kim?". Aralığı fırçalayınca cevap ızgarası
   * o öğrencilere vurguya geçiyor ve soru soru neyi kaçırdıkları görünüyor.
   *
   * Tek eksende (`brushX`): dikey eksen frekans, oradan bir aralık seçmenin
   * anlamı yok.
   */
  $effect(() => {
    const host = brushHost;
    if (!host || !canBrush) return;

    const b = d3brushX<unknown>()
      .extent([
        [LEFT, TOP],
        [W, plotH - BOTTOM],
      ])
      .filter((event: MouseEvent) => !event.button)
      .on("end", (event: D3BrushEvent<unknown>) => {
        if (!event.selection) {
          if (selection.source === KAYNAK) clearSelection();
          return;
        }
        const [x0, x1] = event.selection as [number, number];
        const alt = x.invert(x0);
        const ust = x.invert(x1);
        const secilen = studentIds.filter((_, i) => {
          const p = percentages[i];
          return p >= alt && p <= ust;
        });
        selectStudents(secilen, KAYNAK);
      });

    const sec = d3select(host);
    sec.call(b);

    return () => {
      sec.on(".brush", null);
      sec.selectAll("*").remove();
    };
  });

  /**
   * Bu puanın üstündeki işaret çizgilerinin etiketi çizginin SOLUNA geçer.
   * Etiket her zaman çizginin sağına, `text-anchor` varsayılanı (`start`) ile
   * konuyordu; dış <svg> öğesinde tarayıcı varsayılanı `overflow: hidden`
   * olduğu için viewBox dışına taşan yazı çizilmiyordu — kırpılma sessiz,
   * hata vermiyor. Ölçek yalnız 0–100 aralığını kapsıyor, sağda etiket için
   * pay yok: x(d) = 22 + 2.98d ve 8px'te "Medyan" 31px, yani d > 88.6'da
   * viewBox'ın 320 genişliğini aşıyor ("Mod" d > 93.3, "Ort" d > 94.3).
   * Yüksek başarılı bir sınıfta üç etiketin üçü birden gidebiliyordu.
   * 85 eşiği en geniş etiket olan "Medyan"a pay bırakır; dy kademeleri
   * (10/24/38) aynı kaldığı için üç etiket sola dönse de üst üste binmez.
   */
  const LABEL_FLIP_THRESHOLD = 85;

  /** Etiketin işaret çizgisine uzaklığı (viewBox birimi). */
  const LABEL_GAP = 3;

  const W = 320;
  const H = 140;
  /**
   * EĞRİ YOKKEN GRAFİK BİR ŞERİDE İNER.
   *
   * 140 birimlik gövde dikey eksen için ayrılmıştı; eğri çizilmediğinde o
   * eksenin gösterecek bir şeyi yok ve nokta şeridi 126 birimlik boşluğun
   * altında 20 birimlik bir dipnot gibi kalıyordu. Oysa az öğrencili bir
   * sınıfta noktalar ASIL grafiktir: her biri bir çocuk, tam ve kayıpsız.
   */
  const STRIP_ONLY_H = 46;
  const STRIP_H = 20;
  const LEFT = 22;
  const BOTTOM = 14;
  /**
   * Üst pay — EN ÜSTTEKİ EKSEN ETİKETİ İÇİN.
   *
   * Ölçeğin üst ucu 0 idi, yani en yüksek frekans etiketi `y=0`a düşüyor ve
   * `y(t) + 3` taban çizgisiyle 8px'lik yazının üst yarısı viewBox'ın DIŞINDA
   * kalıyordu: ekranda "6" rakamının yalnız alt yarısı görünüyordu. SVG'de
   * taşan içerik hata vermez, sessizce kırpılır.
   */
  const TOP = 7;

  let showCurve = $derived(stats !== null && stats.n >= MIN_CURVE_N);
  let showSkewness = $derived(stats !== null && stats.n >= MIN_SKEWNESS_N);

  let curve = $derived(showCurve ? densityCurve(percentages, BIN_WIDTH) : []);

  /**
   * Dikey eksenin tepesi. Eğrinin tepesiyle kutunun tavanı arasında bir tutam
   * boşluk bırakılıyor; eğri tavana yapışırsa tepenin nerede olduğu görünmez.
   */
  let yMax = $derived(Math.max(1, ...curve.map((p) => p.y)) * 1.15);

  let plotH = $derived(showCurve ? H : STRIP_ONLY_H);

  let x = $derived(scaleLinear<number, number>().domain([0, 100]).range([LEFT, W]));
  let y = $derived(scaleLinear<number, number>().domain([0, yMax]).range([plotH - BOTTOM, TOP]));

  /** Frekans ekseni TAM SAYI: "2.5 öğrenci" diye bir şey yok. */
  let yTicks = $derived(
    Array.from(new Set(y.ticks(3).map(Math.round))).filter((t) => t <= yMax),
  );
  let xTicks = $derived(x.ticks(5));

  let lineGenerator = $derived(
    line<{ x: number; y: number }>()
      .x((p) => x(p.x))
      .y((p) => y(p.y))
      .curve(curveMonotoneX),
  );

  let areaGenerator = $derived(
    area<{ x: number; y: number }>()
      .x((p) => x(p.x))
      .y0(y(0))
      .y1((p) => y(p.y))
      .curve(curveMonotoneX),
  );

  /**
   * Mod null olabilir: tepe noktası yoksa çizgisi de çizilmez. Olmayan bir
   * tepeyi grafiğe koymak dağılımın yanlış okunmasına yol açardı.
   */
  let markers = $derived(
    stats === null
      ? []
      : [
          /*
            MOD ARTIK HİÇ ÇİZİLMİYOR. Ayrık değerli sürekli puanlarda örneklem
            modu tanımsız; `modeOf` onu 10 puanlık aralıklardan ÜRETİYOR ve
            değer aralık genişliğiyle birlikte oynuyor (aynı altı puan için
            bin 5/10/20/25 → null/null/70/null). Değeri bir görüntü sabitine
            bağlı olan şey istatistik değildir. `modeOf`un beraberlikte null
            dönme kararı doğruydu — ama doğru çözüm boş bırakmak değil,
            istatistiği hiç raporlamamak.
          */
          { label: "Medyan", value: stats.median, dy: 10 },
          { label: "Ort", value: stats.mean, dy: 24 },
        ],
  );

  function clamp(v: number): number {
    return Math.min(Math.max(v, 0), 100);
  }
</script>

<figure
  bind:this={figureEl}
  class="relative m-0 flex h-full flex-col rounded-lg border border-default-medium
         bg-neutral-primary-medium p-4 shadow-sm"
>
  <figcaption class="text-xs font-semibold uppercase tracking-wider text-body-subtle">
    Puan dağılımı
  </figcaption>

  {#if stats === null}
    <p class="mt-1 text-xs text-body-subtle">Sonuç girilmemiş.</p>
  {:else}
    <!--
      GENİŞLİK SINIRI ARTIK IZGARA İZİNDE, BURADA DEĞİL.

      SVG'de `max-w-[640px]` vardı; kart ondan geniş olabildiği için kartın
      içinde ölü boşluk kalıyordu. Üst sınır analiz ızgarasının iz tanımına
      taşındı (`minmax(20rem, 40rem)`), yani kart ne kadarsa SVG de o kadar.

      Sınırın kendisi hâlâ gerekli: viewBox oranı sabit olduğu için SVG
      genişledikçe TÜM iç ölçüler (8px eksen yazısı, 1.6px eğri kalınlığı)
      doğrusal büyüyor — SVG ölçeklenmesi tipografiyi ölçeklemez, ölçek DIŞINA
      çıkarır. 40rem tavanı en fazla ~1.9 kat büyümeye izin veriyor.
    -->
    <svg
      class="mt-2.5 w-full"
      viewBox="0 0 {W} {plotH + STRIP_H}"
      role="img"
      aria-label="Puan dağılım eğrisi: yatay eksen puan, dikey eksen frekans"
    >
      <!--
        Eksen ve ızgara: veri değil, okuma çerçevesi — soluk gri.
        DİKEY EKSEN YALNIZ EĞRİ VARKEN. Nokta şeridinin frekans ekseni yok;
        boş bir dikey çizgi okunacak bir ölçek varmış izlenimi verirdi.
      -->
      {#if showCurve}
        <line x1={LEFT} y1={TOP} x2={LEFT} y2={plotH - BOTTOM} class="stroke-default-medium" />
      {/if}
      <line x1={LEFT} y1={plotH - BOTTOM} x2={W} y2={plotH - BOTTOM} class="stroke-default-medium" />

      {#each showCurve ? yTicks : [] as t (t)}
        <text
          x={LEFT - 4}
          y={y(t) + 3}
          text-anchor="end"
          class="fill-body-subtle"
          style="font-size: 8px; font-variant-numeric: tabular-nums"
        >
          {t}
        </text>
      {/each}

      <!-- Ana veri: eğrinin kendisi, koyu gri/lacivert — değerlendirme değil, ölçüm. -->
      {#if curve.length > 0}
        <path d={areaGenerator(curve) ?? ""} class="fill-gray-800 dark:fill-gray-200" opacity="0.14" />
        <path d={lineGenerator(curve) ?? ""} fill="none" class="stroke-gray-800 dark:stroke-gray-200" stroke-width="1.6" />
      {/if}

      <!-- Geçme eşiği: değerlendirme çizgisi, kırmızı. -->
      <line
        x1={x(clamp(threshold))}
        y1={TOP}
        x2={x(clamp(threshold))}
        y2={plotH - BOTTOM}
        class="stroke-red-600 dark:stroke-red-400"
        stroke-dasharray="3 3"
      >
        <title>Geçme eşiği %{threshold}</title>
      </line>

      <!-- Mod/medyan/ortalama çizgileri de değerlendirme okuması: aynı kırmızı. -->
      {#each markers as m (m.label)}
        <!--
          Sağ uçtaki etiket çizginin soluna döner ve tutunma noktası `end`
          olur; böylece yazı viewBox'ın sağ kenarından taşıp kırpılmaz.
        -->
        {@const flipLeft = m.value > LABEL_FLIP_THRESHOLD}
        <line
          x1={x(clamp(m.value))}
          y1={TOP}
          x2={x(clamp(m.value))}
          y2={plotH - BOTTOM}
          class="stroke-red-600 dark:stroke-red-400"
        >
          <title>{m.label}: %{m.value.toFixed(1)}</title>
        </line>
        <text
          x={x(clamp(m.value)) + (flipLeft ? -LABEL_GAP : LABEL_GAP)}
          y={m.dy}
          text-anchor={flipLeft ? "end" : "start"}
          class="fill-red-600 dark:fill-red-400"
          style="font-size: 8px"
        >
          {m.label}
        </text>
      {/each}

      <!--
        Ham puanlar. Eğri bir KESTİRİM; noktalar gerçeğin kendisi. İkisi aynı
        eksende yan yana durunca eğrinin nerede yumuşattığı da görünüyor.
        Eşiğin altındaki nokta kırmızı — bu da bir değerlendirme okuması.
      -->
      {#each percentages as p, i (i)}
        {@const sid = studentIds[i]}
        {@const r = showCurve ? 2.5 : 4}
        {@const renk =
          p >= threshold ? "fill-gray-800 dark:fill-gray-200" : "fill-red-600 dark:fill-red-400"}
        <!--
          İKİ DAL, BİR DÜZİNE ÜÇLÜ OPERATÖR DEĞİL. `studentIds` verilmediğinde
          nokta etkileşimsiz; her özniteliği ayrı ayrı koşullamak hem okunmuyor
          hem de `role` statik olarak görünmediği için erişilebilirlik
          denetimini yanıltıyordu ("tabindex var ama etkileşimli öğe değil").
        -->
        {#if sid === undefined}
          <circle cx={x(clamp(p))} cy={plotH - BOTTOM + 9} {r} class={renk}></circle>
        {:else}
          {@const uzerinde = selection.hoveredStudentId === sid}
          <circle
            cx={x(clamp(p))}
            cy={plotH - BOTTOM + 9}
            r={uzerinde ? r + 2 : r}
            class="{renk} cursor-pointer transition-all"
            opacity={selection.hasStudent(sid) ? 1 : 0.2}
            onpointerenter={(e) => balonGoster(e, p, sid)}
            onpointermove={(e) => balonGoster(e, p, sid)}
            onpointerleave={balonGizle}
            onclick={() => ogrenciSec(sid)}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                ogrenciSec(sid);
              }
            }}
            onfocus={() => hoverStudent(sid)}
            onblur={balonGizle}
            role="button"
            tabindex="0"
            aria-label={`${studentLabel ? studentLabel(sid) : "Öğrenci"}, puan ${p.toFixed(0)}`}
            aria-pressed={selection.studentSet.has(sid)}
          ></circle>
        {/if}
      {/each}

      <!--
        FIRÇA EN ÜSTTE: kendi saydam dikdörtgeni tıklamaları yakalıyor.
        `canBrush` yoksa hiç çizilmiyor — seçim üretemeyecek bir etkileşim
        göstermek yanıltıcı olurdu.
      -->
      {#if canBrush}
        <!--
          KAPLAMA ETKİN KALMALI. Bir ara `pointer-events-none` verilmişti;
          `.overlay` d3-brush'ta tam da YENİ SEÇİMİ BAŞLATAN dikdörtgendir,
          kapatınca boş alandan fırça başlatmak imkânsız hâle geliyordu —
          grafik "hiç etkileşimli değil" görünüyordu.

          Burada çakışma da yok: nokta şeridi fırça alanının (`extent`)
          ALTINDA, eksenin dışında duruyor.
        -->
        <g
          bind:this={brushHost}
          class="[&_.selection]:fill-primary-600/15 [&_.selection]:stroke-primary-600"
        ></g>
      {/if}

      {#each xTicks as t (t)}
        <text
          x={x(t)}
          y={plotH + STRIP_H - 2}
          text-anchor={t === 0 ? "start" : t >= 100 ? "end" : "middle"}
          class="fill-body-subtle"
          style="font-size: 8px; font-variant-numeric: tabular-nums"
        >
          {t}
        </text>
      {/each}
    </svg>

    <!--
      EĞRİ YOKKEN "hesaplanamıyor" DEMEK YANLIŞTI: hesaplanabiliyordu, biz
      çizmemeye KARAR verdik. Sebebi de söylenmeli, yoksa öğretmen eksik bir
      şey olduğunu sanır. Nokta şeridi bir teselli ödülü değil — altı öğrenci
      için verinin tam ve kayıpsız gösterimi; eğri ise en iyi hâlde bir
      kestirim.
    -->
    <!--
      `pointer-events-none` ŞART: balon imlecin altında kalırsa noktanın
      `pointerleave`ini tetikler, balon kapanır, imleç tekrar noktaya düşer —
      sonsuz titreme.
    -->
    {#if balon}
      {@const solda = balon.x > (figureEl?.clientWidth ?? 0) / 2}
      <div
        class="pointer-events-none absolute z-10 max-w-[16rem] rounded-lg border
               border-default-medium bg-neutral-primary-medium px-2.5 py-1.5 text-xs shadow-lg"
        style="left: {balon.x}px; top: {balon.y}px;
               transform: translate({solda ? '-100%' : '0'}, -100%) translate({solda ? -10 : 10}px, -10px)"
      >
        <p class="font-semibold text-heading">
          {studentLabel ? studentLabel(balon.studentId) : "Öğrenci"}
        </p>
        <p class="tnum mt-0.5 {balon.pct >= threshold ? 'text-body-subtle' : 'text-red-600 dark:text-red-400'}">
          Puan %{balon.pct.toFixed(0)}{#if balon.pct < threshold} — eşiğin altında{/if}
        </p>
      </div>
    {/if}

    {#if !showCurve}
      <p class="mt-1 text-xs text-body-subtle">
        Her nokta bir öğrenci; kesikli çizgi geçme eşiği. Dağılım eğrisi
        {stats.n} öğrenciyle çizilmiyor — {MIN_CURVE_N} kişinin altında eğrinin
        şekli sınıfı değil, hesabın düzleştirme ayarını gösterir.
      </p>
    {:else}
      <p class="mt-1 text-xs text-body-subtle">
        Yatay: puan · Dikey: frekans. Kesikli çizgi geçme eşiği; koyu çizgiler
        medyan ve ortalama. Alttaki noktalar tek tek öğrenciler.
      </p>
    {/if}

    <!--
      ÖLÇÜLER YATAY. Dikey liste dar bir sütun izlenimi veriyor ve grafiğin
      altına uzun bir metin kuyruğu ekliyordu; beş sayı yan yana tek bakışta
      okunuyor ve mod/medyan/ortalama sırası da böyle görünüyor.
    -->
    <!--
      SIRA DEĞİŞTİ VE "Mod" ÇIKTI.

      Mod ayrık değerli sürekli puanlarda tanımsız; buradaki değer 10 puanlık
      aralıklardan üretiliyordu ve aralık genişliğiyle oynuyordu. Kalıcı bir
      "—" ile onun altına iliştirilmiş bir özür, öğretmene eksik bir özellik
      izlenimi veriyordu; oysa eksik olan bir şey yok, hesaplanması gereken
      bir şey yok.

      Yerine EN DÜŞÜK–EN YÜKSEK geldi. Sıra istatistikleri her n'de tanımlı,
      hiçbir varsayım gerektirmiyor ve öğretmenin gerçekten sorduğu şeye —
      sınıf ne kadar yayılmış — sd'den daha doğrudan cevap veriyor.

      Çarpıklık yalnız n ≥ 50'de. Altında sayı basmak, standart hatası
      değerin on katı olan bir büyüklüğü iki ondalıkla raporlamak olurdu.
    -->
    <dl class="mt-[5px] flex flex-wrap gap-x-5 gap-y-[5px] border-t border-default-medium pt-[5px]">
      {#each [["Öğrenci", String(stats.n)], ["En düşük – en yüksek", `${stats.min.toFixed(0)} – ${stats.max.toFixed(0)}`], ["Medyan", stats.median.toFixed(1)], ["Ortalama", stats.mean.toFixed(1)], ["Std. sapma", stats.sd.toFixed(1)], ...(showSkewness && stats.skewness !== null ? [["Çarpıklık", stats.skewness.toFixed(2)]] : [])] as [label, value] (label)}
        <div>
          <dt class="text-xs text-body-subtle">{label}</dt>
          <dd class="tnum text-[15px] leading-5 text-heading">{value}</dd>
        </div>
      {/each}
    </dl>

    <!--
      Çarpıklık cümlesi de aynı kapının arkasında. Eskiden n'den bağımsız
      basılıyor, altına "bu sayı oynak" diye bir dipnot ekleniyordu — ama
      öğretmen CÜMLEYİ okuyor, dipnotu değil. n=6'da −0.06'nın %95 aralığı
      `skewLabel`in üretebildiği bütün etiketleri kapsıyordu: yani ekranda
      kesin kipte yazan cümle, rastgele bir etiketti.
    -->
    {#if showSkewness}
      <p class="mt-[5px] text-xs leading-5 text-body">{skewLabel(stats.skewness)}</p>
    {/if}
  {/if}
</figure>
