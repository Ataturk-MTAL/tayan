<script lang="ts">
  import { Button } from "flowbite-svelte";
  import { MinusOutline, PlusOutline } from "flowbite-svelte-icons";

  type Props = {
    zoom: number;
    onzoom: (value: number) => void;
    onfit: () => void;
  };
  let { zoom, onzoom, onfit }: Props = $props();

  /**
   * Basamaklı seviyeler: rastgele bir sayıya değil, tahmin edilebilir yere gider.
   *
   * İlk basamak, SheetPreview'daki MIN_ZOOM ile AYNI olmak zorunda (ikisi de
   * 0.5). "Uzaklaştır" `zoom <= ZOOM_STEPS[0]` ile kapanıyor; basamak tabanı
   * zoom tabanının üstünde kalırsa aradaki bant düğmeyle ulaşılamaz olur.
   * Denendi ve geri alındı: MIN_ZOOM 0.2'ye indirilip diziye 0.25 eklendiğinde
   * "Sığdır"ın dar bölmede ürettiği 0,22'de Uzaklaştır (0,22 ≤ 0,25) kapalı
   * kalıyor, Yakınlaştır ise 0,22'den doğrudan 0,5'e sıçrayıp 0,25'i atlıyordu.
   */
  export const ZOOM_STEPS = [0.5, 0.65, 0.8, 1, 1.25, 1.5, 2, 3];

  /** Kayan nokta payı: 0.5 × 1.08 gibi çarpımlar basamağa tam oturmaz. */
  const EPSILON = 0.001;

  /**
   * Arama YÖNE duyarlı olmak zorunda: ⌘+tekerlek ve ⌘± sürekli çalıştığı için
   * zoom çoğu zaman iki basamağın ARASINDA duruyor (⌘+tekerlek bir tıkta
   * 0,5 × 1,08 = 0,54 yapıyor). Tek bir `findIndex(s => s >= zoom)` + index±1
   * bu durumda yukarı yönde bir basamak atlıyordu: 0,54'te bulunan 0,65'e +1
   * eklenip 0,8'e gidiliyordu. Doğrusu yukarı yönde zoom'dan KESİN büyük ilk
   * basamak, aşağı yönde KESİN küçük son basamak.
   */
  function step(direction: 1 | -1) {
    if (direction === 1) {
      const higher = ZOOM_STEPS.find((s) => s > zoom + EPSILON);
      onzoom(higher ?? ZOOM_STEPS[ZOOM_STEPS.length - 1]);
      return;
    }
    const lower = ZOOM_STEPS.filter((s) => s < zoom - EPSILON);
    onzoom(lower.length === 0 ? ZOOM_STEPS[0] : lower[lower.length - 1]);
  }
</script>

<!--
  `flex-wrap` + her öğede `shrink-0`: çubuğun ezilme sırası bilerek sabitlendi.

  Flowbite `Button`'ın taban sınıfı `inline-flex items-center justify-center` —
  içinde ne `shrink-0` ne `whitespace-nowrap` var. Yani düğmeler min-content'e
  ("Sığdır", "100%" tek kelime) kadar EZİLİR, ama oradan aşağı inemez; toplam
  min-content (≈315 px) bölmenin genişliğini aşınca fazlalık kabın DIŞINA akar.
  Yan yana modda bu istisna değil kural: 1024 px pencerede önizleme bölmesine
  220 px kalıyor. Taşan düğmelerin üstünü ya panelin opak zemini örtüyor ya da
  pencere kenarında kalıyorlar; app.css'teki html,body{overflow:hidden} yüzünden
  kaydırma çubuğu da yok, yani düğme ULAŞILAMAZ oluyordu.

  Çözüm ezmek değil, ALTA İNDİRMEK: `flex-wrap` ile sığmayan öğe ikinci satıra
  geçer, `shrink-0` ile de öğeler o satıra geçmeden önce okunmaz hâle
  gelmez — düğme yazısı hep tam boyunda kalır.
-->
<div
  class="flex shrink-0 flex-wrap items-center gap-2 border-b border-gray-200 bg-white px-3 py-1.5
         dark:border-gray-700 dark:bg-gray-800"
>
  <Button
    size="xs"
    color="light"
    class="!p-1.5 shrink-0"
    disabled={zoom <= ZOOM_STEPS[0]}
    title="Uzaklaştır (⌘−)"
    aria-label="Uzaklaştır"
    onclick={() => step(-1)}
  >
    <MinusOutline class="h-3.5 w-3.5" />
  </Button>

  <!--
    `w-12` sabit 48 px istiyor ve bu bilinçli: yüzde %50 → %100 → %125 arası
    değişirken kutu genişliği sabit kalmazsa yanındaki düğmeler her adımda
    zıplar. Sabit genişlik ancak `shrink-0` ile gerçekten sabittir — onsuz flex
    öğesi olarak 48 px'in altına ezilir ve rakam kırpılırdı.
  -->
  <span class="tnum w-12 shrink-0 text-center text-xs text-gray-600 dark:text-gray-300">
    {Math.round(zoom * 100)}%
  </span>

  <Button
    size="xs"
    color="light"
    class="!p-1.5 shrink-0"
    disabled={zoom >= ZOOM_STEPS[ZOOM_STEPS.length - 1]}
    title="Yakınlaştır (⌘+)"
    aria-label="Yakınlaştır"
    onclick={() => step(1)}
  >
    <PlusOutline class="h-3.5 w-3.5" />
  </Button>

  <Button
    size="xs"
    color="light"
    class="ml-2 shrink-0"
    title="Sayfayı panele sığdır"
    onclick={onfit}
  >
    Sığdır
  </Button>

  <Button
    size="xs"
    color="light"
    class="shrink-0"
    title="Gerçek boyut (⌘0)"
    onclick={() => onzoom(1)}
  >
    100%
  </Button>

  <!--
    İpucu KISALTILMADI ve `shrink-0` da verilmedi: sığdığı sürece `ml-auto` onu
    sağa yaslıyor, sığmadığında `flex-wrap` sayesinde ikinci satıra inip orada
    tam hâliyle duruyor. Çubuğun feda edilebilir tek öğesi bu; düğmeler değil.
  -->
  <span class="ml-auto text-xs text-gray-400 dark:text-gray-500">⌘ + tekerlek</span>
</div>
