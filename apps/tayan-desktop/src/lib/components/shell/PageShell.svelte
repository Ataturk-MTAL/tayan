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
   *
   * ŞERİDİN YÜKSEKLİĞİ SABİT (`--tayan-header-h`) ve İÇERİĞİNDEN BAĞIMSIZ.
   * Eskiden içeriğe göre büyüyordu ve bu üç ayrı zıplama üretiyordu:
   *   1. Alt başlığı olan sayfa, olmayandan bir satır uzundu.
   *   2. Alt başlıklar `loading ? null : ...` olduğu için AYNI sayfada veri
   *      gelince şerit büyüyordu — sayfa gözünüzün önünde kayıyordu.
   *   3. Eylem kümesi dar pencerede ikinci satıra sarınca bir satır daha.
   * Çekmece de şeridin altından başladığı için üst kenarı bunların hepsiyle
   * birlikte oynuyordu. Sabit yükseklik üçünü birden kesiyor; karşılığında
   * eylemler artık SARMIYOR, sığmazsa kendi içinde yatay kayıyor.
   *
   * İKİNCİL BİLGİ TEK SATIRDA: bölüm adı ile alt başlık `·` ile birleşip
   * başlığın altındaki tek satıra giriyor. Ayrı satırlar olsalardı şerit üç
   * satıra çıkar ve sabit yükseklik onları kırpardı.
   */
  import type { Snippet } from "svelte";
  import { page } from "$app/state";
  import { closeDrawer, drawer, toggleDrawer } from "$lib/ui/drawer.svelte";
  import { activeSection } from "$lib/ui/nav.svelte";
  import { BarsOutline, CloseOutline } from "flowbite-svelte-icons";

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

  /**
   * Bulunulan bölüm — çekmeceyle AYNI kaynaktan.
   *
   * Şerit, bölüm listesini kendi kopyasında tutmuyor: `nav.svelte.ts`ten
   * okuyor. Bir bölüm eklendiğinde ya da adı değiştiğinde çekmece ve şerit
   * birlikte güncelleniyor; iki kopya birbirinden ayrı düşemiyor.
   *
   * Ana sayfada (`/`) `null` döner ve etiket hiç çizilmez — ana sayfa
   * bölümlerin üstünde durur, içlerinden biri değildir.
   */
  let section = $derived(activeSection(page.url.pathname));

  /**
   * Bölüm etiketi yalnız başlıktan FARKLIYSA gösterilir.
   *
   * Bölümün liste sayfasında ikisi çakışıyor: `/exams` başlığı "Sınavlar",
   * bölüm adı da "Sınavlar" — üst üste iki kez aynı kelime, hiçbir şey
   * eklemeden yer kaplıyordu. Etiket asıl işini alt sayfalarda görüyor:
   * `/questions/new` başlığı "Yeni soru", etiketi "Sorular".
   *
   * Karşılaştırma Türkçe yerelinde: `toLowerCase()` "SINAVLAR"ı "sinavlar"
   * yapıp "Sınavlar" ile eşleşmesini engellerdi (I → i, oysa Türkçede I → ı).
   */
  let sectionLabel = $derived(
    section && section.label.toLocaleLowerCase("tr") !== title.toLocaleLowerCase("tr")
      ? section.label
      : null,
  );

  /**
   * Başlığın altındaki TEK ikincil satır: bölüm adı ve alt başlık birlikte.
   *
   * İkisi ayrı satırlarda olsaydı şerit üç satıra çıkardı; sabit yükseklikte
   * üçüncü satır kırpılırdı. `·` ile birleştirmek ikisini de tek satıra
   * sığdırıyor: "Sorular · 4 / 4 soru".
   *
   * BOŞ OLSA DA SATIR ÇİZİLİYOR (aşağıda `&nbsp;` ile). İkisi de yokken satır
   * hiç çizilmeseydi başlık dikeyde ortalanıp YUKARI kayardı — yükseklik sabit
   * kalsa bile başlığın yeri sayfadan sayfaya oynardı, ki şikâyet tam olarak
   * bu oynamaydı.
   */
  let secondaryLine = $derived([sectionLabel, subtitle].filter(Boolean).join(" · "));
</script>

