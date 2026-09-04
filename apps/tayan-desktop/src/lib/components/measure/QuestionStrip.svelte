<script lang="ts">
  import { questionPoints, scoreBadge, type Question, type ScoreBadge } from "$lib/types";

  /**
   * Sınavın kesintisiz soru şeridi.
   *
   * Sarılmaz, ölçeklenir: sayfanın tek kesintisiz çizgisi olarak durur ve
   * sınavın tamamı tek bakışta okunur. Her tuşun durumu tuşun KENDİSİNDE
   * yazılıdır — ayrı bir açıklama sütununa bakmak gerekmez.
   *
   * "Tek bakışta" bir SÜS DEĞİL, şeridin tek ürünü: tuş ölçüsü soru sayısına
   * göre daralıyor (aşağıdaki YOGUN_ESIK) ki 40 soruluk bir sınav varsayılan
   * pencereye sığmaya devam etsin. Kaydırma son çare. Taban ölçüsü eşikte
   * 24px'ten 20px'e atladığı için kırılma noktası tek bir bölme DEĞİL, iki
   * ayrı hesabın küçüğü (şeride kalan yer için aşağıdaki YOGUN_ESIK bloğu):
   *  · 1280px, şeride ~883px → 32 tuşa kadar taban 24px: 32 × 24 = 768 ≤ 883,
   *    sığar. 32'nin üstünde taban 20px: 44 × 20 = 880 ≤ 883, 45 × 20 = 900
   *    > 883. Yani 45. tuştan itibaren kayar.
   *  · 1024px asgarisi, şeride ~627px → burada 32 tuşa kadar taban HÂLÂ 24px:
   *    26 × 24 = 624 ≤ 627 ama 27 × 24 = 648 > 627. Yani 27. tuştan itibaren
   *    kayar. Eski yorumdaki "~31 tuş" sayısı eşiğin ALTINDAKİ 24px'lik tuşu
   *    unutup 627/20 diye her yerde 20px saymaktan geliyordu, yanlıştı.
   *
   * Kırmızı burada da yalnızca değerlendirme demektir: kırmızı bir tuş, o
   * sorunun ölçülmüş ve zayıf çıkmış olduğunu söyler.
   */
  type Props = {
    questions: Question[];
    activeId?: string | null;
    onselect?: (id: string) => void;
  };

  let { questions, activeId = null, onselect }: Props = $props();

  /**
   * Beş düzeyli rozet Tailwind sınıfı olarak.
   *
   * Eskiden CSS değişkeniydi (`--color-mark-*`); Flowbite temasında bu
   * değişkenler yok. Beş düzey birbirinden ayrı renk taşımalı — hepsini
   * gri/kırmızıya indirmek "çok iyi" ile "orta"yı ayırt edilemez yapardı.
   * Kırmızı yalnız "zayıf" düzeyinde: kural burada da geçerli, kırmızı
   * yalnızca değerlendirmenin en olumsuz ucunu işaretliyor.
   */
  const BADGE_CLASS: Record<ScoreBadge, string> = {
    excellent: "bg-green-700 dark:bg-green-400",
    good: "bg-primary-600 dark:bg-primary-400",
    fair: "bg-amber-500 dark:bg-amber-400",
    poor: "bg-red-600 dark:bg-red-400",
    untested: "bg-gray-300 dark:bg-gray-600",
  };

  const BADGE_LABEL: Record<ScoreBadge, string> = {
    excellent: "çok iyi",
    good: "iyi",
    fair: "orta",
    poor: "zayıf",
    untested: "denenmemiş",
  };

  /**
   * Tuş ölçüsü soru sayısına bağlı: sabit 24px taban, şeridin varlık
   * nedenini (sınavın tamamı tek bakışta) uzun sınavlarda öldürüyordu.
   *
   * Ölçüler — hepsi border-box, yani min-w dolguyu ve kenarlığı İÇERİR:
   *  · Public Sans'ın tabular rakam ilerleyişi 0.70em (fontun kendi tnum
   *    lookup'ından ölçüldü: 1400/2000 upem). "40" 13px'te 18.2px, 11px'te
   *    15.4px yer kaplar. Eski yorumun "14px" sayısı yanlıştı.
   *  · Şeride kalan yer = pencere − 224 (w-56 çekmece) − 1 (border-r)
   *    − ~172 ("SINAV" ~77 + "40 SORU" ~95, px-5 dolgularıyla):
   *    1280px varsayılanda ~883px, 1024px asgarisinde ~627px.
   *  · 40 soru × 24px = 960px > 883px: şerit VARSAYILAN pencerede bile
   *    kayıyordu. 40 × 20px = 800px ≤ 883px → sınav yine tek bakışta.
   *
   * Yoğun modda rakam 11px'e iniyor. Eski gerekçe ("20px'lik tuşun padding
   * kutusu 19px, 18.2px oraya sığıyor") yanlıştı — metin padding kutusuna
   * değil İÇERİK kutusuna dizilir ve border-box'ta içerik kutusu = taban
   * − kenarlık − dolgu:
   *  · 20px taban, px-[2px], border-l 1px → içerik 20 − 1 − 4 = 15px.
   *    13px'lik "40" (18.2px) oraya SIĞMAZ, iki yandan 1.6px taşar.
   *  · 24px taban, px-[5px], border-l 1px → içerik 24 − 1 − 10 = 13px.
   *    18.2px orada da taşar. Yani içerik kutusu hiçbir modda ölçüt değil.
   * Gerçek ölçüt komşu rakamlar arası boşluk (= taban − rakam genişliği,
   * çünkü rakam tuşta ortalanıyor ve tuşlar bitişik): 20px tabanda 13px ile
   * 20 − 18.2 = 1.8px kalır, rakamlar birbirine yapışır; 11px ile
   * 20 − 15.4 = 4.6px açılır. 24px tabanda 13px zaten 24 − 18.2 = 5.8px
   * bırakıyor, geniş mod bu yüzden 13px'i koruyabiliyor. leading-5
   * değişmediği için şerit yüksekliği iki modda da 48px.
   */
  const YOGUN_ESIK = 32;

  let yogun = $derived(questions.length > YOGUN_ESIK);
  let tusOlcu = $derived(yogun ? "min-w-[20px] px-[2px]" : "min-w-[24px] px-[5px]");
  let sayiOlcu = $derived(yogun ? "text-[11px]" : "text-[13px]");
