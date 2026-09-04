<script lang="ts">
  /**
   * Sayfa kabı: sabit başlık şeridi + kendi içinde kayan içerik.
   *
   * HER SAYFA AYNI İSKELETİ KULLANIR. Kaydırmayı sayfa sayfa kurmak bu
   * projede zaten bir kez ters gitti: kabı olmayan sayfada belge kayıyor,
   * sol çekmece de içerikle birlikte yukarı gidiyordu. Tek kap, tek davranış.
   *
   * `actions` yuvası başlığın sağına düşer — "Yeni soru", "PDF kaydet" gibi
   * sayfanın ana eylemi her sayfada aynı yerde durur.
   */
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    /** Başlığın altındaki tek satırlık açıklama. */
    subtitle?: string | null;
    /** Başlığın sağındaki eylemler. */
    actions?: Snippet;
    children: Snippet;
    /**
     * İçeriğin KENDİ kaydırıcısı varsa (editör, tam yükseklikli düzen) kabuk
     * kaydırmayı üstlenmemeli: iç içe iki kaydırıcıda fare tekerleğinin
     * hangisini süreceği belirsizleşir.
     */
    scroll?: boolean;
  };

  let { title, subtitle = null, actions, children, scroll = true }: Props = $props();
</script>

<div class="flex h-full min-h-0 flex-col">
  <header
    class="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b
           border-gray-200 bg-white px-6 py-4 dark:border-gray-700 dark:bg-gray-800"
  >
    <!--
      `title` ÖZNİTELİĞİ `truncate`IN ŞARTI. Şerit daraldığında bu kutu
      (`min-w-0` sayesinde) min-content'in altına inip başlığı üç noktayla
      kesiyor — kesilen metin başka hiçbir yerde yazmıyor. Native ipucu, uzun
      sınav adının fareyle de olsa TAM okunabilmesini sağlıyor.
    -->
    <div class="min-w-0">
      <h1 class="truncate text-xl font-semibold text-gray-900 dark:text-white" {title}>
        {title}
      </h1>
      {#if subtitle}
        <p class="mt-0.5 truncate text-sm text-gray-500 dark:text-gray-400" title={subtitle}>
          {subtitle}
        </p>
      {/if}
    </div>

    {#if actions}
      <!--
        `ml-auto` VAR, `shrink-0` YOK, `min-w-0` DA YOK — üçü de bilerek.

        DÜZELTME: burada eskiden "shrink-0 kalkınca küme başlığın altına inmez,
        kendi içinde iki sıraya bölünür" yazıyordu; bu CSS olarak YANLIŞTI.
        Flexbox §9.3'te öğeleri satırlara dağıtan adım HİPOTETİK ana boyuta
        (`flex: 0 1 auto` için max-content) bakar; `flex-shrink` o karara hiç
        girmez, `min-width` ise yalnız YUKARI kelepçeler — temel boyut zaten
        max-content olduğu için `min-width: 0` onu DÜŞÜREMEZ. Küçültme satır
        kırmadan SONRA gelir. Ölçü
        (1024 px pencere, `/exams/[id]`): şerit iç genişliği 1024 − 224
        (çekmece) − 1 (kenarlık) − 48 (px-6) = 751 px; gerçekçi bir sınav
        başlığı ~300 px + gap 12 px + eylem kümesi ~630 px (rozet + "Kâğıt
        ayarları" + w-24 seçim kutusu + "Cevap anahtarı" + "PDF kaydet" +
        "Yayınla" + 5 × gap-2) = ~940 px > 751 → küme ZATEN ikinci satıra
        iniyor, `shrink-0` olsa da olmasa da.

        Geriye kalan iki etki gerçek:
        • `ml-auto` o ikinci satırda kümeyi sağa yaslıyor (otomatik kenar
          boşluğu `justify-between`den önce boşluğu yutar) — sayfanın ana
          eylemi her sayfada aynı yerde duruyor. Asıl düzeltme bu.
        • `shrink-0`ın yokluğu yalnız kümenin TEK BAŞINA satırı aştığı durumda
          (~630 px bugün 751 px'e sığıyor, daha uzun etiketlerde aşabilir) işe
          yarıyor: küme kırpılmak yerine kendi `flex-wrap`'inde sarıyor.
          Emniyet supabı, bugünkü davranış değil.

        `min-w-0` BİLEREK EKLENMEDİ: sarmalı bir flex kabının min-content'i
        zaten en geniş TEK düğme kadar (~150 px), yani kutu satır genişliğine
        kadar min-w-0 olmadan da küçülebiliyor. min-w-0 ancak tek bir düğme
        şeritten genişse fark ederdi — ve o durumda `justify-end` taşmayı SOL
        (başlangıç) yöne akıtırdı; app.css'teki `html, body { overflow: hidden }`
        ile sola taşan ilk düğmeler ULAŞILAMAZ olurdu. `whitespace-nowrap` ise
        düğme etiketlerinin ("Kâğıt ayarları") iki satıra bölünmesini engelliyor.
      -->
      <div class="ml-auto flex flex-wrap items-center justify-end gap-2 whitespace-nowrap">
        {@render actions()}
      </div>
    {/if}
  </header>

  <div class="min-h-0 flex-1" class:overflow-auto={scroll} class:p-6={scroll}>
    {@render children()}
  </div>
</div>
