<script lang="ts">
  /**
   * Madde haritası — güçlük × ayırt edicilik saçılımı.
   *
   * ÖĞRETMENİN SORUSU: "Hangi soruyu atmalıyım, hangisini düzeltmeliyim?"
   * Tablo bu soruyu cevaplıyor ama satır satır okumak gerekiyor; 20 soruda göz
   * iki sütun arasında gidip geliyor ve sorunlu maddeler öbek hâlinde
   * görünmüyor. Saçılımda her soru bir nokta ve sorunlular KENDİ BÖLGELERİNDE
   * toplanıyor — tek bakışta okunuyor.
   *
   * İKİ EKSEN, ÇÜNKÜ İKİ AYRI ÖLÇÜ. Güçlük ve ayırt edicilik bağımsız: çok
   * kolay bir soru da çok zor bir soru da ayırt etmez, ama sebepleri ve
   * çözümleri farklıdır. İkisini tek çubuğa bindirmek madde analizinde en sık
   * yapılan grafik hatasıdır.
   *
   * DİKEY BANTLAR Ebel'in yerleşik eşikleri (Türkçe ölçme kitaplarında aynı):
   * D ≥ 0.40 çok iyi, 0.30–0.39 iyi, 0.20–0.29 düzeltilmeli, 0.20 altı zayıf.
   * Sıfırın ALTI ayrı bir bölge: orada madde ters ayırıyor, yani iyi öğrenciler
   * yanlış yapmış — teşhis "zayıf" değil, "anahtar şüpheli".
   *
   * YATAY BANTLAR güçlüğün yerleşik yorumu: p ≤ 0.30 zor, 0.30–0.70 orta,
   * p ≥ 0.70 kolay.
   */
  import {
    Delaunay,
    brush as d3brush,
    select as d3select,
    zoom as d3zoom,
    zoomIdentity,
    type D3BrushEvent,
    type D3ZoomEvent,
    type ZoomTransform,
  } from "d3";
  import { scaleLinear } from "d3";
  import {
    clearSelection,
    hoverQuestion,
    selectQuestions,
    selection,
    toggleQuestion,
  } from "$lib/ui/analysis-selection.svelte";
  import {
    MIN_DISCRIMINATION_N,
    REVERSE_DISCRIMINATION,
    type ItemStat,
  } from "$lib/analysis/item-stats";

  type Props = {
    items: ItemStat[];
    /** Balonda sorunun adını gösterebilmek için — kimlik → kısa ad. */
    label?: (questionId: string) => string;
    /** Sonuç girilen öğrenci sayısı — ayırt ediciliğin anlamlı olup olmadığı. */
    studentCount: number;
  };

  let { items, studentCount, label }: Props = $props();

  const W = 320;
  const H = 210;
  const LEFT = 26;
  const RIGHT = 8;
  const TOP = 8;
  const BOTTOM = 26;

  /** Ayırt ediciliğin "çok iyi" sayıldığı eşik — Ebel. */
  const GOOD_D = 0.4;
  /** Kabul edilebilirliğin alt sınırı. */
  const WEAK_D = 0.2;

  /** Güçlüğün zor/orta/kolay sınırları. */
  const P_ZOR = 0.3;
  const P_KOLAY = 0.7;

  let hasDiscrimination = $derived(
    studentCount >= MIN_DISCRIMINATION_N && items.some((i) => i.discrimination !== null),
  );

  /**
   * Dikey eksenin alt ucu.
   *
   * TERS BÖLGE HER ZAMAN GÖRÜNÜR: en düşük D pozitif olsa bile eksen −0.3'e
   * kadar iniyor. Aksi hâlde ölçek veriye göre kayar ve "bu sınavda ters madde
   * yok" bilgisi — ki değerli bir bilgidir — grafikten okunamaz olurdu.
   */
  let dMin = $derived(Math.min(-0.3, ...items.map((i) => i.discrimination ?? 0)) - 0.05);

  /**
   * ZOOM DÖNÜŞÜMÜ.
   *
   * Ölçekler dönüştürülüyor, SVG'ye `transform` verilmiyor: grup ölçeklenseydi
   * nokta yarıçapları, yazı boyutları ve çizgi kalınlıkları da büyürdü —
   * yakınlaşmak yerine "zoomlanmış görüntü" olurdu. Ölçek dönüştürülünce
   * yalnız KONUMLAR değişiyor, tipografi sabit kalıyor.
   */
  let zoomT = $state<ZoomTransform>(zoomIdentity);

  let x0 = $derived(scaleLinear<number, number>().domain([0, 1]).range([LEFT, W - RIGHT]));
  let y0 = $derived(scaleLinear<number, number>().domain([dMin, 1]).range([H - BOTTOM, TOP]));

  let x = $derived(zoomT.rescaleX(x0));
  let y = $derived(zoomT.rescaleY(y0));

  let zoomlu = $derived(zoomT.k !== 1 || zoomT.x !== 0 || zoomT.y !== 0);

  type Durum = "ters" | "zayif" | "saglikli";

  let noktalar = $derived(
    items
      .filter((i) => i.discrimination !== null)
      .map((i) => {
        const d = i.discrimination as number;
        const durum: Durum =
          d <= REVERSE_DISCRIMINATION ? "ters" : d < WEAK_D ? "zayif" : "saglikli";
        return { questionId: i.questionId, order: i.order, p: i.difficulty, d, durum };
      }),
  );

  /*
    RENK KODU sayfanın kuralına uyuyor: kırmızı YALNIZ değerlendirme bulgusu.
    Ters ayıran madde gerçek bir bulgudur, orada kırmızı yerinde. Zayıf madde
    bir uyarı — amber; aynı ölçek "sonucu daha önce girilmiş" uyarısında da
    kullanılıyor. Sağlıklı madde nötr.
  */
  const RENK: Record<Durum, string> = {
    ters: "fill-red-600 dark:fill-red-400",
    zayif: "fill-amber-500 dark:fill-amber-400",
    saglikli: "fill-gray-700 dark:fill-gray-300",
  };

  /** Nokta yarıçapı (viewBox birimi). */
  const R = 6;

  /**
   * Nokta konumları — ETİKET KAYDIRMA YOK.
   *
   * Önceden çakışan etiketler en yakın boş yöne kaydırılıyor ve araya
   * bağlayıcı çizgi çekiliyordu. İki sorunu vardı:
   *
   *   1. Etiket dairesinin İÇİNDE oturmuyordu; grafik "sayılar ve baloncuklar
   *      birbirinden kopmuş" görünüyordu.
   *   2. Yerleşim EKRAN KOORDİNATINDA hesaplandığı için her zoom adımında
   *      yeniden çalışıyordu: daire düzgün kayarken etiket bir anda başka bir
   *      aday yöne SIÇRIYORDU. Kullanıcının gördüğü "zoomda senkron hareket
   *      edemiyor" tam olarak buydu.
   *
   * Çözüm etiketi oynatmak değil, ETİKET SAYISINI AZALTMAK: yalnız sorunlu
   * maddeler ve imlecin üstündeki madde numara taşıyor. Sağlıklı maddeler
   * çıplak nokta; kimliğini imleç balonu söylüyor. Grafiğin işi zaten
   * "hangileri sorunlu" — 20 numarayı birden okutmak o işe hizmet etmiyordu.
   */
  let yerlesim = $derived(
    noktalar.map((n) => ({ ...n, cx: x(n.p), cy: y(n.d) })),
  );

  /**
   * En yakın nokta arayıcı.
   *
   * `Delaunay.find` imlecin en yakınındaki noktayı SABİT ZAMANDA buluyor —
   * her noktaya ayrı ayrı işaretçi olayı bağlamaya gerek kalmıyor. Asıl
   * kazanç bu değil ama: fırçanın kaplaması bütün alanı yakaladığı için
   * noktalara doğrudan `pointerenter` ZATEN ulaşmıyordu. Delaunay ile imleç
   * boşlukta gezerken bile en yakın madde vurgulanıyor.
   */
  let delaunay = $derived(
    yerlesim.length > 0
      ? Delaunay.from(yerlesim.map((n) => [n.cx, n.cy] as [number, number]))
      : null,
  );

  /** İmlecin bir noktaya "yakın" sayıldığı en büyük uzaklık (viewBox birimi). */
  const HOVER_MESAFE = 18;

  let figureEl = $state<HTMLElement | null>(null);
  let svgEl = $state<SVGSVGElement | null>(null);
  let zoomHost = $state<SVGRectElement | null>(null);

  function zoomSifirla() {
    zoomT = zoomIdentity;
    const host = zoomHost;
    if (host) d3select(host).call(d3zoom<SVGRectElement, unknown>().transform, zoomIdentity);
  }

  /** Balonun konumu ve içeriği. `null` = balon yok. */
  let balon = $state<{
    x: number;
    y: number;
    order: number;
    p: number;
    d: number;
    questionId: string;
  } | null>(null);

  /**
   * Ekran koordinatını viewBox koordinatına çevirir.
   *
   * `getScreenCTM().inverse()` SVG'nin KENDİ dönüşüm matrisini kullanıyor:
   * kart genişliği, viewBox oranı, sayfa kaydırması — hepsi hesaba katılmış
   * oluyor. Oranı elle hesaplamak (clientWidth / viewBox genişliği) kart her
   * yeniden boyutlandığında yanlışa düşerdi.
   */
  function viewBoxKoordinat(event: PointerEvent): [number, number] | null {
    const ctm = svgEl?.getScreenCTM();
    if (!ctm) return null;
    const nokta = new DOMPoint(event.clientX, event.clientY).matrixTransform(ctm.inverse());
    return [nokta.x, nokta.y];
  }

  /**
   * İmleç hareketi — EN YAKIN maddeyi Delaunay ile bulur.
   *
   * Noktalara tek tek `pointerenter` bağlamak burada ÇALIŞMIYOR: fırçanın
   * kaplaması bütün alanı yakalıyor ve olaylar noktalara hiç ulaşmıyor.
   * Kaplamayı geçirgen yapmak da fırçayı bozuyordu (yeni seçim başlatan
   * dikdörtgen odur). Delaunay ikisini birden çözüyor — üstelik imlecin tam
   * noktanın üstünde olması da gerekmiyor, yakınında olması yetiyor.
   */
  function imlecHareket(event: PointerEvent) {
    const koordinat = viewBoxKoordinat(event);
    const kap = figureEl?.getBoundingClientRect();
    if (!koordinat || !delaunay || !kap) return;

    const [px, py] = koordinat;
    const i = delaunay.find(px, py);
    const n = yerlesim[i];
    if (!n || Math.hypot(n.cx - px, n.cy - py) > HOVER_MESAFE) {
      imlecCik();
      return;
    }

    balon = {
      x: event.clientX - kap.left,
      y: event.clientY - kap.top,
      order: n.order,
      p: n.p,
      d: n.d,
      questionId: n.questionId,
    };
    hoverQuestion(n.questionId);
  }

  function imlecCik() {
    balon = null;
    hoverQuestion(null);
  }

  const KAYNAK = "madde haritası";

  let brushHost = $state<SVGGElement | null>(null);

  /**
   * FIRÇA — "şu bölgedeki sorular" demenin en doğrudan yolu.
   *
   * ZOOMLA DOĞRU ÇALIŞIYOR ÇÜNKÜ KARŞILAŞTIRMA EKRAN KOORDİNATINDA.
   * Fırça viewBox koordinatı veriyor; `yerlesim` içindeki `cx`/`cy` de
   * zoom'la dönüştürülmüş ölçeklerden geliyor. İkisi aynı uzayda, dolayısıyla
   * yakınlaştırılmış görünümde çevrelenen bölge tam da ekranda görünen
   * noktaları seçiyor. Veri uzayında (p, D) karşılaştırsaydık zoom'un ters
   * dönüşümünü elle uygulamak gerekirdi ve bir işaret hatası sessizce yanlış
   * soruları seçerdi.
   */
  $effect(() => {
    const host = brushHost;
    if (!host || !hasDiscrimination) return;

    const b = d3brush<unknown>()
      .extent([
        [LEFT, TOP],
        [W - RIGHT, H - BOTTOM],
      ])
      .filter((event: MouseEvent) => !event.button && !event.shiftKey)
      .on("end", (event: D3BrushEvent<unknown>) => {
        if (!event.selection) {
          if (selection.source === KAYNAK) clearSelection();
          return;
        }
        const [[x0, y0], [x1, y1]] = event.selection as [[number, number], [number, number]];
        selectQuestions(
          yerlesim
            .filter((n) => n.cx >= x0 && n.cx <= x1 && n.cy >= y0 && n.cy <= y1)
            .map((n) => n.questionId),
          KAYNAK,
        );
      });

    const sec = d3select(host);
    sec.call(b);

    return () => {
      sec.on(".brush", null);
      sec.selectAll("*").remove();
    };
  });

  /**
   * ZOOM — tekerlek yakınlaştırır, Shift+sürükle kaydırır.
   *
   * FIRÇAYLA BÖLÜŞÜM NET: düz sürükleme SEÇİM (asıl iş), tekerlek ZOOM,
   * Shift+sürükle KAYDIRMA. Fırçanın `filter`ı da `!event.shiftKey` diyor,
   * yani Shift basılıyken fırça hiç başlamıyor — iki davranış aynı sürüklemeyi
   * paylaşmıyor.
   */
  $effect(() => {
    const host = zoomHost;
    if (!host || !hasDiscrimination) return;

    const z = d3zoom<SVGRectElement, unknown>()
      .scaleExtent([1, 8])
      .translateExtent([
        [LEFT, TOP],
        [W - RIGHT, H - BOTTOM],
      ])
      .filter((event: WheelEvent | MouseEvent) => {
        if (event.type === "wheel") return true;
        return event.shiftKey && !(event as MouseEvent).button;
      })
      .on("zoom", (event: D3ZoomEvent<SVGRectElement, unknown>) => {
        zoomT = event.transform;
      });

    d3select(host).call(z);
    return () => {
      d3select(host).on(".zoom", null);
    };
  });

  let sayim = $derived({
    ters: noktalar.filter((n) => n.durum === "ters").length,
    zayif: noktalar.filter((n) => n.durum === "zayif").length,
  });