<div class="flex h-full min-h-0 flex-col">
  <header
    class="flex h-[var(--tayan-header-h)] shrink-0 items-center gap-3 border-b
           border-gray-200 bg-white px-6 dark:border-gray-700 dark:bg-gray-800"
  >
    <!--
      `flex-wrap` YOK ve `py-4` YOK — ikisi de bilerek.

      Sarma, şeridin yüksekliğini içeriğine bağlıyordu: dar pencerede eylem
      kümesi ikinci satıra iniyor ve şerit büyüyordu. Dikey iç boşluk da aynı
      şeyi yapıyordu — satır sayısı arttıkça yükseklik onunla artıyordu.
      Yükseklik artık `h-[var(--tayan-header-h)]` ile sabit, dikey hizalama
      `items-center`ın işi.

      BEDELİ GERÇEK: eylemler artık sarmıyor. `minWidth: 1024` penceresinde en
      kalabalık şerit (`/exams/[id]`: rozet + "Kâğıt ayarları" + seçim kutusu +
      "Cevap anahtarı" + "PDF kaydet" + "Yayınla" ≈ 630 px) marka ve düğmeyle
      birlikte sınıra dayanıyor. Taşarsa KIRPILMIYOR: eylem kümesinin kendi
      `overflow-x-auto`su var, yatay kayarak hepsine ulaşılabiliyor. Kırpma,
      "Yayınla" düğmesini ulaşılamaz yapardı.
    -->
    <div class="flex min-w-0 flex-1 items-center gap-3">
      <!--
        ÇEKMECE DÜĞMESİ HER SAYFADA AYNI YERDE. Çekmece kapandığında onu geri
        getirecek tek tutamak bu; sayfaya özel olsaydı bir sayfada kaybolur ve
        menü geri alınamaz hâle gelirdi. Kapak `PageShell`'de olduğu için 11
        sayfanın hepsi tek düzenlemeyle kapsanıyor.

        `aria-expanded` + `aria-controls`: ekran okuyucu düğmenin bir şeyi AÇIP
        KAPADIĞINI ve neyi kontrol ettiğini ancak bu ikisiyle bilebilir; ikon
        tek başına "çubuklar" diye okunurdu. `aria-label` duruma göre değişiyor
        — etiket, basınca ne OLACAĞINI söylemeli.
      -->
      <button
        type="button"
        class="-ml-2 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg
               text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-900
               dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white"
        aria-label={drawer.open ? "Menüyü kapat" : "Menüyü aç"}
        aria-expanded={drawer.open}
        aria-controls="main-drawer"
        title={drawer.open ? "Menüyü kapat" : "Menüyü aç"}
        onclick={toggleDrawer}
      >
        <!--
          İKON DURUMU SÖYLER. Aç/kapa aynı düğmede olduğu için ikon sabit
          kalsaydı, çekmece açıkken üç çubuk "menüyü aç" der gibi durur ve
          kapatma tutamağı olduğu anlaşılmazdı. ✕ hem kapatmayı hem de o anda
          açık bir katman olduğunu gösteriyor.
        -->
        {#if drawer.open}
          <CloseOutline class="h-5 w-5" />
        {:else}
          <BarsOutline class="h-5 w-5" />
        {/if}
      </button>

      <!--
        MARKA DÜĞMENİN SAĞINDA, en solda DEĞİL.

        Çekmece sol kenardan giriyor; onu çağıran düğme de o kenarda durmalı.
        Düğme ✕'e döndüğünde tam çekmecenin sol üst köşesinin üstüne oturuyor
        ve kapattığı paneli görsel olarak "kapatıyor". Marka öne alınsaydı ✕
        yaklaşık 120 px içeri kayar, kapattığı şeyle bağı kopardı. Material
        app bar, Gmail ve Flowbite'ın kendi dashboard navbar'ı da bu sırada:
        önce çekmece düğmesi, sonra marka.

        KARŞI ARGÜMAN GERÇEK: "sol üstteki logo = ana sayfa" webin en köklü
        alışkanlığı ve marka bir düğme boyu içeri kayınca o refleks zayıflıyor.
        Burada kaybediyor çünkü çekmece bu uygulamanın BİRİNCİL gezinmesi ve
        menüyü açmak, ana sayfaya gitmekten kat kat sık bir eylem — ekranın
        sonsuz büyüklükteki sol üst köşesi sık olana verilmeli.

        TEK BAĞLANTI, TEK AD: karo ile kelime ayrı hedefler değil. İkisi ayrı
        `<a>` olsaydı ekran okuyucu "T" ve "TAYAN" diye iki ayrı bağlantı
        okurdu; ikisi de aynı yere gidiyor.
      -->
      <a
        href="/"
        class="flex shrink-0 items-center gap-2 no-underline"
        aria-label="TAYAN — Ana sayfa"
        aria-current={page.url.pathname === "/" ? "page" : undefined}
        onclick={closeDrawer}
      >
        <!--
          `rounded-md`, `rounded-lg` DEĞİL. Flowbite'ın tema belirteçleri
          `--radius-lg`i 16 px yapıyor (Tailwind'in varsayılanı 8 px); 32 px'lik
          bu karoda 16 px tam yarıçap demek, yani karo DAİREYE dönüşüyordu.
          `rounded-md` Flowbite'ın ezmediği bir belirteç (6 px) ve markayı
          yeniden yuvarlatılmış kare yapıyor.

          Yarıçap ölçeğinin tamamı geri alınmıyor: büyük yarıçaplar Flowbite
          temasının kendisi ve istenen görünüm; sorun yalnız bu boyuttaki kutuda.
        -->
        <span
          class="flex h-8 w-8 items-center justify-center rounded-md bg-primary-600
                 text-sm font-bold text-white"
        >
          T
        </span>
        <span class="text-base font-bold tracking-tight text-gray-900 dark:text-white">
          TAYAN
        </span>
      </a>

      <!--
        Ayraç: marka uygulamanın kimliği, başlık ise o anki sayfa. Araya çizgi
        girmeseydi "TAYAN Yeni soru" tek bir söz öbeği gibi okunurdu.
        `aria-hidden`: ekran okuyucuya söyleyecek bir şeyi yok.
      -->
      <div
        class="h-8 w-px shrink-0 bg-gray-200 dark:bg-gray-700"
        aria-hidden="true"
      ></div>

      <!--
        `title` ÖZNİTELİĞİ `truncate`IN ŞARTI. Şerit daraldığında bu kutu
        (`min-w-0` sayesinde) min-content'in altına inip başlığı üç noktayla
        kesiyor — kesilen metin başka hiçbir yerde yazmıyor. Native ipucu, uzun
        sınav adının fareyle de olsa TAM okunabilmesini sağlıyor.

        SIKIŞMADA İLK BU KÜÇÜLÜR: marka ve ayraç `shrink-0`, bu kutu değil.
        Şerit daralınca kimlik ve çekmece düğmesi yerinde kalır, kesilen
        yalnız sayfa başlığı olur — ve o zaten `title` özniteliğinde tam
        hâliyle duruyor.
      -->
      <div class="min-w-0">
        <h1 class="truncate text-xl font-semibold text-gray-900 dark:text-white" {title}>
          {title}
        </h1>

        <!--
          İKİNCİL SATIR HER ZAMAN ÇİZİLİR, boşken bile (`&nbsp;`).

          Koşullu çizilseydi yükseklik sabit kalsa da BAŞLIK oynardı: satır
          varken çift, yokken tek satır dikeyde ortalanır ve başlığın taban
          çizgisi sayfadan sayfaya yaklaşık 10 px kayardı. Görünmez bir satır,
          başlığı her sayfada aynı yükseklikte tutuyor.

          Bölüm adı ile alt başlık burada `·` ile birleşiyor: "Sorular · 4 / 4
          soru". Bölüm adı çekmecenin okuduğu listeden geliyor — menü
          değişince bu satır da kendiliğinden değişiyor.

          `title` özniteliği yalnız DOLU satırda: boş satıra ipucu bağlamak
          fareyi orada bekleyen kullanıcıya boş bir kutu gösterirdi.
        -->
        <p
          class="mt-0.5 truncate text-sm text-gray-500 dark:text-gray-400"
          title={secondaryLine || undefined}
        >
          {#if secondaryLine}{secondaryLine}{:else}&nbsp;{/if}
        </p>
      </div>
    </div>

    {#if actions}
      <!--
        SARMA YERİNE YATAY KAYMA. Bu küme eskiden `flex-wrap` ile ikinci satıra
        iniyordu; şeridin yüksekliğini değiştiren üç sebepten biri buydu.
        Sabit yükseklikte sarma artık mümkün değil, ama taşmayı KIRPMAK da
        seçenek değil: `min-w-0` + `overflow-x-auto` ile küme kendi içinde
        yatay kayıyor, en sağdaki "Yayınla" düğmesine hep ulaşılabiliyor.

        ÖLÇÜ (1024 px — `minWidth`, yani mümkün en dar pencere; `/exams/[id]`):
        şerit iç genişliği 1024 − 48 (px-6) = 976 px. Sabit olanlar: çekmece
        düğmesi 40 px + marka ~130 px + ayraç 1 px + 3 × gap-3 (36 px) = 207 px.
        Eylem kümesi ~630 px (rozet + "Kâğıt ayarları" + w-24 seçim kutusu +
        "Cevap anahtarı" + "PDF kaydet" + "Yayınla" + 5 × gap-2). Geriye
        başlığa 976 − 207 − 630 = 139 px kalıyor — başlık üç noktayla kesilir
        ama kümenin tamamı görünür. Daha uzun etiketlerde küme kayma moduna
        geçer; kaybolan bir şey olmaz.

        `ml-auto`: kümeyi sağa yaslıyor, sayfanın ana eylemi her sayfada aynı
        yerde duruyor. `whitespace-nowrap`: "Kâğıt ayarları" gibi iki kelimeli
        etiketlerin alt satıra bölünmesini engelliyor — sabit yükseklikte o
        bölünme düğmeyi kırpardı.
      -->
      <div
        class="ml-auto flex min-w-0 items-center justify-end gap-2 overflow-x-auto
               whitespace-nowrap"
      >
        {@render actions()}
      </div>
    {/if}
  </header>

  <div class="min-h-0 flex-1" class:overflow-auto={scroll} class:p-6={scroll}>
    {@render children()}
  </div>
</div>