</script>

{#if questions.length > 0}
  <div class="flex shrink-0 items-stretch border-b border-gray-300 bg-white dark:border-gray-600 dark:bg-gray-800">
    <span
      class="flex items-center px-5 text-[11px] font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400"
    >
      Sınav
    </span>

    <!--
      Sarma yok: şerit bir liste değil, tek bir çizgi. Ama kaçış valfi şart:
      tuşun min-w tabanı (yukarıda) flex sıkıştırmasının önünde durduğu için
      N × taban kabı aştığında yığın küçülemiyor; kapta `overflow` yokken taşma
      görünür biçimde dışarı akıyor ve sağdaki "N soru" sayacının üstüne
      biniyordu (1024px'te kaba 627px kalıyor, 60 soru × 20px = 1200px istiyor).
      overflow-x-auto taşmayı şeridin KENDİ içine hapsediyor; `min-w-0` da kabın
      küçülebilmesi için gerekli, ikisi de kalıyor.

      overflow-y-hidden ŞART: CSS Overflow'a göre bir eksen `visible` değilken
      diğerinin `visible`'ı `auto`'ya ÇÖZÜLÜR — yani tek başına overflow-x-auto
      bu kabı aynı anda DİKEY kaydırma kabı da yapıyordu. Kap `items-stretch`
      ile tuşların yüksekliğine (10 + 20 + 5 + 3 + 10 = 48px) kilitli; klasik
      (üste binmeyen) kaydırıcılı platformlarda — WebView2/Windows,
      WebKitGTK/Linux — yatay çubuk bu 48px'ten ~15px yiyince içerik sığmıyor
      ve anlamsız bir dikey çubuk daha açılıyordu.

      Çubuğu 6px'e çekiyoruz çünkü hidden artık kırpıyor: durum çizgisi tuşun
      35–38px bandında, altında 10px dolgu var. 15px'lik varsayılan çubuk
      çizgiyi (şeridin tek ürününü) kırpardı; 6px dolgunun içinde kalıyor.
      Yan fayda: macOS'un üste binen — yani görünmeyen — çubuğunun yerine
      görünür bir çubuk koyuyor, şeridin kaydığına dair TEK ipucu bu.
    -->
    <div
      class="flex min-w-0 flex-1 items-stretch overflow-x-auto overflow-y-hidden
             [&::-webkit-scrollbar]:h-[6px] [&::-webkit-scrollbar-track]:bg-transparent
             [&::-webkit-scrollbar-thumb]:bg-gray-300 dark:[&::-webkit-scrollbar-thumb]:bg-gray-600"
    >
      {#each questions as q, i (q.id)}
        {@const badge = scoreBadge(q.stats)}
        <!--
          Taban min-w (min-w-0 değil): tuşa OKUNUR bir zemin. min-w-0 olsaydı
          flex-1 tuşları kaba göre ezerdi — 1024px asgarisinde şeride kalan
          ~627px'e 40 soru sıkıştırmak tuş başına 627/40 ≈ 15.7px demek.
          40 soru yoğun moda düştüğü için rakam 11px, "40" ise 11 × 0.70 × 2
          = 15.4px (Public Sans tabular rakam 0.70em; fontun tnum lookup'ı
          rakamları 1400/2000 upem'lik gövdeye çeviriyor). Komşu rakamlar
          arasında 15.7 − 15.4 ≈ 0.3px kalırdı, yani yapışırlardı. 20px taban
          bunu 20 − 15.4 = 4.6px'te tutuyor, sığmayan fazlalık da ebeveynin
          kendi yatay kaydırmasına gidiyor.
          (Eski yorumun "600px'te tuş 13px, iç kutu 2px" sayıları hiçbir
          hesaptan çıkmıyordu — 600/40 zaten 15px eder — atıldı.)
        -->
        <button
          type="button"
          class="group relative {tusOlcu} flex-1 border-l border-gray-200 py-2.5
                 text-center transition-colors hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-700
                 {q.id === activeId ? 'bg-gray-50 dark:bg-gray-700' : ''}"
          title="{i + 1}. soru · {questionPoints(q)} puan · {BADGE_LABEL[badge]}"
          onclick={() => onselect?.(q.id)}
        >
          <span class="tnum block {sayiOlcu} leading-5" class:font-bold={q.id === activeId}>
            {i + 1}
          </span>
          <!-- Durum tuşun kendisinde: altındaki çizgi ölçümün rengidir. -->
          <span
            class="mt-[5px] block h-[3px] w-full {BADGE_CLASS[badge]}"
            aria-hidden="true"
          ></span>
        </button>
      {/each}
    </div>

    <span
      class="tnum flex items-center border-l border-gray-200 px-5 text-[11px] font-semibold uppercase
             tracking-wider text-gray-500 dark:border-gray-700 dark:text-gray-400"
    >
      {questions.length} soru
    </span>
  </div>
{/if}