</script>

<figure
  bind:this={figureEl}
  class="relative m-0 flex h-full flex-col rounded-lg border border-default-medium
         bg-neutral-primary-medium p-4 shadow-sm"
>
  <figcaption
    class="flex items-center justify-between gap-2.5 text-xs font-semibold uppercase
           tracking-wider text-body-subtle"
  >
    <span>Madde haritası</span>
    <!--
      Sıfırlama YALNIZ zoomlanmışken görünüyor: hiç kullanılmamış bir denetim,
      kullanıcıya "burada bir şey var mı" diye düşündürmekten başka iş görmez.
    -->
    {#if zoomlu}
      <button
        type="button"
        class="rounded-sm px-1.5 py-0.5 text-xs text-body-subtle transition-colors
               hover:bg-neutral-tertiary-medium hover:text-body"
        onclick={zoomSifirla}
      >
        Yakınlaştırmayı sıfırla
      </button>
    {/if}
  </figcaption>

  {#if items.length === 0}
    <p class="mt-1 text-xs text-body-subtle">Sonuç girilmemiş.</p>
  {:else if !hasDiscrimination}
    <!--
      Ayırt edicilik yoksa saçılımın DİKEY EKSENİ yok; yine de nokta koymak,
      olmayan bir ölçüyü varmış gibi göstermek olurdu. Onun yerine yalnız
      güçlük şeridi: aynı yatay eksen, tek boyut, hiçbir iddia kaybı yok.
    -->
    <p class="mt-1 text-xs text-body-subtle">
      Ayırt edicilik {studentCount} öğrenciyle hesaplanmıyor ({MIN_DISCRIMINATION_N} gerekiyor);
      harita yalnız güçlük eksenini gösteriyor.
    </p>
    <svg
      class="mt-2.5 w-full"
      viewBox="0 0 {W} 54"
      role="img"
      aria-label="Soruların güçlük şeridi"
    >
      <line x1={LEFT} y1={30} x2={W - RIGHT} y2={30} class="stroke-default-medium" />
      {#each items as item (item.questionId)}
        <circle cx={x(item.difficulty)} cy={30} r="4" class="fill-gray-700 dark:fill-gray-300">
          <title>Soru {item.order} · güçlük %{(item.difficulty * 100).toFixed(0)}</title>
        </circle>
      {/each}
      {#each [0, 0.5, 1] as t (t)}
        <text
          x={x(t)}
          y={48}
          text-anchor={t === 0 ? "start" : t === 1 ? "end" : "middle"}
          class="fill-body-subtle"
          style="font-size: 8px"
        >
          {(t * 100).toFixed(0)}
        </text>
      {/each}
    </svg>
  {:else}
    <!--
      İŞARETÇİ OLAYLARI SVG'DE, NOKTALARDA DEĞİL. Fırçanın kaplaması bütün
      alanı yakalıyor; olay oradan KABARARAK buraya ulaşıyor ve en yakın nokta
      Delaunay ile bulunuyor. Noktalara tek tek dinleyici bağlamak fırça
      yüzünden çalışmıyordu.
    -->
    <svg
      bind:this={svgEl}
      class="mt-2.5 w-full"
      viewBox="0 0 {W} {H}"
      role="img"
      aria-label="Madde haritası: yatay eksen güçlük, dikey eksen ayırt edicilik"
      onpointermove={imlecHareket}
      onpointerleave={imlecCik}
    >
      <!--
        Bölge zeminleri ÖNCE çiziliyor ki noktalar üstünde kalsın. Kırmızı bir
        dolgu değil, çok soluk bir alan: bölgenin kendisi bulgu değil, oraya
        düşen madde bulgu.
      -->
      <rect
        x={LEFT}
        y={y(REVERSE_DISCRIMINATION)}
        width={W - RIGHT - LEFT}
        height={Math.max(y(dMin) - y(REVERSE_DISCRIMINATION), 0)}
        class="fill-red-600 dark:fill-red-400"
        opacity="0.08"
      />
      <rect
        x={LEFT}
        y={y(WEAK_D)}
        width={W - RIGHT - LEFT}
        height={Math.max(y(REVERSE_DISCRIMINATION) - y(WEAK_D), 0)}
        class="fill-amber-500 dark:fill-amber-400"
        opacity="0.08"
      />

      <!--
        ZOOM YAKALAYICI. Saydam ama olayları yakalıyor; fırçanın ALTINDA
        duruyor ki düz sürükleme fırçaya, tekerlek buraya gelsin.
      -->
      <rect
        bind:this={zoomHost}
        x={LEFT}
        y={TOP}
        width={W - RIGHT - LEFT}
        height={H - BOTTOM - TOP}
        fill="transparent"
      />

      <!-- Güçlük sınırları — okuma çerçevesi, veri değil. -->
      {#each [P_ZOR, P_KOLAY] as p (p)}
        <line
          x1={x(p)}
          y1={TOP}
          x2={x(p)}
          y2={H - BOTTOM}
          class="stroke-default-medium"
          stroke-dasharray="2 3"
        />
      {/each}

      <!-- Ebel eşikleri. Sıfır TAM çizgi: işaretin değiştiği yer. -->
      {#each [0, WEAK_D, GOOD_D] as g (g)}
        <line
          x1={LEFT}
          y1={y(g)}
          x2={W - RIGHT}
          y2={y(g)}
          class="stroke-default-medium"
          stroke-dasharray={g === 0 ? undefined : "2 3"}
        />
        <text
          x={LEFT - 3}
          y={y(g) + 3}
          text-anchor="end"
          class="fill-body-subtle"
          style="font-size: 8px; font-variant-numeric: tabular-nums"
        >
          {g.toFixed(1)}
        </text>
      {/each}

      <line x1={LEFT} y1={TOP} x2={LEFT} y2={H - BOTTOM} class="stroke-default-medium" />

      <!--
        NOKTA DEĞİL, SORU NUMARASI. Saçılımdaki çıplak bir nokta "hangi soru"
        sorusuna cevap vermez ve öğretmen tabloya dönüp aramak zorunda kalır.
        Numara doğrudan yazılınca harita kendi başına eyleme geçirilebiliyor.
      -->
      {#each yerlesim as n (n.order)}
        {@const uzerinde = selection.hoveredQuestionId === n.questionId}
        {@const etiketli = n.durum !== "saglikli" || uzerinde}
        {@const gorunur = selection.hasQuestion(n.questionId)}
        <g>
          <circle
            cx={n.cx}
            cy={n.cy}
            r={uzerinde ? R + 3 : R}
            class="{RENK[n.durum]} transition-all"
            opacity={gorunur ? 0.9 : 0.18}
          />
          <!--
            NUMARA DAİRENİN İÇİNDE, ORTALI. Kaydırma yok: etiket her zaman
            noktasının merkezinde. Çakışan iki nokta gerçekten çakışıyor —
            bu bilginin kendisi (iki maddenin p ve D değeri yakın); imleç
            balonu hangisi olduğunu söylüyor.
          -->
          {#if etiketli}
            <text
              x={n.cx}
              y={n.cy + 3}
              text-anchor="middle"
              class="fill-white dark:fill-gray-900"
              opacity={gorunur ? 1 : 0.25}
              style="font-size: 8px; font-weight: 600; font-variant-numeric: tabular-nums"
              aria-hidden="true"
            >
              {n.order}
            </text>
          {/if}
        </g>
      {/each}

      <!--
        FIRÇA KATMANI EN ÜSTTE. Kaplaması ETKİN — d3-brush'ta yeni seçimi
        başlatan dikdörtgen odur. İmleç sorunu Delaunay ile çözüldüğü için
        kaplamayı geçirgen yapmaya gerek kalmadı.
      -->
      <g
        bind:this={brushHost}
        class="[&_.selection]:fill-primary-600/15 [&_.selection]:stroke-primary-600"
      ></g>

      {#each [0, P_ZOR, P_KOLAY, 1] as p (p)}
        <text
          x={x(p)}
          y={H - BOTTOM + 11}
          text-anchor={p === 0 ? "start" : p === 1 ? "end" : "middle"}
          class="fill-body-subtle"
          style="font-size: 8px; font-variant-numeric: tabular-nums"
        >
          {(p * 100).toFixed(0)}
        </text>
      {/each}
      <text x={LEFT} y={H - 3} class="fill-body-subtle" style="font-size: 8px">zor</text>
      <text
        x={W - RIGHT}
        y={H - 3}
        text-anchor="end"
        class="fill-body-subtle"
        style="font-size: 8px">kolay</text
      >
    </svg>

    <!--
      BALON HTML, SVG DEĞİL. SVG <text> ölçekle birlikte büyüyor ve satır
      sarma bilmiyor; soru adı gibi değişken uzunlukta bir metin oraya
      sığdırılamaz. HTML balon figürün üstünde mutlak konumlanıyor, kendi
      tipografisiyle.

      `pointer-events-none` ŞART: balon imlecin altında kalırsa noktanın
      `pointerleave`ini tetikler, balon kapanır, imleç tekrar noktaya düşer —
      sonsuz titreme.
    -->
    {#if balon}
      {@const solda = balon.x > (figureEl?.clientWidth ?? 0) / 2}
      <div
        class="pointer-events-none absolute z-10 max-w-[16rem] rounded-lg border
               border-default-medium bg-neutral-primary-medium px-2.5 py-1.5 text-xs
               shadow-lg"
        style="left: {balon.x}px; top: {balon.y}px;
               transform: translate({solda ? '-100%' : '0'}, -50%) translateX({solda ? -10 : 10}px)"
      >
        <p class="font-semibold text-heading">
          Soru {balon.order}{#if label}
            — {label(balon.questionId)}{/if}
        </p>
        <p class="tnum mt-0.5 text-body-subtle">
          Güçlük %{(balon.p * 100).toFixed(0)} · Ayırt edicilik {balon.d.toFixed(2)}
        </p>
        <p class="mt-0.5 text-body-subtle">
          {#if balon.d <= REVERSE_DISCRIMINATION}
            <span class="text-red-600 dark:text-red-400">Ters ayırıyor — anahtarı kontrol et</span>
          {:else if balon.d < WEAK_D}
            <span class="text-amber-600 dark:text-amber-400">Ayırt etmiyor</span>
          {:else}
            Sağlıklı
          {/if}
        </p>
      </div>
    {/if}

    <!--
      BALON HTML, SVG DEĞİL: SVG <text> ölçekle büyüyor ve satır sarma bilmiyor.
      `pointer-events-none` ŞART — imlecin altına girerse SVG'nin
      `pointerleave`ini tetikler, balon kapanır, imleç tekrar noktaya düşer:
      sonsuz titreme.
    -->
    {#if balon}
      {@const solda = balon.x > (figureEl?.clientWidth ?? 0) / 2}
      <div
        class="pointer-events-none absolute z-10 max-w-[16rem] rounded-lg border
               border-default-medium bg-neutral-primary-medium px-2.5 py-1.5 text-xs shadow-lg"
        style="left: {balon.x}px; top: {balon.y}px;
               transform: translate({solda ? '-100%' : '0'}, -50%) translateX({solda ? -12 : 12}px)"
      >
        <p class="font-semibold text-heading">
          Soru {balon.order}{#if label}
            — {label(balon.questionId)}{/if}
        </p>
        <p class="tnum mt-0.5 text-body-subtle">
          Güçlük %{(balon.p * 100).toFixed(0)} · Ayırt edicilik {balon.d.toFixed(2)}
        </p>
        <p class="mt-0.5">
          {#if balon.d <= REVERSE_DISCRIMINATION}
            <span class="text-red-600 dark:text-red-400">Ters ayırıyor — anahtarı kontrol et</span>
          {:else if balon.d < WEAK_D}
            <span class="text-amber-600 dark:text-amber-400">Ayırt etmiyor</span>
          {:else}
            <span class="text-body-subtle">Sağlıklı</span>
          {/if}
        </p>
      </div>
    {/if}

    <p class="mt-1 text-xs text-body-subtle">
      Yatay: güçlük (sağa gittikçe kolay) · Dikey: ayırt edicilik. Kesikli yatay
      çizgiler 0.20 ve 0.40 — ölçmede yerleşik eşikler. Sıfırın altı ters ayıran
      bölge. Sürükle: bölge seç · Tekerlek: yakınlaştır · Shift+sürükle: kaydır.
    </p>

    <!--
      SAYIM METİN OLARAK DA VAR. Grafik "nerede" sorusuna cevap veriyor; "kaç
      tane" sorusuna bir cümle daha hızlı cevap veriyor — ve grafiğin taşıdığı
      bulgu ekran okuyucuya ancak böyle ulaşıyor.
    -->
    {#if sayim.ters > 0 || sayim.zayif > 0}
      <ul class="mt-1 space-y-0.5 text-xs">
        {#if sayim.ters > 0}
          <li class="text-red-600 dark:text-red-400">
            {sayim.ters} soru ters ayırıyor — iyi öğrenciler yanlış yapmış, önce cevap
            anahtarını kontrol et.
          </li>
        {/if}
        {#if sayim.zayif > 0}
          <li class="text-amber-600 dark:text-amber-400">
            {sayim.zayif} soru ayırt etmiyor (D &lt; {WEAK_D}).
          </li>
        {/if}
      </ul>
    {:else}
      <p class="mt-1 text-xs text-body-subtle">Ayırt ediciliği zayıf soru yok.</p>
    {/if}
  {/if}
</figure>
