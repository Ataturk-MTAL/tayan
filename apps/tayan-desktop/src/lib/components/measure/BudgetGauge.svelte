<script lang="ts">
  /**
   * Bütçe köşede bir sayı değil, dolan ve GÖRÜNÜR biçimde taşan bir ölçüdür.
   * Taşma gizlenmez: taştığı an kırmızı bölge çubuğun dışına çıkar, çünkü
   * 100 puanı aşan bir sınav öğretmenin fark etmesi gereken bir hatadır.
   */
  type Props = {
    label: string;
    value: number;
    target: number;
    unit: string;
  };

  let { label, value, target, unit }: Props = $props();

  /** Çubuğun piksel genişliği. Aşağıdaki `w-[160px]` ile AYNI kalmalı: taşma bloğu
   *  bu sayıyı ölçek olarak kullanıyor, yüzdeyle değil (akış kabında yüzde
   *  çözülemez — kabın genişliği zaten içeriğinden geliyor). */
  const BAR_WIDTH_PX = 160;
  /** Taşmanın üst sınırı bir TAM bütçe: kırmızı en çok dolu çubuk kadar uzar,
   *  yani doyum noktası ratio 2 (200/100 puan, 40/20 soru). */
  const MAX_OVER_RATIO = 1;

  let ratio = $derived(target > 0 ? value / target : 0);
  let fillPct = $derived(Math.min(ratio, 1) * 100);
  /**
   * Taşan kısım çubukla AYNI ölçekte çizilir: 1 birim taşma = 160px, yani
   * 140/100 puan → 64px kırmızı. Bir ara bu sınır %25'e (ratio 1.25) çekilmişti;
   * o hâlde 125/100 ile 300/100 aynı 40px'lik şeridi çiziyor, taşmanın
   * BÜYÜKLÜĞÜ okunmuyordu — oysa ölçünün ürünü tam olarak o büyüklük. Sınır
   * geri alındı; sayının kırmızının altında kalması sorunu kırpmayla değil,
   * aşağıdaki akış yerleşimiyle çözüldü.
   */
  let overWidthPx = $derived(
    ratio > 1 ? Math.min(ratio - 1, MAX_OVER_RATIO) * BAR_WIDTH_PX : 0
  );
  let over = $derived(value > target);
</script>

<div class="flex items-center gap-2.5">
  <span class="whitespace-nowrap text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
    {label}
  </span>

  <!--
    Çubuk + taşma tek bir AKIŞ kabında: taşma bloğu artık `absolute` değil,
    çubuğun sağındaki gerçek bir kardeş. İki nedenle:
    (1) Konumlandırılmış öğe, boyama sırasında konumlandırılmamış satır içi
        metnin HER ZAMAN üstünde kalır (CSS 2.1 Ek E: konumlandırılmış
        torunlar 8. adım, satır içi kutular 7. adım). Eski `absolute` blokla
        sağdaki "{value} / {target} {unit}" okuması arasında satırın yalnız
        gap-2.5'i (10px) vardı: 140/100 puanda 64px'lik blok o boşluğu yiyip
        sayının üstüne biniyor, 200/100'de (160px) sayıyı tümüyle siliyordu.
        Akışta kardeş olunca örtüşme fiziksel olarak imkânsız; okumaya her
        durumda gap-2.5 kadar temiz mesafe kalıyor.
    (2) Yer akıştan alındığı için taşma YOKKEN hiç yer tutmuyor. Koşulsuz bir
        `mr-[44px]` her ölçüye kalıcı 44px bindiriyordu: /exams/[id] başlık
        altı şeridinde iki ölçü var, yani 1024px'lik en dar pencerede şeride
        kalan 751px'in (1024 − 224 çekmece − 1 kenarlık − 48 px-6) 88px'i
        normal durumda boşa gidiyor, üstelik çubukla sayı arasında
        açıklanamayan bir boşluk olarak görünüyordu.
    Şerit `flex-wrap`: taşma büyüyünce ölçü sığmazsa alt satıra iner. En geniş
    hâli bile (40 + 10 + 160 + 160 + 10 + ~90 ≈ 470px) 751px'in altında.
  -->
  <div class="flex shrink-0 items-center">
    <div class="relative h-[10px] w-[160px] border border-gray-300 bg-gray-100 dark:border-gray-600 dark:bg-gray-700">
      <div class="absolute inset-y-0 left-0 bg-gray-800 dark:bg-gray-200" style="width: {fillPct}%"></div>
    </div>
    {#if overWidthPx > 0}
      <!--
        Taşan kısım çubuğun DIŞINDA, kenarlığın bittiği yerden başlar. h-[12px]
        çubuğun 10px'inden 1px yukarı, 1px aşağı taşar (items-center ortalıyor):
        kırmızı, ölçünün devamı değil ondan çıkan bir şerit gibi okunsun. Kırmızı
        burada dekor değil, değerlendirme: bütçe aşıldı.
      -->
      <div
        class="h-[12px] bg-red-600 dark:bg-red-400"
        style="width: {overWidthPx}px"
        aria-hidden="true"
      ></div>
    {/if}
  </div>

  <!--
    shrink-0 + whitespace-nowrap: "{value} / {target} {unit}" boşluklu bir
    dizge, yani sarma noktası sunuyor. Sınav başlığı satırı kalabalıklaştığında
    (kitapçık uyarısı, "N soru bankada yok", süre, derleme durumu aynı
    `flex-wrap` satırında) ölçünün KENDİSİ ortadan bölünüyordu: "85 / 100" bir
    satırda, "puan" alt satırda kalıyor ve `items-center` yüzünden çubuk ile
    yazı hizası kayıyordu. Etikette nowrap zaten vardı, değerde yoktu. Artık
    okuma tek parça; yer yetmezse ebeveynin `flex-wrap`'iyle ölçünün TAMAMI
    alt satıra iniyor.
  -->
  <span
    class="tnum shrink-0 whitespace-nowrap text-[13px] leading-5"
    class:text-red-600={over}
    class:dark:text-red-400={over}
    class:font-semibold={over}
  >
    {value} / {target} {unit}
  </span>
</div>
